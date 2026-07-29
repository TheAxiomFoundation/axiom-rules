# PROGRESS — relation slot entities follow-up (issue #137)

Branch: `fix/relation-slot-entities`, extending the four existing #137 commits
without rewriting them. Network access is prohibited.

## State

Amendments A and B are implemented. The full default-feature `cargo test`
suite is green. Work is moving to the staged-corpus sweep and 26/32 dynamic
acceptance transcript.

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

## Next

1. Run the five-corpus sweep, report the orientation census and every
   inconsistent relation, and reproduce the amended 26/32 transcript.
2. Run all repository gates, request an independent review, fix actionable
   findings, and write the completed report to `out.md`.
