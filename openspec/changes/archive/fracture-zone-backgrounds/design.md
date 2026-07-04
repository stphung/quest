> Backported design record. Sources: docs/plans/2026-02-28-fracture-zone-backgrounds-design.md.

## 2026-02-28-fracture-zone-backgrounds-design.md

# Fracture Zone Backgrounds Design (Zones 12-30)

## Problem

All 19 fracture zones (12-30) currently fall through to `config_fallback()` in `zone_bg.rs`, rendering the same generic grey background. Each zone has rich narrative identity that should be reflected in its combat scene background.

## Approach

Per-zone config functions following the existing pattern in `zone_bg.rs`. Each zone gets a unique `ZoneSceneConfig` with sky gradient, celestial type, terrain profiles, ground detail, weather, and overlay. New FX types are added where existing ones don't capture the fracture zone themes.

## Design Principles

- **Chapter cohesion**: Zones within a chapter share a base palette with progressive intensity
- **Cosmic horror arc**: Chapters 1-2 feel physical/elemental, 3-4 shift to abstract emptiness, 5-6 break reality itself
- **Progressive emptiness**: Terrain, ground detail, and weather thin out as zones deepen, culminating in Z29 (near-empty silence) and Z30 (the wound)

## New FX Types

### Weather Types (6 new)
| Type | Visual | Chapters |
|------|--------|----------|
| `AshRain` | Dense downward grey/orange ash particles | Ch.1 |
| `GlassShards` | Reflective falling fragments, alternating bright/dim | Ch.2 |
| `DriftingAsh` | Slow sparse dark ash floating laterally | Ch.3 |
| `DustMotes` | Faint amber particles suspended in stillness | Ch.4 |
| `StaticNoise` | Random flickering characters (visual snow) | Ch.5 |
| `FractureMotes` | Dim pulsing particles that appear/vanish | Ch.6 |

### Celestial Types (3 new)
| Type | Visual | Chapters |
|------|--------|----------|
| `CrackedSky` | Bright fracture lines across upper sky | Ch.1-2 |
| `VoidRift` | Dark tear in sky with faint purple edge glow | Ch.3-4 |
| `Flicker` | Stars/points that blink in/out of existence | Ch.5-6 |

### Overlay Functions (6 new)
| Overlay | Effect | Chapters |
|---------|--------|----------|
| `overlay_heat_distortion` | Subtle horizontal waver of bg colors in lower half | Ch.1 |
| `overlay_mirror_flash` | Brief reflective bright streaks | Ch.2 |
| `overlay_consuming_dark` | Bottom-up darkness creep that pulses | Ch.3 |
| `overlay_hollow_echo` | Periodic desaturation wave sweeping across scene | Ch.4 |
| `overlay_reality_tear` | Horizontal bands of color inversion flickering | Ch.5 |
| `overlay_wound_pulse` | Slow full-scene brightness oscillation near-black to dim | Ch.6 |

## Per-Zone Configs

### Ch.1: The Red Fault (Z12-14) — Physical, volcanic, shattered earth

**Z12 Splintered Rim**: Smoky orange-brown sky, CrackedSky celestial, jagged `▲` ridgeline in rust-red, `▓` dark volcanic near terrain (filled), rubble ground (`=` `:`), AshRain (light), heat_distortion overlay.

**Z13 Ember Ravine**: Deep red-orange sky fading to near-black, EmberGlow celestial, low `~` heat-haze far terrain in orange, `█` black basalt near terrain (filled), lava channel ground (`=` `~`), Embers (heavy), lava_glow overlay.

**Z14 Heart of the Fault**: Bright crimson sky to dark blood red, EmberGlow celestial, `╱` fracture lines in glowing red, `▓` charred stone near terrain (filled), cinder ground (`:` `.`), AshRain (heavy), heat_distortion overlay.

### Ch.2: The Mirror Scar (Z15-17) — Crystallized light, reflections, prismatic

**Z15 Shard Fields**: Cold silver-blue sky to pale grey, CrackedSky celestial, `◆` crystalline peaks in icy blue, `│` thin crystal pillars (sparse, unfilled), glass fragment ground (`*` `.`), GlassShards (light), mirror_flash overlay.

**Z16 Refraction Steps**: White-blue sky to deep indigo, BioLuminescent celestial, `▽` inverted terrain in prismatic tones, `░` translucent fill in blue-white, refractive points ground (`:` `*`), GlassShards (medium), crystal_shimmer overlay.

