/// Enemy sprite templates for 3D rendering
use ratatui::style::Color;

use crate::core::game_state::GameState;
use crate::zones::get_zone;

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

// ── 8 Base Sprite Archetypes (legacy ASCII art, kept for reference) ──────────

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

// ── Pixel Sprite Definitions ─────────────────────────────────────────
//
// Pixel format per character:
//   '.' = transparent (preserve zone background)
//   'D' = dark outline
//   'B' = body primary color
//   'S' = shade (darker body)
//   'E' = eye / accent color
//   'H' = highlight (bright accent)
//
// Rows are processed in pairs (row 0+1, row 2+3, ...) to produce one
// terminal cell row each, using ▀ / ▄ / █ half-block characters.

pub struct PixelSprite {
    pub rows: &'static [&'static str],
}

// HUMANOID: Armored warrior. Crested helm, broad pauldrons, weapon-arm raised, two legs.
// 16 pixels wide, 18 pixels tall → 9 cell rows
pub const PIXEL_HUMANOID: PixelSprite = PixelSprite {
    rows: &[
        "....HDDDDH......",
        "...DBBBBBBD.....",
        "...DBEEBBBD.....",
        "...DBBBBSBD.....",
        "....DDDDDD......",
        "..HDBBBBBDDH....",
        ".DBBBBBBBBBBBD..",
        "HDBSBBBBBBBBSDH.",
        ".DBBBBBBBBBBD...",
        ".DBSBBBBBBSBD...",
        "..DBBHBBHBBD....",
        "..DBBBBBBBD.....",
        "..DBBD.DBBD.....",
        "..DBBD.DBBD.....",
        "..DBSD.DBSD.....",
        "..DBBD.DBBD.....",
        "..DSSD.DSSD.....",
        "..DDDD.DDDD.....",
    ],
};

// INSECT: Bug with antennae, segmented oval body, 3 pairs of jointed legs, compound eyes.
// 18 pixels wide, 16 pixels tall → 8 cell rows
pub const PIXEL_INSECT: PixelSprite = PixelSprite {
    rows: &[
        "D.....D....D.....D",
        "..D..D......D..D..",
        "....DBBBBBBBBD....",
        "....DEEBBBEEBD....",
        "....DBBBBBBBD.....",
        ".DBBBBBBBBBBBBBD..",
        "DBBHBBBBBBBBBBHBD.",
        "DBSSBBBBBBBBSSBD..",
        "DBBBBBBBBBBBBBBBD.",
        ".DBD.DBBBBBBD.DBD.",
        ".BD..DBBBBBD...DB.",
        ".D...DBBBBD.....D.",
        "....DBBBBBBD......",
        "....DBBSSBD.......",
        "....DBBBBD........",
        "....DDDDD.........",
    ],
};

// QUADRUPED: Four-legged beast. Profile view, prominent head, muscular barrel body, four legs.
// 18 pixels wide, 18 pixels tall → 9 cell rows
pub const PIXEL_QUADRUPED: PixelSprite = PixelSprite {
    rows: &[
        ".DBD.......DBD....",
        ".DBBD.....DBSD....",
        "..DBBBBBBBBBD.....",
        "..DBEBBBBEBBD.....",
        "..DBBBBBBBBBD.....",
        "..DBHBBBBHBBD.....",
        "..DBBBBBBBBBD.....",
        ".DBBSSBBBBSSBD....",
        ".DBBBBBBBBBSBD....",
        ".DBBBBBBBBBBD.....",
        "..DBBBBBBBBD......",
        "..DBD...DBBD......",
        ".DBD....DSBD......",
        ".DBD....DBBD......",
        ".DSD....DSSD......",
        ".DDD....DDDD......",
        "..................",
        "..................",
    ],
};

// SERPENT: Cobra strike pose, flared hood, S-curve body with scale pattern.
// 16 pixels wide, 18 pixels tall → 9 cell rows
pub const PIXEL_SERPENT: PixelSprite = PixelSprite {
    rows: &[
        "....HDDDDH......",
        "...DBBBBBBBD....",
        "..DBBBBBBBBBD...",
        ".DBBEBBBBEBBD...",
        "..DBBBBBBBBD....",
        "...DBBEBBD......",
        "....DBBBD.......",
        ".....DBBD.......",
        "..DBBBBD........",
        ".DBSBBBD........",
        ".DBBBBD.........",
        "......DBBBBD....",
        ".....DBSBBBD....",
        "......DBBBD.....",
        "..DBBBD.........",
        "...DBBBD........",
        "....DBSD........",
        ".....HDD........",
    ],
};

