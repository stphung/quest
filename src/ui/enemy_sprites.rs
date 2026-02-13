/// Enemy sprite templates for 3D rendering
use ratatui::style::Color;

use crate::core::game_state::GameState;
use crate::zones::get_zone;

pub struct EnemySprite {
    pub base_art: &'static str,
    #[allow(dead_code)]
    pub width: usize,
    #[allow(dead_code)]
    pub height: usize,
}

impl EnemySprite {
    pub const fn new(art: &'static str, width: usize, height: usize) -> Self {
        Self {
            base_art: art,
            width,
            height,
        }
    }
}

// ── 8 Base Sprite Archetypes ────────────────────────────────────────

pub const SPRITE_INSECT: EnemySprite = EnemySprite::new(
    r"    ╲│╱  ╲│╱
      ╲╱  ╲╱
     ┌──────┐
    ╱│ ●  ● │╲
   ╱ │  ▼▼  │ ╲
   ╲ │▒▒▒▒▒▒│ ╱
    ╲└──────┘╱
    ╱├──────┤╲
   ╱ ╰──────╯ ╲
  ╱╱            ╲╲",
    16,
    10,
);

pub const SPRITE_QUADRUPED: EnemySprite = EnemySprite::new(
    r"   ╱▲    ▲╲
  ╱  ╱╲  ╱╲  ╲
 │  ● ╱██╲ ●  │
 │   ╱████╲   │
 │  │ ▼══▼ │  │
  ╲ ╰══════╯ ╱
   ╲ ██████ ╱
    ▐██████▌
   ╱╱ ╱╲╱╲ ╲╲
  ╰╯ ╰╯  ╰╯ ╰╯",
    16,
    10,
);

pub const SPRITE_SERPENT: EnemySprite = EnemySprite::new(
    r"       ╱╲
    ╱▓▓▓▓╲
   │ ◆  ◆ │
   │  ╲╱╲  │
   ╰┐ ▼▼ ┌╯
  ╱▓╰════╯▓╲
 │▓▓╲      ╱▓│
  ╲▓▓╲  ╱╱▓╱
   ╲▓▓╲╱╱▓╱
    ╰══╲╱══╯",
    15,
    10,
);

pub const SPRITE_HUMANOID: EnemySprite = EnemySprite::new(
    r"     ╱══╲
    ╱ ▓▓ ╲
    │ ●  ● │
    │  ▼   │
    ╰┬────┬╯
   ╱─┤████├─╲
  ╱  │████│  ╲
     │████│
     ├─┬┬─┤
     ╰─╯╰─╯",
    15,
    10,
);

pub const SPRITE_AVIAN: EnemySprite = EnemySprite::new(
    r"       ╱╲
╲     ╱████╲     ╱
 ╲   │ ◆  ◆ │   ╱
  ╲  │  ╲╱   │  ╱
   ╲ ╰──────╯ ╱
    ╲ ▒████▒ ╱
     ╲ ████ ╱
      ╲▒▒▒▒╱
      ╱╲  ╱╲
     ╱╱ ╲╱ ╲╲",
    17,
    10,
);

pub const SPRITE_ELEMENTAL: EnemySprite = EnemySprite::new(
    r"    ╱░░░░╲
   ╱░▒▒▒▒░╲
  │░▒ ◆◆ ▒░│
  │░▒▓▓▓▓▒░│
  │░▒▓██▓▒░│
  │░▒▓▓▓▓▒░│
  │░▒▒▒▒▒▒░│
   ╲░░░░░░╱
    ╲░░░░╱
      ░░",
    14,
    10,
);

pub const SPRITE_TITAN: EnemySprite = EnemySprite::new(
    r"   ═══════════
   ║ ●     ● ║
   ║    ▼▼    ║
   ║ ╱════╲  ║
  ╔╩════════╩╗
  ║██████████║
  ║██████████║
  ╚╦════════╦╝
   ║║      ║║
   ╩╩      ╩╩",
    15,
    10,
);

pub const SPRITE_HORROR: EnemySprite = EnemySprite::new(
    r"   ╱╲  ╱╲  ╱╲
  ╱ ▒▓▒▓▒▓▒ ╲
 │ ●  ◆  ● ◆│
 │ ▓▒░▒▓░▒▓ │
 │ ╱▓█▓▓█▓╲ │
  ╲▓█▓▒▒▓█▓╱
  ╱▒╲▓▓▓▓╱▒╲
 ╱▒╱╲╲▒▒╱╱▒╲╲
 ╲╱  ╲╲╱╱  ╲╱
      ╲╱╲╱",
    16,
    10,
);

