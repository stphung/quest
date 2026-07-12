# Tasks — Act 2 Era Pacing: 3-Month Balanced Campaign

## 1. Constants

- [x] 1.1 `CAP_GROWTH` 1.36 → 1.46 and `DARK_TAKES_PER_DAY` 0.0006 → 0.0007 in `src/vessel/colony.rs`, with doc comments updated to say why each value is what it is
- [x] 1.2 Update the constants' doc comments to reference the tuned target (~3-month balanced era) and the gates that enforce it

## 2. Gates

- [x] 2.1 Tighten `strategy_sweep_holds_the_campaign_envelope`: balanced 15–30 crossings, 2.5–4.5 months, ≥84% saved; refresh the measured-values comment block (2026-07-12 post-retune numbers)
- [x] 2.2 Follow the era-window assertion in `dock_time_across_charge_policies` to 2.5–4.5 months; refresh its measured comment
- [x] 2.3 Full ferryman + voyage suites green: `cargo test --release --test ferryman_tests --test voyage_tests`

## 3. Fallout review

- [x] 3.1 Run `cargo test overlay_snapshot` and `cargo test snapshot`; review any Reckoning/Dock diffs (expedition sizes derived from `CAP_GROWTH`) and re-bless only the expected numeric changes
- [x] 3.2 `cargo run --release --bin voyage_simulator` (voyage layer untouched — must stay green unchanged)

## 4. Docs and spec

- [x] 4.1 `src/vessel/CLAUDE.md`: constants table rows (`CAP_GROWTH`, `DARK_TAKES_PER_DAY`) and the speed-vs-salvation paragraph's measured figures
- [x] 4.2 `docs/dossiers/act2-pilgrimage.md`: refresh-history entry for the retune; update figures stated as current
- [x] 4.3 Archive the change (folds the MODIFIED envelope requirement into `openspec/specs/vessel-act2/spec.md`); `openspec validate --specs`

## 5. Verification

- [ ] 5.1 `make check`
- [ ] 5.2 Push and update PR #731's description to declare the balance change (it previously said "no gameplay changes")
