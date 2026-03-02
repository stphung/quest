//! Pixel sprite definitions for fracture zone chapters 4-6.
//! Regal, Automaton (Hollow Throne), Spectral, Resonant (Wailing Reach), Primordial, Fossilized (Origin Wound).

use super::enemy_sprites::PixelSprite;

// REGAL: Ancient crowned guardian, processional sentinel. Tall upright posture,
// floating cracked crown (H highlights), robed/cloaked lower body, ceremonial staff,
// ornate shoulder pauldrons, glowing eyes of ancient authority.
// 18 pixels wide, 18 pixels tall -> 9 cell rows
pub const PIXEL_REGAL: PixelSprite = PixelSprite {
    rows: &[
        "..H.HDHDHD.H......",
        "...DHHHHHHD.......",
        "...DBBBBBD........",
        "...DBEEBBD........",
        "...DBBBSBD........",
        "..HDDDDDDDDH......",
        ".DBBHBBBBHBBBD....",
        "DBBBBBBBBBBBBBD...",
        ".DBBBBBBBBBBBDBD..",
        ".DSBBBBBBBBSDDBBD.",
        "..DBBBBBBBBD..DBD.",
        "..DSBBBBBSBD..DD..",
        "..DBBBHBBBD.......",
        "..DSBBBBBSD.......",
        "..DBBBBBBD........",
        ".DSBBBBBSBD.......",
        ".DBBBBBBBBD.......",
        ".DDDDDDDDDDD......",
    ],
};

// AUTOMATON: Clockwork construct, mechanical guardian. Geometric body segments,
// gear-like patterns, blade-arm and shield-arm, boxy head with single eye-slit,
// rigid vertical posture, sharp corners. Ancient robot/clockwork soldier.
// 18 pixels wide, 18 pixels tall -> 9 cell rows
pub const PIXEL_AUTOMATON: PixelSprite = PixelSprite {
    rows: &[
        "....DDDDDDDD......",
        "....DSSBBSSD......",
        "....DEEEEEED......",
        "....DDDDDDDD......",
        ".DDDDBBBBBDDDD....",
        "DBBBDSBDBDSDBBBD..",
        "DBBHDBBSBDDBBHD...",
        "DDDDDBBBBDDDDDD...",
        "..DDDBSBSBDDDD....",
        "..DBDDBBBDDDBBD...",
        "..DBDDSHSDDDBSD...",
        "..DBDDBBBBDDBBD...",
        "..DDDDBBBBDDDDD...",
        "...DDBBD.DBBDD....",
        "...DDBSD.DSBDD....",
        "...DDBBD.DBBDD....",
        "..DDDSSD.DSSDD....",
        "..DDDDD..DDDDD....",
    ],
};

// SPECTRAL: Translucent sound-based entity. Made of vibrating waves/distortion.
// Heavy H and S suggesting transparency, irregular wavering edges, scattered border
// pixels, dense central core, suggestion of sound waves, no legs -- floating with
// trailing wisps. Overall shape suggests a frozen scream / open maw.
// 18 pixels wide, 18 pixels tall -> 9 cell rows
pub const PIXEL_SPECTRAL: PixelSprite = PixelSprite {
    rows: &[
        ".H.....HH.....H...",
        "..H..DHHHD..H.....",
        "...DHHHHHHHD......",
        "..DHHEHHHEHHD.....",
        "..DHHHHHHHHHHD....",
        ".DHBHHHHHHHBHD....",
        ".DHBBHHHHHBBHD.H..",
        "DHBBBHHHHBBBBHD...",
        ".DHBBBBBBBBHD.....",
        "..DHBBBBBBHD.H....",
        ".H.DHBBBHD........",
        "....DHHHD..H......",
        ".....DHHD.........",
        "..H...DHD.........",
        "......DHD.H.......",
        ".......HD.........",
        "........H.........",
        "..................",
    ],
};

// RESONANT: Creature made of crystallized/solidified sound. Concentric ring patterns,
// tuning-fork-like paired upper protrusions, H highlights radiating outward,
// more geometric than Spectral, stands on a point like a spinning top,
// narrow bottom, wide middle, bell-shaped lower body.
// 18 pixels wide, 18 pixels tall -> 9 cell rows
pub const PIXEL_RESONANT: PixelSprite = PixelSprite {
    rows: &[
        ".DBD........DBD...",
        ".DBBD......DBBD...",
        ".DBSD......DSBD...",
        ".DBBD......DBBD...",
        "..DBBDDDDDDBBBD...",
        "..DBBBBBBBBBBD....",
        ".HDBDBBBBBBDBDH...",
        "HDBBDBBBBBBDBBDH..",
        "DBBBDBBBBBBDBBBDH.",
        ".DBBBBBBBBBBBBBD..",
        "..DBBHDBBBDHBBD...",
        "...DBBDBBBDBBD....",
        "....DBBBBBBD......",
        ".....DBBBD........",
        "......DBBD........",
        "......DBSD........",
        ".......DD.........",
        "..................",
    ],
};

// PRIMORDIAL: Massive root-like ancient entity -- the first creature.
// Thick trunk-like central body with branching root appendages, deep ancient eyes
// set in bark-like texture, crown of root tendrils reaching upward, lower body
// splits into root-legs spreading wide.
// 20 pixels wide (special exception), 18 pixels tall -> 9 cell rows
pub const PIXEL_PRIMORDIAL: PixelSprite = PixelSprite {
    rows: &[
        ".DBD..DBBD..DBD.....",
        "DBBD.DBSSBD.DBBD....",
        ".DBBDBBBBBBDBBD.....",
        "..DBBBEBBEBBBBD.....",
        "..DBBBBBBBBBBBD.....",
        ".DBBSBBBBBBBSBBD....",
        ".DBBBBHBBHBBBBD.....",
        "DBBSBBBBBBBBSBBBD...",
        "DBBBBBBBBBBBBBBBBD..",
        ".DBBSBBBBBBBSBBD....",
        "..DBBBHBBHBBBD......",
        "..DBBSBBBBBSBD......",
        "..DBBBBBBBBBD.......",
        ".DBBD.DBBD.DBBBD....",
        "DBBSD.DBSD..DBSBD...",
        "DBBBD.DBBD..DBBBD...",
        "DSSSD.DSSD...DSSSD..",
        "DDDDD.DDDD...DDDDD..",
    ],
};

// FOSSILIZED: Petrified ancient creature, half-turned to stone. Asymmetric: left
// side is organic (normal B body, limb shapes) while right side is stone (heavy S/D,
// angular, geometric). Visible fossil bone impressions in the stone half (E chars
// for bone outlines). Crack line running diagonally where organic meets stone.
// Hunched/stooped posture suggesting weight of ages.
// 18 pixels wide, 18 pixels tall -> 9 cell rows
pub const PIXEL_FOSSILIZED: PixelSprite = PixelSprite {
    rows: &[
        "....DDDDDDDD......",
        "...DBBBBDSSD......",
        "...DBEEBDESSD.....",
        "...DBBBBDDSSD.....",
        "....DDDDDDDD......",
        "..DBBBBDDSSSSD....",
        ".DBBBHBDDSSESSD...",
        ".DBBBBDDDSSESSD...",
        "..DBBDDDDSSSSD....",
        "..DBDDDDDSSD......",
        "..DBBDDDSSSD......",
        "..DBSDDDDSSD......",
        "...DDDDDDSSD......",
        "..DBD..DDSSD......",
        ".DBBD..DDSSD......",
        ".DBSD..DDSSDD.....",
        "..DDD..DDSSD......",
        ".......DDDDD......",
    ],
};
