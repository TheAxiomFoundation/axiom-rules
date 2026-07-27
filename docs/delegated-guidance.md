# Delegated agency guidance: `supplies` relation design

Status: proposal for discussion on issue #117. This note is intentionally
design-only. No schema or compiler implementation should begin until the shape
is agreed.

## Decision summary

1. Add `guidance/` as an atomic RuleSpec document root. Guidance modules use
   the same singular `module.source_verification.corpus_citation_path`
   requirement as other provision-backed atomic modules.
2. Extend the existing `kind: source_relation` graph with `type: supplies`.
   Do not add a parallel top-level `overrides:` graph. Supplying a value within
   delegated authority does not override the statute.
3. Keep the stable delegation vocabulary in the delegating provision's module.
   Put each period-specific fulfillment relation in the guidance module and
   point it back to the exact delegation relation. This avoids making a stable
   statute import and change for every annual guidance document.
4. Use exact canonical rule references for every endpoint and an explicit
   delegated-value/supplied-value binding. Bare value names are unsafe in the
   presence of cross-module and cross-kind name collisions.
5. Add a typed program applicability period. A `supplies` relation is valid
   only when its typed period exactly matches the composed program's typed
   period, and execution of that artifact is limited to that period.
6. Preserve validated source relations in the compiled artifact. A relation
   that is checked and then discarded is not first-class and cannot support a
   closure query.
7. Resolve the guidance module through `ModuleSource` and its singular corpus
   citation through an injected, pinned corpus index. Citation syntax alone is
   not proof that the cited provision exists.

## Why `supplies`, not `overrides`

RuleSpec already represents legal and provenance edges as non-executable
`source_relation` rules. Its closed relation vocabulary includes `delegates`,
`implements`, `sets`, `amends`, and other relations. A same-kind `sets` edge can
also bind a downstream parameter or derived implementation into a delegated
upstream slot.

Annual IRS guidance in the §32(j) case fills slots that Congress directed the
Secretary to calculate. It does not displace contradictory statutory text.
Calling that edge `overrides` would encode the wrong authority rule. A distinct
`supplies` relation should reuse the compatible binding mechanics of `sets`
while adding stricter document-class, delegation, temporal, corpus, origin, and
closure checks. True amendment, supersession, and conflict remain separate
relations.

## Proposed RuleSpec shape

### Stable declaration in the delegating provision

The statute owns a stable `delegates` relation and the complete set of values
whose current fulfillment is required for closure:

```yaml
# us/statutes/26/32/j.yaml
format: rulespec/v1
module:
  source_verification:
    corpus_citation_path: us/statutes/26/32/j
rules:
  - name: eitc_inflation_adjustment_delegation
    kind: source_relation
    source_relation:
      type: delegates
      target: us:statutes/26/32/j#eitc_inflation_adjustment
      authority: us:agencies/treasury
      values:
        - us:statutes/26/32/j#eitc_earned_income_amounts
        - us:statutes/26/32/j#eitc_maximum_credit_amounts
```

For `type: delegates`, `values` is an optional additive list. Existing
`delegates` records without it remain valid, but they cannot be the
`delegated_by` endpoint of a `supplies` relation because the compiler would
have no closed vocabulary against which to validate supplied values.

Each entry is an exact canonical rule ID, not a local name. It must resolve to
an executable parameter or derived slot owned by the same atomic module as the
delegation relation.

### Period-specific fulfillment in the guidance module

The guidance module imports the higher-authority provision, defines the
published values as ordinary provision-backed rules, and declares how those
rules fulfill the delegation:

