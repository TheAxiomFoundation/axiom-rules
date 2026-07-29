# PROGRESS — relation slot entities follow-up (issue #137)

Branch: `fix/relation-slot-entities`, extending the four existing #137 commits
without rewriting them. Network access is prohibited.

## State

Amendment work has started from `b30ea2a`. The prior report in `out.md` has
been read, and the checkout is clean apart from that intentionally untracked
report. The branch is four commits ahead of local `origin/main`.

## Done

- Confirmed the verified gate findings in the prior report:
  - closure-membership errors reject five legitimate published modules;
  - the hardcoded legacy relation alias is content-specific;
  - declaration-based binding diagnostics point consumers toward the
    zero-producing order for `us:statutes/26/32`.
- Confirmed the active branch and preserved all four existing commits.

## Next

1. Add failing tests for default warnings, strict compile errors, legacy labels,
   and the five closure-kind cases; then implement Amendment A.
2. Add failing tests for usage-derived orientation and binding behavior; then
   implement Amendment B.
3. Run the five-corpus sweep, report the orientation census and every
   inconsistent relation, and reproduce the amended 26/32 transcript.
4. Run all repository gates, request an independent review, fix actionable
   findings, and write the completed report to `out.md`.
