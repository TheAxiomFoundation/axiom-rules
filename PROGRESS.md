# PROGRESS — relation slot entities follow-up (issue #137)

Branch: `fix/relation-slot-entities`, extending the four existing #137 commits
without rewriting them. Network access is prohibited.

## State

Complete. Amendments A, B, and C, all repository gates, the final corpus and
26/32 reruns, and independent review are complete. The completed report is in
`out.md`; no push was performed.

## Done

- Confirmed the verified gate findings in the prior report:
  - closure-membership errors reject five legitimate published modules;
  - the hardcoded legacy relation alias is content-specific;
  - declaration-based binding diagnostics point consumers toward the
    zero-producing order for `us:statutes/26/32`.
- Confirmed the active branch and preserved all four existing commits.
- Added `CompileOptions::strict_relation_entities`.
- Ratcheted relation argument validation:
  - arity mismatch remains a hard error in every mode;
  - shape-invalid labels warn by default, leave `slot_entities` empty, and
    error in strict mode;
  - well-shaped closure-unknown kinds warn by default, remain verbatim in the
    artifact, and error in strict mode.
- Deleted the `member_of_individuals_household` content-specific alias.
- Routed RuleSpec diagnostics into artifact compile diagnostics without
  serializing them, preserving canonical module paths.
- Documented compile and binding strictness separately.
- Red evidence: the focused test build failed because
  `CompileOptions::strict_relation_entities` did not exist.
- Green evidence: 82 RuleSpec tests, 12 module-source tests, and five
  namespace-ratchet unit tests pass.
- Added deterministic program-level relation usage analysis shared by compile
  validation and dataset binding:
  - aggregate owner entity constrains `current_slot`;
  - predicate/value entity constrains `related_slot`;
  - membership and derived-relation source uses propagate structural kinds;
  - versioned rules inspect executable versions rather than phantom base
    semantics;
  - conflicting uses leave a slot unresolved instead of choosing arbitrarily.
- Added `warning[relation_orientation_mismatch]`, strict compile promotion,
  artifact reload recomputation, and source-fidelity preservation.
- Binding now uses executable orientation for used relations and declaration
  order only for unused relations.
- Red evidence: the 26/32-shaped fixture had no compile warning, warned on the
  working tuple, and did not warn on the empty-lookup tuple; the membership
  fixture also had no orientation warning.
- Green evidence: all three orientation tests pass, including strict/reload
  behavior and derived-relation membership. Full `cargo test` passes.
- Re-ran the staged default-mode corpus sweep:
  - 40/46 modules compile, with the same six pre-existing failures as the
    baseline;
  - successful compiles emit 39 warnings across 26 invocations: 32
    `relation_orientation_mismatch`, five
    `unknown_relation_argument_entity`, one
    `invalid_relation_argument_entity_shape`, and one pre-existing
    `non_exhaustive_match`;
  - the five closure-authority gaps compile successfully by default.
- Completed the orientation census:
  - the required 42-relation US cohort has 19 usage-consistent, 22
    usage-inconsistent, and one unused declaration;
  - the literal five-root total includes eight additional UK declarations and
    is 21 consistent, 28 inconsistent, and one unused across 50 declarations;
  - NZ, BE, and GH contain no declared-argument relations.
- Re-ran the amended 26/32 dynamic fixture:
  - compilation preserves declared slots `[TaxUnit, Person]` and emits the
    orientation warning for executable order `[Person, TaxUnit]`;
  - the executable-order tuple returns count 1 with no bind warning;
  - the declaration-order tuple returns count 0 with two bind warnings, one
    for each reversed slot.
- Independent evidence review reproduced the corpus counts, census, worklist,
  and 26/32 transcript without a gate-blocking finding.
- Independent code review found that a nested aggregate's predicate/value
  entity could contaminate its enclosing relation's orientation.
- Added a nested `sum_related` regression covering both compile and bind
  diagnostics. Red evidence derived `[Payment, TaxUnit]` for the outer
  `[Person, TaxUnit]` relation and `[Payment, Payment]` for the inner
  `[Payment, Person]` relation. Treating nested aggregate internals as a
  separate execution context makes both executable tuple orders warning-free.
- Focused re-review found that an untyped outer relation still suppressed the
  nested scalar traversal. A second red regression observed no executable-use
  diagnostic for reversed inner declaration `[Person, Payment]`. Relation
  traversal now carries an optional current kind, retaining the independently
  derived inner `[Payment, Person]` orientation; compile warns on the reversed
  declaration and binding leaves the working tuple warning-free.
- Final post-review gates:
  - `cargo test`: 256 passed, zero failed;
  - schema feature tests: 12 passed, zero failed;
  - `cargo fmt --all --check`: exit 0;
  - `cargo check --target wasm32-unknown-unknown --no-default-features`:
    exit 0;
  - `cargo clippy --all-targets --all-features`: exit 0;
  - `git diff --check origin/main...HEAD`: exit 0.
- Final post-fix corpus rerun remains 40/46 with 39 warning emissions across
  26 successful invocations and the same six baseline failures.
- Final fresh 26/32 rerun remains count 1 with zero bind warnings for
  `[Person, TaxUnit]`, and count 0 with two bind warnings for
  `[TaxUnit, Person]`.
- Final focused independent review reports no remaining actionable finding.
- Wrote the completed report, including all census worklists and numbers, to
  `out.md`.

## Next

No implementation work remains. Deliver `out.md`; leave the branch unpushed.