// ── Boss Crown Patterns ─────────────────────────────────────────────

pub const BOSS_CROWN: &str = "--- \u{2605} ---";
pub const ZONE_BOSS_CROWN: &str = "=== \u{2605} ===";

// ── Sprite Archetype Enum ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteArchetype {
    Insect,
    Quadruped,
    Serpent,
    Humanoid,
    Avian,
    Elemental,
    Titan,
    Horror,
}

impl SpriteArchetype {
    pub fn sprite(&self) -> &'static EnemySprite {
        match self {
            SpriteArchetype::Insect => &SPRITE_INSECT,
            SpriteArchetype::Quadruped => &SPRITE_QUADRUPED,
            SpriteArchetype::Serpent => &SPRITE_SERPENT,
            SpriteArchetype::Humanoid => &SPRITE_HUMANOID,
            SpriteArchetype::Avian => &SPRITE_AVIAN,
            SpriteArchetype::Elemental => &SPRITE_ELEMENTAL,
            SpriteArchetype::Titan => &SPRITE_TITAN,
            SpriteArchetype::Horror => &SPRITE_HORROR,
        }
    }
}

// ── Enemy Tier Enum ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyTier {
    Normal,
    DungeonElite,
    SubzoneBoss,
    DungeonBoss,
    ZoneBoss,
}

/// Detects the enemy tier from the current game state.
pub fn detect_enemy_tier(game_state: &GameState) -> EnemyTier {
    let enemy = match &game_state.combat_state.current_enemy {
        Some(e) => e,
        None => return EnemyTier::Normal,
    };

    let in_dungeon = game_state.active_dungeon.is_some();

    if in_dungeon {
        if enemy.name.starts_with("Boss ") {
            return EnemyTier::DungeonBoss;
        }
        if enemy.name.starts_with("Elite ") {
            return EnemyTier::DungeonElite;
        }
        return EnemyTier::Normal;
    }

    if game_state.zone_progression.fighting_boss {
        // Check if this is the zone boss (final subzone) or a subzone boss
        let zone_id = game_state.zone_progression.current_zone_id;
        let subzone_id = game_state.zone_progression.current_subzone_id;
        if let Some(zone) = get_zone(zone_id) {
            let is_final_subzone = subzone_id == zone.subzones.len() as u32;
            if is_final_subzone {
                return EnemyTier::ZoneBoss;
            }
        }
        return EnemyTier::SubzoneBoss;
    }

    EnemyTier::Normal
}

// ── Zone Color Palette ──────────────────────────────────────────────

pub struct ZoneColorPalette {
    pub primary: Color,
    pub secondary: Color,
}

/// Returns the zone color palette (ANSI-16 colors only).
pub fn zone_palette(zone_id: u32) -> ZoneColorPalette {
    match zone_id {
        1 => ZoneColorPalette {
            primary: Color::Green,
            secondary: Color::Yellow,
        },
        2 => ZoneColorPalette {
            primary: Color::DarkGray,
            secondary: Color::Green,
        },
        3 => ZoneColorPalette {
            primary: Color::Gray,
            secondary: Color::White,
        },
        4 => ZoneColorPalette {
            primary: Color::Magenta,
            secondary: Color::LightRed,
        },
        5 => ZoneColorPalette {
            primary: Color::LightRed,
            secondary: Color::Yellow,
        },
        6 => ZoneColorPalette {
            primary: Color::Cyan,
            secondary: Color::White,
        },
        7 => ZoneColorPalette {
            primary: Color::LightMagenta,
            secondary: Color::Cyan,
        },
        8 => ZoneColorPalette {
            primary: Color::Blue,
            secondary: Color::Cyan,
        },
        9 => ZoneColorPalette {
            primary: Color::White,
            secondary: Color::Yellow,
        },
        10 => ZoneColorPalette {
            primary: Color::Yellow,
            secondary: Color::White,
        },
        11 => ZoneColorPalette {
            primary: Color::LightRed,
            secondary: Color::Magenta,
        },
        _ => ZoneColorPalette {
            primary: Color::Red,
            secondary: Color::Yellow,
        },
    }
}

// ── Zone Suffix-to-Archetype Mapping ────────────────────────────────

