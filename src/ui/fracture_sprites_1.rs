//! Pixel sprite definitions for fracture zone chapters 1-3.
//! Fractured, Molten (Red Fault), Crystalline, Mirrored (Mirror Scar), Abyssal, Devouring (Black Mouth).

use super::enemy_sprites::PixelSprite;

// FRACTURED: Cracked bipedal stone golem with magma veins. Asymmetric — left shoulder higher.
// S chars form crack lines through the B body. H highlights where magma seeps through.
// 18 pixels wide, 18 pixels tall -> 9 cell rows
pub const PIXEL_FRACTURED: PixelSprite = PixelSprite {
    rows: &[
        "....DDDDD.........",
        "...DBBSBBD........",
        "...DBEBBED........",
        "...DBSBBBD........",
        "..DDDSBDDDDDD.....",
        ".DBBHSBBBBBBBDD...",
        "DBBBSHBBSBBBBBBD..",
        "DBBHBBSBBBBHBBBD..",
        ".DBBSBBBSBBBBBD...",
        ".DBHSBBBBBSBBD....",
        "..DBBSBHSBBBD.....",
        "..DBBBSBBBBBD.....",
        "..DBBSD.DSBBD.....",
        "..DBHD..DBHBD.....",
        "..DBSD..DSBBD.....",
        "..DBBD..DBBSD.....",
        "..DSSD..DDSSD.....",
        "..DDDD...DDDD.....",
    ],
};

// MOLTEN: Amorphous lava creature, no legs — pools at the base. Dripping protrusions.
// H highlights for hot spots, S for cooling crusted areas. Blobby, organic shape.
// 18 pixels wide, 18 pixels tall -> 9 cell rows
pub const PIXEL_MOLTEN: PixelSprite = PixelSprite {
    rows: &[
        "......H...........",
        ".....DHD.....H....",
        "....DHHBBD..DHD...",
        "...DHHHHBBBD.D....",
        "..DHHEEHBBBBD.....",
        "..DHHBBHBBBBD.....",
        ".DBHHHBBHSBBD.....",
        ".DBBHBBBBBSBD.D...",
        "..DBBHBBBSBD.DBD..",
        "..DBBBHBBBD..DSD..",
        "...DBBBSBBD...D...",
        "..DBBBBHBBBD......",
        ".DBSBBBBBSBBD.....",
        "DBSSBBHBBSSBD.....",
        "DBSBBBBBBBSSBD....",
        "DSSBBHBBBBSSD.....",
        "DSSSSSBBSSSSD.....",
        "DDDDDDDDDDDDD.....",
    ],
};

// CRYSTALLINE: Angular geometric crystal being. Sharp faceted outlines. Tall narrow silhouette.
// Diamond-shaped head, spike protrusions. Alternating B/H for refraction. Clean, geometric.
// 18 pixels wide, 18 pixels tall -> 9 cell rows
pub const PIXEL_CRYSTALLINE: PixelSprite = PixelSprite {
    rows: &[
        "........DH........",
        ".......DHBD.......",
        "......DHBHBD......",
        ".....DHBEEBHD.....",
        "......DHBHBD......",
        ".....DDHBHBDD.....",
        "..DH.DHBHBHBD.HD..",
        "..DBDDBHBHBDDDHD..",
        "...DDDBBHBBDDDD...",
        "......DBHBD.......",
        "......DBHBD.......",
        ".....DHBHBHD......",
        ".....DBHBHBD......",
        "......DBHBD.......",
        "......DBHBD.......",
        ".....DDHBHDD......",
        "....DD..D..DD.....",
        "...DD.......DD....",
    ],
};

// MIRRORED: Reflective doppelganger, PERFECTLY left-right symmetric. Eerie smooth outline.
// Alternating B/H vertical stripes for mirror surface. Featureless face — only 2 E eyes.
// 18 pixels wide, 18 pixels tall -> 9 cell rows
pub const PIXEL_MIRRORED: PixelSprite = PixelSprite {
    rows: &[
        ".....DDDDDDDD.....",
        "....DBHBHBHBBD....",
        "....DHBEEBHBD.....",
        "....DBHBHBHBBD....",
        ".....DDDDDDD......",
        "...DDBHBHBHBDD....",
        "..DBHBHBHBHBHBD...",
        "..DHBHBHBHBHBHD...",
        "..DBHBHBHBHBHBD...",
        "..DHBHBHBHBHBHD...",
        "...DBHBHBHBHBD....",
        "...DHBHBHBHBHD....",
        "....DBHBHBHBD.....",
        "....DBHD.DHBD.....",
        "....DBHD.DHBD.....",
        "....DBHD.DHBD.....",
        "....DDHD.DHDD.....",
        "....DDDD.DDDD.....",
    ],
};

// ABYSSAL: Dark void entity. Large gaping maw as central feature. Mostly D and S (very dark).
// Wispy dissolving edges. Tentacle appendages. Hollow interior — more negative space than body.
// 18 pixels wide, 18 pixels tall -> 9 cell rows
pub const PIXEL_ABYSSAL: PixelSprite = PixelSprite {
    rows: &[
        ".D............D...",
        "..DD...DD...DD....",
        "...DDDDDDDDD......",
        "..DD..DEED..DD....",
        ".DD.DDEEEDDD.DD...",
        ".D.DDE...EEDD.D...",
        "DD.DD.....DDD.DD..",
        "DD.DDE...EEDD..DD.",
        ".D..DDDDDDD..D.D..",
        ".DD..DDSDD...DD...",
        "..DD..DDD..DD.....",
        "...DDDDDDDDD......",
        "..DD.DDDDD.DD.....",
        ".DD...DDD...DD....",
        ".D.....D.....D....",
        "DD.....D......D...",
        ".D.............D..",
        "................D.",
    ],
};

// DEVOURING: Multi-mawed horror with mouths/teeth scattered across body. Bulging, wide, low.
// Multiple E eye-mouths at different spots. H highlights as teeth. Oozing S texture. Hunched.
// 18 pixels wide, 18 pixels tall -> 9 cell rows
pub const PIXEL_DEVOURING: PixelSprite = PixelSprite {
    rows: &[
        "..................",
        "......DDDDD.......",
        "....DDBSBSBDD.....",
        "...DBEHHHEBBBD....",
        "..DBBSSBSSBBBD....",
        ".DDBBBBBBBBBDD....",
        "DBBEHHEBBSBBBBDD..",
        "DBBBSSBBEHHEBBBBD.",
        "DBBBBBBBBSSBBBBBD.",
        "DBSBBEHHEBBBSBBD..",
        "DBBBSSSBBBBBBBBD..",
        "DBBBBBBBBHHEBBBD..",
        ".DBSBBBBBSSBBBD...",
        ".DDBBBSBBBBBDD....",
        "..DDBBBBBBBDD.....",
        "..DBSD.D.DBSD.....",
        "..DBBD.D.DBBD.....",
        "..DDDD.D.DDDD.....",
    ],
};