**Z17 Hall of Second Suns**: Bright white sky to blinding teal, CrackedSky celestial, `═` horizontal light beams, `▒` frosted glass terrain, points of light ground (`*` `·`), Sparkles (heavy), mirror_flash overlay.

### Ch.3: The Black Mouth (Z18-20) — Consuming darkness, ash, hungering void

**Z18 Ashen Verge**: Dark grey sky to charcoal, Overcast celestial, `~` rolling ash dunes in grey, `▓` dark stone near terrain (filled), fine ash ground (`.` `;`), DriftingAsh (light), consuming_dark overlay.

**Z19 Throat of the World**: Near-black sky to pure black, None celestial, `▼` downward stalactites in dark tones, `█` solid darkness near terrain (filled), no ground detail, DriftingAsh (heavy), consuming_dark overlay.

**Z20 The Black Mouth**: Deep purple-black sky to black, VoidRift celestial, `╲` angled void edges, `░` barely visible near terrain, no ground detail, VoidParticles weather, consuming_dark overlay.

### Ch.4: The Hollow Throne (Z21-23) — Ancient emptiness, fossilized grandeur

**Z21 Sunken Processional**: Amber-grey sky to dark stone, Wisps celestial, `║` tall pillars in amber, `▒` carved stone near terrain (filled), floor tile ground (`:` `.`), DustMotes (light), hollow_echo overlay.

**Z22 The Pale Archive**: Pale bone-white sky to dusty grey, None celestial, `│` library shelves in pale tones, `░` faded fill in bone color, crystal tablet ground (`·` `;`), DustMotes (medium), hollow_echo overlay.

**Z23 The Hollow Throne**: Cold grey sky to deep void-black, VoidRift celestial, `╫` ornate throne structure, `▓` obsidian floor near terrain (filled), no ground detail, DustMotes (sparse), hollow_echo overlay.

### Ch.5: The Wailing Reach (Z24-26) — Reality dissolving, existential horror

**Z24 The Stillborn Sea**: Flat grey sky (no gradient — lifeless) to slightly darker grey, None celestial, `~` flat waterline in grey (no wave motion — still water), `▒` grey seabed near terrain (filled), dead shore ground (`~` `.`), None weather (absolute stillness), hollow_echo overlay.

**Z25 Resonance Fault**: Deep teal sky to vibrating purple, Flicker celestial, `│` vibrating crystal pillars, `░` humming ground in teal, resonance point ground (`*` `:`), StaticNoise (light), reality_tear overlay.

**Z26 The Wailing Reach**: Flickering between dark and light sky, Flicker celestial, `╱`/`╲` alternating broken lines, `░` unstable near terrain, no ground detail, StaticNoise (heavy), reality_tear overlay.

### Ch.6: The Origin Wound (Z27-30) — Primordial, cosmic, the first break

**Z27 The Scar Root**: Dark rust sky to near-black, Stars (sparse) celestial, `╱` root tendrils in rust-red, `▓` petrified fracture near terrain (dark, filled), root vein ground (`:` `.`), FractureMotes (light), wound_pulse overlay.

**Z28 Echoing Abyss**: Pure black sky (uniform void), Flicker celestial, no far terrain (vast emptiness), `░` faint distant edges near terrain, no ground detail, FractureMotes (sparse), wound_pulse overlay.

**Z29 Threshold of Silence**: Near-white sky fading to black (light dying), None celestial, `─` single fading horizon line, no near terrain, no ground detail, None weather (silence), wound_pulse overlay.

**Z30 The Origin Wound**: Deep void-purple sky to absolute black, Flicker (rare) celestial, `╳` fracture cross pattern in dim purple, `░` barely visible near terrain, no ground detail, FractureMotes (faint), wound_pulse overlay.

## Implementation Scope

### Files Modified
- `src/ui/zone_bg.rs` — Main implementation: 19 new config functions, expanded match, 6 new weather paint functions, 3 new celestial paint functions, 6 new overlay functions

### Files Not Modified
- `src/ui/scene_fx.rs` — No changes needed (shared utilities already sufficient)
- `src/zones/` — No changes needed (zone data already defined)

### Estimated Additions
- ~15 new enum variants (6 weather + 3 celestial)
- ~19 new config functions (~30-40 lines each)
- ~15 new rendering functions (weather/celestial/overlay, ~20-40 lines each)
- ~1 expanded match arm block
- Total: ~1,000-1,200 new lines in zone_bg.rs

## Testing

- Visual verification via `cargo run -- --debug` (debug menu can travel to any zone)
- Ensure `cargo clippy` and `cargo test` pass (no logic changes, pure rendering)
- Verify no performance regression with the new rendering functions