/// Returns the sprite archetype for a zone enemy suffix.
/// Falls back to the zone default if the suffix is unrecognized.
pub fn archetype_for_suffix(zone_id: u32, suffix: &str) -> SpriteArchetype {
    let s = suffix.to_lowercase();
    let matched = match zone_id {
        1 => match s.as_str() {
            "beetle" | "wasp" => Some(SpriteArchetype::Insect),
            "rabbit" | "boar" => Some(SpriteArchetype::Quadruped),
            "serpent" => Some(SpriteArchetype::Serpent),
            _ => None,
        },
        2 => match s.as_str() {
            "wolf" => Some(SpriteArchetype::Quadruped),
            "spider" => Some(SpriteArchetype::Insect),
            "bat" => Some(SpriteArchetype::Avian),
            "treant" => Some(SpriteArchetype::Titan),
            "wisp" => Some(SpriteArchetype::Elemental),
            _ => None,
        },
        3 => match s.as_str() {
            "goat" => Some(SpriteArchetype::Quadruped),
            "eagle" => Some(SpriteArchetype::Avian),
            "golem" | "yeti" => Some(SpriteArchetype::Titan),
            "harpy" => Some(SpriteArchetype::Humanoid),
            _ => None,
        },
        4 => match s.as_str() {
            "skeleton" | "mummy" => Some(SpriteArchetype::Humanoid),
            "spirit" => Some(SpriteArchetype::Elemental),
            "gargoyle" => Some(SpriteArchetype::Titan),
            "specter" => Some(SpriteArchetype::Horror),
            _ => None,
        },
        5 => match s.as_str() {
            "salamander" => Some(SpriteArchetype::Serpent),
            "phoenix" | "drake" => Some(SpriteArchetype::Avian),
            "imp" => Some(SpriteArchetype::Humanoid),
            "elemental" => Some(SpriteArchetype::Elemental),
            _ => None,
        },
        6 => match s.as_str() {
            "mammoth" => Some(SpriteArchetype::Titan),
            "wendigo" | "wraith" => Some(SpriteArchetype::Horror),
            "bear" => Some(SpriteArchetype::Quadruped),
            "wyrm" => Some(SpriteArchetype::Serpent),
            _ => None,
        },
        7 => match s.as_str() {
            "construct" | "golem" => Some(SpriteArchetype::Titan),
            "guardian" => Some(SpriteArchetype::Humanoid),
            "sprite" => Some(SpriteArchetype::Elemental),
            "watcher" => Some(SpriteArchetype::Horror),
            _ => None,
        },
        8 => match s.as_str() {
            "kraken" => Some(SpriteArchetype::Horror),
            "shark" => Some(SpriteArchetype::Quadruped),
            "naga" => Some(SpriteArchetype::Serpent),
            "leviathan" => Some(SpriteArchetype::Titan),
            "siren" => Some(SpriteArchetype::Humanoid),
            _ => None,
        },
        9 => match s.as_str() {
            "griffin" | "roc" | "wyvern" => Some(SpriteArchetype::Avian),
            "djinn" | "sylph" => Some(SpriteArchetype::Elemental),
            _ => None,
        },
        10 => match s.as_str() {
            "titan" | "colossus" => Some(SpriteArchetype::Titan),
            "lord" | "king" | "champion" => Some(SpriteArchetype::Humanoid),
            _ => None,
        },
        11 => match s.as_str() {
            "beast" => Some(SpriteArchetype::Quadruped),
            "horror" | "terror" => Some(SpriteArchetype::Horror),
            "fiend" => Some(SpriteArchetype::Humanoid),
            "monster" => Some(SpriteArchetype::Titan),
            _ => None,
        },
        _ => None,
    };

    matched.unwrap_or(zone_default_archetype(zone_id))
}

/// Returns the default archetype for a zone (used when suffix doesn't match).
fn zone_default_archetype(zone_id: u32) -> SpriteArchetype {
    match zone_id {
        1 | 2 | 6 => SpriteArchetype::Quadruped,
        3 => SpriteArchetype::Titan,
        4 | 10 => SpriteArchetype::Humanoid,
        5 | 7 => SpriteArchetype::Elemental,
        8 => SpriteArchetype::Serpent,
        9 => SpriteArchetype::Avian,
        11 => SpriteArchetype::Horror,
        _ => SpriteArchetype::Quadruped,
    }
}

