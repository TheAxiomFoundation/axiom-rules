# Delegated agency guidance: `supplies` relation design

Status: proposal for discussion on issue #117. This note is intentionally
design-only. No schema or compiler implementation should begin until the shape
is agreed.

## Current facts that constrain the design

RuleSpec already represents legal and provenance edges as non-executable
`kind: source_relation` rules. Its closed vocabulary includes `delegates`,
`implements`, `sets`, `amends`, and other relations. Same-kind `sets` edges can
also copy an implementation's parameter versions or derived semantics into an
upstream executable node.

The concrete EITC encoding does not have statute-owned executable parameter
slots for the annual amounts. `us:statutes/26/32` directly imports the guidance
module and its formulas consume guidance-owned parameters. The current module
target is `us:statutes/26/32`, while its corpus join key is the different,
singular document-class path `us/statute/26/32`.

The engine also has no typed composed-program applicability period, no corpus
resolver, and no retained source-relation graph in `ProgramSpec` or compiled
artifacts. It can validate citation syntax, but not whether a citation resolves
in a pinned corpus release.

Those facts rule out pretending that a new enum spelling alone solves #117.

## Decisions

1. Add `guidance/` as an atomic RuleSpec document root. A guidance module that
   participates in `supplies` must have one singular, resolving
   `module.source_verification.corpus_citation_path`.
2. Extend the existing source-relation graph with `type: supplies`. Do not add
   a parallel top-level `overrides:` graph. Filling a value within delegated
   authority does not override the statute.
3. Put the period-specific relation in the delegating provision module for the
   first additive migration, alongside its existing executable import. This is
   the placement requested by the issue and does not require inventing
   statute-owned empty parameter slots.
4. Model the provision's delegated values as typed legal-contract identifiers,
   not fake executable nodes. A supply binding maps one exact contract ID to
   one exact guidance-owned executable rule ID.
5. `supplies` is a legal/provenance and closure relation. It must not reuse the
   current destructive `sets` lowering, which would copy guidance semantics
   under the statute node's identity and make the result look statute-backed.
6. Add a typed root `applicability_period`. It must exactly match each supply
   relation used by the composition. Runtime query periods must be contained
   by that applicability interval and be cadence-compatible; universal exact
   query equality would incorrectly prohibit monthly queries within annual or
   fiscal-year guidance.
7. Preserve validated legal relations and computed delegation-closure results
   in compiled artifacts. Absence in a legacy artifact means unreported, not
   satisfied.
8. Resolve RuleSpec modules through `ModuleSource` and corpus citations through
   an injected, pinned corpus index. A missing corpus capability fails closed
   only for the new certification contract; legacy graphs remain unchanged.

## Proposed RuleSpec shape

The following EITC snippets illustrate the schema mechanics. They are not a
complete §32(j) legal vocabulary and must not be used as a certification
fixture without enumerating every amount directly adjusted under §32(b)(2) and
§32(i)(1).

### Guidance as an atomic document-backed module

```yaml
# us/guidance/irs/rev-proc-2025-32/earned-income-credit.yaml
format: rulespec/v1
module:
  source_verification:
    corpus_citation_path: us/guidance/irs/rev-proc-2025-32
rules:
  - name: eitc_earned_income_amounts
    kind: parameter
    dtype: Money
    unit: USD
    indexed_by: qualifying_child_count
    source: Rev. Proc. 2025-32 section 3.06(1)
    versions:
      - effective_from: 2026-01-01
        effective_to: 2026-12-31
        values:
          0: 8680
          1: 13020
          2: 18290
          3: 18290
```

`supplied_by` will address the exact fragmentless module target under
`guidance/`. The module target and corpus path are different identity spaces:
the former follows RuleSpec's plural repository root, while the latter must
match the corpus record exactly.

The corpus already has a singular document-root node for this revenue
procedure. A migration may cite that node if it honestly grounds the encoded
provision, or split the current page-14/page-15 encoding into singular
page/provision modules for more precise grounding. It must not restore plural
node provenance.

### Delegation vocabulary and fulfillment on the provision

```yaml
# excerpt from us/statutes/26/32.yaml
format: rulespec/v1
imports:
  - us:guidance/irs/rev-proc-2025-32/earned-income-credit
module:
  source_verification:
    corpus_citation_path: us/statute/26/32
rules:
  - name: eitc_inflation_adjustment_delegation
    kind: source_relation
    source_relation:
      type: delegates
      target: us:statutes/26/32
      authority: us:agencies/treasury
      values:
        - id: us:statutes/26/32#delegated.eitc_earned_income_amounts
          shape:
            kind: parameter
            dtype: Money
            unit: USD
            indexed_by: qualifying_child_count

  - name: rev_proc_2025_32_supplies_eitc_2026
    kind: source_relation
    source_relation:
      type: supplies
      delegated_by: us:statutes/26/32#eitc_inflation_adjustment_delegation
      supplied_by: us:guidance/irs/rev-proc-2025-32/earned-income-credit
      period:
        period_kind: tax_year
        start: 2026-01-01
        end: 2026-12-31
      bindings:
        - delegated_value: us:statutes/26/32#delegated.eitc_earned_income_amounts
          supplied_value: us:guidance/irs/rev-proc-2025-32/earned-income-credit#eitc_earned_income_amounts
```

