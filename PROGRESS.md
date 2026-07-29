# PROGRESS — relation slot entities follow-up (issue #137)

Branch: `fix/relation-slot-entities`, extending the four existing #137 commits
without rewriting them. Network access is prohibited.

## State

Amendment A is implemented and focused tests are green. Work is moving to the
usage-derived orientation tests and implementation.

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

## Next

1. Add failing tests for usage-derived orientation and binding behavior; then
   implement Amendment B.
2. Run the five-corpus sweep, report the orientation census and every
   inconsistent relation, and reproduce the amended 26/32 transcript.
3. Run all repository gates, request an independent review, fix actionable
   findings, and write the completed report to `out.md`.