// AVIAN: Front-facing raptor. Wide M-wingspan, hooked beak, layered feathers, spread talons.
// 20 pixels wide, 18 pixels tall → 9 cell rows
pub const PIXEL_AVIAN: PixelSprite = PixelSprite {
    rows: &[
        "........DDDDD.......",
        ".......DBBBBBD......",
        ".......DBEBBBD......",
        ".......DBHBBD.......",
        "....DDDDBBBBDDDD....",
        "..DHBBBBBBBBBBBHBD..",
        ".DBBSSBBBBBBBBSSBBHD",
        "DBBBBBBBBBBBBBBBBBD.",
        ".DBBSSBBBBBBBBSSBD..",
        "...DDDBBBBBBBBDDD...",
        "......DBBBBBBD......",
        "......DBBBSSBD......",
        ".......DBBBD........",
        "......DBBD.DBD......",
        ".....DBD....DBD.....",
        "....DBHD....DBHD....",
        "...DBBD......DBBD...",
        "...DDD........DDD...",
    ],
};

// ELEMENTAL: Amorphous energy entity. H core, irregular spiked outline, E accent sparks, asymmetric.
// 18 pixels wide, 18 pixels tall → 9 cell rows
pub const PIXEL_ELEMENTAL: PixelSprite = PixelSprite {
    rows: &[
        "....H......H......",
        "...DHHBD...H......",
        "..DBHHHBBD........",
        ".DBHHHHHBBD.H.....",
        ".DBHHEEHBBBD......",
        "DBHHHEEBBBBD......",
        "DBBBHHHBBBBD......",
        ".DBBHBBBSBD.......",
        ".DBBBBBHSBD.......",
        "..DBBBSBBD........",
        "..DBBBBBBD.H......",
        "...DBHBBD.........",
        "...DBBSBD.........",
        "....DBBBD.........",
        "....DBHD..........",
        "...DBD.H..........",
        "..DD..............",
        "..................",
    ],
};

// TITAN: Enormous armored colossus. Tiny head, massive pauldrons, thick barrel torso, column legs.
// 20 pixels wide, 18 pixels tall → 9 cell rows
pub const PIXEL_TITAN: PixelSprite = PixelSprite {
    rows: &[
        "......HDDDDH........",
        "......DBBBBD........",
        "......DBEEBD........",
        "......DBBSBD........",
        "..HDDDDBBBBDDDDH....",
        "..DBBHBBBBBBHBBBD...",
        ".DBBBBBBBBBBBBBSBD..",
        ".DBBHBBBBBBBBHBBBD..",
        ".DBSBBHHBBBBBBBSBD..",
        ".DBBSBBBBBBBBSBBBD..",
        ".DBBBBBBHBBBBBSBD...",
        "..DBBBBBBBBBBBBD....",
        "...DBBBD..DBBBD.....",
        "...DBSBD..DBSBD.....",
        "...DBBBD..DBBBD.....",
        "...DBBBD..DBBBD.....",
        "...DSSSD..DSSSD.....",
        "...DDDDD..DDDDD.....",
    ],
};

// HORROR: Asymmetric eldritch abomination. Scattered E eyes, irregular tentacles, mottled body.
// 20 pixels wide, 18 pixels tall → 9 cell rows
pub const PIXEL_HORROR: PixelSprite = PixelSprite {
    rows: &[
        "..E.........E.......",
        "..D....E............",
        "....DDDDDDDDDD......",
        "...DBBBSBBEBBD......",
        "..DBHSBBBSBBBBBD....",
        "..DBEBBBSBBHBBBD....",
        ".DBBBHBBBBSBBBBBBD..",
        ".DBBSBBEBBBBBSBD.DD.",
        ".DBBBBBSBBHBBBBD....",
        "..DBBEBBBBBBBSBD....",
        "..DBBBSBBHBBBD......",
        "...DDDDDDDDDD.......",
        "....DBBBD...........",
        "...DBBD.DD..........",
        "...DBD..D...........",
        "....DD..D...........",
        "....D...............",
        "................E...",
    ],
};

