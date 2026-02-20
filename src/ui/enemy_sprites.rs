//! Enemy sprite rendering: struct definitions, archetype enum, color palettes,
//! and sprite selection logic.
//!
//! Sprite constant data (legacy ASCII, pixel art, suffix mapping tables)
//! lives in `enemy_sprite_data`.

use ratatui::style::Color;

use crate::core::game_state::GameState;
use crate::zones::get_zone;

// Re-export all sprite data so existing internal consumers still compile.
pub use super::enemy_sprite_data::*;

#[allow(dead_code)]
pub struct EnemySprite {
    pub base_art: &'static str,
    pub width: usize,
    pub height: usize,
}

#[allow(dead_code)]
impl EnemySprite {
    pub const fn new(art: &'static str, width: usize, height: usize) -> Self {
        Self {
            base_art: art,
            width,
            height,
        }
    }
}

pub struct PixelSprite {
    pub rows: &'static [&'static str],
}

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
    Undead,
    Aquatic,
    Plant,
}

impl SpriteArchetype {
    #[allow(dead_code)]
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
            SpriteArchetype::Undead => &SPRITE_HUMANOID,
            SpriteArchetype::Aquatic => &SPRITE_SERPENT,
            SpriteArchetype::Plant => &SPRITE_TITAN,
        }
    }

    pub fn pixel_sprite(&self) -> &'static PixelSprite {
        match self {
            SpriteArchetype::Insect => &PIXEL_INSECT,
            SpriteArchetype::Quadruped => &PIXEL_QUADRUPED,
            SpriteArchetype::Serpent => &PIXEL_SERPENT,
            SpriteArchetype::Humanoid => &PIXEL_HUMANOID,
            SpriteArchetype::Avian => &PIXEL_AVIAN,
            SpriteArchetype::Elemental => &PIXEL_ELEMENTAL,
            SpriteArchetype::Titan => &PIXEL_TITAN,
            SpriteArchetype::Horror => &PIXEL_HORROR,
            SpriteArchetype::Undead => &PIXEL_UNDEAD,
            SpriteArchetype::Aquatic => &PIXEL_AQUATIC,
            SpriteArchetype::Plant => &PIXEL_PLANT,
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

// ── Sprite Color Palette (RGB-based for pixel art) ───────────────────

pub struct SpriteColorPalette {
    pub dark: Color,      // outline - very dark version of body
    pub body: Color,      // primary body color
    pub shade: Color,     // dimmed body (~60% of body)
    pub eye: Color,       // accent color (from zone secondary)
    pub highlight: Color, // bright accent (~130% of body, clamped to 255)
}

/// Converts an ANSI-16 color to an approximate RGB tuple.
pub fn ansi_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Black => (0, 0, 0),
        Color::Red => (170, 0, 0),
        Color::Green => (0, 170, 0),
        Color::Yellow => (170, 170, 0),
        Color::Blue => (0, 0, 170),
        Color::Magenta => (170, 0, 170),
        Color::Cyan => (0, 170, 170),
        Color::Gray => (170, 170, 170),
        Color::DarkGray => (85, 85, 85),
        Color::LightRed => (255, 85, 85),
        Color::LightGreen => (85, 255, 85),
        Color::LightYellow => (255, 255, 85),
        Color::LightBlue => (85, 85, 255),
        Color::LightMagenta => (255, 85, 255),
        Color::LightCyan => (85, 255, 255),
        Color::White => (255, 255, 255),
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (128, 128, 128),
    }
}

fn scale_channel(v: u8, factor: f32) -> u8 {
    ((v as f32 * factor).round() as u16).min(255) as u8
}