/// Archetype matching for dungeon enemies with generic names (Orc, Troll, etc.)
fn dungeon_generic_archetype(suffix: &str) -> SpriteArchetype {
    match suffix.to_lowercase().as_str() {
        "orc" => SpriteArchetype::Humanoid,
        "troll" => SpriteArchetype::Titan,
        "drake" => SpriteArchetype::Avian,
        "crusher" => SpriteArchetype::Titan,
        "beast" | "fiend" => SpriteArchetype::Quadruped,
        "horror" | "terror" => SpriteArchetype::Horror,
        "render" | "maw" => SpriteArchetype::Horror,
        _ => SpriteArchetype::Quadruped,
    }
}

/// Archetype matching for boss enemies using keyword matching on the boss name.
fn boss_name_archetype(boss_name: &str) -> Option<SpriteArchetype> {
    let name = boss_name.to_lowercase();
    // Check specific creature keywords first (before generic title keywords)
    if name.contains("spider") || name.contains("sporeling") || name.contains("arachne") {
        Some(SpriteArchetype::Insect)
    } else if name.contains("wolf") || name.contains("bear") || name.contains("beast") {
        Some(SpriteArchetype::Quadruped)
    } else if name.contains("treant")
        || name.contains("giant")
        || name.contains("golem")
        || name.contains("colossus")
        || name.contains("titan")
        || name.contains("mammoth")
        || name.contains("leviathan")
        || name.contains("behemoth")
    {
        Some(SpriteArchetype::Titan)
    } else if name.contains("wyrm")
        || name.contains("serpent")
        || name.contains("naga")
        || name.contains("salamander")
    {
        Some(SpriteArchetype::Serpent)
    } else if name.contains("horror")
        || name.contains("wraith")
        || name.contains("specter")
        || name.contains("kraken")
        || name.contains("avatar")
        || name.contains("frozen one")
        || name.contains("drowned")
        || name.contains("broodmother")
    {
        Some(SpriteArchetype::Horror)
    } else if name.contains("drake")
        || name.contains("phoenix")
        || name.contains("harpy")
        || name.contains("roc")
    {
        Some(SpriteArchetype::Avian)
    } else if name.contains("elemental")
        || name.contains("wisp")
        || name.contains("sprite")
        || name.contains("storm")
        || name.contains("tempest")
        || name.contains("incarnate")
    {
        Some(SpriteArchetype::Elemental)
    } else if name.contains("skeleton")
        || name.contains("king")
        || name.contains("lord")
        || name.contains("queen")
        || name.contains("chief")
        || name.contains("warlord")
        || name.contains("commander")
        || name.contains("warden")
        || name.contains("admiral")
        || name.contains("knight")
        || name.contains("guardian")
        || name.contains("matriarch")
        || name.contains("sentinel")
    {
        Some(SpriteArchetype::Humanoid)
    } else {
        None
    }
}

// ── Main Sprite Selection Function ──────────────────────────────────

/// Gets the appropriate sprite for an enemy based on zone context.
/// Uses zone_id for zone-themed suffix matching with archetype fallbacks.
pub fn get_sprite_for_enemy(enemy_name: &str, zone_id: u32) -> &'static EnemySprite {
    // Extract the suffix (last word of the name)
    let suffix = enemy_name.split_whitespace().last().unwrap_or(enemy_name);

    // Strip "Elite " or "Boss " prefix for dungeon enemies
    let clean_name = enemy_name
        .strip_prefix("Elite ")
        .or_else(|| enemy_name.strip_prefix("Boss "))
        .unwrap_or(enemy_name);
    let clean_suffix = clean_name.split_whitespace().last().unwrap_or(clean_name);

    // 1. Try zone-based suffix matching
    let archetype = archetype_for_suffix(zone_id, suffix);
    // If the suffix matched a known zone enemy, use that archetype
    if suffix_is_known_for_zone(zone_id, suffix) {
        return archetype.sprite();
    }

    // 2. Try boss keyword matching
    if let Some(boss_archetype) = boss_name_archetype(enemy_name) {
        return boss_archetype.sprite();
    }

    // 3. Try dungeon generic name matching (for "Orc", "Troll", etc.)
    let generic = dungeon_generic_archetype(clean_suffix);
    if clean_suffix.to_lowercase() != suffix.to_lowercase() || is_generic_suffix(clean_suffix) {
        return generic.sprite();
    }

    // 4. Fall back to zone default
    zone_default_archetype(zone_id).sprite()
}

