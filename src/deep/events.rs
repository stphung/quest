//! Event resolution for The Deep mission check-in system.
//!
//! Events fire at 25 %, 50 %, and 75 % through a mission. The player can
//! resolve them manually, or they are auto-resolved if the player is away.

use crate::deep::clock::pending_events;
use crate::deep::types::{
    ActiveMission, EventResolution, EventType, MercArchetype, MercStatus, Mercenary,
};

// =========================================================================
// Manual resolution
// =========================================================================

/// Resolve a specific event on an active mission.
///
/// Returns `Err` if:
/// - `event_index` is out of bounds
/// - the event has already been resolved
pub fn resolve_event(
    mission: &mut ActiveMission,
    event_index: usize,
    resolution: EventResolution,
) -> Result<(), &'static str> {
    let event = mission
        .events
        .get_mut(event_index)
        .ok_or("event index out of bounds")?;

    if event.resolved {
        return Err("event already resolved");
    }

    event.resolved = true;
    event.resolution = Some(resolution);
    mission.events_resolved += 1;
    Ok(())
}

// =========================================================================
// Auto-resolution
// =========================================================================

/// Auto-resolves all pending (triggered but unresolved) events using
/// [`EventResolution::AutoResolved`].
///
/// Called when the player hasn't checked in — mission still completes normally.
pub fn auto_resolve_pending_events(mission: &mut ActiveMission, now_unix: i64) {
    let indices = pending_events(mission, now_unix);
    for i in indices {
        // Safety: pending_events only returns valid indices.
        if let Some(event) = mission.events.get_mut(i) {
            event.resolved = true;
            event.resolution = Some(EventResolution::AutoResolved);
            mission.events_resolved += 1;
        }
    }
}

// =========================================================================
// Flavor text
// =========================================================================

/// Returns flavor text for an event + resolution combination.
///
/// Covers all 5 [`EventType`] × 4 [`EventResolution`] combinations (20 texts).
pub fn event_flavor_text(event_type: EventType, resolution: EventResolution) -> &'static str {
    match (event_type, resolution) {
        // CaveIn
        (EventType::CaveIn, EventResolution::Safe) => {
            "Your squad found an alternate route around the collapse."
        }
        (EventType::CaveIn, EventResolution::Archetype) => {
            "Your Saboteur rigged a controlled blast to clear the debris."
        }
        (EventType::CaveIn, EventResolution::Risky) => {
            "The squad charged through the unstable passage."
        }
        (EventType::CaveIn, EventResolution::AutoResolved) => {
            "The squad carefully navigated around the obstruction."
        }

        // Ambush
        (EventType::Ambush, EventResolution::Safe) => {
            "The squad fell back and chose a defensible position to wait out the threat."
        }
        (EventType::Ambush, EventResolution::Archetype) => {
            "Your Vanguard held the line while the rest of the squad pushed through."
        }
        (EventType::Ambush, EventResolution::Risky) => {
            "The squad charged into the ambush, routing the attackers at great cost."
        }
        (EventType::Ambush, EventResolution::AutoResolved) => {
            "The squad quietly slipped past the hostile forces without engaging."
        }

        // FloodedPassage
        (EventType::FloodedPassage, EventResolution::Safe) => {
            "The squad waited for the water level to drop before proceeding."
        }
        (EventType::FloodedPassage, EventResolution::Archetype) => {
            "Your Arcanist channeled an elemental ward, safely parting the floodwaters."
        }
        (EventType::FloodedPassage, EventResolution::Risky) => {
            "The squad waded through the flooded tunnel, braving the treacherous current."
        }
        (EventType::FloodedPassage, EventResolution::AutoResolved) => {
            "The squad found a higher path skirting the flooded section."
        }

        // AncientDoor
        (EventType::AncientDoor, EventResolution::Safe) => {
            "The squad marked the door's location for later and pressed on."
        }
        (EventType::AncientDoor, EventResolution::Archetype) => {
            "Your Scout deciphered the ancient glyphs and bypassed the lock mechanism."
        }
        (EventType::AncientDoor, EventResolution::Risky) => {
            "The squad forced the ancient door open, triggering a defensive ward."
        }
        (EventType::AncientDoor, EventResolution::AutoResolved) => {
            "The squad left the sealed door undisturbed and found another way through."
        }

        // Tremor
        (EventType::Tremor, EventResolution::Safe) => {
            "The squad halted and braced until the shaking subsided."
        }
        (EventType::Tremor, EventResolution::Archetype) => {
            "Your Medic quickly assessed for injuries and guided the squad through the aftershocks."
        }
        (EventType::Tremor, EventResolution::Risky) => {
            "The squad sprinted through the trembling corridor before the ceiling gave way."
        }
        (EventType::Tremor, EventResolution::AutoResolved) => {
            "The squad sheltered in a reinforced alcove until the tremors passed."
        }
    }
}

