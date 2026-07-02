# Save-format compatibility corpus

Save files committed in the format the game wrote when each generation was
created. `tests/save_compat_tests.rs` loads every file through the game's
real deserialization paths; a failure there means a change broke loading of
existing player saves.

Rules:

- **Committed files are frozen.** Never edit or regenerate them to make a
  failing test pass — fix the compatibility break instead
  (`#[serde(default)]`, `#[serde(alias = "...")]`, or a migration).
- After an intentional, migration-backed format change, create a NEW
  generation directory (`v2/`, `v3/`, …) with the generator
  (`cargo test --test save_compat_tests regenerate_save_corpus -- --ignored`
  pointed at the new directory) and keep all older generations loading.
- `v1/loom_legacy_v1.json` is hand-written (pre-version-2 loom save) and is
  never regenerated — it pins the loader's version-reset path.

| Generation | Created | Notes |
|------------|---------|-------|
| `v1/` | 2026-07 | Initial corpus: 4 character saves + deep/haven/loom/enhancement/achievements |