/// Checks if a suffix is a known zone enemy suffix.
fn suffix_is_known_for_zone(zone_id: u32, suffix: &str) -> bool {
    let s = suffix.to_lowercase();
    match zone_id {
        1 => matches!(
            s.as_str(),
            "beetle" | "rabbit" | "wasp" | "boar" | "serpent"
        ),
        2 => matches!(s.as_str(), "wolf" | "spider" | "bat" | "treant" | "wisp"),
        3 => matches!(s.as_str(), "goat" | "eagle" | "golem" | "yeti" | "harpy"),
        4 => matches!(
            s.as_str(),
            "skeleton" | "mummy" | "spirit" | "gargoyle" | "specter"
        ),
        5 => matches!(
            s.as_str(),
            "salamander" | "phoenix" | "imp" | "drake" | "elemental"
        ),
        6 => matches!(
            s.as_str(),
            "mammoth" | "wendigo" | "wraith" | "bear" | "wyrm"
        ),
        7 => matches!(
            s.as_str(),
            "construct" | "guardian" | "sprite" | "watcher" | "golem"
        ),
        8 => matches!(
            s.as_str(),
            "kraken" | "shark" | "naga" | "leviathan" | "siren"
        ),
        9 => matches!(s.as_str(), "griffin" | "djinn" | "sylph" | "roc" | "wyvern"),
        10 => matches!(
            s.as_str(),
            "titan" | "colossus" | "lord" | "king" | "champion"
        ),
        11 => matches!(
            s.as_str(),
            "beast" | "horror" | "fiend" | "terror" | "monster"
        ),
        _ => false,
    }
}

