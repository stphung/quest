//! Haven room definitions, skill tree structure, and tier costs.

use serde::{Deserialize, Serialize};

/// Room identifiers in the Haven skill tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HavenRoomId {
    // Root
    Hearthstone,
    // Combat branch
    Armory,
    TrainingYard,
    TrophyHall,
    Watchtower,
    AlchemyLab,
    WarRoom,
    // QoL branch
    Bedroom,
    Garden,
    Library,
    FishingDock,
    Workshop,
    Vault,
    // Special buildings
    StormForge,
}

impl HavenRoomId {
    /// All room IDs in tree order
    pub const ALL: [HavenRoomId; 14] = [
        HavenRoomId::Hearthstone,
        HavenRoomId::Armory,
        HavenRoomId::TrainingYard,
        HavenRoomId::TrophyHall,
        HavenRoomId::Watchtower,
        HavenRoomId::AlchemyLab,
        HavenRoomId::WarRoom,
        HavenRoomId::Bedroom,
        HavenRoomId::Garden,
        HavenRoomId::Library,
        HavenRoomId::FishingDock,
        HavenRoomId::Workshop,
        HavenRoomId::Vault,
        HavenRoomId::StormForge,
    ];

    /// Display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            HavenRoomId::Hearthstone => "Hearthstone",
            HavenRoomId::Armory => "Armory",
            HavenRoomId::TrainingYard => "Training Yard",
            HavenRoomId::TrophyHall => "Trophy Hall",
            HavenRoomId::Watchtower => "Watchtower",
            HavenRoomId::AlchemyLab => "Alchemy Lab",
            HavenRoomId::WarRoom => "War Room",
            HavenRoomId::Bedroom => "Bedroom",
            HavenRoomId::Garden => "Garden",
            HavenRoomId::Library => "Library",
            HavenRoomId::FishingDock => "Fishing Dock",
            HavenRoomId::Workshop => "Workshop",
            HavenRoomId::Vault => "Vault",
            HavenRoomId::StormForge => "Storm Forge",
        }
    }

    /// Flavor description for detail panel
    pub fn description(&self) -> &'static str {
        match self {
            HavenRoomId::Hearthstone => "A crackling fire burns at the heart of your Haven, its embers never quite dying out. Even when you're away, its warmth keeps your skills sharp.",
            HavenRoomId::Armory => "Whetstones and weapon oil fill the air with a sharp, metallic tang. Every blade here has been honed to a razor's edge, and their fury flows into whoever wields them.",
            HavenRoomId::TrainingYard => "The clang of steel on wood echoes through the yard at all hours. Sweat-stained targets and chalk-drawn footwork patterns mark the path to mastery.",
            HavenRoomId::TrophyHall => "Glass cases display the spoils of a hundred battles — a dragon's scale, a bandit lord's signet ring, a shard of cursed obsidian. Their presence draws more treasure your way.",
            HavenRoomId::Watchtower => "A spiral staircase leads to a narrow platform where hawks nest and cold wind bites. Hours spent scanning the horizon have taught you to spot a weakness before your enemy even knows it's there.",
            HavenRoomId::AlchemyLab => "Bubbling flasks and copper coils crowd every surface, filling the room with a warm, herbal haze. The potions brewed here mend wounds faster than any battlefield medic could dream.",
            HavenRoomId::WarRoom => "Faded footwork circles are carved into the stone floor, each one paired with strike marks on the opposing wall — one high, one low, in rapid succession. The room teaches your muscles what your mind already knows: one strike is never enough.",
            HavenRoomId::Bedroom => "Heavy curtains block out every sliver of light, and the bed is piled high with furs. In this perfect darkness, your body recovers with an almost unnatural speed.",
            HavenRoomId::Garden => "Water trickles from a carved stone fountain into a shallow basin where lily pads drift. Tending this garden teaches a stillness that makes even the longest fishing wait feel brief.",
            HavenRoomId::Library => "A reading nook tucked beneath a stained-glass window, surrounded by towers of scrolls and ink-stained notes. The more you read, the more the world reveals its hidden trials to you.",
            HavenRoomId::FishingDock => "Morning mist clings to the water as your line breaks the stillness. The fish here bite in pairs, and those who cast long enough swear they've felt something vast stir in the deep — something most anglers will never be ready for.",
            HavenRoomId::Workshop => "Sawdust and iron filings crunch underfoot as you pass workbenches cluttered with half-finished tools and polishing rigs. Gear crafted here always seems to turn out a cut above the rest.",
            HavenRoomId::Vault => "Behind a door that only opens to your touch, shelves of dark wood cradle the weapons and armor you've sworn never to lose. The vault doesn't care how many times the world starts over — it keeps its promises.",
            HavenRoomId::StormForge => "A forge of black iron sits beneath an open sky, struck by lightning that never stops. It took more prestiges than most adventurers will ever earn just to lay these stones, and the forging demands you sacrifice more still. The anvil will not wake for just anyone — only hands that have felt the Storm Leviathan's fury carry the spark needed to ignite the forge and shape Stormbreaker from raw thunder.",
        }
    }

    /// Parent room(s) that must be T1+ to unlock this room.
    /// Returns empty slice for Hearthstone (root).
    /// Capstones require both parents.
    pub fn parents(&self) -> &'static [HavenRoomId] {
        match self {
            HavenRoomId::Hearthstone => &[],
            // Combat branch
            HavenRoomId::Armory => &[HavenRoomId::Hearthstone],
            HavenRoomId::TrainingYard => &[HavenRoomId::Armory],
            HavenRoomId::TrophyHall => &[HavenRoomId::Armory],
            HavenRoomId::Watchtower => &[HavenRoomId::TrainingYard],
            HavenRoomId::AlchemyLab => &[HavenRoomId::TrophyHall],
            HavenRoomId::WarRoom => &[HavenRoomId::Watchtower, HavenRoomId::AlchemyLab],
            // QoL branch
            HavenRoomId::Bedroom => &[HavenRoomId::Hearthstone],
            HavenRoomId::Garden => &[HavenRoomId::Bedroom],
            HavenRoomId::Library => &[HavenRoomId::Bedroom],
            HavenRoomId::FishingDock => &[HavenRoomId::Garden],
            HavenRoomId::Workshop => &[HavenRoomId::Library],
            HavenRoomId::Vault => &[HavenRoomId::FishingDock, HavenRoomId::Workshop],
            // StormForge requires both capstones
            HavenRoomId::StormForge => &[HavenRoomId::WarRoom, HavenRoomId::Vault],
        }
    }

    /// Child rooms that this room unlocks when built to T1+.
    #[allow(dead_code)] // Will be used for UI graph rendering
    pub fn children(&self) -> &'static [HavenRoomId] {
        match self {
            HavenRoomId::Hearthstone => &[HavenRoomId::Armory, HavenRoomId::Bedroom],
            HavenRoomId::Armory => &[HavenRoomId::TrainingYard, HavenRoomId::TrophyHall],
            HavenRoomId::TrainingYard => &[HavenRoomId::Watchtower],
            HavenRoomId::TrophyHall => &[HavenRoomId::AlchemyLab],
            HavenRoomId::Watchtower => &[HavenRoomId::WarRoom],
            HavenRoomId::AlchemyLab => &[HavenRoomId::WarRoom],
            HavenRoomId::WarRoom => &[HavenRoomId::StormForge],
            HavenRoomId::Bedroom => &[HavenRoomId::Garden, HavenRoomId::Library],
            HavenRoomId::Garden => &[HavenRoomId::FishingDock],
            HavenRoomId::Library => &[HavenRoomId::Workshop],
            HavenRoomId::FishingDock => &[HavenRoomId::Vault],
            HavenRoomId::Workshop => &[HavenRoomId::Vault],
            HavenRoomId::Vault => &[HavenRoomId::StormForge],
            HavenRoomId::StormForge => &[],
        }
    }

    /// Whether this room is a capstone (requires two parents)
    #[allow(dead_code)] // Will be used for UI styling
    pub fn is_capstone(&self) -> bool {
        matches!(
            self,
            HavenRoomId::WarRoom | HavenRoomId::Vault | HavenRoomId::StormForge
        )
    }

    /// Get the depth of this room in the tree (0 = root, 4 = capstones, 5 = StormForge)
    pub fn depth(&self) -> u8 {
        match self {
            HavenRoomId::Hearthstone => 0,
            HavenRoomId::Armory | HavenRoomId::Bedroom => 1,
            HavenRoomId::TrainingYard
            | HavenRoomId::TrophyHall
            | HavenRoomId::Garden
            | HavenRoomId::Library => 2,
            HavenRoomId::Watchtower
            | HavenRoomId::AlchemyLab
            | HavenRoomId::FishingDock
            | HavenRoomId::Workshop => 3,
            HavenRoomId::WarRoom | HavenRoomId::Vault => 4,
            HavenRoomId::StormForge => 5,
        }
    }

    /// Maximum tier for this room (most rooms are 3, StormForge and FishingDock have special max)
    pub fn max_tier(&self) -> u8 {
        match self {
            HavenRoomId::StormForge => 1,  // Single tier only
            HavenRoomId::FishingDock => 4, // Has tier 4 for max fishing rank
            _ => 3,
        }
    }
}