// UNDEAD: Skeletal warrior. Hollow eyes, exposed ribs, bony arms, leg bones.
// 16 pixels wide, 18 pixels tall → 9 cell rows
pub const PIXEL_UNDEAD: PixelSprite = PixelSprite {
    rows: &[
        "....HDDDDH......",
        "...DBBEBBD......",
        "...DBEEBBD......",
        "....DDDDD.......",
        "...DBSBSBD......",
        "..HDBBBBBBDH....",
        "..DSBSDBSBD.....",
        "..DBBBBBBBD.....",
        "...DDDDDDD......",
        "..DBBBD.DBD.....",
        "..DBBD..DBD.....",
        "..DBBD..DBD.....",
        "...DDD..DDD.....",
        "..DBD....DBD....",
        ".DBBD....DBBD...",
        ".DBSD....DSBD...",
        "..DDD....DDD....",
        "................",
    ],
};

// PLANT: Aggressive flora. Leafy canopy, thorned vines, root legs.
// 16 pixels wide, 18 pixels tall → 9 cell rows
pub const PIXEL_PLANT: PixelSprite = PixelSprite {
    rows: &[
        "...HBHBHBHB.....",
        "..HBBBBBBBBD....",
        "..DBEBEBBBD.....",
        "..DBBBBBBBD.....",
        ".DBHBBBBBBBD....",
        "DBBBBBBBBBBBD...",
        "DBSBBBHBBSBBBD..",
        "DBBBBBBBBBBBD...",
        ".DBBHBBHBBBD....",
        "..DBBBBBBBD.....",
        "DBBBBBBBBBBBBBD.",
        "DBBSBBBBSBBBBBD.",
        ".DSBBBBBBBSBD...",
        "..DDBBBBBBDD....",
        "...DBBBBBBD.....",
        "..DBSD.DBSD.....",
        ".DBBBD.DBBBD....",
        "DDSSD...DSSDD...",
    ],
};

// AQUATIC: Sea creature. Streamlined torpedo body, fin ridges, deep-water coloring.
// 18 pixels wide, 16 pixels tall → 8 cell rows
pub const PIXEL_AQUATIC: PixelSprite = PixelSprite {
    rows: &[
        "..DBBBBBBBBBBBBBD.",
        ".DEBBBBBBBBBBBBBED",
        "DBSBBBBBBBBBBBBSBD",
        "DEBBBBBBBBBBBBBBBD",
        "DBSSBBBBBBBBBBBSBD",
        ".DBBBBBBBBBBBBBBD.",
        "DBBSBBBBBBBBSBBD..",
        ".DBBBBBBBBBBBBD...",
        "DBBSSBBBBBBSSBD...",
        ".DBBBBBBBBBBBBD...",
        "..DBBBBBBBBBBD....",
        "DBBBBBBBBBBBBBBBD.",
        ".DSBBBBBBBBBSBBD..",
        "..DBBBBBBBBBBD....",
        "...DBBBBBBBBBD....",
        "....DHBBBBBHD.....",
    ],
};

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

/// Contrast ratio between two RGB colors (always ≥ 1.0).
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

    // dark: ~20% of body per channel. This gives a deep tinted shadow that forms the
    // sprite outline. For very dark primaries like Blue (0,0,170), even 20% gives low
    // contrast — this is an inherent limitation of dark ANSI hues, and is accepted.
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

    // eye: secondary zone color, blended toward White when contrast against body is too
    // low (< 1.5×). This ensures E pixels (eyes / accents) are visible even when the
    // secondary color has similar luminance to the primary (e.g. Yellow on Green).
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

    // highlight: ~140% of body clamped to 255. When all channels are already at max
    // (near-white body), simple scaling produces the same color as body. In that case
    // shift toward warm white (reduced blue) so H pixels have a visible warm shimmer.
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

// ── Zone Suffix-to-Archetype Mapping ────────────────────────────────

