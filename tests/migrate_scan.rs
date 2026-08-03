//! The migrate-scan detector must find real hand-written expansions (which
//! lower as nested binary and/or chains), ignore already-migrated sugar, and
//! refuse near-miss shapes.

use axiom_rules_engine::migrate::scan_source;

const HAND_EXPANDED: &str = r#"
format: rulespec/v1
rules:
  - name: filing_status_is_valid
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: |-
          (
            status_single
            and not status_married_separate
            and not status_joint
            and not status_head_of_household
          )
          or (
            not status_single
            and status_married_separate
            and not status_joint
            and not status_head_of_household
          )
          or (
            not status_single
            and not status_married_separate
            and status_joint
            and not status_head_of_household
          )
          or (
            not status_single
            and not status_married_separate
            and not status_joint
            and status_head_of_household
          )
"#;

#[test]
fn finds_the_hand_expanded_shape_through_binary_nesting() {
    let hits = scan_source(HAND_EXPANDED).expect("module lowers");
    assert_eq!(hits.len(), 1, "exactly one site: {hits:?}");
    assert_eq!(hits[0].rule, "filing_status_is_valid");
    assert_eq!(hits[0].site, "versions[0].expr");
    assert_eq!(hits[0].arity, 4);
}

#[test]
fn ignores_already_migrated_sugar() {
    let rulespec = r#"
format: rulespec/v1
rules:
  - name: filing_status_is_valid
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: exactly_one(status_single, status_married_separate, status_joint, status_head_of_household)
"#;
    let hits = scan_source(rulespec).expect("module lowers");
    assert!(hits.is_empty(), "sugar is not a candidate: {hits:?}");
}

#[test]
fn finds_the_minimum_arity_pair() {
    let rulespec = r#"
format: rulespec/v1
rules:
  - name: one_election
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: |-
          (takes_standard_deduction and not itemizes)
          or (not takes_standard_deduction and itemizes)
"#;
    let hits = scan_source(rulespec).expect("module lowers");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].arity, 2);
}

#[test]
fn refuses_or_of_ands_that_is_not_exactly_one() {
    // Overlapping branches: no negations of the complement set.
    let rulespec = r#"
format: rulespec/v1
rules:
  - name: any_route_applies
    kind: derived
    entity: Household
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: |-
          (is_elderly and has_income)
          or (is_disabled and has_income)
"#;
    let hits = scan_source(rulespec).expect("module lowers");
    assert!(
        hits.is_empty(),
        "overlapping branches are not a gate: {hits:?}"
    );
}

#[test]
fn refuses_mismatched_branch_and_base_counts() {
    // Three branches over two bases (an at-least pattern, not exactly-one).
    let rulespec = r#"
format: rulespec/v1
rules:
  - name: lopsided
    kind: derived
    entity: Household
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: |-
          (a and not b)
          or (not a and b)
          or (a and b)
"#;
    let hits = scan_source(rulespec).expect("module lowers");
    assert!(hits.is_empty(), "three branches over two bases: {hits:?}");
}

#[test]
fn finds_the_flat_pairwise_exclusion_idiom() {
    // The Hawaii/Wisconsin shape: disjunction of the statuses, then a NOT
    // for every unordered pair.
    let rulespec = r#"
format: rulespec/v1
rules:
  - name: filing_status_is_valid
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: |-
          (a or b or c or d)
          and not (a and b)
          and not (a and c)
          and not (a and d)
          and not (b and c)
          and not (b and d)
          and not (c and d)
"#;
    let hits = scan_source(rulespec).expect("module lowers");
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].idiom, "pairwise_exclusions");
    assert_eq!(hits[0].arity, 4);
}

#[test]
fn finds_the_factored_triangular_exclusion_idiom() {
    // The North Dakota shape: each status excludes the ones after it.
    let rulespec = r#"
format: rulespec/v1
rules:
  - name: filing_status_is_valid
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: |-
          (a or b or c or d)
          and not (a and (b or c or d))
          and not (b and (c or d))
          and not (c and d)
"#;
    let hits = scan_source(rulespec).expect("module lowers");
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].idiom, "pairwise_exclusions");
    assert_eq!(hits[0].arity, 4);
}

#[test]
fn refuses_incomplete_pair_coverage() {
    // Missing not(c and d): at-most-one does not hold, so this is not a gate.
    let rulespec = r#"
format: rulespec/v1
rules:
  - name: leaky
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: |-
          (a or b or c or d)
          and not (a and (b or c or d))
          and not (b and (c or d))
"#;
    let hits = scan_source(rulespec).expect("module lowers");
    assert!(
        hits.is_empty(),
        "incomplete coverage is not a gate: {hits:?}"
    );
}

#[test]
fn refuses_pairwise_shape_with_extra_conjuncts() {
    // A residency conjunct rides along: the chain is not a pure gate.
    let rulespec = r#"
format: rulespec/v1
rules:
  - name: entangled
    kind: derived
    entity: TaxUnit
    dtype: Judgment
    versions:
      - effective_from: 2026-01-01
        formula: |-
          is_resident
          and (a or b)
          and not (a and b)
"#;
    let hits = scan_source(rulespec).expect("module lowers");
    assert!(hits.is_empty(), "extra conjuncts are not a gate: {hits:?}");
}