// =========================================================================
// Archetype mapping
// =========================================================================

/// Returns which [`MercArchetype`] gets the bonus resolution for each event type.
pub fn archetype_for_event(event_type: EventType) -> MercArchetype {
    match event_type {
        EventType::CaveIn => MercArchetype::Saboteur,
        EventType::Ambush => MercArchetype::Vanguard,
        EventType::FloodedPassage => MercArchetype::Arcanist,
        EventType::AncientDoor => MercArchetype::Scout,
        EventType::Tremor => MercArchetype::Medic,
    }
}

// =========================================================================
// Archetype resolution eligibility
// =========================================================================

/// Returns `true` if the squad contains the required archetype for the
/// archetype-resolution option on a given event.
///
/// The merc must not have [`MercStatus::Lost`] status to be eligible.
pub fn can_use_archetype_resolution(
    mission: &ActiveMission,
    event_index: usize,
    mercs: &[&Mercenary],
) -> bool {
    let event = match mission.events.get(event_index) {
        Some(e) => e,
        None => return false,
    };

    let required_archetype = archetype_for_event(event.event_type);

    // Only squad members that are in the mission's squad list matter.
    mercs.iter().any(|m| {
        mission.squad.contains(&m.id)
            && m.archetype == required_archetype
            && m.status != MercStatus::Lost
    })
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep::types::{
        ActiveMission, EventResolution, EventType, MercStatus, MissionEvent, MissionType,
    };

    fn make_mission_with_events(events: Vec<MissionEvent>) -> ActiveMission {
        ActiveMission {
            id: 1,
            mission_type: MissionType::Expedition,
            layer: 1,
            squad: vec![1, 2],
            start_time: 0,
            duration_secs: 3600,
            cost: 50,
            events,
            events_resolved: 0,
        }
    }

    fn make_event(trigger_at_percent: u8, resolved: bool) -> MissionEvent {
        MissionEvent {
            trigger_at_percent,
            event_type: EventType::CaveIn,
            resolved,
            resolution: if resolved {
                Some(EventResolution::Safe)
            } else {
                None
            },
        }
    }

    fn make_merc(id: u32, archetype: MercArchetype, status: MercStatus) -> Mercenary {
        Mercenary {
            id,
            name: "Test Merc".to_string(),
            archetype,
            level: 1,
            power: 20,
            resilience: 20,
            status,
            missions_completed: 0,
            injury_cooldown: 0,
        }
    }

    // -- resolve_event --

    #[test]
    fn resolve_event_success() {
        let events = vec![make_event(25, false)];
        let mut mission = make_mission_with_events(events);

        let result = resolve_event(&mut mission, 0, EventResolution::Safe);
        assert!(result.is_ok());
        assert!(mission.events[0].resolved);
        assert_eq!(mission.events[0].resolution, Some(EventResolution::Safe));
        assert_eq!(mission.events_resolved, 1);
    }

    #[test]
    fn resolve_event_fails_on_already_resolved() {
        let events = vec![make_event(25, true)];
        let mut mission = make_mission_with_events(events);

        let result = resolve_event(&mut mission, 0, EventResolution::Safe);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "event already resolved");
    }

    #[test]
    fn resolve_event_fails_on_invalid_index() {
        let events = vec![make_event(25, false)];
        let mut mission = make_mission_with_events(events);

        let result = resolve_event(&mut mission, 5, EventResolution::Safe);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "event index out of bounds");
    }

    #[test]
    fn resolve_event_increments_counter() {
        let events = vec![make_event(25, false), make_event(50, false)];
        let mut mission = make_mission_with_events(events);

        resolve_event(&mut mission, 0, EventResolution::Risky).unwrap();
        resolve_event(&mut mission, 1, EventResolution::Archetype).unwrap();
        assert_eq!(mission.events_resolved, 2);
    }

    // -- auto_resolve_pending_events --

    #[test]
    fn auto_resolve_resolves_triggered_events() {
        let events = vec![MissionEvent {
            trigger_at_percent: 25,
            event_type: EventType::Ambush,
            resolved: false,
            resolution: None,
        }];
        let mut mission = make_mission_with_events(events);
        // 50% progress — event at 25% has triggered
        auto_resolve_pending_events(&mut mission, 1800);

        assert!(mission.events[0].resolved);
        assert_eq!(
            mission.events[0].resolution,
            Some(EventResolution::AutoResolved)
        );
        assert_eq!(mission.events_resolved, 1);
    }

    #[test]
    fn auto_resolve_skips_not_yet_triggered() {
        let events = vec![MissionEvent {
            trigger_at_percent: 75,
            event_type: EventType::Tremor,
            resolved: false,
            resolution: None,
        }];
        let mut mission = make_mission_with_events(events);
        // Only 10% progress — event at 75% has not triggered
        auto_resolve_pending_events(&mut mission, 360);

        assert!(!mission.events[0].resolved);
        assert_eq!(mission.events_resolved, 0);
    }

    #[test]
    fn auto_resolve_skips_already_resolved() {
        let events = vec![make_event(25, true)]; // already resolved
        let mut mission = make_mission_with_events(events);
        let original_resolved = mission.events_resolved;

        auto_resolve_pending_events(&mut mission, 1800);

        // Counter should not change since event was already resolved
        assert_eq!(mission.events_resolved, original_resolved);
    }

    // -- archetype_for_event --

    #[test]
    fn cave_in_maps_to_saboteur() {
        assert_eq!(archetype_for_event(EventType::CaveIn), MercArchetype::Saboteur);
    }

    #[test]
    fn ambush_maps_to_vanguard() {
        assert_eq!(archetype_for_event(EventType::Ambush), MercArchetype::Vanguard);
    }

    #[test]
    fn flooded_passage_maps_to_arcanist() {
        assert_eq!(
            archetype_for_event(EventType::FloodedPassage),
            MercArchetype::Arcanist
        );
    }

    #[test]
    fn ancient_door_maps_to_scout() {
        assert_eq!(archetype_for_event(EventType::AncientDoor), MercArchetype::Scout);
    }

    #[test]
    fn tremor_maps_to_medic() {
        assert_eq!(archetype_for_event(EventType::Tremor), MercArchetype::Medic);
    }

    // -- can_use_archetype_resolution --

    #[test]
    fn archetype_resolution_available_with_correct_merc() {
        let events = vec![MissionEvent {
            trigger_at_percent: 25,
            event_type: EventType::CaveIn, // requires Saboteur
            resolved: false,
            resolution: None,
        }];
        let mut mission = make_mission_with_events(events);
        mission.squad = vec![1];

        let saboteur = make_merc(1, MercArchetype::Saboteur, MercStatus::Ready);
        let mercs: Vec<&Mercenary> = vec![&saboteur];

        assert!(can_use_archetype_resolution(&mission, 0, &mercs));
    }

    #[test]
    fn archetype_resolution_not_available_wrong_archetype() {
        let events = vec![MissionEvent {
            trigger_at_percent: 25,
            event_type: EventType::CaveIn, // requires Saboteur
            resolved: false,
            resolution: None,
        }];
        let mut mission = make_mission_with_events(events);
        mission.squad = vec![1];

        let vanguard = make_merc(1, MercArchetype::Vanguard, MercStatus::Ready);
        let mercs: Vec<&Mercenary> = vec![&vanguard];

        assert!(!can_use_archetype_resolution(&mission, 0, &mercs));
    }

    #[test]
    fn archetype_resolution_not_available_if_merc_lost() {
        let events = vec![MissionEvent {
            trigger_at_percent: 25,
            event_type: EventType::CaveIn,
            resolved: false,
            resolution: None,
        }];
        let mut mission = make_mission_with_events(events);
        mission.squad = vec![1];

        let lost_saboteur = make_merc(1, MercArchetype::Saboteur, MercStatus::Lost);
        let mercs: Vec<&Mercenary> = vec![&lost_saboteur];

        assert!(!can_use_archetype_resolution(&mission, 0, &mercs));
    }

    #[test]
    fn archetype_resolution_not_available_invalid_event_index() {
        let mission = make_mission_with_events(vec![]);
        let mercs: Vec<&Mercenary> = vec![];
        assert!(!can_use_archetype_resolution(&mission, 99, &mercs));
    }

    #[test]
    fn archetype_resolution_merc_not_in_squad() {
        let events = vec![MissionEvent {
            trigger_at_percent: 25,
            event_type: EventType::CaveIn,
            resolved: false,
            resolution: None,
        }];
        let mut mission = make_mission_with_events(events);
        mission.squad = vec![99]; // merc 1 is NOT in this squad

        let saboteur = make_merc(1, MercArchetype::Saboteur, MercStatus::Ready);
        let mercs: Vec<&Mercenary> = vec![&saboteur];

        assert!(!can_use_archetype_resolution(&mission, 0, &mercs));
    }
}