For `type: delegates`, `values` is an optional additive list of typed legal
contracts. Existing delegation relations without it remain valid, but they
cannot be a `delegated_by` endpoint because the compiler would have no closed
authority vocabulary. IDs are exact, module-owned contract identifiers.
They do not claim that an empty/pending executable parameter exists.

Each `shape` is validated against the supplying RuleSpec declaration before
lowering, while parameter dtype and other source facts are still available.
The supplies contract requires exact equality, including presence versus
absence; the permissive "only compare when both sides declare a field" behavior
of existing `sets` is insufficient.

For `type: supplies`, validation is type-specific. It does not require the
legacy generic `source_relation.target`: `delegated_by`, `supplied_by`,
`period`, and non-empty `bindings` are its complete payload. These structures
deny unknown fields, so a misspelled legal claim cannot deserialize and vanish.

The supply relation's loader-stamped origin must be the same atomic provision
module that owns `delegated_by`. Every `supplied_value` must instead have the
exact loader-stamped guidance origin named by `supplied_by`. A module cannot
self-assert either endpoint's identity.

### Composed program period

```yaml
format: rulespec/v1
module:
  kind: composition
applicability_period:
  period_kind: tax_year
  start: 2026-01-01
  end: 2026-12-31
imports:
  - us:statutes/26/32
rules: []
```

`applicability_period` is an executable program contract and belongs at the
RuleSpec/`ProgramSpec` root, not in descriptive `module.summary`. The composed
root is authoritative; imported modules do not overwrite it.

Supply relations carry their own periods, so the compiler does not need to
retain an imported module-level period that the current merge model would
discard. Each supplied rule's effective versions must cover the complete
relation interval.

The full `PeriodSpec` shape, rather than the shorthand string `"2026"`, avoids
ambiguity among calendar, tax, fiscal, and custom years. Invalid bounds fail.
At runtime, a query must be wholly contained in the artifact applicability
interval and have a compatible period kind. A tax-year-only program can impose
exact equality as a narrower contract; it is not the universal rule.

## Non-executable relation semantics

`supplies` must not copy parameter versions or derived formulas between nodes.
The current `sets` implementation mutates the upstream target while retaining
that target's ID and source fields. Reusing it would erase the fact that the
value came from guidance and would conflict directly with issue #115.

The existing executable import remains responsible for making the guidance
rule available to formulas. `supplies` proves why that reachable guidance node
is legally in the program, which delegation contract it fulfills, and for what
period. The compiled node keeps its guidance ID, guidance corpus citation, and
provision-backed annotation.

Moving the annual relation physically into the guidance module and reversing
the import direction is a possible later architecture, but the concrete module
cannot do that additively today: statute formulas consume guidance parameters
directly and RuleSpec cannot declare an empty indexed pending parameter slot.
That move first needs a separately designed late-bound/delegated-slot contract.
The initial relation-on-provision design states this limitation rather than
silently manufacturing slots.

## Certification compilation and closure

Ordinary schema parsing can validate the local typed shape. Full certification
compilation of a composed root performs cross-module, period, corpus, and
closure checks after all imports have resolved.

The new `guidance/` root is strict: every guidance-owned executable node that
is reachable from the composition's declared outputs must be justified by
exactly one valid legal relation for the applicability period. A plain
`guidance/` import therefore cannot evade the checks merely by omitting
`supplies`.

For every referenced `delegated_by`, the union of bindings valid for the
applicability period must equal the delegation's declared value IDs. Extra
values are unauthorized; missing values leave the delegation open. The
certification compiler rejects both, including the zero-supplier case.
General tooling may display the retained open state, but it must not label it
satisfied.

This closure test operates on the provision encoding. The compiler can prove
that every supplied value is in the provision's declared delegation vocabulary
and that the vocabulary is completely fulfilled. It cannot independently make
the legal judgment that an encoder chose the correct vocabulary. Corpus
grounding, proof review, and oracles remain necessary.

## Compile-time gates

Certification compilation of `supplies` proceeds fail-closed:

1. Resolve `delegated_by` to exactly one provision-backed, atomic
   `type: delegates` relation in the same origin module as the supply relation.
   Require non-empty, unique, canonical value contracts and strict shapes.
2. Resolve `supplied_by` as an exact fragmentless atomic module target under
   `guidance/`. Resolve its singular citation through the pinned corpus index
   and require the corpus document class to be guidance.
3. Resolve every binding. Require each delegated side in the referenced
   vocabulary and each supplied side to be an executable rule whose actual
   origin is exactly `supplied_by`.
4. Compare the contract shape to the source RuleDefinition exactly before
   lowering. Never accept the existing `sets` behavior that silently continues
   when both endpoints are unresolved.
5. Require exact equality between the supply relation period and the composed
   root applicability period. Require complete effective-version coverage by
   the supplied rules.
