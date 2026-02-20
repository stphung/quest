//! Enemy sprite constant data: legacy ASCII sprites, half-block pixel sprites,
//! archetype mapping tables, and zone suffix lookups.
//!
//! Pure data — no rendering logic, no frame access.

use super::enemy_sprites::{EnemySprite, PixelSprite, SpriteArchetype};

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
pub fn zone_default_archetype(zone_id: u32) -> SpriteArchetype {
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
pub fn dungeon_generic_archetype(suffix: &str) -> SpriteArchetype {
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
pub fn boss_name_archetype(boss_name: &str) -> Option<SpriteArchetype> {
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

/// Checks if a suffix is a known zone enemy suffix.
pub fn suffix_is_known_for_zone(zone_id: u32, suffix: &str) -> bool {
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
pub fn is_generic_suffix(suffix: &str) -> bool {
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
