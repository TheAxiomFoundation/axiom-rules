# Progress — released binary CLI documentation

Branch: `docs/released-binary-cli-contract` (from `origin/main`).

## State

The released v0.1.1 and current-`main` CLI contracts are separated throughout
the command guidance found by the documentation audit. Verification remains.

## Done

- Confirmed the starting worktree was clean and contained no user changes.
- Confirmed `docs/install.md` names release `v0.1.0` on lines 13 and 25, then
  gives an unlabeled `compile-composed --rulespec-root` example on lines 40–43.
- Confirmed `README.md` gives the source-built `compile --rulespec-root`
  contract on lines 60–63 without explicitly distinguishing it from the
  released binary linked immediately above.
- Confirmed `docs/jurisdiction-repos.md` gives
  `compile --rulespec-root` on lines 52–55 without identifying it as the
  source-built `main` contract.
- Confirmed the requested scope is documentation only.
- Updated `docs/install.md` to download and attest release `v0.1.1`.
- Documented the known macOS `shasum --check` trailing-blank-line warning and
  the successful `OK` result.
- Added the verified v0.1.1 `compile` and `run-compiled` sequence, with no root
  flag, and separately retained the source-built `main` examples for `compile`
  and `compile-composed` with repeatable `--rulespec-root`.
- Separated the v0.1.1 and current-`main` CLI contracts in `README.md`, keeping
  the source-built `compile` and `compile-composed` documentation.
- Labeled the release and current-`main` import-resolution contracts in
  `docs/jurisdiction-repos.md`; the explicit root example now uses `cargo run`.
- Labeled the current-`main` root and `compile-composed` requirements in
  `docs/rulespec.md`, and distinguished the released v0.1.1 contract.

## Next

- Verify line numbers, links, wording, and the final documentation-only diff.
- Record the final report in the requested output file.
