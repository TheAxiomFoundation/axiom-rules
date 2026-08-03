//! Corpus migrations: engine-versioned codemods with machine-checked gates.
//!
//! A migration pairs a detector (spec-level, via the engine's own lowering —
//! never text patterns) with a rewriter and an equivalence gate. This module
//! carries the detector side; see the corpus-migrations design (#152).
//!
//! The pilot detector finds hand-expanded exactly-one patterns: an `or` whose
//! n branches are each an `and` of the same n base judgments, branch i
//! asserting base i and negating the rest — the shape `exactly_one(...)`
//! replaces (#142). Detection walks the serialized `ProgramSpec` JSON, the
//! same `kind:`-tagged form compiled artifacts carry, so structural equality
//! of base judgments is exact serialization equality.

use serde_json::Value;

use crate::rulespec::{RuleSpecError, lower_rulespec_str};

/// One detected hand-expanded exactly-one site.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ExpandedExactlyOne {
    /// Name of the derived rule the expression belongs to.
    pub rule: String,
    /// Where in the rule the expression sits: `expr` or `versions[i].expr`.
    pub site: String,
    /// Number of mutually exclusive base judgments.
    pub arity: usize,
    /// Which hand-written idiom matched: `or_of_ands` (branch i asserts base
    /// i, negates the rest) or `pairwise_exclusions` (a disjunction of the
    /// bases conjoined with NOT terms forbidding every unordered pair —
    /// covers both the flat all-pairs and the factored triangular forms).
    pub idiom: &'static str,
}

/// Scan one RuleSpec module source for hand-expanded exactly-one patterns.
///
/// Lowers through the real loader; a source that does not lower is the
/// caller's to report as unscanned rather than silently pattern-free.
pub fn scan_source(source: &str) -> Result<Vec<ExpandedExactlyOne>, RuleSpecError> {
    let program = lower_rulespec_str(source)?;
    let value = serde_json::to_value(&program)
        .expect("ProgramSpec serialization is infallible for lowered programs");
    let mut found = Vec::new();
    if let Some(derived) = value.get("derived").and_then(Value::as_array) {
        for rule in derived {
            let name = rule
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("<unnamed>")
                .to_string();
            // Versions are the authored surface; the rule-level `expr`
            // mirrors one of them, so scanning both would double-count.
            let versions = rule.get("versions").and_then(Value::as_array);
            match versions {
                Some(versions) if !versions.is_empty() => {
                    for (index, version) in versions.iter().enumerate() {
                        if let Some(expr) = version.get("expr") {
                            collect_sites(
                                expr,
                                &name,
                                &format!("versions[{index}].expr"),
                                &mut found,
                            );
                        }
                    }
                }
                _ => {
                    if let Some(expr) = rule.get("expr") {
                        collect_sites(expr, &name, "expr", &mut found);
                    }
                }
            }
        }
    }
    Ok(found)
}

fn collect_sites(value: &Value, rule: &str, site: &str, found: &mut Vec<ExpandedExactlyOne>) {
    let detected = expanded_exactly_one(value)
        .map(|(arity, bases)| (arity, bases, "or_of_ands"))
        .or_else(|| {
            pairwise_exclusions(value).map(|(arity, bases)| (arity, bases, "pairwise_exclusions"))
        });
    if let Some((arity, bases, idiom)) = detected {
        found.push(ExpandedExactlyOne {
            rule: rule.to_string(),
            site: site.to_string(),
            arity,
            idiom,
        });
        // The branches of a detected expansion are its own machinery; only
        // the base judgments can legitimately contain further candidates.
        for base in bases {
            collect_sites(base, rule, site, found);
        }
        return;
    }
    // Only MAXIMAL and/or chains are candidates: descend to the flattened
    // leaves of a chain, never into intermediate same-kind nodes, so a
    // proper sub-chain of a wider `or` cannot fire as its own gate.
    for kind in ["or", "and"] {
        if let Some(leaves) = flatten_chain(value, kind) {
            for leaf in leaves {
                collect_sites(leaf, rule, site, found);
            }
            return;
        }
    }
    match value {
        Value::Object(map) => {
            for child in map.values() {
                collect_sites(child, rule, site, found);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_sites(child, rule, site, found);
            }
        }
        _ => {}
    }
}