/// Get the prestige rank cost for a specific tier and room.
/// Costs scale with depth: root is cheapest, capstones are most expensive.
/// Special rooms have unique costs.
pub fn tier_cost(room: HavenRoomId, tier: u8) -> u32 {
    // Special room costs
    match room {
        HavenRoomId::StormForge => {
            // Single tier, costs 25 PR
            if tier == 1 {
                25
            } else {
                0
            }
        }
        HavenRoomId::FishingDock => {
            // T1-3 follow normal depth 3 costs, T4 is special
            match tier {
                1 => 2,
                2 => 4,
                3 => 6,
                4 => 10, // Special T4 cost
                _ => 0,
            }
        }
        _ => {
            let depth = room.depth();
            match (depth, tier) {
                // Depth 0 (Hearthstone): 1/2/3
                (0, 1) => 1,
                (0, 2) => 2,
                (0, 3) => 3,
                // Depth 1 (Armory, Bedroom): 1/3/5
                (1, 1) => 1,
                (1, 2) => 3,
                (1, 3) => 5,
                // Depth 2-3 (mid-tree): 2/4/6
                (2..=3, 1) => 2,
                (2..=3, 2) => 4,
                (2..=3, 3) => 6,
                // Depth 4 (capstones): 3/5/7
                (4, 1) => 3,
                (4, 2) => 5,
                (4, 3) => 7,
                _ => 0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rooms_have_nonempty_name_and_description() {
        for room in HavenRoomId::ALL {
            assert!(!room.name().is_empty(), "{:?} has empty name", room);
            assert!(
                !room.description().is_empty(),
                "{:?} has empty description",
                room
            );
        }
    }

    #[test]
    fn test_specific_room_names() {
        assert_eq!(HavenRoomId::Hearthstone.name(), "Hearthstone");
        assert_eq!(HavenRoomId::Armory.name(), "Armory");
        assert_eq!(HavenRoomId::TrainingYard.name(), "Training Yard");
        assert_eq!(HavenRoomId::TrophyHall.name(), "Trophy Hall");
        assert_eq!(HavenRoomId::Watchtower.name(), "Watchtower");
        assert_eq!(HavenRoomId::AlchemyLab.name(), "Alchemy Lab");
        assert_eq!(HavenRoomId::WarRoom.name(), "War Room");
        assert_eq!(HavenRoomId::Bedroom.name(), "Bedroom");
        assert_eq!(HavenRoomId::Garden.name(), "Garden");
        assert_eq!(HavenRoomId::Library.name(), "Library");
        assert_eq!(HavenRoomId::FishingDock.name(), "Fishing Dock");
        assert_eq!(HavenRoomId::Workshop.name(), "Workshop");
        assert_eq!(HavenRoomId::Vault.name(), "Vault");
        assert_eq!(HavenRoomId::StormForge.name(), "Storm Forge");
    }

    #[test]
    fn test_hearthstone_is_root_with_no_parents() {
        assert!(HavenRoomId::Hearthstone.parents().is_empty());
        assert_eq!(HavenRoomId::Hearthstone.depth(), 0);
    }

    #[test]
    fn test_capstones_require_both_parents() {
        assert_eq!(
            HavenRoomId::WarRoom.parents(),
            &[HavenRoomId::Watchtower, HavenRoomId::AlchemyLab]
        );
        assert_eq!(
            HavenRoomId::Vault.parents(),
            &[HavenRoomId::FishingDock, HavenRoomId::Workshop]
        );
        assert_eq!(
            HavenRoomId::StormForge.parents(),
            &[HavenRoomId::WarRoom, HavenRoomId::Vault]
        );
    }

    #[test]
    fn test_single_parent_rooms() {
        assert_eq!(HavenRoomId::Armory.parents(), &[HavenRoomId::Hearthstone]);
        assert_eq!(HavenRoomId::Bedroom.parents(), &[HavenRoomId::Hearthstone]);
        assert_eq!(HavenRoomId::TrainingYard.parents(), &[HavenRoomId::Armory]);
        assert_eq!(HavenRoomId::TrophyHall.parents(), &[HavenRoomId::Armory]);
        assert_eq!(
            HavenRoomId::Watchtower.parents(),
            &[HavenRoomId::TrainingYard]
        );
        assert_eq!(
            HavenRoomId::AlchemyLab.parents(),
            &[HavenRoomId::TrophyHall]
        );
        assert_eq!(HavenRoomId::Garden.parents(), &[HavenRoomId::Bedroom]);
        assert_eq!(HavenRoomId::Library.parents(), &[HavenRoomId::Bedroom]);
        assert_eq!(HavenRoomId::FishingDock.parents(), &[HavenRoomId::Garden]);
        assert_eq!(HavenRoomId::Workshop.parents(), &[HavenRoomId::Library]);
    }

    #[test]
    fn test_children_mirror_parents() {
        // Every room listed as a parent of X should list X as a child.
        for room in HavenRoomId::ALL {
            for parent in room.parents() {
                assert!(
                    parent.children().contains(&room),
                    "{:?} should be a child of {:?}",
                    room,
                    parent
                );
            }
        }
    }

    #[test]
    fn test_storm_forge_has_no_children() {
        assert!(HavenRoomId::StormForge.children().is_empty());
    }

    #[test]
    fn test_hearthstone_children() {
        assert_eq!(
            HavenRoomId::Hearthstone.children(),
            &[HavenRoomId::Armory, HavenRoomId::Bedroom]
        );
    }

    #[test]
    fn test_is_capstone() {
        assert!(HavenRoomId::WarRoom.is_capstone());
        assert!(HavenRoomId::Vault.is_capstone());
        assert!(HavenRoomId::StormForge.is_capstone());

        assert!(!HavenRoomId::Hearthstone.is_capstone());
        assert!(!HavenRoomId::Armory.is_capstone());
        assert!(!HavenRoomId::Bedroom.is_capstone());
        assert!(!HavenRoomId::Watchtower.is_capstone());
    }

    #[test]
    fn test_depth_values_for_every_room() {
        assert_eq!(HavenRoomId::Hearthstone.depth(), 0);
        assert_eq!(HavenRoomId::Armory.depth(), 1);
        assert_eq!(HavenRoomId::Bedroom.depth(), 1);
        assert_eq!(HavenRoomId::TrainingYard.depth(), 2);
        assert_eq!(HavenRoomId::TrophyHall.depth(), 2);
        assert_eq!(HavenRoomId::Garden.depth(), 2);
        assert_eq!(HavenRoomId::Library.depth(), 2);
        assert_eq!(HavenRoomId::Watchtower.depth(), 3);
        assert_eq!(HavenRoomId::AlchemyLab.depth(), 3);
        assert_eq!(HavenRoomId::FishingDock.depth(), 3);
        assert_eq!(HavenRoomId::Workshop.depth(), 3);
        assert_eq!(HavenRoomId::WarRoom.depth(), 4);
        assert_eq!(HavenRoomId::Vault.depth(), 4);
        assert_eq!(HavenRoomId::StormForge.depth(), 5);
    }

    #[test]
    fn test_max_tier_special_cases() {
        assert_eq!(HavenRoomId::StormForge.max_tier(), 1);
        assert_eq!(HavenRoomId::FishingDock.max_tier(), 4);
        // Everything else caps at 3.
        assert_eq!(HavenRoomId::Hearthstone.max_tier(), 3);
        assert_eq!(HavenRoomId::Armory.max_tier(), 3);
        assert_eq!(HavenRoomId::WarRoom.max_tier(), 3);
        assert_eq!(HavenRoomId::Vault.max_tier(), 3);
    }

    #[test]
    fn test_tier_cost_hearthstone_depth_0() {
        assert_eq!(tier_cost(HavenRoomId::Hearthstone, 1), 1);
        assert_eq!(tier_cost(HavenRoomId::Hearthstone, 2), 2);
        assert_eq!(tier_cost(HavenRoomId::Hearthstone, 3), 3);
    }

    #[test]
    fn test_tier_cost_depth_1_rooms() {
        for room in [HavenRoomId::Armory, HavenRoomId::Bedroom] {
            assert_eq!(tier_cost(room, 1), 1);
            assert_eq!(tier_cost(room, 2), 3);
            assert_eq!(tier_cost(room, 3), 5);
        }
    }

    #[test]
    fn test_tier_cost_depth_2_and_3_rooms() {
        for room in [
            HavenRoomId::TrainingYard,
            HavenRoomId::TrophyHall,
            HavenRoomId::Garden,
            HavenRoomId::Library,
            HavenRoomId::Watchtower,
            HavenRoomId::AlchemyLab,
            HavenRoomId::FishingDock, // T1-3 follow normal depth-3 costs
            HavenRoomId::Workshop,
        ] {
            assert_eq!(tier_cost(room, 1), 2, "{:?} T1", room);
            assert_eq!(tier_cost(room, 2), 4, "{:?} T2", room);
            assert_eq!(tier_cost(room, 3), 6, "{:?} T3", room);
        }
    }

    #[test]
    fn test_tier_cost_capstones() {
        for room in [HavenRoomId::WarRoom, HavenRoomId::Vault] {
            assert_eq!(tier_cost(room, 1), 3, "{:?} T1", room);
            assert_eq!(tier_cost(room, 2), 5, "{:?} T2", room);
            assert_eq!(tier_cost(room, 3), 7, "{:?} T3", room);
        }
    }

    #[test]
    fn test_tier_cost_fishing_dock_special_t4() {
        assert_eq!(tier_cost(HavenRoomId::FishingDock, 4), 10);
    }

    #[test]
    fn test_tier_cost_storm_forge() {
        assert_eq!(tier_cost(HavenRoomId::StormForge, 1), 25);
        // Invalid tiers for a single-tier room return 0.
        assert_eq!(tier_cost(HavenRoomId::StormForge, 0), 0);
        assert_eq!(tier_cost(HavenRoomId::StormForge, 2), 0);
    }

    #[test]
    fn test_tier_cost_invalid_tier_returns_zero() {
        // Tier 0 and out-of-range tiers are not valid for any normal room.
        assert_eq!(tier_cost(HavenRoomId::Hearthstone, 0), 0);
        assert_eq!(tier_cost(HavenRoomId::Hearthstone, 4), 0);
        assert_eq!(tier_cost(HavenRoomId::WarRoom, 5), 0);
        assert_eq!(tier_cost(HavenRoomId::FishingDock, 5), 0);
    }

    #[test]
    fn test_all_array_has_fourteen_unique_rooms() {
        assert_eq!(HavenRoomId::ALL.len(), 14);
        let mut seen = std::collections::HashSet::new();
        for room in HavenRoomId::ALL {
            assert!(seen.insert(room), "Duplicate room {:?} in ALL", room);
        }
    }
}
