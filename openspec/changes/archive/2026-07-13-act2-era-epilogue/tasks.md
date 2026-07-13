# Tasks — Act 2 Era Epilogue

## 1. Colony state + playback

- [x] 1.1 Add `era_end_shown: bool` to `ColonyState` (`serde(default)`), and `take_era_end_playback() -> Option<ScenePlayback>` gated on `era_over() && !era_end_shown` (mirrors `take_finale_playback`)
- [x] 1.2 Author the epilogue content in `scenes.rs`: fixed opening/closing beats + conditioned account builder (delivered, taken = `INITIAL_SOULS − souls_delivered`, crossings, days at sea from `CrossingRecords`, districts standing), closing on the Verity and the door ajar; payout note "the era is over"
- [x] 1.3 Unit/integration tests: plays once then never again; `era_end_shown` round-trips `colony.json` (and old saves missing the field load with it false); account text matches a known colony's numbers; does not fire while `era_over()` is false

## 2. main.rs wiring

- [x] 2.1 Delivery branch (`era_over()` case): keep setting `last_crossing_complete`, drop the one-line "The last crossing" modal push
- [x] 2.2 Clear-screen drain: when `scene_play`/`scene_modal` are empty and `v.arrived()`, take the era-end playback from the colony and play it via `ScenePlay`; save colony (+ character) after the take so the one-shot flag persists
- [x] 2.3 Verify ordering by hand (`QUEST_ACT2=1` + `QUEST_VOYAGE_TIME_SCALE` dev clock or fixture): finale → landfall/district/milestone moments → epilogue; and the quit-at-era-end reload case plays it on next boot

## 3. Post-era rendering

- [x] 3.1 `render_dock`: early `era_over()` branch — authored quiet-harbor lines, no charge bar, no jump preview, no `[J]` hint
- [x] 3.2 `render_record`: era-account block above the crossing log when `era_over()`
- [x] 3.3 `fixtures::colony_era_complete()` (souls_remaining 0, six districts, populated records, `era_end_shown` true) + overlay snapshots for the era-over Dock and Record; review and bless
- [x] 3.4 Check the Reckoning view post-era for the same misleading-affordance problem the Dock had (yard buys still work by spec — but confirm nothing renders as a next-crossing promise); fix copy if needed

## 4. Spec + docs

- [x] 4.1 `src/vessel/CLAUDE.md`: update the `last_crossing_complete` field note and colony.rs row (epilogue, quiet dock, era summary); root CLAUDE.md untouched (no constants moved)
- [x] 4.2 Dossier: short refresh-history entry (the dead-end is closed; held-for-later title-screen question unchanged)
- [x] 4.3 Archive the change (folds the MODIFIED requirement); `openspec validate --specs`

## 5. Verification

- [x] 5.1 Targeted: `cargo test --release --test ferryman_tests` + colony unit tests + `cargo test overlay_snapshot` + `QUEST_ACT2=1 cargo test flag_on`
- [x] 5.2 `make check`
- [x] 5.3 Push; new draft PR (the prior branch PR is merged — this is a fresh PR from the restarted branch)
