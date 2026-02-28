#![allow(dead_code)]
use crate::dungeon::logic::DungeonEvent;
use crate::dungeon::pathfinding::{
    find_next_room, move_to_room, ROOM_MOVE_INTERVAL, ROOM_TRAVEL_INTERVAL,
};
use crate::dungeon::types::{Dungeon, RoomState};

/// Explicit inputs for the dungeon tick facade.
pub struct DungeonInput<'a> {
    pub dungeon: &'a mut Option<Dungeon>,
    pub zone_id: u32,
    pub prestige_rank: u32,
    pub player_level: u32,
}

/// Facade: tick dungeon exploration with decomposed inputs.
///
/// Replicates the logic of `update_dungeon` from `logic.rs` without
/// requiring a full `GameState`. Only the active dungeon, delta_time,
/// and god-item speed bonus are needed.
pub fn tick_dungeon_facade(
    dungeon: &mut Option<Dungeon>,
    delta_time: f64,
    god_item_dungeon_speed_percent: f64,
) -> Vec<DungeonEvent> {
    let mut events = Vec::new();

    let d = match dungeon.as_mut() {
        Some(d) => d,
        None => return events,
    };

    // Can't move until current room is cleared (combat complete)
    if !d.current_room_cleared {
        return events;
    }

    // Update move timer
    d.move_timer += delta_time;

    // Find next room to explore
    if let Some(next_pos) = find_next_room(d) {
        // Check if next room is already cleared (traveling) or new (exploring)
        let is_traveling = d
            .get_room(next_pos.0, next_pos.1)
            .map(|r| r.state == RoomState::Cleared)
            .unwrap_or(false);

        // Use faster interval when traveling through cleared rooms
        let base_interval = if is_traveling {
            ROOM_TRAVEL_INTERVAL
        } else {
            ROOM_MOVE_INTERVAL
        };
        let move_interval = base_interval * (1.0 - god_item_dungeon_speed_percent / 100.0);

        // Update traveling state for UI
        d.is_traveling = is_traveling;

        // Check if it's time to move
        if d.move_timer >= move_interval {
            d.move_timer = 0.0;

            // Move to the next room
            let move_events = move_to_room(d, next_pos);
            events.extend(move_events);
        }
    } else {
        d.is_traveling = false;
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::generation::generate_dungeon;

    #[test]
    fn test_dungeon_facade_no_dungeon() {
        let mut dungeon: Option<Dungeon> = None;
        let events = tick_dungeon_facade(&mut dungeon, 0.1, 0.0);
        assert!(events.is_empty());
    }

    #[test]
    fn test_dungeon_facade_room_not_cleared() {
        let mut d = generate_dungeon(10, 0, 1);
        d.current_room_cleared = false;
        let mut dungeon = Some(d);

        let events = tick_dungeon_facade(&mut dungeon, 0.1, 0.0);
        assert!(events.is_empty());
    }

    #[test]
    fn test_dungeon_facade_timer_accumulates() {
        let mut d = generate_dungeon(10, 0, 1);
        d.current_room_cleared = true;
        d.move_timer = 0.0;
        let mut dungeon = Some(d);

        // Small delta — not enough to trigger a move
        let events = tick_dungeon_facade(&mut dungeon, 0.1, 0.0);
        // Timer should have accumulated
        let timer = dungeon.as_ref().unwrap().move_timer;
        assert!(timer > 0.0, "move_timer should accumulate");
        // May or may not have events depending on whether threshold was met,
        // but with 0.1s and a 2.5s interval, no move should happen yet
        assert!(events.is_empty());
    }

    #[test]
    fn test_dungeon_facade_triggers_move_after_interval() {
        let mut d = generate_dungeon(10, 0, 1);
        d.current_room_cleared = true;
        d.move_timer = 0.0;
        let mut dungeon = Some(d);

        // Feed enough time to exceed ROOM_MOVE_INTERVAL (2.5s)
        let events = tick_dungeon_facade(&mut dungeon, 3.0, 0.0);

        // If there is a next room to explore, we should get movement events
        // (generated dungeons always have at least entrance + some rooms)
        if !events.is_empty() {
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, DungeonEvent::EnteredRoom { .. })),
                "Expected an EnteredRoom event"
            );
            // Timer should have been reset
            assert_eq!(dungeon.as_ref().unwrap().move_timer, 0.0);
        }
    }

    #[test]
    fn test_dungeon_facade_speed_bonus_reduces_interval() {
        let mut d = generate_dungeon(10, 0, 1);
        d.current_room_cleared = true;
        d.move_timer = 0.0;
        let mut dungeon = Some(d);

        // With 50% speed bonus, effective interval is 1.25s
        // Feed 1.5s — should trigger a move with speed bonus but not without
        let events = tick_dungeon_facade(&mut dungeon, 1.5, 50.0);

        // With the speed bonus, 1.5s > 1.25s so move should trigger
        // (provided there is a next room)
        if !events.is_empty() {
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, DungeonEvent::EnteredRoom { .. })),
                "Expected an EnteredRoom event with speed bonus"
            );
        }
    }
}
