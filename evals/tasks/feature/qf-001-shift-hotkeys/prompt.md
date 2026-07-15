Quest's base-game hotkeys are letters ([P]restige, [H]aven, [S]oulforge,
[G] Stormglass, [D]eep, and so on). The UX rule for this screen is that
every letter hotkey must also accept its shifted variant — players with
Caps Lock on, or who habitually hold Shift, should get the same behavior
as the lowercase press.

Currently, pressing Shift+H with the Haven discovered does nothing, while
`h` opens the Haven overlay as expected. Every other overlay hotkey on the
base screen honors its uppercase variant; the Haven hotkey is the odd one
out.

Make the Haven hotkey accept the shifted variant like the rest, following
the same pattern the neighboring hotkeys use. The behavior must remain
gated on Haven discovery exactly as the lowercase path is.

The input replay test `uppercase_hotkey_variant_also_opens_overlay`
(headless harness driving the real input dispatcher) covers this behavior.
