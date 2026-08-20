# Stage-3 NZ final verification cures

Audit basis: `stage3-nz/sol-verify-r3.md`, read end to end before changes.
Only its open findings 1, 2, and 6 were changed. Findings 3, 4, 5, and 7
were rechecked after the cures.

- Starting audit commit: `d32cf980bdd5a3b7a7e582604dffa90687784b69`
- Final implementation commit: `d9069693e92ee338ab2c1d62461e73bcb02f195d`
- Ops harness base, read only: `bcf631b5`
- Release binary SHA-256: `3a37cd2bfa570569252b97ac0f563fcf089fd97f43b208943c15ae28bb445e34`
- Harness patch SHA-256: `11018bb4eb935bbb27ac1ce80e5dd008da045d0d28827c64cf0dfbbb6745087a`

No push or PR was made. No `PROGRESS.md` was created, edited, deleted, or
materialized. Clean-checkout builds used `git archive` into scratch trees with
the tracked root `PROGRESS.md` explicitly excluded.

## Cure 1 — typed child-gross provenance

### Implementation

The aggregation plan now binds every `ChildGrossAmount` to a public numeric,
Child-scoped rule in a production-validated compiled source artifact. The
compiled aggregation artifact embeds and digests that source artifact, and its
surfaced plan digest binds both the canonical plan and the source-artifact
digest. Coordinated source changes therefore change the plan identity.

The materialization layer maintains a private `OperandProvenance` index:

- request scalars are always `RequestSupplied`;
- only successful execution of the bound embedded engine artifact constructs
  `EngineComputed { source_artifact_digest, rule_id, stage: Gross }`;
- supplying a scalar and a same-name computation recipe leaves the operand
  request-supplied; and
- the family reduction compares the exact marker before reading any child
  gross operand.

The named rejection is:

```text
InvalidChildGrossProvenance {
  operation,
  input,
  child,
  expected,
  found,
}
```

For the committed audit reproduction it renders:

```text
child-gross operand `best_start_after_per_child_abatement` for `best_start_total` on child `child-0` has provenance `request_supplied`, expected `engine_computed(source_artifact=sha256:60cf4aef86b144b9acf65cdf9c1f0c43a29b03122858d9a44b2f716ed3aa5525,rule_id=nz:statutes/income_tax/family_scheme/tax_credits#best_start_tax_credit_before_abatement,stage=gross)`
```

Committed regressions:

- `renamed_child_gross_plan.yaml`, SHA-256
  `5f649402d4d221cefbb11770daa72260ae6de0fe9ec757487ef3640d08657cbc`;
- `renamed_child_gross_request.json`, SHA-256
  `7648a58de650e2fa717ac18d8a3a6d758801617210fdcb95a650f5d9bd85e810`;
- `relabelled_request_scalar_is_rejected_by_named_engine_provenance_guard`;
- `request_scalar_cannot_launder_through_a_same_name_engine_recipe`; and
- source-artifact, advertised-source-digest, plan, constitution, and phase
  tamper mutants.

The two audit comments were restored in `per_child_expressible.yaml`. Its exact
SHA-256 is the required
`39011e1c1f512d98c969b951ff91420d4996e18ccdea6770762611d51b49867d`.

### Guard reversion

This was repeated against an archive-derived copy of the final implementation.
With only the provenance comparison changed to `false && ...`, the audit attack
went green:

```text
mutant_exit=0
families.status=determined
best_start_total=6082
best_start_total_before_abatement=6082
plan_digest=sha256:b169151f941d7d891f75c5f0175a8703c0de1cc1f5d159af9e87a0cf7056e5a2
```

After restoring the comparison and rebuilding the same plan, the attack went
red with exit 1 and the named error above. The plan digest remained
`sha256:b169151f941d7d891f75c5f0175a8703c0de1cc1f5d159af9e87a0cf7056e5a2`.
The scratch source then compared byte-for-byte with the committed source.

## Cure 2 — family-level Knowledge propagation

### Implementation

The family envelope is now `AggregationFamilyKnowledge`, with exactly the two
stage-2 statuses `determined` and `indeterminate`. Once family topology is
known, an indeterminate envelope retains `value: Some(families)` so members and
children remain complete. Its reasons are the sorted union of every consumed
indeterminate partner value, scalar, count, predicate, child age, age-band
value, and child scalar.

The committed regression is
`stage3-nz/probes/mixed_evidence_probe.sh`, backed by
`mixed_conflict_care_and_unknown_age_indetermines_family_without_losing_topology`.
It sets child 0 care to Conflict and child 1 age to Unknown and asserts the
family status, reasons, scalars, counts, and exact topology.

### Guard reversion

With only the nonempty-reasons branch forced back to `determined`, the
committed probe exited 1. The raw result reproduced the audit defect without
losing topology:

```text
families_status=determined
members=child-0,child-1,partner,primary
children=child-0,child-1
best_start_total=indeterminate
dependent_child_count=indeterminate
youngest_child_age=indeterminate
```

After restoring the branch, rebuilding, and rerunning the same probe:

```text
families_status=indeterminate
members=child-0,child-1,partner,primary
children=child-0,child-1
best_start_total=indeterminate
dependent_child_count=indeterminate
youngest_child_age=indeterminate
```

The restored probe exited 0, and the scratch source compared byte-for-byte
with the committed source.

The five prior named Knowledge probes each passed one test:

```text
lifted_derived_relation_never_drops_unknown_or_conflict
unknown_and_conflict_survive_derived_relation_gates_for_counts_and_sums
indeterminate_relation_knowledge_cannot_produce_a_smaller_determined_family
omitted_relation_is_unknown_but_evidenced_complete_empty_is_known_empty
unknown_status_survives_materialization_and_indeterminates_phase_two_count
```

## Cure 3 — citation accuracy

I checked each corrected citation against the official New Zealand Legislation
text on 19 August 2026.

- [MC 9](https://www.legislation.govt.nz/act/public/2007/0097/latest/DLM1518490.html)
  was split into (1)(a), (1)(b), and (2)-(3). Its operative phrases are
  “is not financially independent”, “is attending school or a tertiary
  educational establishment”, and “The Commissioner must determine the
  period”. The full conjunction is used only for age 18.
- [YA 1](https://www.legislation.govt.nz/act/public/2007/0097/latest/DLM1520575.html)
  now supplies ages 14-17: “is aged 15 years or less” and “is aged 16 or 17
  years and is not financially independent”. Its age-18 branch says “is aged
  18 years”, which the plan combines with the MC 9 facts.
- [MC 10](https://www.legislation.govt.nz/act/public/2007/0097/latest/DLM1518492.html)
  keeps principal-caregiver qualification at subsection (3), including
  “totalling at least one-third”. The exclusive-care mechanic cites subsection
  (4), whose operative phrase is “periods during which the principal caregiver
  has exclusive care”.
- [MD 10](https://www.legislation.govt.nz/act/public/2007/0097/latest/DLM1518534.html)
  is now included for the applicable calculation: subsection (2) supplies the
  formula introduced by “calculated using the formula”, and subsection (3)(d)
  defines “weekly periods”.
- [MC 7](https://www.legislation.govt.nz/act/public/2007/0097/latest/DLM1518486.html)
  says “This section applies when a person has a spouse” and, outside IWTC,
  “the Commissioner must determine which”. The plan no longer labels generic
  partner presence as entitlement or selection. It instead accepts the family
  unit's adult reference person for the declared caller-evidenced partner
  relation, and explicitly leaves MC 7(1) applicability and MC 7(2) allocation
  or Commissioner selection out of scope.

These remain Tier-B plan inputs and cited limitations. No legal reading became
an engine default. The exact citation regression
`nz_tier_b_legal_choices_are_explicit_cited_inputs_and_limitations` passed.

The source plan, request, expected result, four published schemas, CLI fixture,
compiled source binding, and harness were regenerated. Relevant final values:

- aggregation-plan file SHA-256:
  `311f64cb64c3b5a4b0f49fd2ff634cda43cb327d4a148683784a8fe1705787a6`;
- full compiled source artifact SHA-256:
  `355659bf30cef49c440d5fa24014f1bf6c9a17964befb164880196950462f3b2`;
- full compiled aggregation artifact SHA-256:
  `46d9bb6cbd03d69cde3419b409225e62f23f43056fc9133340b0c7138eba4f2c`;
- bound plan digest:
  `sha256:7bf9c7984041b15bedcaeba32e282654a33de823594fef2eecd9989ccfd4cde2`;
- bound source-artifact digest:
  `sha256:5162363e74b0e5246e3da3d42513ccbbcc3dd42cf655e1c3a4b71540ae2888bd`;
- constitution digest:
  `sha256:94545f002fd9b1596e7186684c75b335a58e2d9a20a2871512f9b147565d9bc3`.

## Preserved findings and test totals

Finding 3 remained resolved:

- `reviewer_best_start_plan_is_rejected_by_named_family_scope_guard`: 1 passed;
- `every_stage3_structural_guard_kills_its_isolated_plan_mutant`: 1 passed,
  covering all 11 isolated structural mutants.

Finding 4 remained resolved. The final patch was applied to a fresh copy made
from `git -C ops show`/the equivalent path-limited archive at `bcf631b5`; its
dirty-checkout, unbound-binary, wrong-commit, wrong-digest, foreign-plan, and
operational-boundary mutants all passed:

```text
python3 -m pytest -q test_declared_closures.py
14 passed, 0 failed
```

Finding 5 remained resolved. The production registry/source loader, CLI exact
fixture, source/plan/constitution/phase tamper test, duplicate registration,
raw-sidecar rejection, and schemas passed. `schema_golden` passed 8 tests.

Finding 7 remained resolved. Final full-suite reruns were:

| Suite | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| flag off | 285 | 0 | 0 |
| `unit-derivation` | 331 | 0 | 0 |
| all features | 347 | 0 | 0 |
| copied harness mutants | 14 | 0 | — |

The flag-off current and parent (`d32cf980`) release binaries compiled the same
176-output format-2 composition artifact byte-for-byte. Both artifact SHA-256
values are
`355659bf30cef49c440d5fa24014f1bf6c9a17964befb164880196950462f3b2`,
and their top-level help was byte-identical.

## End-to-end parity

Two independent external invocations ran from the patched harness copy. Each
invocation itself compiled and evaluated twice and byte-compared its two fresh
passes.

| Check | Run 1 | Run 2 |
| --- | ---: | ---: |
| amount/control cells agreeing to cent | 1,454 / 1,976 | 1,454 / 1,976 |
| outside-cent dispositions | 522 | 522 |
| class B / class C / class A / class D | 520 / 2 / 0 / 0 | 520 / 2 / 0 / 0 |
| ordinary engine evaluations per pass | 881 | 881 |
| unit-aggregation evaluations per pass | 1,411 | 1,411 |

Both `comparison.csv` files are byte-identical, with SHA-256
`ccaa4dcb61b112587b47afb0e1892f670df354670fcd35f4d801edc621dd4bf2`.
The sorted disposition identity over `scenario_id`, `weekly_wage`, `column`,
`classification`, `reason_code`, and `reason_title` contains the same 522 IDs
in both runs and has SHA-256
`44063af7ec7c3a4dc6a296b70efc18914762a0d88e59594610e025c4d671e14e`.

The ordinary compiled artifact is the frozen
`355659bf30cef49c440d5fa24014f1bf6c9a17964befb164880196950462f3b2`.
The intentionally regenerated unit-aggregation artifact is
`46d9bb6cbd03d69cde3419b409225e62f23f43056fc9133340b0c7138eba4f2c`.

## Honesty notes

- No comparison cell or disposition changed.
- I do not claim the two external `comparison.json` files are byte-identical:
  each truthfully embeds its own output-directory path. That one path causes
  the corresponding external `SHA256SUMS` line to differ. The requested
  `comparison.csv` files are byte-identical, all 522 disposition identities
  are identical, and each invocation's two internal fresh passes compared all
  generated artifacts byte-for-byte.
- The ops repository remained read-only. The regenerated change exists only as
  `stage3-nz/ops-harness.patch` and in scratch copies.
- `d9069693e92ee338ab2c1d62461e73bcb02f195d` is the final implementation SHA
  pinned by the binary and harness. The later certificate/report commit does
  not alter implementation or harness execution.