/// Returns the arity and base judgments when `value` is an or-chain of n
/// and-chains over the same n base judgments in the canonical exactly-one
/// expansion shape. Hand-chained `and`/`or` lower as nested binary pairs, so
/// both chains are flattened associatively before the shape check — the flat
/// n-ary form (which only the retired inline desugar produced) matches too.
fn expanded_exactly_one(value: &Value) -> Option<(usize, Vec<&Value>)> {
    let branches = flatten_chain(value, "or")?;
    let n = branches.len();
    if n < 2 {
        return None;
    }
    let first = flatten_chain(branches[0], "and")?;
    if first.len() != n {
        return None;
    }
    // Base judgment j comes from branch 0: positive at position 0, negated
    // elsewhere. Every base must be extractable or the shape does not match.
    let mut base: Vec<&Value> = Vec::with_capacity(n);
    for (position, item) in first.iter().enumerate() {
        if position == 0 {
            base.push(item);
        } else {
            base.push(not_item(item)?);
        }
    }
    for (branch_index, branch) in branches.iter().enumerate() {
        let items = flatten_chain(branch, "and")?;
        if items.len() != n {
            return None;
        }
        for (position, item) in items.iter().enumerate() {
            let matches = if position == branch_index {
                **item == *base[position]
            } else {
                not_item(item).is_some_and(|inner| *inner == *base[position])
            };
            if !matches {
                return None;
            }
        }
    }
    Some((n, base))
}

/// Returns the arity and bases when `value` is the disjunction+exclusions
/// idiom: an and-chain containing exactly one or-chain over n distinct bases
/// (at least one holds) plus NOT terms whose forbidden pairs cover every
/// unordered pair of bases (at most one holds) — and nothing else. Each NOT
/// body must be `x and y` or `x and (y1 or y2 or ...)` with every operand a
/// base; coverage is checked as a set, so the flat all-pairs form and the
/// factored triangular form both match, in any order.
fn pairwise_exclusions(value: &Value) -> Option<(usize, Vec<&Value>)> {
    let leaves = flatten_chain(value, "and")?;
    let mut bases: Option<Vec<&Value>> = None;
    let mut exclusion_bodies: Vec<&Value> = Vec::new();
    for leaf in &leaves {
        if flatten_chain(leaf, "or").is_some() {
            if bases.is_some() {
                return None;
            }
            bases = Some(flatten_chain(leaf, "or")?);
        } else if let Some(body) = not_item(leaf) {
            exclusion_bodies.push(body);
        } else {
            return None;
        }
    }
    let bases = bases?;
    let n = bases.len();
    if n < 2 || exclusion_bodies.is_empty() {
        return None;
    }
    let keys: Vec<String> = bases.iter().map(|base| base.to_string()).collect();
    let index_of = |value: &Value| -> Option<usize> {
        let key = value.to_string();
        keys.iter().position(|candidate| *candidate == key)
    };
    if keys.iter().collect::<std::collections::HashSet<_>>().len() != n {
        return None;
    }
    let mut covered = std::collections::HashSet::new();
    for body in exclusion_bodies {
        let pair = flatten_chain(body, "and")?;
        if pair.len() != 2 {
            return None;
        }
        // Accept either orientation: `x and (…)` or `(…) and x`.
        let (single, group) = if index_of(pair[0]).is_some() {
            (pair[0], pair[1])
        } else {
            (pair[1], pair[0])
        };
        let left = index_of(single)?;
        match flatten_chain(group, "or") {
            Some(rest) => {
                for member in rest {
                    let right = index_of(member)?;
                    if right == left {
                        return None;
                    }
                    covered.insert((left.min(right), left.max(right)));
                }
            }
            None => {
                let right = index_of(group)?;
                if right == left {
                    return None;
                }
                covered.insert((left.min(right), left.max(right)));
            }
        }
    }
    if covered.len() != n * (n - 1) / 2 {
        return None;
    }
    Some((n, bases))
}

/// Flatten an associative `and`/`or` chain into its leaves, in source order.
/// Nested binary nodes of the same kind dissolve; anything else is a leaf.
fn flatten_chain<'v>(value: &'v Value, kind: &str) -> Option<Vec<&'v Value>> {
    let map = value.as_object()?;
    if map.get("kind")?.as_str()? != kind {
        return None;
    }
    let items = map.get("items")?.as_array()?;
    let mut leaves = Vec::new();
    for item in items {
        match flatten_chain(item, kind) {
            Some(nested) => leaves.extend(nested),
            None => leaves.push(item),
        }
    }
    Some(leaves)
}

fn not_item(value: &Value) -> Option<&Value> {
    let map = value.as_object()?;
    if map.get("kind")?.as_str()? != "not" {
        return None;
    }
    map.get("item")
}
