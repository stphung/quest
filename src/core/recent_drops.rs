use std::collections::VecDeque;

use crate::items::types::Rarity;

/// A recently gained item or fish for display in the Loot panel
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RecentDrop {
    pub name: String,
    pub rarity: Rarity,
    pub equipped: bool,
    pub icon: &'static str,
    /// Equipment slot name (e.g. "Weapon", "Armor"), empty for non-equipment
    pub slot: String,
    /// Stat summary (e.g. "+8 STR +3 DEX +Crit"), empty for non-equipment
    pub stats: String,
}

/// Max number of recent drops to track
pub const MAX_RECENT_DROPS: usize = 10;

/// Add a recent drop to the deque, evicting the oldest if at capacity.
pub fn add_recent_drop(
    drops: &mut VecDeque<RecentDrop>,
    name: String,
    rarity: Rarity,
    equipped: bool,
    icon: &'static str,
    slot: String,
    stats: String,
) {
    if drops.len() >= MAX_RECENT_DROPS {
        drops.pop_back();
    }
    drops.push_front(RecentDrop {
        name,
        rarity,
        equipped,
        icon,
        slot,
        stats,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_recent_drop_single() {
        let mut drops = VecDeque::new();

        add_recent_drop(
            &mut drops,
            "Iron Sword".to_string(),
            Rarity::Common,
            true,
            "\u{2694}",
            "Weapon".to_string(),
            "+2 STR".to_string(),
        );

        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].name, "Iron Sword");
        assert_eq!(drops[0].rarity, Rarity::Common);
        assert!(drops[0].equipped);
        assert_eq!(drops[0].slot, "Weapon");
        assert_eq!(drops[0].stats, "+2 STR");
    }

    #[test]
    fn test_add_recent_drop_fifo_order() {
        let mut drops = VecDeque::new();

        add_recent_drop(
            &mut drops,
            "First".to_string(),
            Rarity::Common,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        add_recent_drop(
            &mut drops,
            "Second".to_string(),
            Rarity::Rare,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        add_recent_drop(
            &mut drops,
            "Third".to_string(),
            Rarity::Epic,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );

        // Most recent should be at front
        assert_eq!(drops.len(), 3);
        assert_eq!(drops[0].name, "Third");
        assert_eq!(drops[1].name, "Second");
        assert_eq!(drops[2].name, "First");
    }

    #[test]
    fn test_add_recent_drop_caps_at_max() {
        let mut drops = VecDeque::new();

        // Fill to the cap
        for i in 0..10 {
            add_recent_drop(
                &mut drops,
                format!("Item {i}"),
                Rarity::Common,
                false,
                "",
                "".to_string(),
                "".to_string(),
            );
        }
        assert_eq!(drops.len(), 10);

        // Adding one more should evict the oldest
        add_recent_drop(
            &mut drops,
            "Overflow".to_string(),
            Rarity::Legendary,
            true,
            "",
            "".to_string(),
            "".to_string(),
        );
        assert_eq!(drops.len(), 10);
        assert_eq!(drops[0].name, "Overflow");
        // "Item 0" (the oldest) should have been evicted
        assert!(drops.iter().all(|d| d.name != "Item 0"));
        // "Item 1" should still be present as the last element
        assert_eq!(drops[9].name, "Item 1");
    }

    #[test]
    fn test_add_recent_drop_at_exact_cap_boundary() {
        let mut drops = VecDeque::new();

        // Add exactly MAX_RECENT_DROPS items
        for i in 0..10 {
            add_recent_drop(
                &mut drops,
                format!("Item {i}"),
                Rarity::Common,
                false,
                "",
                "".to_string(),
                "".to_string(),
            );
        }
        assert_eq!(drops.len(), 10);

        // Add two more, should still be capped at 10
        add_recent_drop(
            &mut drops,
            "Extra1".to_string(),
            Rarity::Common,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        add_recent_drop(
            &mut drops,
            "Extra2".to_string(),
            Rarity::Common,
            false,
            "",
            "".to_string(),
            "".to_string(),
        );
        assert_eq!(drops.len(), 10);
        assert_eq!(drops[0].name, "Extra2");
        assert_eq!(drops[1].name, "Extra1");
    }

    #[test]
    fn test_add_recent_drop_preserves_all_fields() {
        let mut drops = VecDeque::new();

        add_recent_drop(
            &mut drops,
            "Legendary Blade".to_string(),
            Rarity::Legendary,
            true,
            "\u{2694}",
            "Weapon".to_string(),
            "+20 STR +10 DEX +Crit".to_string(),
        );

        let drop = &drops[0];
        assert_eq!(drop.name, "Legendary Blade");
        assert_eq!(drop.rarity, Rarity::Legendary);
        assert!(drop.equipped);
        assert_eq!(drop.icon, "\u{2694}");
        assert_eq!(drop.slot, "Weapon");
        assert_eq!(drop.stats, "+20 STR +10 DEX +Crit");
    }

    #[test]
    fn test_empty_deque_has_zero_length() {
        let drops: VecDeque<RecentDrop> = VecDeque::new();
        assert_eq!(drops.len(), 0);
        assert!(drops.is_empty());
    }

    #[test]
    fn test_max_recent_drops_constant() {
        assert_eq!(MAX_RECENT_DROPS, 10);
    }
}
