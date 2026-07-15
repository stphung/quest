The prestige progress gauge in the hero panel (the bar labelled
`Lv X/Y to <Tier> (P<rank>)`) renders shorter than it should at every
level. A brand-new Level 1 character shows a completely empty gauge, where
it used to show a small amount of fill (level 1 of 10 toward Bronze). The
regression is visible across all terminal size tiers that render the hero
panel.

The gauge is meant to show plain progress toward the next prestige tier's
required level: at Level 1 with a Level-10 requirement the bar should be
10% full, and it should reach 100% exactly when the requirement is met.

The UI snapshot tests (which compare full rendered frames against
committed reference snapshots) caught the regression — excerpt attached.
In the diffs, the lines starting with `-` are the expected (correct)
frames and `+` is what currently renders.

Fix the gauge so it fills to the documented ratio again.
