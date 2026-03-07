# Loom Flow View Redesign: Living Machines

## Summary

Replace the current text-table Flow View with an animated factory floor rendered using `scene_fx` cell buffers. Six machine nodes in a 3x2 grid, each with unique animated textures, buffer bars, and recipe input slots. Port labels below each box show connections without drawn wires. A sidebar shows full detail for the selected node. Animations freeze on stall. Responsive fallbacks at smaller terminal sizes.

## Motivation

The current Flow View is a data table with paired rows showing node headers, buffer bars, and pipe connectors. It works but doesn't scale visually when pipes cross between non-paired nodes, and lacks the spatial "factory" feel of games like Factorio and Shapez. Players should be able to look at the Loom and see machines running, resources flowing, and bottlenecks at a glance.

## Overall Layout

```
+======================================+====================+
|                                      |                    |
|          Factory Floor               |      Sidebar       |
|        (3x2 machine grid)           |   (selected node   |
|                                      |    detail)         |
|                                      |                    |
+======================================|                    |
|        Pattern Bar (3 rows)          |                    |
+======================================+====================+
```

- **Factory floor** (~60% width): 3 rows x 2 columns of animated machine nodes. Arrow keys move selection. Primary visual area.
- **Sidebar** (~20 cols fixed right): Detail for selected node -- level, buffer, production rate, all pipes, all possible recipes with input slot indicators.
- **Pattern bar** (3 rows, bottom-left): Active pattern progress (unchanged from current).

Node grid positions are fixed by archetype pair:

| Row | Left | Right |
|-----|------|-------|
| 0 | Ember Spindle | Void Condenser |
| 1 | Reflection Lens | Memory Archive |
| 2 | Silence Well | Resonance Forge |

## Node Box Anatomy

Each node is roughly 22 cols x 6 rows:

```
+----- Ember Spindle --- Lv.5 ----+
| ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ |    <- animated texture (2 rows)
| ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ |
| ########.............. 18/40    |    <- buffer bar
| [*Emb] [*Void] > ForgedLt      |    <- recipe slots
+---------------------------------+
 ->V ->R ->M    <-F                    <- port labels (outside box)
```

### Title Bar

Node name and level. Selected node gets a bright border, unselected gets a dim border.

### Animated Texture (2 rows)

Unique per node type, cycles every ~300ms using `scene_fx::current_millis()`. Freezes and dims when stalled. Locked nodes show a lock icon instead.

| Node | Texture | Feel |
|------|---------|------|
| Ember Spindle | `~ ~ ~` shifting wave | Flickering heat |
| Reflection Lens | `. * . *` twinkling | Light refracting |
| Void Condenser | `: : : :` dripping | Dark matter condensing |
| Memory Archive | `x x x` crosshatch | Woven patterns |
| Silence Well | `_ _ _ _` still ripple | Calm surface |
| Resonance Forge | `~ ~ ~` vibrating | Sound waves |

Animation is a column offset shift computed at render time: `offset = (current_millis() / 300) % pattern_length`. Each node type uses a different shift direction (horizontal for Ember, vertical for Resonance, brightness pulse for Void).

### Buffer Bar

`#` filled / `.` empty. Colored green (< 75%), yellow (75-90%), red (> 90% or stalled). Shows `current/capacity` numerically after the bar.

### Recipe Slots

Shows the best active or candidate recipe for this node's nature. Two input indicators followed by an arrow and output name:

- `[*Emb]` -- filled, this resource is arriving via pipe (bright)
- `[oSlnc]` -- empty, not connected (dim red)
- `> ForgedLt` -- output name (bright when producing, dim when missing input)

When producing, the `>` arrow pulses on a 500ms cycle.

When no recipes are discovered, shows `? + ? > ???` dimmed.

### Port Labels

Sit outside the node box, below it. Use single-letter color-coded abbreviations:

- **E** = Ember Spindle (orange)
- **V** = Void Condenser (purple)
- **R** = Reflection Lens (cyan)
- **M** = Memory Archive (yellow)
- **S** = Silence Well (gray)
- **F** = Resonance Forge (blue)

Format: outgoing on left (`->V ->R ->M`), incoming on right (`<-F`), separated by gap.

Under-construction pipes blink on a 500ms cycle.

Max port labels per node: `->X ->X ->X  <-X <-X <-X` (~20 chars), fits under the box.

## Selection and Connection Highlighting

When a node is selected:
1. Its border brightens (dim `+--+` to bright `+==+`)
2. Its port labels brighten
3. Matching port labels on connected nodes also brighten (e.g., selecting Ember makes `<-E` on Void brighten)

This lets players visually trace connections without drawn wires.

## Sidebar Detail Panel

~20 columns wide. Updates when selection changes. Content top to bottom:

1. **Node identity**: Name, level, nature type (Heat/Void/Form/etc)
2. **Buffer + rate**: Bar and exact numbers with more room than the node box
3. **Recipe list**: All recipes matching this node's nature from the recipe registry. Each shows two input slots as filled/empty with output name. Active recipes bright, inactive dim. This is the key planning tool.
4. **Pipe list**: All outgoing and incoming pipes with flow rate and tier
5. **Controls**: Context-sensitive key hints ([B]uild, [U]pgrade, [D]emolish, [S]plit)

## Animation System

All animations are computed at render time from `scene_fx::current_millis()`. No new state or tick counters needed.

- **Texture cycling**: `frame = (current_millis() / 300) % frame_count`
- **Production pulse**: `bright = (current_millis() / 500) % 2 == 0`
- **Construction blink**: `visible = (current_millis() / 500) % 2 == 0`
- **Stall**: Freeze at frame 0, dim to dark gray
- **Locked**: No texture, lock icon, no animation

Performance: purely cosmetic render-time computation. The existing 100ms tick loop already redraws every frame, giving ~10fps animation.

## Rendering Approach

Switch from `Paragraph` with `Line`/`Span` (row-based text) to `scene_fx` cell buffer rendering (`SceneCell`, `put_text`, `put_cell`, `render_buffer`). This gives per-character control needed for:
- Placing node boxes at exact grid coordinates
- Animating texture patterns with color per cell
- Drawing buffer bars with per-cell coloring
- Highlighting ports across distant nodes

The sidebar can remain `Paragraph`-based since it's standard text.

## Responsive Sizing

| Tier | Min Size | Behavior |
|------|----------|----------|
| L/XL | 80x30+ | Full layout: factory floor + sidebar + pattern bar |
| M | 60x24+ | Drop sidebar, show 2-line detail strip at bottom instead of pattern bar |
| S | 40x16+ | Fall back to existing text-based List+Detail view |

## Edge Cases

- **No archetype**: Archetype selection screen (unchanged)
- **One node unlocked**: Only that node renders as a machine. Others show as dim locked boxes.
- **No pipes**: No port labels. Clean factory floor with isolated machines.
- **No recipes discovered**: Recipe slots show `? + ? > ???` dimmed. Sidebar says "No recipes discovered."
- **Stalled node**: Texture freezes, dims. Buffer bar turns red. Immediately obvious.

## Relationship to Other Views

The Flow View becomes the default/primary Loom view. List+Detail and Codex remain as separate Tab-accessible views, unchanged. They serve as supplementary data views for detailed pipe management and recipe browsing.
