//! Zone and subzone data definitions.

#![allow(dead_code)]

/// Represents a zone in the game world.
#[derive(Debug, Clone)]
pub struct Zone {
    pub id: u32,
    pub name: &'static str,
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
    pub depth: u32,
    pub boss: SubzoneBoss,
}

/// Boss guarding the exit of a subzone.
#[derive(Debug, Clone)]
pub struct SubzoneBoss {
    pub name: &'static str,
    pub is_zone_boss: bool,
}

/// Returns all zones in the game (zones 1-10).
pub fn get_all_zones() -> Vec<Zone> {
    vec![
        // Tier 1: Nature's Edge (P0) - 3 subzones each
        Zone {
            id: 1,
            name: "Meadow",
            prestige_requirement: 0,
            min_level: 1,
            max_level: 10,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Sunny Fields",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Field Guardian",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Overgrown Thicket",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Thicket Horror",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Mushroom Caves",
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
            prestige_requirement: 0,
            min_level: 10,
            max_level: 25,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Forest Edge",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Alpha Wolf",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Twisted Woods",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Corrupted Treant",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Spider's Hollow",
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
            prestige_requirement: 5,
            min_level: 25,
            max_level: 40,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Rocky Foothills",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Bandit King",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Frozen Peaks",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Ice Giant",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Dragon's Perch",
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
            prestige_requirement: 5,
            min_level: 40,
            max_level: 55,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Outer Sanctum",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Skeleton Lord",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Sunken Temple",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Spectral Guardian",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Sealed Catacombs",
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
            prestige_requirement: 10,
            min_level: 55,
            max_level: 70,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Scorched Badlands",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Ash Walker Chief",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Lava Rivers",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Magma Serpent",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Obsidian Fortress",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Fire Giant Warlord",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Magma Core",
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
            prestige_requirement: 10,
            min_level: 70,
            max_level: 85,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Snowbound Plains",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Dire Wolf Alpha",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Glacier Maze",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Ice Wraith Lord",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Frozen Lake",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Lake Horror",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Permafrost Tomb",
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
            prestige_requirement: 15,
            min_level: 85,
            max_level: 100,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Glittering Tunnels",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Gem Golem",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Prismatic Halls",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Prism Elemental",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Resonance Depths",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Echo Wraith",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Heart Crystal",
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
            prestige_requirement: 15,
            min_level: 100,
            max_level: 115,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Coral Gardens",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Merfolk Warlord",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Drowned Streets",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Drowned Admiral",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Abyssal Palace",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Pressure Beast",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Throne of Tides",
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
            prestige_requirement: 20,
            min_level: 115,
            max_level: 130,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Cloud Docks",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Harpy Matriarch",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Sky Bridges",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Wind Elemental Lord",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Stormfront",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Storm Drake",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Eye of the Storm",
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
            prestige_requirement: 20,
            min_level: 130,
            max_level: 150,
            requires_weapon: true,
            weapon_name: Some("Stormbreaker"),
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Lightning Fields",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Spark Colossus",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Thunder Halls",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Storm Knight Commander",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Generator Core",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Core Warden",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "Apex Spire",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "The Undying Storm",
                        is_zone_boss: true,
                    },
                },
            ],
        },
        // Zone 11: The Expanse - Infinite post-game zone (unlocked by GameComplete achievement)
        Zone {
            id: 11,
            name: "The Expanse",
            prestige_requirement: 0, // Unlocked by achievement, not prestige
            min_level: 150,
            max_level: u32::MAX,
            requires_weapon: false,
            weapon_name: None,
            subzones: vec![
                Subzone {
                    id: 1,
                    name: "Void's Edge",
                    depth: 1,
                    boss: SubzoneBoss {
                        name: "Void Sentinel",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 2,
                    name: "Eternal Storm",
                    depth: 2,
                    boss: SubzoneBoss {
                        name: "Tempest Incarnate",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 3,
                    name: "Abyssal Rift",
                    depth: 3,
                    boss: SubzoneBoss {
                        name: "Rift Behemoth",
                        is_zone_boss: false,
                    },
                },
                Subzone {
                    id: 4,
                    name: "The Endless",
                    depth: 4,
                    boss: SubzoneBoss {
                        name: "Avatar of Infinity",
                        is_zone_boss: true,
                    },
                },
            ],
        },
    ]
}

/// Gets a zone by its ID.
pub fn get_zone(zone_id: u32) -> Option<Zone> {
    get_all_zones().into_iter().find(|z| z.id == zone_id)
}

/// Gets a subzone within a zone.
pub fn get_subzone(zone_id: u32, subzone_id: u32) -> Option<(Zone, Subzone)> {
    let zone = get_zone(zone_id)?;
    let subzone = zone.subzones.iter().find(|s| s.id == subzone_id)?.clone();
    Some((zone, subzone))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_count() {
        let zones = get_all_zones();
        assert_eq!(zones.len(), 11); // Including Zone 11: The Expanse
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
        for zone in &zones {
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
        assert!(get_zone(12).is_none());
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

        // Invalid zone
        assert!(get_subzone(12, 1).is_none());
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
}
