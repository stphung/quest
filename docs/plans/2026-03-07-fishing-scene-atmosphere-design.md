# Fishing Scene Atmospheric Enhancement

Stylized/atmospheric improvements to the fishing scene's mountains, sky, and horizon transition. Enhances mood and depth without adding visual clutter.

## Current State

The sky half of `fishing_scene.rs` (`SkyShape::draw()`) renders:
- Sky gradient (blue/dusk shifting)
- Celestial body (sun/moon pixel circle)
- Stars (twinkling at dusk)
- Clouds (5 small drifting pixel clusters)
- Shoreline silhouettes (2 sine-wave layers in flat colors)
- Sailboat (detailed pixel art)

The shoreline "mountains" are smooth sine-wave humps in two flat colors with no internal texture. The sky gradient is clean but uniform. The transition between sky/land/water lacks atmospheric depth.

## Design

### 1. Mountain Silhouettes

Replace the 2-layer sine-wave shoreline (lines 294-320) with a 3-layer mountain system.

**Shape:** Each layer uses 2-3 summed sine waves at different frequencies for jagged-yet-smooth ridgelines. Far layer has gentler, taller peaks; near layer has sharper, shorter ones. `hash2d`-based per-column jitter adds irregularity.

**Color depth:**
- **Far range** -- most faded, blue-grey (aerial perspective). Minimal internal color variation.
- **Mid range** -- moderate saturation, slight vertical gradient per column (darker base, lighter peak).
- **Near range** -- most saturated dark green-grey, strongest vertical gradient. Top 1-2 pixels get tree-fringe texture via `hash2d` alternating lighter/darker pixels.

**Dusk:** All layers shift with the existing `dusk` parameter. Static shapes, no drift animation.

### 2. Sky Atmosphere

Two new elements painted between gradient and celestial body.

**Haze bands** -- 2-3 thin bands (1-2px tall) in the lower third of the sky. Slightly warmer/lighter tint blended at ~15-20% opacity over gradient. Drift at ~0.5x cloud speed. Pick up warm tones during dusk.

**High-altitude wisps** -- 3-4 wispy streaks in the upper sky half. Short pixel runs (3-6 wide) at slight diagonal. Barely visible -- few percent brighter than gradient. Drift at ~0.3x cloud speed. Thinner/more translucent than clouds to maintain visual hierarchy.

### 3. Horizon Transition

**Mist layer** -- Semi-transparent fog at mountain bases, 2-4px tall, overlapping near mountain bottom and water top. Rendered as patches via slow sine-wave pattern (not solid). Drifts at ~0.6x cloud speed. Blends near-mountain base color and sky horizon color at ~25-30% opacity.

**Light bleed** -- When celestial body is behind/near a mountain silhouette, adjacent mountain edge pixels get subtle bright halo (~20% blend toward orb color). Gold for sun, cool white for moon.

**Water side unchanged** -- Existing horizon foam line stays as-is.

### Rendering Order

Updated `SkyShape::draw()` order:

1. Sky gradient (existing)
2. Horizon color bleed (existing)
3. Haze bands (new)
4. High-altitude wisps (new)
5. Celestial body (existing)
6. Stars (existing)
7. Clouds (existing)
8. Mountain silhouettes -- 3 layers far-to-near (replaces shoreline)
9. Mountain light bleed (new)
10. Horizon mist (new)
11. Sailboat (existing)

## Scope

All changes are within `SkyShape` in `src/ui/fishing_scene.rs`. No new files, no logic changes, no new dependencies. The `scene_fx` helpers (`hash2d`, `lerp_rgb`, `lerp_channel`) already provide everything needed.
