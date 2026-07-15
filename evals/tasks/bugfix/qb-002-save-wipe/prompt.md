A player upgraded Quest from a version released before Act 2 shipped and
their high-prestige character was silently replaced by a fresh Level 1
character. Nothing crashed; the character just came up empty.

Context on how loading works in this codebase: the account/character
loaders fall back to a default state when JSON parsing fails, so a
deserialization error never surfaces as a crash — it silently wipes the
player's progress. The committed save corpus (`tests/fixtures/saves/v1/`)
exists precisely to catch this: those files are real saves written by
older versions of the game, frozen forever, and a corpus test failure
means existing player saves no longer load.

QA bisected the regression to a recent "attribute cleanup" in the
character persistence code. Old save files — which predate several fields
that newer versions write — no longer deserialize.

Fix the persistence code so that saves written by pre-Act 2 versions load
correctly again. Per this repo's conventions, save-format compatibility is
fixed in the type definitions (defaults/aliases/migrations), never by
editing the frozen corpus files.