```yaml
# us/guidance/irs/rev-proc-2025-32/earned-income-credit.yaml
format: rulespec/v1
imports:
  - us:statutes/26/32/j
module:
  source_verification:
    corpus_citation_path: us/guidance/irs/rev-proc-2025-32/page-14
applicability_period:
  period_kind: tax_year
  start: 2026-01-01
  end: 2026-12-31
rules:
  # The parameter/table rules transcribed from the guidance appear here.

  - name: supplies_eitc_2026_amounts
    kind: source_relation
    source_relation:
      type: supplies
      delegated_by: us:statutes/26/32/j#eitc_inflation_adjustment_delegation
      supplied_by: us:guidance/irs/rev-proc-2025-32/earned-income-credit
      period:
        period_kind: tax_year
        start: 2026-01-01
        end: 2026-12-31
      bindings:
        - delegated_value: us:statutes/26/32/j#eitc_earned_income_amounts
          supplied_value: us:guidance/irs/rev-proc-2025-32/earned-income-credit#eitc_earned_income_amounts
        - delegated_value: us:statutes/26/32/j#eitc_maximum_credit_amounts
          supplied_value: us:guidance/irs/rev-proc-2025-32/earned-income-credit#eitc_maximum_credit_amounts
```

`delegated_by` is the exact generated ID of a `type: delegates` source-relation
rule. `supplied_by` is an exact, fragmentless atomic module target under
`guidance/`. The loader-stamped origin of the `supplies` rule must equal
`supplied_by`; a module cannot claim another document's identity.

Each binding is directional. `delegated_value` names the slot authorized by
the higher-authority provision and `supplied_value` names the executable rule
transcribed from the guidance. This deliberately does not accept the issue's
bare list of names: the two sides need not share a name, and name-only matching
would recreate the identity-collision defect addressed by issue #115.

The period is the existing full `PeriodSpec` shape, not the shorthand string
`"2026"` and not a rule's `period: Month`/`TaxYear` cadence. Exact kind, start,
and end dates avoid ambiguity among calendar years, tax years, fiscal years,
and custom periods.

### Composed program contract

An `axiom-compose` root that uses supplied guidance must emit the same typed
period:

```yaml
format: rulespec/v1
module:
  kind: composition
applicability_period:
  period_kind: tax_year
  start: 2026-01-01
  end: 2026-12-31
imports:
  - us:guidance/irs/rev-proc-2025-32/earned-income-credit
rules: []
```

`applicability_period` is a program contract and therefore belongs at the
RuleSpec/`ProgramSpec` top level, not inside descriptive `module.summary`.
Imported declarations do not select it: the root document is authoritative.
It is optional for legacy programs and required when the root's merged legal
graph contains a `supplies` relation.

A compiled artifact carrying `applicability_period` must reject an execution
query for any other period. Merely checking the period during composition while
allowing the artifact to execute for a later year would leave the stale-guidance
defect intact.

## Compile-time semantics

Compilation of a graph containing `type: supplies` proceeds fail-closed:

1. Resolve `delegated_by` to exactly one imported `source_relation` whose type
   is `delegates`. Its loader-stamped origin must be an atomic, provision-backed
   module, and it must declare a non-empty `values` vocabulary.
2. Resolve `supplied_by` as an exact atomic RuleSpec module under `guidance/`.
   The supplying relation and every `supplied_value` must have that exact
   loader-stamped origin.
3. Require the guidance module's singular
   `source_verification.corpus_citation_path`, require its document class to be
   `guidance`, and resolve that exact path through the pinned corpus index used
   for the compilation. A syntactically valid path that is absent from the
   pinned corpus fails.
4. Require exact equality among the relation period, the supplying guidance
   module's `applicability_period`, and the composed root's
   `applicability_period`.
5. Resolve every `delegated_value` and `supplied_value`. Every delegated side
   must be in the referenced delegation's declared vocabulary. Every supplied
   side must be an executable parameter or derived rule from `supplied_by`.
6. Apply the existing `sets` compatibility checks to each binding: both sides
   have the same executable kind, and their index, unit, entity, dtype, and
   rule cadence are compatible. The supplied rule must have effective-dated
   semantics covering the full applicability period.
7. Reject duplicate bindings, more than one supplier for the same delegated
   value in an applicability period, and overlapping/conflicting supply
   relations.
8. For closure, take the union of valid bindings for each
   `(delegated_by, applicability_period)`. It must equal the delegation's
   declared value set. Rejecting unauthorized extra values is necessary but
   not sufficient: missing delegated values must also fail.
