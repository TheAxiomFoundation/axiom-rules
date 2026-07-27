# Progress — released binary CLI documentation

Branch: `docs/released-binary-cli-contract` (from `origin/main`).

## State

The released-binary install path is corrected and ready for verification. The
README and jurisdiction documentation still need their contract labels.

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

## Next

- Separate and label the two CLI contracts in `README.md`.
- Label the source-built contract in `docs/jurisdiction-repos.md`.
- Verify line numbers, links, wording, and the final documentation-only diff.
- Record the final report in the requested output file.