/// Checks if a suffix is from the generic dungeon enemy name pool.
fn is_generic_suffix(suffix: &str) -> bool {
    matches!(
        suffix.to_lowercase().as_str(),
        "orc"
            | "troll"
            | "drake"
            | "crusher"
            | "render"
            | "maw"
            | "beast"
            | "fiend"
            | "horror"
            | "terror"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sprite_for_orc() {
        let sprite = get_sprite_for_enemy("Grizzled Orc", 0);
        assert_eq!(sprite.height, 10);
        assert!(sprite.base_art.contains("●"));
    }

    #[test]
    fn test_get_sprite_for_drake() {
        let sprite = get_sprite_for_enemy("Dark Drake", 0);
        assert_eq!(sprite.height, 10);
        assert!(sprite.base_art.contains("◆"));
    }

    #[test]
    fn test_get_sprite_default() {
        let sprite = get_sprite_for_enemy("Unknown Monster", 0);
        assert_eq!(sprite.height, 10);
    }

    #[test]
    fn test_sprite_dimensions() {
        assert_eq!(SPRITE_INSECT.height, 10);
        assert_eq!(SPRITE_QUADRUPED.height, 10);
        assert_eq!(SPRITE_SERPENT.height, 10);
        assert_eq!(SPRITE_HUMANOID.height, 10);
        assert_eq!(SPRITE_AVIAN.height, 10);
        assert_eq!(SPRITE_ELEMENTAL.height, 10);
        assert_eq!(SPRITE_TITAN.height, 10);
        assert_eq!(SPRITE_HORROR.height, 10);
    }

    #[test]
    fn test_zone_sprite_selection() {
        // Zone 1 (Meadow): Beetle -> INSECT
        let sprite = get_sprite_for_enemy("Meadow Beetle", 1);
        assert_eq!(sprite.base_art, SPRITE_INSECT.base_art);

        // Zone 2 (Dark Forest): Spider -> INSECT
        let sprite = get_sprite_for_enemy("Shadow Spider", 2);
        assert_eq!(sprite.base_art, SPRITE_INSECT.base_art);

        // Zone 5 (Volcanic): Phoenix -> AVIAN
        let sprite = get_sprite_for_enemy("Flame Phoenix", 5);
        assert_eq!(sprite.base_art, SPRITE_AVIAN.base_art);
    }

    #[test]
    fn test_zone_sprite_defaults() {
        // Unknown enemy name within a zone should get zone default
        // Zone 1 default: QUADRUPED
        let sprite = get_sprite_for_enemy("Unknown Creature", 1);
        assert_eq!(sprite.base_art, SPRITE_QUADRUPED.base_art);

        // Zone 8 default: SERPENT
        let sprite = get_sprite_for_enemy("Unknown Creature", 8);
        assert_eq!(sprite.base_art, SPRITE_SERPENT.base_art);
    }

    #[test]
    fn test_all_zone_sprites_are_10_lines() {
        let archetypes = [
            SpriteArchetype::Insect,
            SpriteArchetype::Quadruped,
            SpriteArchetype::Serpent,
            SpriteArchetype::Humanoid,
            SpriteArchetype::Avian,
            SpriteArchetype::Elemental,
            SpriteArchetype::Titan,
            SpriteArchetype::Horror,
        ];

        for archetype in &archetypes {
            assert_eq!(
                archetype.sprite().height,
                10,
                "{:?} has wrong height",
                archetype
            );
        }
    }

    #[test]
    fn test_zone_palette() {
        // Verify each zone returns a palette with ANSI-16 colors
        for zone_id in 1..=11 {
            let palette = zone_palette(zone_id);
            // Just verify primary and secondary are assigned
            assert!(
                palette.primary != palette.secondary,
                "Zone {} should have distinct primary/secondary colors",
                zone_id
            );
        }
    }

    #[test]
    fn test_all_zones_have_sprite_coverage() {
        // Every zone should return a valid sprite for any enemy name
        for zone_id in 1..=11 {
            let sprite = get_sprite_for_enemy("SomeRandomEnemy", zone_id);
            assert_eq!(
                sprite.height, 10,
                "Zone {} default sprite should be 10 lines",
                zone_id
            );
        }
    }

    #[test]
    fn test_archetype_for_suffix_all_zones() {
        // Zone 1
        assert_eq!(archetype_for_suffix(1, "Beetle"), SpriteArchetype::Insect);
        assert_eq!(archetype_for_suffix(1, "Boar"), SpriteArchetype::Quadruped);
        assert_eq!(archetype_for_suffix(1, "Serpent"), SpriteArchetype::Serpent);

        // Zone 4
        assert_eq!(
            archetype_for_suffix(4, "Skeleton"),
            SpriteArchetype::Humanoid
        );
        assert_eq!(
            archetype_for_suffix(4, "Spirit"),
            SpriteArchetype::Elemental
        );
        assert_eq!(archetype_for_suffix(4, "Specter"), SpriteArchetype::Horror);

        // Zone 9
        assert_eq!(archetype_for_suffix(9, "Griffin"), SpriteArchetype::Avian);
        assert_eq!(archetype_for_suffix(9, "Djinn"), SpriteArchetype::Elemental);

        // Zone 10
        assert_eq!(archetype_for_suffix(10, "Titan"), SpriteArchetype::Titan);
        assert_eq!(archetype_for_suffix(10, "Lord"), SpriteArchetype::Humanoid);
    }

    #[test]
    fn test_dungeon_generic_matching() {
        // Dungeon enemies with generic names should match via suffix
        let sprite = get_sprite_for_enemy("Grizzled Orc", 1);
        assert_eq!(sprite.base_art, SPRITE_HUMANOID.base_art);

        let sprite = get_sprite_for_enemy("Elite Darken Horror", 3);
        assert_eq!(sprite.base_art, SPRITE_HORROR.base_art);

        let sprite = get_sprite_for_enemy("Boss Savage Troll", 5);
        assert_eq!(sprite.base_art, SPRITE_TITAN.base_art);
    }

    #[test]
    fn test_boss_name_matching() {
        // Named bosses should match via keyword
        let sprite = get_sprite_for_enemy("Sporeling Queen", 1);
        assert_eq!(sprite.base_art, SPRITE_INSECT.base_art);

        let sprite = get_sprite_for_enemy("Frost Wyrm", 3);
        assert_eq!(sprite.base_art, SPRITE_SERPENT.base_art);

        let sprite = get_sprite_for_enemy("Alpha Wolf", 2);
        assert_eq!(sprite.base_art, SPRITE_QUADRUPED.base_art);

        let sprite = get_sprite_for_enemy("Corrupted Treant", 2);
        assert_eq!(sprite.base_art, SPRITE_TITAN.base_art);
    }

    #[test]
    fn test_enemy_tier_enum() {
        // Just test the enum values exist and are distinct
        assert_ne!(EnemyTier::Normal, EnemyTier::DungeonElite);
        assert_ne!(EnemyTier::SubzoneBoss, EnemyTier::ZoneBoss);
        assert_ne!(EnemyTier::DungeonBoss, EnemyTier::Normal);
    }
}