9. Normalize each valid binding to the existing same-kind binding operation so
   execution uses the guidance value, while retaining the original
   `delegates` and `supplies` records in deterministic compiled metadata.

An ordinary `imports:` edge remains necessary to load executable dependencies,
but it carries no delegation semantics and never satisfies these checks by
itself.

## Artifact representation and closure

Today non-`sets` source relations are validated during RuleSpec lowering and
then discarded. The additive IR/artifact change should retain a deterministic
`source_relations` collection with, at minimum:

- canonical relation ID and loader-stamped origin;
- relation type;
- exact endpoints;
- typed period, when present;
- exact value bindings; and
- singular corpus citation path for each provision-backed endpoint.

Legacy artifacts without this collection remain loadable. Absence means
"delegation closure is unreported", not "closed". Consumers can ask whether a
delegation is fulfilled by querying retained, compiler-validated edges; a
cached boolean is unnecessary and risks drifting from the graph.

## Required negative tests

Every rejection gate introduced by implementation needs a fixture that
constructs the rejected input. At minimum:

- `supplied_by` module missing, non-canonical, fragmented, or outside
  `guidance/`;
- supplying relation or supplied rule whose actual origin differs from
  `supplied_by`;
- missing, malformed, non-guidance, or non-resolving singular corpus citation;
- compilation with `supplies` but no pinned corpus resolver;
- `delegated_by` missing, wrong relation type, synthesized, or not
  provision-backed;
- delegation with no declared values;
- absent composed-program period and each kind/start/end period mismatch;
- execution query outside the artifact applicability period;
- unknown, bare-name, wrong-origin, or unauthorized delegated value;
- unknown, non-executable, synthesized, or wrong-origin supplied value;
- parameter/derived kind mismatch and every existing `sets` compatibility
  mismatch;
- supplied versions that do not cover the applicability period;
- duplicate, overlapping, or conflicting supplies; and
- a delegation whose required values are only partially supplied.

Positive tests should cover a single guidance module, closure split across two
singular provision/page modules, artifact round-trip retention, in-memory
module/corpus sources, filesystem roots, and legacy documents/artifacts with no
new fields.

## Compatibility and rollout

The schema changes are additive: a new atomic root, one relation enum value,
optional fields on `delegates`, a supplies-specific payload, an optional
program period, and an optional artifact relation collection. Existing modules
and artifacts continue to parse and execute unchanged.

The old compilation entry points have no corpus capability. Add a compile
entry point/context accepting both a `ModuleSource` and a pinned corpus index.
If a `supplies` relation reaches an entry point without that capability, reject
that new relation explicitly; do not silently downgrade the corpus-resolution
gate. Legacy graphs that contain no `supplies` relation need no corpus index.

Adding `guidance/` can happen without immediately rejecting existing
`policies/` modules. Reserving `policies/` exclusively for compositions is not
an additive change in this repository: `policies/` is currently an allowed
atomic root and existing modules depend on that contract, while generated
compositions already use a separate ephemeral `module.kind: composition`
surface. Migrate document-backed guidance first; any later namespace
restriction needs its own deprecation plan.

## Concrete-case corrections needed before migration

The issue's example uses the removed plural
`source_verification.corpus_citation_paths` form for pages 14 and 15. The
current engine intentionally requires one singular corpus provision join key
per atomic source/proof node and rejects the plural field. The revenue-procedure
encoding therefore cannot be stamped provision-backed as one two-page node
without changing that invariant.

The correct migration is either:

- split the values into singular page/provision-backed guidance modules and
  let multiple `supplies` relations jointly close the delegation; or
- add one canonical corpus provision node that genuinely encompasses the
  whole table and cite that one node.

Restoring plural provenance would make the node-level provision identity
ambiguous and would conflict with issue #115.

Finally, this engine can currently prove that a RuleSpec module resolves, but
it cannot prove that a corpus path resolves because no corpus index is part of
its compilation capability. A `ModuleSource` check plus citation-shape
validation is not the compile-time corpus check requested by #117. The pinned
corpus resolver and typed period emitted by `axiom-compose` are prerequisites,
not details the engine should infer.