/// Returns the sprite archetype for a zone enemy suffix.
/// Falls back to the zone default if the suffix is unrecognized.
pub fn archetype_for_suffix(zone_id: u32, suffix: &str) -> SpriteArchetype {
    let s = suffix.to_lowercase();
    let matched = match zone_id {
        1 => match s.as_str() {
            "beetle" | "wasp" | "mantis" => Some(SpriteArchetype::Insect),
            "rabbit" | "boar" | "hare" | "toad" => Some(SpriteArchetype::Quadruped),
            "serpent" => Some(SpriteArchetype::Serpent),
            "grub" => Some(SpriteArchetype::Insect),
            "sprout" => Some(SpriteArchetype::Plant),
            _ => None,
        },
        2 => match s.as_str() {
            "wolf" | "lynx" => Some(SpriteArchetype::Quadruped),
            "spider" | "moth" => Some(SpriteArchetype::Insect),
            "bat" => Some(SpriteArchetype::Avian),
            "treant" => Some(SpriteArchetype::Titan),
            "wisp" => Some(SpriteArchetype::Elemental),
            "hollow" => Some(SpriteArchetype::Horror),
            "shambler" => Some(SpriteArchetype::Plant),
            "wretch" => Some(SpriteArchetype::Humanoid),
            _ => None,
        },
        3 => match s.as_str() {
            "goat" | "ram" => Some(SpriteArchetype::Quadruped),
            "eagle" | "condor" => Some(SpriteArchetype::Avian),
            "golem" | "yeti" | "troll" | "gargoyle" => Some(SpriteArchetype::Titan),
            "harpy" | "bandit" => Some(SpriteArchetype::Humanoid),
            _ => None,
        },
        4 => match s.as_str() {
            "skeleton" | "mummy" | "revenant" | "lich" => Some(SpriteArchetype::Undead),
            "spirit" => Some(SpriteArchetype::Elemental),
            "gargoyle" => Some(SpriteArchetype::Titan),
            "specter" => Some(SpriteArchetype::Horror),
            "shade" => Some(SpriteArchetype::Horror),
            "cultist" => Some(SpriteArchetype::Humanoid),
            "apparition" => Some(SpriteArchetype::Elemental),
            _ => None,
        },
        5 => match s.as_str() {
            "salamander" | "cinderwyrm" => Some(SpriteArchetype::Serpent),
            "phoenix" | "drake" => Some(SpriteArchetype::Avian),
            "imp" | "ashborn" => Some(SpriteArchetype::Humanoid),
            "elemental" | "magmite" => Some(SpriteArchetype::Elemental),
            "hellhound" => Some(SpriteArchetype::Quadruped),
            "infernal" => Some(SpriteArchetype::Horror),
            _ => None,
        },
        6 => match s.as_str() {
            "mammoth" | "glacial" => Some(SpriteArchetype::Titan),
            "wendigo" | "wraith" => Some(SpriteArchetype::Horror),
            "bear" | "moose" => Some(SpriteArchetype::Quadruped),
            "wyrm" => Some(SpriteArchetype::Serpent),
            "banshee" => Some(SpriteArchetype::Elemental),
            "imp" => Some(SpriteArchetype::Humanoid),
            "revenant" => Some(SpriteArchetype::Undead),
            _ => None,
        },
        7 => match s.as_str() {
            "construct" | "golem" | "sentinel" => Some(SpriteArchetype::Titan),
            "guardian" => Some(SpriteArchetype::Humanoid),
            "sprite" | "shard" | "prism" => Some(SpriteArchetype::Elemental),
            "watcher" | "echo" => Some(SpriteArchetype::Horror),
            "crawler" => Some(SpriteArchetype::Insect),
            _ => None,
        },
        8 => match s.as_str() {
            "kraken" => Some(SpriteArchetype::Horror),
            "shark" | "ray" | "lurker" => Some(SpriteArchetype::Aquatic),
            "naga" => Some(SpriteArchetype::Serpent),
            "leviathan" => Some(SpriteArchetype::Titan),
            "siren" => Some(SpriteArchetype::Humanoid),
            "eel" | "hydra" => Some(SpriteArchetype::Serpent),
            "drowned" => Some(SpriteArchetype::Undead),
            _ => None,
        },
        9 => match s.as_str() {
            "griffin" | "roc" | "wyvern" | "pegasus" | "stormhawk" => Some(SpriteArchetype::Avian),
            "djinn" | "sylph" | "zephyr" => Some(SpriteArchetype::Elemental),
            "manticore" => Some(SpriteArchetype::Quadruped),
            "cloudwalker" => Some(SpriteArchetype::Humanoid),
            _ => None,
        },
        10 => match s.as_str() {
            "titan" | "colossus" | "juggernaut" | "breaker" => Some(SpriteArchetype::Titan),
            "lord" | "king" | "champion" | "warlord" | "stormknight" => {
                Some(SpriteArchetype::Humanoid)
            }
            "thunderborn" => Some(SpriteArchetype::Elemental),
            _ => None,
        },
        11 => match s.as_str() {
            "beast" => Some(SpriteArchetype::Quadruped),
            "horror" | "terror" | "abomination" | "rift" | "amalgam" => {
                Some(SpriteArchetype::Horror)
            }
            "fiend" => Some(SpriteArchetype::Humanoid),
            "monster" => Some(SpriteArchetype::Titan),
            "void" => Some(SpriteArchetype::Elemental),
            "remnant" => Some(SpriteArchetype::Undead),
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
    } else if name.contains("revenant") || name.contains("lich") || name.contains("mummy") {
        Some(SpriteArchetype::Undead)
    } else if name.contains("shark") || name.contains("lurker") || name.contains("ray") {
        Some(SpriteArchetype::Aquatic)
    } else if name.contains("shambler") || name.contains("sprout") {
        Some(SpriteArchetype::Plant)
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
    // If the suffix matched a known zone enemy, use that archetype
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

/// Checks if a suffix is a known zone enemy suffix.
fn suffix_is_known_for_zone(zone_id: u32, suffix: &str) -> bool {
    let s = suffix.to_lowercase();
    match zone_id {
        1 => matches!(
            s.as_str(),
            "beetle"
                | "rabbit"
                | "wasp"
                | "boar"
                | "serpent"
                | "grub"
                | "hare"
                | "toad"
                | "mantis"
                | "sprout"
        ),
        2 => matches!(
            s.as_str(),
            "wolf"
                | "spider"
                | "bat"
                | "treant"
                | "wisp"
                | "lynx"
                | "moth"
                | "hollow"
                | "shambler"
                | "wretch"
        ),
        3 => matches!(
            s.as_str(),
            "goat"
                | "eagle"
                | "golem"
                | "yeti"
                | "harpy"
                | "ram"
                | "condor"
                | "troll"
                | "bandit"
                | "gargoyle"
        ),
        4 => matches!(
            s.as_str(),
            "skeleton"
                | "mummy"
                | "spirit"
                | "gargoyle"
                | "specter"
                | "revenant"
                | "shade"
                | "lich"
                | "cultist"
                | "apparition"
        ),
        5 => matches!(
            s.as_str(),
            "salamander"
                | "phoenix"
                | "imp"
                | "drake"
                | "elemental"
                | "cinderwyrm"
                | "ashborn"
                | "magmite"
                | "hellhound"
                | "infernal"
        ),
        6 => matches!(
            s.as_str(),
            "mammoth"
                | "wendigo"
                | "wraith"
                | "bear"
                | "wyrm"
                | "moose"
                | "banshee"
                | "imp"
                | "glacial"
                | "revenant"
        ),
        7 => matches!(
            s.as_str(),
            "construct"
                | "guardian"
                | "sprite"
                | "watcher"
                | "golem"
                | "shard"
                | "crawler"
                | "prism"
                | "sentinel"
                | "echo"
        ),
        8 => matches!(
            s.as_str(),
            "kraken"
                | "shark"
                | "naga"
                | "leviathan"
                | "siren"
                | "eel"
                | "ray"
                | "lurker"
                | "drowned"
                | "hydra"
        ),
        9 => matches!(
            s.as_str(),
            "griffin"
                | "djinn"
                | "sylph"
                | "roc"
                | "wyvern"
                | "zephyr"
                | "pegasus"
                | "manticore"
                | "cloudwalker"
                | "stormhawk"
        ),
        10 => matches!(
            s.as_str(),
            "titan"
                | "colossus"
                | "lord"
                | "king"
                | "champion"
                | "warlord"
                | "juggernaut"
                | "thunderborn"
                | "stormknight"
                | "breaker"
        ),
        11 => matches!(
            s.as_str(),
            "beast"
                | "horror"
                | "fiend"
                | "terror"
                | "monster"
                | "abomination"
                | "void"
                | "rift"
                | "amalgam"
                | "remnant"
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
