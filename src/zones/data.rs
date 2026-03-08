//! Zone and subzone data definitions.

#![allow(dead_code)]

use std::sync::LazyLock;

/// Represents a zone in the game world.
#[derive(Debug, Clone)]
pub struct Zone {
    pub id: u32,
    pub name: &'static str,
    /// Short flavor text describing the zone's atmosphere and history.
    pub description: &'static str,
    pub subzones: Vec<Subzone>,
    pub prestige_requirement: u32,
    pub min_level: u32,
    pub max_level: u32,
    /// If true, completing this zone requires forging a legendary weapon (see issue #20)
    pub requires_weapon: bool,
    /// Name of the legendary weapon for this zone (if requires_weapon is true)
    pub weapon_name: Option<&'static str>,
}

/// Represents a subzone within a zone.
#[derive(Debug, Clone)]
pub struct Subzone {
    pub id: u32,
    pub name: &'static str,
    /// Short flavor text describing this subzone.
    pub description: &'static str,
    pub depth: u32,
    pub boss: SubzoneBoss,
}

/// Boss guarding the exit of a subzone.
#[derive(Debug, Clone)]
pub struct SubzoneBoss {
    pub name: &'static str,
    pub is_zone_boss: bool,
}

/// All zones in the game, initialized once on first access.
static ALL_ZONES: LazyLock<Vec<Zone>> = LazyLock::new(|| {
    vec![
        // Tier 1: Nature's Edge (P0) - 3 subzones each
        Zone {
            id: 1,
            name: "Meadow",
            description: "Rolling grasslands where wildflowers hide teeth. Many adventurers begin here. Fewer leave than you'd think.",
            prestige_requirement: 0,
            min_level: 1,
            max_level: 10,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Sunny Fields",
                    description: "Tall grass sways in a warm breeze. The buzzing isn't all bees.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Field Guardian",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Overgrown Thicket",
                    description: "Thorned vines knot overhead, swallowing the path. Something breathes in the undergrowth.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Thicket Horror",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Mushroom Caves",
                    description: "Bioluminescent spores drift through damp air. The fungus down here has a will of its own.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Sporeling Queen",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 2,
            name: "Dark Forest",
            description: "Ancient trees twist toward a sky they'll never reach. The canopy hasn't let sunlight through in centuries.",
            prestige_requirement: 0,
            min_level: 10,
            max_level: 25,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Forest Edge",
                    description: "The treeline stands like a wall. Past it, the birdsong stops.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Alpha Wolf",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Twisted Woods",
                    description: "The trees grow at wrong angles here, as if recoiling from something deeper in.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Corrupted Treant",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Spider's Hollow",
                    description: "Silk threads catch what little light remains. The webs are older than the trees.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Broodmother Arachne",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // Tier 2: Civilization's Remnants (P5) - 3 subzones each
        Zone {
            id: 3,
            name: "Mountain Pass",
            description: "A trade route abandoned after the last war. Bandits and worse things have claimed the heights.",
            prestige_requirement: 5,
            min_level: 25,
            max_level: 40,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Rocky Foothills",
                    description: "Loose scree and overturned wagons mark where the road gave out.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Bandit King",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Frozen Peaks",
                    description: "The wind cuts like a blade up here. Frost-rimed bones jut from the snow.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Ice Giant",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Dragon's Perch",
                    description: "Claw marks score the clifftop stone. The air reeks of ozone and old fire.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Frost Wyrm",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 4,
            name: "Ancient Ruins",
            description: "A civilization that mastered death and was consumed by it. Their wards still flicker in the dark.",
            prestige_requirement: 5,
            min_level: 40,
            max_level: 55,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Outer Sanctum",
                    description: "Crumbling pillars frame a courtyard where the dead stand guard over nothing.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Skeleton Lord",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Sunken Temple",
                    description: "The floor has collapsed into flooded corridors. Pale lights drift beneath the water.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Spectral Guardian",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Sealed Catacombs",
                    description: "Someone sealed these tombs from the outside. The scratching on the inner walls never stopped.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Lich King's Shade",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // Tier 3: Elemental Forces (P10) - 4 subzones each
        Zone {
            id: 5,
            name: "Volcanic Wastes",
            description: "The earth split open here long ago and never healed. Ash falls like grey snow on a land that remembers fire.",
            prestige_requirement: 10,
            min_level: 55,
            max_level: 70,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Scorched Badlands",
                    description: "Cracked earth radiates heat. The horizon shimmers and lies.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Ash Walker Chief",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Lava Rivers",
                    description: "Molten channels carve the landscape into shrinking islands of stone.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Magma Serpent",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Obsidian Fortress",
                    description: "A stronghold forged from cooled lava. Its builders worship the thing that lives beneath.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Fire Giant Warlord",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Magma Core",
                    description: "The heart of the volcano. The heat here doesn't just burn. It thinks.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Infernal Titan",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 6,
            name: "Frozen Tundra",
            description: "An endless white silence where the cold has a patience that outlasts everything.",
            prestige_requirement: 10,
            min_level: 70,
            max_level: 85,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Snowbound Plains",
                    description: "The snow is deep enough to swallow a horse. Howls carry for miles.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Dire Wolf Alpha",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Glacier Maze",
                    description: "Walls of blue ice shift when you aren't looking. The maze remembers its last visitors.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Ice Wraith Lord",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Frozen Lake",
                    description: "The ice is clear enough to see the bottom. Something down there sees you back.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Lake Horror",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Permafrost Tomb",
                    description: "A king was buried here in ice so cold it stopped time itself. He's still dreaming.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Frozen One",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // Tier 4: Hidden Depths (P15) - 4 subzones each
        Zone {
            id: 7,
            name: "Crystal Caverns",
            description: "The crystals sing at frequencies that rearrange thought. Miners went in for gems and came out as prophets.",
            prestige_requirement: 15,
            min_level: 85,
            max_level: 100,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Glittering Tunnels",
                    description: "Every surface reflects your torchlight a thousand times. Not all the reflections are yours.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Gem Golem",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Prismatic Halls",
                    description: "Light bends through crystal columns, splitting into colors that have no name.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Prism Elemental",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Resonance Depths",
                    description: "The hum of the crystals grows deafening. Step wrong and the harmonics will shatter bone.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Echo Wraith",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Heart Crystal",
                    description: "A single crystal the size of a cathedral pulses at the center of the earth.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Crystal Colossus",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 8,
            name: "Sunken Kingdom",
            description: "A drowned empire that refused to die. The sea took their land but not their pride, or their army.",
            prestige_requirement: 15,
            min_level: 100,
            max_level: 115,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Coral Gardens",
                    description: "Living coral has overgrown marble streets. The anemones sway toward warm blood.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Merfolk Warlord",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Drowned Streets",
                    description: "Barnacle-crusted buildings line avenues where fish swim through broken windows.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Drowned Admiral",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Abyssal Palace",
                    description: "The pressure is crushing. The palace doors stand open, as if expecting guests.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Pressure Beast",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Throne of Tides",
                    description: "A throne of black coral sits at the lowest point of the world. Its occupant has been waiting.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Drowned King",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // Tier 5: Ascending (P20) - 4 subzones each
        Zone {
            id: 9,
            name: "Floating Isles",
            description: "Shattered fragments of earth hang in an open sky. The ground fell away long ago. Only the stubborn parts remain.",
            prestige_requirement: 20,
            min_level: 115,
            max_level: 130,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Cloud Docks",
                    description: "Rotting airship moorings creak in the wind. The crews are long gone. The harpies aren't.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Harpy Matriarch",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Sky Bridges",
                    description: "Chains of ancient iron link the islands. The wind never stops trying to cut them.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Wind Elemental Lord",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Stormfront",
                    description: "Lightning arcs between the isles in rhythms that almost sound like language.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Storm Drake",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Eye of the Storm",
                    description: "Perfect stillness at the center of violence. The sky above is impossibly clear.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Tempest Lord",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 10,
            name: "Storm Citadel",
            description: "A fortress built from living lightning, older than the sky itself. It was not made to be entered.",
            prestige_requirement: 20,
            min_level: 130,
            max_level: 150,
            requires_weapon: true,
            weapon_name: Some("Stormbreaker"),
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Lightning Fields",
                    description: "The ground crackles with each step. Static pulls at your blade like a living thing.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Spark Colossus",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Thunder Halls",
                    description: "Every footfall echoes as thunder. The Citadel's knights march in endless patrols.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Storm Knight Commander",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Generator Core",
                    description: "The machine that powers the storm. It has been running since before memory.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Core Warden",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Apex Spire",
                    description: "The highest point in the world. The storm is not weather. It is a will.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Undying Storm",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // Zone 11: The Expanse - Infinite post-game zone (unlocked by StormsEnd achievement)
        Zone {
            id: 11,
            name: "The Expanse",
            description: "Beyond the storm lies what was always there. Raw, unformed reality stretching past the edges of the world.",
            prestige_requirement: 25,
            min_level: 150,
            max_level: u32::MAX,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Void's Edge",
                    description: "The last solid ground before everything dissolves into possibility.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Void Sentinel",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Eternal Storm",
                    description: "A storm with no beginning and no end. It was here before the Citadel learned to harness it.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Tempest Incarnate",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Abyssal Rift",
                    description: "A wound in reality that goes down forever. Things climb out of it that have no names yet.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Rift Behemoth",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "The Endless",
                    description: "There is nothing here but you and the infinite. It is enough.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Avatar of Infinity",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // ── Chapter 1: The Red Fault (Zones 12-14) ──────────────────────
        Zone {
            id: 12,
            name: "Splintered Rim",
            description: "The continent's wound begins here — a shattered ridge where heat and ash pour upward through fractured stone.",
            prestige_requirement: 50,
            min_level: 165,
            max_level: 180,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Rimwatch Scree",
                    description: "Loose rubble slides toward a glowing crack. The air tastes of iron and burnt earth.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Rimclaw Stalker",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Ashsplit Road",
                    description: "A trade road split clean in half by the fault. Ash drifts across the gap like grey snow.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Cinder Hound",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Broken Survey",
                    description: "An abandoned surveyor's camp teeters on the edge of a thermal vent. The instruments still spin.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Searback Ram",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Bloodglass Shelf",
                    description: "Volcanic glass the color of arterial blood juts from the cliff face. It cuts anything that touches it.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Shard-Tusk Brute",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "The First Fissure",
                    description: "The original crack in the world. Red light pulses from below in a slow, steady rhythm.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "Fault Stalker",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 13,
            name: "Ember Ravine",
            description: "A deep canyon where the walls glow with trapped heat. Embers drift upward like inverse rain.",
            prestige_requirement: 50,
            min_level: 180,
            max_level: 195,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Coalwind Narrows",
                    description: "Hot gusts funnel through tight passages, carrying sparks and the smell of burning coal.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Maw of Soot",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Smelter Steps",
                    description: "Carved steps descend into a natural forge. The stone itself has been tempered by centuries of heat.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Crucible Knight",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Cauterized Span",
                    description: "A bridge of fused rock crosses the ravine. The wound beneath it was sealed by fire, not healed.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Rift Colossus",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Scarforge Hollow",
                    description: "A cavernous chamber where molten metal pools in natural crucibles. Something has been forging here.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Scarbound",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "Ember Ravine",
                    description: "The heart of the ravine. The embers here are not sparks — they are the last breaths of something immense.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "Cinder Maw",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 14,
            name: "Heart of the Fault",
            description: "The deepest reach of the Red Fault, where the world's blood flows openly and the heat has a will.",
            prestige_requirement: 50,
            min_level: 195,
            max_level: 210,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Artery Bridge",
                    description: "A natural bridge spanning a river of magma. The stone pulses with geothermal rhythm.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Veinbreaker",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Magma Choir",
                    description: "The lava sings here — harmonic frequencies that resonate through bone and blade alike.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Ash Cantor",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Severed Descent",
                    description: "A vertical shaft where the fault has cut clean through bedrock. The walls weep molten tears.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Pyre Warden",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Coreglass Throne",
                    description: "A throne of crystallized magma sits at the fault's deepest point, radiating terrible heat.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Rupture Regent",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "Heart of the Fault",
                    description: "The wound's origin. The earth's core is visible through the crack, red and furious and alive.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The Red Tyrant",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // ── Chapter 2: The Mirror Scar (Zones 15-17) ────────────────────
        Zone {
            id: 15,
            name: "Shard Fields",
            description: "Reality has fractured into a plain of mirror shards. Reflections move independently of their sources.",
            prestige_requirement: 75,
            min_level: 210,
            max_level: 225,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Split Horizon",
                    description: "The horizon line has doubled. Two skies compete for the same space.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Glass Hound",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Shard Drift",
                    description: "Floating fragments of mirrored earth drift slowly, each reflecting a different version of the landscape.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Prism Jackal",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Mirror Furrows",
                    description: "Deep grooves scored into reflective ground. Walk the furrows and you see only yourself, endlessly.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Sky Echo",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "White Fracture",
                    description: "A blinding crack in the landscape where all light converges. The fracture hums with static energy.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Faceted",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "Shard Fields",
                    description: "The field stretches to every horizon. Every shard reflects a world that almost was.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "Prism Widow",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 16,
            name: "Refraction Steps",
            description: "Impossible geometry carved from reflected light. Staircases fold back on themselves at angles that shouldn't exist.",
            prestige_requirement: 75,
            min_level: 225,
            max_level: 240,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Bent Causeway",
                    description: "A road that curves in dimensions the eye can't track. Walking straight leads you in circles.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Angle Serpent",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Parallax Gate",
                    description: "A gate that exists in two places at once. Step through and you're not sure which side you've emerged on.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Twin Sight",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Reflected Climb",
                    description: "You climb upward but the ground rises to meet you. Gravity is a suggestion here, not a law.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "The Repeater",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Lightfall Court",
                    description: "A courtyard where light falls like water, pooling in corners and flowing down steps.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Shard Marshal",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "Refraction Steps",
                    description: "The final staircase. Each step refracts you into a slightly different version of yourself.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The Reflection Engine",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 17,
            name: "Hall of Second Suns",
            description: "A sanctuary built around a mirrored sun that was never meant to exist. Its light reveals truths and lies alike.",
            prestige_requirement: 75,
            min_level: 240,
            max_level: 255,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Solar Debris",
                    description: "Fragments of captured sunlight orbit a central point, casting impossible shadows.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Helio Wraith",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "False Noon",
                    description: "It is always midday here. The light comes from everywhere and nowhere. There are no shadows.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "The Doubled King",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Mirror Nave",
                    description: "The central hall of a cathedral made entirely of mirrors. Every surface shows a congregation that isn't there.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Sunshard Titan",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Witness Gallery",
                    description: "A gallery where your reflection watches you from behind the glass. It does not always copy your movements.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Faceless Chorus",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "Hall of Second Suns",
                    description: "Two suns burn at the hall's apex. One is real. The other remembers being real. Neither will dim first.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The Many-Faced Witness",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // ── Chapter 3: The Black Mouth (Zones 18-20) ────────────────────
        Zone {
            id: 18,
            name: "Ashen Verge",
            description: "The edge of the final wound. Ash covers everything in a silence that feels permanent.",
            prestige_requirement: 100,
            min_level: 255,
            max_level: 270,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Charline Flats",
                    description: "A vast flat expanse of charcoal and bone dust. Nothing grows. Nothing has grown for a very long time.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Gravewing",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Gloam Ditch",
                    description: "A trench where the light dims to a perpetual twilight. The ditch breathes cold air upward.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Ash Revenant",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Burnt Procession",
                    description: "A road lined with carbonized statues. They were walking somewhere when the fire found them.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Night Forger",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Veil of Cinders",
                    description: "A curtain of falling ash so thick it obscures all sight. Things move behind it, vast and patient.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Last Pyre",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "Ashen Verge",
                    description: "The last step before the darkness. The ash here is warm, as if something enormous exhaled it moments ago.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "Hollow Giant",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 19,
            name: "Throat of the World",
            description: "The land slopes inward here, as if the earth is swallowing itself. The darkness ahead has weight.",
            prestige_requirement: 100,
            min_level: 270,
            max_level: 285,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Mawgate Steps",
                    description: "Stone steps descend into a throat-like passage. The walls are smooth and organic.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Toothwarden",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Windpipe Hollow",
                    description: "A tunnel that breathes. Air rushes in and out in long, slow cycles. The walls contract.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Black-Lung Behemoth",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Devourer's Span",
                    description: "A bridge of bone and calcified muscle spanning a void that moves. Something below is digesting.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "The Sable Herd",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Gullet Court",
                    description: "A chamber that was once a throne room, now dissolved into biological architecture. The court serves a new master.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Horizon Eater",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "Throat of the World",
                    description: "The deepest point of the passage. The world's throat is open and something ancient waits at the bottom.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "Night Sovereign",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 20,
            name: "The Black Mouth",
            description: "The surface has stopped merely breaking. It has opened wide enough to show hunger.",
            prestige_requirement: 100,
            min_level: 285,
            max_level: 300,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Lip of Unmaking",
                    description: "The rim of the final abyss. Reality frays at the edges here, dissolving into static and void.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "The First Hunger",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Void Saliva Falls",
                    description: "Dark matter cascades downward in slow, viscous sheets. It dissolves anything it touches.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Crawlfather",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Jawbone Causeway",
                    description: "A path built from fossilized teeth the size of buildings. The jaw they belong to has no skull.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "The Unlit Colossus",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Mouth Chapel",
                    description: "A cathedral grown from living darkness. The congregation here worships the act of consumption itself.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Choir Below",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "The Black Mouth",
                    description: "The end of all paths. The mouth is open, patient, and endless. It has been waiting since before the world.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The Mouth Unending",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // ── Chapter 4: The Hollow Throne (Zones 21-23) ─────────────────
        Zone {
            id: 21,
            name: "Sunken Processional",
            description: "A grand ceremonial road descending into a buried kingdom. The walls are crystallized amber. Faded grandeur preserved in perfect silence.",
            prestige_requirement: 150,
            min_level: 300,
            max_level: 315,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Pilgrim's Descent",
                    description: "A spiraling road worn smooth by countless feet that walked it before the world above existed.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Processional Sentinel",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Amber Gallery",
                    description: "The walls are thick with fossilized amber. Shapes move inside it \u{2014} not insects, but people.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Entombed Warden",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Candlebone Walk",
                    description: "Pillars of luminescent bone light the way. They have burned for longer than living memory.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Tallow Knight",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Crown Gate",
                    description: "A gate carved from a single gemstone the size of a cathedral. It is open. It was never meant to be.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Gate Colossus",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "Sunken Processional",
                    description: "The road ends at a vast courtyard. The sky above is stone. The silence is absolute.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The Stone Procession",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 22,
            name: "The Pale Archive",
            description: "Libraries of crystallized knowledge carved into burial niches. Each niche holds a crystal tablet instead of a body. The guardians here protect not from invasion but from understanding.",
            prestige_requirement: 150,
            min_level: 315,
            max_level: 330,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Index Catacombs",
                    description: "Endless shelves carved into burial niches. Each holds a crystal tablet instead of a body.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Scroll Wraith",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Hall of Sealed Words",
                    description: "Words too dangerous to speak are preserved here in solid form. They hum.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "The Censor",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Theorem Vault",
                    description: "Mathematical proofs have crystallized into physical objects. Some of them are wrong. Those ones move.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Paradox Construct",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "The Forbidden Index",
                    description: "The section that was locked when the kingdom still lived. The lock is broken now.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Memory Eater",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "The Pale Archive",
                    description: "The central reading room. The tablets here contain the kingdom's final entry. It is one word, repeated.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The Archivist Eternal",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 23,
            name: "The Hollow Throne",
            description: "The seat of power. The palace is intact but empty. Whatever sat here left willingly \u{2014} or was removed. The room defends its throne as though something still sits there.",
            prestige_requirement: 150,
            min_level: 330,
            max_level: 345,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Antechamber of Echoes",
                    description: "A waiting room where visitors once stood. Their conversations still play on loop.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Echo Warden",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Council of Dust",
                    description: "A round table where advisors sat. Their chairs hold impressions. Their dust holds opinions.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "The Dustbound Chancellor",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Royal Causeway",
                    description: "A bridge of polished obsidian leading to the inner sanctum. It reflects nothing.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Void Mirror Guardian",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Coronation Steps",
                    description: "Steps leading to the throne. Each is engraved with a name. The final name has been scratched out.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Crowned Absence",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "The Hollow Throne",
                    description: "The throne is empty but the room defends it as though something still sits there.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The Hollow Sovereign",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // ── Chapter 5: The Wailing Reach (Zones 24-26) ─────────────────
        Zone {
            id: 24,
            name: "The Stillborn Sea",
            description: "An underground ocean that never came alive. No waves, no life, no temperature. Walking on its surface is like walking on solid grief.",
            prestige_requirement: 200,
            min_level: 345,
            max_level: 360,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Tideless Shore",
                    description: "A beach of grey sand touching water that has never moved. No wind has ever blown here.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Drowned Wanderer",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Salt Meridian",
                    description: "A line of crystallized salt runs across the sea. On either side, the water is different colors.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Brine Phantom",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "The Unborn Reef",
                    description: "Coral formations that grew and then stopped. They are shaped like things that were never alive.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Calcified Leviathan",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Abyssal Shelf",
                    description: "The sea floor drops away into depths that have no bottom. The water below is not water.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Depthless",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "The Stillborn Sea",
                    description: "The center of the sea. The water is perfectly transparent. What lies beneath is visible and should not be.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "Mother of Still Waters",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 25,
            name: "Resonance Fault",
            description: "Sound has crystallized into physical structures. Footsteps create visible ripples. Voices from ages past still echo, frozen in crystal.",
            prestige_requirement: 200,
            min_level: 360,
            max_level: 375,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Harmonic Crust",
                    description: "The ground vibrates at a frequency just below hearing. Your teeth ache. Your bones hum.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Frequency Hound",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Choir Stone",
                    description: "Stone pillars that sing when the wind passes through. There is no wind.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "The Dissonant",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Oscillation Bridge",
                    description: "A bridge that exists only when a specific note is sustained. Silence drops you into nothing.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Null Resonant",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "The Petrified Scream",
                    description: "A scream from millennia ago, frozen in crystal. It is still screaming.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Scream Warden",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "Resonance Fault",
                    description: "The source of all sound in this place. A crack that hums with every frequency at once.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The Unheard Chorus",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 26,
            name: "The Wailing Reach",
            description: "The boundary between existence and void. Things here flicker between real and not. The wailing is the sound of matter deciding whether it exists.",
            prestige_requirement: 200,
            min_level: 375,
            max_level: 390,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Fraying Edge",
                    description: "The ground has gaps \u{2014} not holes, but places where the ground simply isn't. They move.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Border Stalker",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Liminal Corridor",
                    description: "A hallway always the same length no matter how far you walk. The walls are forgetting to be walls.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "The Undefined",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Static Garden",
                    description: "A garden where flowers are made of frozen static. They grow, bloom, and die in visual noise.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Entropy Bloom",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "The Last Light",
                    description: "A single point of light hovering in vast dark. The last thing here certain it exists.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Flickering One",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "The Wailing Reach",
                    description: "The boundary itself. One step more and things stop being things. The wailing is loudest here.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The Voice at the Edge",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // ── Chapter 6: The Origin Wound (Zones 27-30) ──────────────────
        Zone {
            id: 27,
            name: "The Scar Root",
            description: "Petrified tendrils of primordial fracture energy radiate outward like the root system of a dead tree. This is where the cracks began spreading.",
            prestige_requirement: 300,
            min_level: 390,
            max_level: 405,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Rootthread Pass",
                    description: "Thin veins of fracture energy wind through stone. They pulse with a rhythm older than heartbeats.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Root Creeper",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Splinter Nest",
                    description: "A tangle of petrified fracture-roots knotted into a den. Something has been living in the cracks.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Nestbound Horror",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Taproot Chamber",
                    description: "A central root as wide as a river descends into darkness. It feeds on something below.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Taproot Warden",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Fossilized Eruption",
                    description: "A frozen moment: the instant the first crack split the world. The energy is still here, arrested.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Calcified Rupture",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "The Scar Root",
                    description: "The root's origin point. Not rock, not energy \u{2014} the idea of breaking, made physical.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "Root of All Scars",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 28,
            name: "Echoing Abyss",
            description: "A vast emptiness where everything reverberates forever. Actions echo forward and backward in time. Your footsteps arrive before you do.",
            prestige_requirement: 300,
            min_level: 405,
            max_level: 420,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "First Echo",
                    description: "Your footsteps arrive before you do. The echoes here precede their causes.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Precursor Echo",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Reverberation Well",
                    description: "A pit where sound falls forever. Drop a stone and you hear it land yesterday.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Well Dweller",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Temporal Silt",
                    description: "The ground is made of compacted echoes. Dig and you hear conversations from before language.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "The Ancient Noise",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "The Infinite Repeat",
                    description: "A chamber where everything happens again. The monsters here have died a thousand times and remember each.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Once-Slain",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "Echoing Abyss",
                    description: "The center of all echoes. Here, the echo and the original are the same event.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The Eternal Reverberation",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 29,
            name: "Threshold of Silence",
            description: "The last place where sound and light exist. Beyond this, there is only the wound.",
            prestige_requirement: 300,
            min_level: 420,
            max_level: 435,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Dimming Walk",
                    description: "Each step forward, the light dims. Not because darkness grows, but because light has less reason to be.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Fade Walker",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Hush Fields",
                    description: "A plain where sound softens with every yard. At the center, even thought is quiet.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "The Muted",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Shadow of Sound",
                    description: "Not shadow from light \u{2014} shadow from sound. An area where noise casts visible darkness.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Soundshadow Beast",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "The Final Frequency",
                    description: "A single note hangs in the air. The last sound that will ever be heard in this place.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Frequency's End",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "Threshold of Silence",
                    description: "The threshold. One step more and there is no more sound, no more light. Just the wound.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The Silent Warden",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        Zone {
            id: 30,
            name: "The Origin Wound",
            description: "The primordial break. Not a place but an event fossilized into geography. The wound is open, patient, and eternal. It was here before the world it broke.",
            prestige_requirement: 300,
            min_level: 435,
            max_level: u32::MAX,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "The First Crack",
                    description: "The original fracture. Barely wider than a sword, but everything else grew from it.",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Fissure Guardian",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Primordial Scar",
                    description: "The scar tissue of the universe. Layered over itself a billion times, never healing, always growing.",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Scar Titan",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Memory of Wholeness",
                    description: "A pocket where reality remembers being unbroken. It hurts to be here. Wholeness is heavy.",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "The Unbroken",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Wound's Heart",
                    description: "The center of the original break. Not stone, not void \u{2014} a state of matter that exists only here.",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Heart of the Wound",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 5,
                    name: "The Origin Wound",
                    description: "The wound itself. Open, patient, eternal. Something guards the entrance to eternity.",
                    depth: 5,
                    boss: SubzoneBoss {
                        name: "The First and Final",
                        is_zone_boss: true,
                    },
                },
            ],
        },
    ]
});

/// Returns all zones in the game (zones 1-30).
/// Returns a static slice reference — no allocation on each call.
pub fn get_all_zones() -> &'static [Zone] {
    &ALL_ZONES
}

/// Gets a zone by its ID.
/// Uses O(1) array indexing since zone IDs are contiguous 1-30.
pub fn get_zone(zone_id: u32) -> Option<&'static Zone> {
    let zones = get_all_zones();
    let index = zone_id.checked_sub(1)? as usize;
    zones.get(index)
}

/// Gets a subzone within a zone.
/// Uses O(1) array indexing since subzone IDs are contiguous 1-based within each zone.
pub fn get_subzone(zone_id: u32, subzone_id: u32) -> Option<(&'static Zone, &'static Subzone)> {
    let zone = get_zone(zone_id)?;
    let index = subzone_id.checked_sub(1)? as usize;
    let subzone = zone.subzones.get(index)?;
    Some((zone, subzone))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_count() {
        let zones = get_all_zones();
        assert_eq!(zones.len(), 30); // Zones 1-11 + Fracture Zones 12-30
    }

    #[test]
    fn test_zone_names() {
        let zones = get_all_zones();
        assert_eq!(zones[0].name, "Meadow");
        assert_eq!(zones[1].name, "Dark Forest");
        assert_eq!(zones[2].name, "Mountain Pass");
        assert_eq!(zones[3].name, "Ancient Ruins");
        assert_eq!(zones[4].name, "Volcanic Wastes");
        assert_eq!(zones[5].name, "Frozen Tundra");
        assert_eq!(zones[6].name, "Crystal Caverns");
        assert_eq!(zones[7].name, "Sunken Kingdom");
        assert_eq!(zones[8].name, "Floating Isles");
        assert_eq!(zones[9].name, "Storm Citadel");
    }

    #[test]
    fn test_zone_prestige_requirements() {
        let zones = get_all_zones();
        // Tier 1: P0
        assert_eq!(zones[0].prestige_requirement, 0);
        assert_eq!(zones[1].prestige_requirement, 0);
        // Tier 2: P5
        assert_eq!(zones[2].prestige_requirement, 5);
        assert_eq!(zones[3].prestige_requirement, 5);
        // Tier 3: P10
        assert_eq!(zones[4].prestige_requirement, 10);
        assert_eq!(zones[5].prestige_requirement, 10);
        // Tier 4: P15
        assert_eq!(zones[6].prestige_requirement, 15);
        assert_eq!(zones[7].prestige_requirement, 15);
        // Tier 5: P20
        assert_eq!(zones[8].prestige_requirement, 20);
        assert_eq!(zones[9].prestige_requirement, 20);
    }

    #[test]
    fn test_subzone_counts() {
        let zones = get_all_zones();
        // Tiers 1-2: 3 subzones each
        assert_eq!(zones[0].subzones.len(), 3);
        assert_eq!(zones[1].subzones.len(), 3);
        assert_eq!(zones[2].subzones.len(), 3);
        assert_eq!(zones[3].subzones.len(), 3);
        // Tiers 3-5: 4 subzones each
        assert_eq!(zones[4].subzones.len(), 4);
        assert_eq!(zones[5].subzones.len(), 4);
        assert_eq!(zones[6].subzones.len(), 4);
        assert_eq!(zones[7].subzones.len(), 4);
        assert_eq!(zones[8].subzones.len(), 4);
        assert_eq!(zones[9].subzones.len(), 4);
    }

    #[test]
    fn test_zone_bosses() {
        let zones = get_all_zones();

        // Check that last subzone of each zone has is_zone_boss = true
        for zone in zones {
            let last_subzone = zone.subzones.last().unwrap();
            assert!(
                last_subzone.boss.is_zone_boss,
                "Zone {} last subzone boss should be zone boss",
                zone.name
            );

            // Check that non-last subzones are not zone bosses
            for subzone in &zone.subzones[..zone.subzones.len() - 1] {
                assert!(
                    !subzone.boss.is_zone_boss,
                    "Zone {} subzone {} should not be zone boss",
                    zone.name, subzone.name
                );
            }
        }
    }

    #[test]
    fn test_get_zone() {
        assert!(get_zone(1).is_some());
        assert_eq!(get_zone(1).unwrap().name, "Meadow");
        assert!(get_zone(10).is_some());
        assert_eq!(get_zone(10).unwrap().name, "Storm Citadel");
        assert!(get_zone(11).is_some());
        assert_eq!(get_zone(11).unwrap().name, "The Expanse");
        assert!(get_zone(12).is_some());
        assert_eq!(get_zone(12).unwrap().name, "Splintered Rim");
        assert!(get_zone(20).is_some());
        assert!(get_zone(21).is_some());
        assert_eq!(get_zone(21).unwrap().name, "Sunken Processional");
        assert!(get_zone(30).is_some());
        assert!(get_zone(31).is_none());
        assert!(get_zone(0).is_none());
    }

    #[test]
    fn test_get_subzone() {
        let result = get_subzone(1, 1);
        assert!(result.is_some());
        let (zone, subzone) = result.unwrap();
        assert_eq!(zone.name, "Meadow");
        assert_eq!(subzone.name, "Sunny Fields");

        let result = get_subzone(10, 4);
        assert!(result.is_some());
        let (zone, subzone) = result.unwrap();
        assert_eq!(zone.name, "Storm Citadel");
        assert_eq!(subzone.name, "Apex Spire");

        // Zone 11 (The Expanse) exists
        let result = get_subzone(11, 1);
        assert!(result.is_some());
        let (zone, subzone) = result.unwrap();
        assert_eq!(zone.name, "The Expanse");
        assert_eq!(subzone.name, "Void's Edge");

        // Fracture zone exists
        let result = get_subzone(12, 1);
        assert!(result.is_some());
        let (zone, subzone) = result.unwrap();
        assert_eq!(zone.name, "Splintered Rim");
        assert_eq!(subzone.name, "Rimwatch Scree");

        // New zones exist
        let result = get_subzone(21, 1);
        assert!(result.is_some());
        let (zone, subzone) = result.unwrap();
        assert_eq!(zone.name, "Sunken Processional");
        assert_eq!(subzone.name, "Pilgrim's Descent");

        // Invalid zone
        assert!(get_subzone(31, 1).is_none());
        // Invalid subzone
        assert!(get_subzone(1, 5).is_none());
    }

    #[test]
    fn test_weapon_zones() {
        let zones = get_all_zones();

        // Zones 1-9 don't require weapons
        for zone in &zones[0..9] {
            assert!(
                !zone.requires_weapon,
                "Zone {} should not require weapon",
                zone.name
            );
            assert!(zone.weapon_name.is_none());
        }

        // Zone 10 requires the Stormbreaker weapon
        let zone10 = &zones[9];
        assert!(zone10.requires_weapon, "Zone 10 should require weapon");
        assert_eq!(zone10.weapon_name, Some("Stormbreaker"));
    }

    #[test]
    fn test_fracture_zone_12() {
        let zone = get_zone(12).unwrap();
        assert_eq!(zone.name, "Splintered Rim");
        assert_eq!(zone.subzones.len(), 5);
        assert_eq!(zone.prestige_requirement, 50);
        assert_eq!(zone.min_level, 165);
        assert_eq!(zone.max_level, 180);
    }

    #[test]
    fn test_fracture_zone_20() {
        let zone = get_zone(20).unwrap();
        assert_eq!(zone.name, "The Black Mouth");
        assert_eq!(zone.subzones.len(), 5);
        assert_eq!(zone.max_level, 300);
    }

    #[test]
    fn test_fracture_zone_30() {
        let zone = get_zone(30).unwrap();
        assert_eq!(zone.name, "The Origin Wound");
        assert_eq!(zone.subzones.len(), 5);
        assert_eq!(zone.max_level, u32::MAX);
    }

    #[test]
    fn test_all_fracture_zones_have_5_subzones() {
        for zone_id in 12..=30 {
            let zone = get_zone(zone_id).unwrap();
            assert_eq!(
                zone.subzones.len(),
                5,
                "Zone {} ({}) should have 5 subzones",
                zone_id,
                zone.name
            );
        }
    }

    #[test]
    fn test_all_fracture_zones_have_zone_boss() {
        for zone_id in 12..=30 {
            let zone = get_zone(zone_id).unwrap();
            let last = zone.subzones.last().unwrap();
            assert!(
                last.boss.is_zone_boss,
                "Zone {} ({}) final subzone boss should be zone boss",
                zone_id, zone.name
            );
        }
    }

    #[test]
    fn test_fracture_zones_prestige_requirements() {
        let expected: &[(std::ops::RangeInclusive<u32>, u32)] = &[
            (12..=14, 50),
            (15..=17, 75),
            (18..=20, 100),
            (21..=23, 150),
            (24..=26, 200),
            (27..=30, 300),
        ];
        for (range, req) in expected {
            for zone_id in range.clone() {
                let zone = get_zone(zone_id).unwrap();
                assert_eq!(
                    zone.prestige_requirement, *req,
                    "Zone {} should have prestige_requirement {}",
                    zone_id, req
                );
            }
        }
    }

    #[test]
    fn test_fracture_zones_no_weapon_required() {
        for zone_id in 12..=30 {
            let zone = get_zone(zone_id).unwrap();
            assert!(
                !zone.requires_weapon,
                "Zone {} should not require weapon",
                zone_id
            );
            assert!(zone.weapon_name.is_none());
        }
    }
}
