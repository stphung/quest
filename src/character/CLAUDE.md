# Character System

Character attributes, derived stats, prestige progression, and multi-character persistence.

## Module Structure

```
src/character/
├── mod.rs          # Public re-exports
├── attributes.rs   # 6 RPG attributes, modifiers, cap enforcement
├── derived_stats.rs # Combat stats calculated from attributes
├── prestige.rs     # Prestige tiers, multipliers, tier progression
├── manager.rs      # Character CRUD, JSON persistence in ~/.quest/
└── input.rs        # Character select/create/delete/rename input handling
```

## Key Types

### `Attributes` (`attributes.rs`)
Six core RPG attributes stored as `u32` values:
- **STR** (Strength): Physical damage (+2 per modifier)
- **DEX** (Dexterity): Defense and crit chance (+1% crit per modifier)
- **CON** (Constitution): Maximum HP (+10 per modifier)
- **INT** (Intelligence): Magic damage (+2 per modifier)
- **WIS** (Wisdom): XP gain (+5% per modifier)
- **CHA** (Charisma): Prestige multiplier bonus (+10% per modifier)

**Modifier formula**: `(value - 10) / 2` (integer division, minimum 0)

**Attribute caps**: `20 + (5 × prestige_rank)`. Enforced in `attributes.rs`.

### `DerivedStats` (`derived_stats.rs`)
Combat stats calculated from attributes. Recalculated whenever attributes change:
- Max HP, damage (physical + magic), defense, crit chance, crit multiplier
- XP multiplier (from WIS), prestige multiplier (from CHA)

### `PrestigeTier` (`prestige.rs`)
Named tiers from Bronze through Eternal with diminishing-returns XP multipliers.

**Formula**: `multiplier = 1.0 + 0.5 × rank^0.7`
- P0: 1.0x, P1 (Bronze): 1.5x, P5: ~2.7x, P10: ~3.5x, P20: ~5.1x, P100: ~13.3x

## Character Persistence (`manager.rs`)

Characters are saved as individual JSON files in `~/.quest/`:
- File pattern: `~/.quest/{character_name}.json`
- Auto-save every 30 seconds (driven by `main.rs` timer)
- Name validation: 1-20 chars, alphanumeric + spaces, no leading/trailing spaces

### Character CRUD Operations
- `create_character(name)` — Creates new character with base attributes, validates name uniqueness
- `load_character(name)` — Loads from JSON file
- `save_character(state)` — Serializes GameState to JSON
- `delete_character(name)` — Removes JSON file
- `rename_character(old, new)` — Renames file, updates internal state
- `list_characters()` — Lists all `.json` files in `~/.quest/`

## Leveling System

On level-up (handled in `core/game_logic.rs`):
1. +3 random attribute points distributed among STR, DEX, CON, INT, WIS, CHA
2. Points respect attribute caps (base 20 + 5 per prestige rank)
3. Derived stats are recalculated

XP curve: `100 * level^1.5` (XP needed for next level)

## Prestige Flow

1. Player must meet level threshold (`can_prestige()` in `prestige.rs`)
2. Confirmation dialog shown (`ui/prestige_confirm.rs`)
3. `perform_prestige()` resets: level → 1, XP → 0, zone → first, attributes → base
4. Preserves: prestige_rank (incremented), equipment, achievements, haven
5. New attribute cap = 20 + (5 * new_prestige_rank)

## Input Handling (`input.rs`)

Character management screens use a state machine:
- `CharacterSelectState` — List, preview, navigate
- `CharacterCreationState` — Name input with real-time validation
- `CharacterDeleteState` — Requires typing exact name to confirm
- `CharacterRenameState` — Name input with validation against existing names

These states are managed in `main.rs` and rendered by corresponding `ui/character_*.rs` files.

## Patterns

### Adding a New Attribute
1. Add field to `Attributes` struct in `attributes.rs`
2. Add `AttributeType` variant and update `AttributeType::all()`
3. Add derived stat calculations in `derived_stats.rs`
4. Update leveling distribution in `core/game_logic.rs`
5. Update item attribute bonuses in `items/types.rs` `AttributeBonuses`
6. Update scoring weights in `items/scoring.rs`
7. Update UI display in `ui/stats_panel.rs`

### `PrestigeCombatBonuses` (`prestige.rs`)

Flat combat bonuses computed from prestige rank, applied during combat each tick:

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct PrestigeCombatBonuses {
    pub flat_damage: u32,    // Added after Haven % bonus, before enemy defense
    pub flat_defense: u32,   // Added to DEX-based defense
    pub crit_chance: f64,    // Added to DEX-based crit (capped at 15%)
    pub flat_hp: u32,        // Added to combat max HP (not DerivedStats)
}
```

Computed via `PrestigeCombatBonuses::from_rank(rank)` using power-law formulas from `core/constants.rs`:
- `flat_damage = floor(5.0 * rank^0.7)` -- P5: 15, P10: 25, P20: 40
- `flat_defense = floor(3.0 * rank^0.6)` -- P5: 7, P10: 11, P20: 18
- `crit_chance = min(rank * 0.5, 15.0)` -- P10: 5%, P20: 10%, P30: 15%
- `flat_hp = floor(15.0 * rank^0.6)` -- P5: 39, P10: 59, P20: 90

### Adding a New Prestige Benefit
1. Add the benefit logic in `prestige.rs` (or relevant module)
2. Apply it during `perform_prestige()` or in derived stat calculation
3. Display it in `ui/prestige_confirm.rs` dialog
4. Document the benefit in prestige tier descriptions