/// Relative luminance (WCAG 2.1) for an RGB triplet.
fn relative_luminance(r: u8, g: u8, b: u8) -> f32 {
    fn linearize(c: u8) -> f32 {
        let s = c as f32 / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

/// Contrast ratio between two RGB colors (always >= 1.0).
fn contrast_ratio_rgb(a: (u8, u8, u8), b: (u8, u8, u8)) -> f32 {
    let la = relative_luminance(a.0, a.1, a.2) + 0.05;
    let lb = relative_luminance(b.0, b.1, b.2) + 0.05;
    if la > lb {
        la / lb
    } else {
        lb / la
    }
}

/// Builds a `SpriteColorPalette` from a zone id and enemy tier.
pub fn sprite_color_palette(zone_id: u32, tier: EnemyTier) -> SpriteColorPalette {
    let zone_pal = zone_palette(zone_id);

    // For zone boss tier, override primary to LightRed (bright)
    let primary_color = match tier {
        EnemyTier::ZoneBoss => Color::LightRed,
        EnemyTier::SubzoneBoss | EnemyTier::DungeonBoss => zone_pal.secondary,
        _ => zone_pal.primary,
    };

    let (pr, pg, pb) = ansi_to_rgb(primary_color);
    let (er, eg, eb) = ansi_to_rgb(zone_pal.secondary);

    // dark: ~20% of body per channel.
    let dark = Color::Rgb(
        scale_channel(pr, 0.20),
        scale_channel(pg, 0.20),
        scale_channel(pb, 0.20),
    );

    // body: the primary color
    let body = Color::Rgb(pr, pg, pb);

    // shade: ~60% of body
    let shade = Color::Rgb(
        scale_channel(pr, 0.60),
        scale_channel(pg, 0.60),
        scale_channel(pb, 0.60),
    );

    // eye: secondary zone color, blended toward White when contrast is low
    let eye_raw = (er, eg, eb);
    let eye = if contrast_ratio_rgb(eye_raw, (pr, pg, pb)) < 1.5 {
        Color::Rgb(
            ((er as u16 + 255) / 2) as u8,
            ((eg as u16 + 255) / 2) as u8,
            ((eb as u16 + 255) / 2) as u8,
        )
    } else {
        Color::Rgb(er, eg, eb)
    };

    // highlight: ~140% of body clamped to 255
    let hl_r = scale_channel(pr, 1.40);
    let hl_g = scale_channel(pg, 1.40);
    let hl_b = scale_channel(pb, 1.40);
    let highlight = if hl_r == pr && hl_g == pg && hl_b == pb {
        Color::Rgb(255, 255, 200)
    } else {
        Color::Rgb(hl_r, hl_g, hl_b)
    };

    SpriteColorPalette {
        dark,
        body,
        shade,
        eye,
        highlight,
    }
}

// ── Main Sprite Selection Function ──────────────────────────────────

/// Gets the appropriate sprite for an enemy based on zone context.
/// Uses zone_id for zone-themed suffix matching with archetype fallbacks.
#[allow(dead_code)]
pub fn get_sprite_for_enemy(enemy_name: &str, zone_id: u32) -> &'static EnemySprite {
    get_archetype_for_enemy(enemy_name, zone_id).sprite()
}

/// Gets the appropriate archetype for an enemy based on zone context.
/// Shared selection logic used by both ASCII and pixel sprite paths.
pub fn get_archetype_for_enemy(enemy_name: &str, zone_id: u32) -> SpriteArchetype {
    // Extract the suffix (last word of the name)
    let suffix = enemy_name.split_whitespace().last().unwrap_or(enemy_name);

    // Strip "Elite " or "Boss " prefix for dungeon enemies
    let clean_name = enemy_name
        .strip_prefix("Elite ")
        .or_else(|| enemy_name.strip_prefix("Boss "))
        .unwrap_or(enemy_name);
    let clean_suffix = clean_name.split_whitespace().last().unwrap_or(clean_name);

    // 1. Try zone-based suffix matching
    if suffix_is_known_for_zone(zone_id, suffix) {
        return archetype_for_suffix(zone_id, suffix);
    }

    // 2. Try boss keyword matching
    if let Some(boss_archetype) = boss_name_archetype(enemy_name) {
        return boss_archetype;
    }

    // 3. Try dungeon generic name matching (for "Orc", "Troll", etc.)
    let generic = dungeon_generic_archetype(clean_suffix);
    if clean_suffix.to_lowercase() != suffix.to_lowercase() || is_generic_suffix(clean_suffix) {
        return generic;
    }

    // 4. Fall back to zone default
    zone_default_archetype(zone_id)
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
            SpriteArchetype::Undead,
            SpriteArchetype::Aquatic,
            SpriteArchetype::Plant,
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
        assert_eq!(archetype_for_suffix(1, "Sprout"), SpriteArchetype::Plant);

        // Zone 4 — Skeleton and Mummy are now Undead
        assert_eq!(archetype_for_suffix(4, "Skeleton"), SpriteArchetype::Undead);
        assert_eq!(archetype_for_suffix(4, "Mummy"), SpriteArchetype::Undead);
        assert_eq!(
            archetype_for_suffix(4, "Spirit"),
            SpriteArchetype::Elemental
        );
        assert_eq!(archetype_for_suffix(4, "Specter"), SpriteArchetype::Horror);

        // Zone 8 — Shark is now Aquatic
        assert_eq!(archetype_for_suffix(8, "Shark"), SpriteArchetype::Aquatic);
        assert_eq!(archetype_for_suffix(8, "Ray"), SpriteArchetype::Aquatic);
        assert_eq!(archetype_for_suffix(8, "Drowned"), SpriteArchetype::Undead);

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

    #[test]
    fn test_pixel_sprites_have_even_or_odd_rows() {
        // All pixel sprites should have at least 2 rows and consistent structure.
        let archetypes = [
            SpriteArchetype::Insect,
            SpriteArchetype::Quadruped,
            SpriteArchetype::Serpent,
            SpriteArchetype::Humanoid,
            SpriteArchetype::Avian,
            SpriteArchetype::Elemental,
            SpriteArchetype::Titan,
            SpriteArchetype::Horror,
            SpriteArchetype::Undead,
            SpriteArchetype::Aquatic,
            SpriteArchetype::Plant,
        ];

        for archetype in &archetypes {
            let ps = archetype.pixel_sprite();
            assert!(
                ps.rows.len() >= 2,
                "{:?} pixel sprite has fewer than 2 rows",
                archetype
            );
            // Each row must only contain valid pixel characters
            for (i, row) in ps.rows.iter().enumerate() {
                for ch in row.chars() {
                    assert!(
                        matches!(ch, '.' | 'D' | 'B' | 'S' | 'E' | 'H'),
                        "{:?} pixel sprite row {} contains invalid char '{}'",
                        archetype,
                        i,
                        ch
                    );
                }
            }
        }
    }

    #[test]
    fn test_sprite_color_palette_zones() {
        // Should return distinct colors for dark vs body
        for zone_id in 1..=11 {
            let pal = sprite_color_palette(zone_id, EnemyTier::Normal);
            assert_ne!(
                pal.dark, pal.body,
                "Zone {} dark and body should differ",
                zone_id
            );
        }
    }

    #[test]
    fn test_ansi_to_rgb_basic() {
        let (r, g, b) = ansi_to_rgb(Color::White);
        assert_eq!((r, g, b), (255, 255, 255));

        let (r, g, b) = ansi_to_rgb(Color::Black);
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn test_get_archetype_for_enemy_consistency() {
        // get_archetype_for_enemy should produce same archetype as get_sprite_for_enemy
        let test_cases = [
            ("Meadow Beetle", 1u32),
            ("Shadow Spider", 2),
            ("Grizzled Orc", 0),
            ("Alpha Wolf", 2),
            ("Frost Wyrm", 3),
        ];
        for (name, zone) in &test_cases {
            let archetype = get_archetype_for_enemy(name, *zone);
            let sprite = get_sprite_for_enemy(name, *zone);
            assert_eq!(
                archetype.sprite().base_art,
                sprite.base_art,
                "Archetype mismatch for '{}'",
                name
            );
        }
    }
}