6. Reject empty or duplicate bindings, duplicate value contracts, more than one
   supplier for the same delegated value in a period, and overlapping or
   conflicting supplies.
7. Check closure across the complete value vocabulary, including no-supplier
   and partial-supplier cases.
8. Check every reachable guidance-owned executable node so omission of the
   relation itself cannot bypass certification.
9. Retain the validated graph and closure result without rewriting executable
   identities or provenance.

A guidance module compiled alone, with no legal relation in that module, still
gets ordinary source/schema validation. The cross-module closure gate runs on
the composed certification surface, where the delegating module, guidance
module, declared outputs, applicability period, `ModuleSource`, and pinned
corpus index are all present. This distinction avoids both a circular import
and a false claim that standalone authoring validation proves closure.

## Artifact representation

The additive artifact/IR change retains a deterministic `source_relations`
collection with, at minimum:

- canonical relation ID and actual loader-stamped origin;
- relation type and exact endpoints;
- typed period and exact value contracts/bindings;
- singular corpus citation paths for provision-backed endpoints; and
- a derived closure result for each `(delegated_by, applicability_period)`.

Legacy artifacts without the collection remain loadable. Absence means
"delegation closure unreported". A closure result is recomputed and checked at
artifact load like other compiler-derived metadata; it is not source-tamper
evidence.

## Required rejection tests

Every new gate needs a fixture that constructs the rejected input. At minimum:

- `supplied_by` missing, non-canonical, fragmented, outside `guidance/`, absent
  from `ModuleSource`, or different from the supplied rule's actual origin;
- guidance citation missing, malformed, non-guidance, absent from the pinned
  corpus, or compiled without the required corpus capability;
- `delegated_by` missing, unresolved, wrong type, synthesized, from the wrong
  origin, or not provision-backed;
- empty, duplicate, non-canonical, or unknown-field delegation values;
- absent composed applicability period; invalid bounds; or kind/start/end
  mismatch with the supply relation;
- execution outside the artifact interval or with incompatible cadence;
- empty/duplicate/unknown-field bindings;
- bare-name, unknown, unauthorized, or wrong-origin binding endpoints;
- missing or unequal shape fields, including kind, dtype, unit, and index;
- supplied versions that do not cover the applicability interval;
- duplicate, overlapping, or conflicting suppliers;
- no supplier and partial closure; and
- reachable guidance values introduced by a plain import with no legal edge.

Positive tests cover a fully closed guidance relation, closure split across
multiple singular provision modules, artifact round-trip/recomputation,
in-memory module and corpus sources, filesystem roots, contained runtime
periods, and unchanged legacy modules/artifacts.

One adversarial test must prove that a same-named statute rule cannot receive
the guidance rule's relation or provenance, and another must prove that
`supplies` never lowers through the identity-rewriting `sets` path.

## Compatibility and prerequisites

The schema changes are additive: a new atomic root, one source-relation type,
optional typed value contracts on `delegates`, a supplies-specific strict
payload, an optional root applicability period, and an optional artifact legal
graph. Existing modules and artifacts continue to parse and execute unchanged.

Compilation needs a new context/entry point accepting both a `ModuleSource` and
a pinned corpus index. Encountering the new certification surface without its
corpus capability is an explicit error; legacy graphs require none.
`axiom-compose` must emit the typed applicability period and declared outputs
before the engine can perform the requested period and reachability gates.

Adding `guidance/` does not immediately reject existing `policies/` modules.
Reserving `policies/` exclusively for compositions is not additive here:
`policies/` is currently an allowed atomic root, while generated compositions
already use a distinct ephemeral `module.kind: composition` surface. Migrate
document-backed guidance first; any hard namespace restriction needs a
separate deprecation plan.

## Corrections to the issue as filed

- `overrides` is the wrong relation for a value supplied under §32(j);
  `supplies` should extend the existing legal graph.
- The engine already has `delegates` and executable `sets` relations, but
  currently discards non-`sets` edges from artifacts.
- The concrete statute module is `us:statutes/26/32`, not
  `us:statutes/26/32/j`, and the corpus identity is the different singular
  `us/statute/26/32`.
- The current guidance module's plural `corpus_citation_paths` violates the
  engine's singular provision identity. A real migration must use the existing
  honest document-root corpus node or split into singular page/provision nodes.
- `eitc_maximum_credit_amounts` is not itself a dollar amount directly adjusted
  by §32(j). It is derived from statutory rates and the adjusted earned-income
  amount (or published as part of prescribed tables). A correct delegation
  vocabulary must distinguish directly delegated amounts from published
  derived conveniences; the compiler cannot infer that legal distinction from
  names.
- The issue's bare list of value names is not collision-safe and cannot express
  different legal-contract and guidance-rule identities.
- A period check is impossible until the composed program carries a typed
  period, and a corpus-resolution check is impossible until compilation is
  given a pinned corpus index.
- Reusing `sets` would falsely retain statutory identity on guidance semantics,
  so `supplies` must remain non-destructive.
