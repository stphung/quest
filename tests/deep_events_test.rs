//! Integration tests for the deep::events module.

use quest::deep::events::{
    archetype_for_event, auto_resolve_pending_events, can_use_archetype_resolution, resolve_event,
};
use quest::deep::types::{
    ActiveMission, EventResolution, EventType, MercArchetype, MercStatus, Mercenary, MissionEvent,
    MissionType,
};

// =========================================================================
// Helpers
// =========================================================================

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

fn make_event_type(event_type: EventType, trigger_at_percent: u8, resolved: bool) -> MissionEvent {
    MissionEvent {
        trigger_at_percent,
        event_type,
        resolved,
        resolution: if resolved {
            Some(EventResolution::Safe)
        } else {
            None
        },
    }
}

fn make_cave_in_event(resolved: bool) -> MissionEvent {
    make_event_type(EventType::CaveIn, 25, resolved)
}

fn make_merc(id: u32, archetype: MercArchetype, status: MercStatus) -> Mercenary {
    Mercenary {
        id,
        name: format!("Merc {}", id),
        archetype,
        level: 1,
        power: 20,
        resilience: 20,
        status,
        missions_completed: 0,
        injury_cooldown: 0,
    }
}

// =========================================================================
// resolve_event
// =========================================================================

#[test]
fn resolve_event_success_marks_resolved() {
    let mut mission = make_mission_with_events(vec![make_cave_in_event(false)]);
    let result = resolve_event(&mut mission, 0, EventResolution::Safe);
    assert!(result.is_ok());
    assert!(mission.events[0].resolved);
    assert_eq!(mission.events[0].resolution, Some(EventResolution::Safe));
    assert_eq!(mission.events_resolved, 1);
}

#[test]
fn resolve_event_archetype_resolution() {
    let mut mission = make_mission_with_events(vec![make_cave_in_event(false)]);
    resolve_event(&mut mission, 0, EventResolution::Archetype).unwrap();
    assert_eq!(mission.events[0].resolution, Some(EventResolution::Archetype));
}

#[test]
fn resolve_event_risky_resolution() {
    let mut mission = make_mission_with_events(vec![make_cave_in_event(false)]);
    resolve_event(&mut mission, 0, EventResolution::Risky).unwrap();
    assert_eq!(mission.events[0].resolution, Some(EventResolution::Risky));
}

#[test]
fn resolve_event_fails_on_already_resolved() {
    let mut mission = make_mission_with_events(vec![make_cave_in_event(true)]);
    let result = resolve_event(&mut mission, 0, EventResolution::Safe);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "event already resolved");
}

#[test]
fn resolve_event_fails_on_invalid_index() {
    let mut mission = make_mission_with_events(vec![make_cave_in_event(false)]);
    let result = resolve_event(&mut mission, 99, EventResolution::Safe);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "event index out of bounds");
}

#[test]
fn resolve_event_fails_on_empty_events_list() {
    let mut mission = make_mission_with_events(vec![]);
    let result = resolve_event(&mut mission, 0, EventResolution::Safe);
    assert!(result.is_err());
}

#[test]
fn resolve_multiple_events_increments_counter() {
    let mut mission = make_mission_with_events(vec![
        make_cave_in_event(false),
        make_event_type(EventType::Ambush, 50, false),
    ]);

    resolve_event(&mut mission, 0, EventResolution::Safe).unwrap();
    assert_eq!(mission.events_resolved, 1);

    resolve_event(&mut mission, 1, EventResolution::Risky).unwrap();
    assert_eq!(mission.events_resolved, 2);
}

// =========================================================================
// auto_resolve_pending_events
// =========================================================================

#[test]
fn auto_resolve_resolves_triggered_event() {
    let mut mission = make_mission_with_events(vec![MissionEvent {
        trigger_at_percent: 25,
        event_type: EventType::Ambush,
        resolved: false,
        resolution: None,
    }]);
    // 50% progress at t=1800 (mission duration 3600s starting at t=0)
    auto_resolve_pending_events(&mut mission, 1800);

    assert!(mission.events[0].resolved);
    assert_eq!(
        mission.events[0].resolution,
        Some(EventResolution::AutoResolved)
    );
    assert_eq!(mission.events_resolved, 1);
}

#[test]
fn auto_resolve_does_not_resolve_future_events() {
    let mut mission = make_mission_with_events(vec![MissionEvent {
        trigger_at_percent: 75,
        event_type: EventType::Tremor,
        resolved: false,
        resolution: None,
    }]);
    // Only 10% progress — event at 75% hasn't triggered yet
    auto_resolve_pending_events(&mut mission, 360);

    assert!(!mission.events[0].resolved);
    assert_eq!(mission.events_resolved, 0);
}

#[test]
fn auto_resolve_skips_already_resolved_events() {
    let mut mission = make_mission_with_events(vec![make_cave_in_event(true)]);
    // progress = 50%
    auto_resolve_pending_events(&mut mission, 1800);
    // events_resolved was 0 at start (make_mission_with_events initializes it to 0)
    // already-resolved event should not bump the counter again
    assert_eq!(mission.events_resolved, 0);
}

#[test]
fn auto_resolve_handles_multiple_pending_events() {
    let mut mission = make_mission_with_events(vec![
        MissionEvent {
            trigger_at_percent: 25,
            event_type: EventType::CaveIn,
            resolved: false,
            resolution: None,
        },
        MissionEvent {
            trigger_at_percent: 50,
            event_type: EventType::FloodedPassage,
            resolved: false,
            resolution: None,
        },
        MissionEvent {
            trigger_at_percent: 75,
            event_type: EventType::AncientDoor,
            resolved: false,
            resolution: None,
        },
    ]);
    // 100% progress — all events triggered
    auto_resolve_pending_events(&mut mission, 3600);

    assert!(mission.events[0].resolved);
    assert!(mission.events[1].resolved);
    assert!(mission.events[2].resolved);
    assert_eq!(mission.events_resolved, 3);
}

// =========================================================================
// archetype_for_event
// =========================================================================

#[test]
fn cave_in_requires_saboteur() {
    assert_eq!(archetype_for_event(EventType::CaveIn), MercArchetype::Saboteur);
}

#[test]
fn ambush_requires_vanguard() {
    assert_eq!(archetype_for_event(EventType::Ambush), MercArchetype::Vanguard);
}

#[test]
fn flooded_passage_requires_arcanist() {
    assert_eq!(
        archetype_for_event(EventType::FloodedPassage),
        MercArchetype::Arcanist
    );
}

#[test]
fn ancient_door_requires_scout() {
    assert_eq!(archetype_for_event(EventType::AncientDoor), MercArchetype::Scout);
}

#[test]
fn tremor_requires_medic() {
    assert_eq!(archetype_for_event(EventType::Tremor), MercArchetype::Medic);
}

// =========================================================================
// can_use_archetype_resolution
// =========================================================================

#[test]
fn archetype_resolution_available_correct_merc_in_squad() {
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
        event_type: EventType::CaveIn,
        resolved: false,
        resolution: None,
    }];
    let mut mission = make_mission_with_events(events);
    mission.squad = vec![1];

    let scout = make_merc(1, MercArchetype::Scout, MercStatus::Ready);
    let mercs: Vec<&Mercenary> = vec![&scout];

    assert!(!can_use_archetype_resolution(&mission, 0, &mercs));
}

#[test]
fn archetype_resolution_not_available_merc_is_lost() {
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
fn archetype_resolution_not_available_merc_not_in_squad() {
    let events = vec![MissionEvent {
        trigger_at_percent: 25,
        event_type: EventType::Ambush, // requires Vanguard
        resolved: false,
        resolution: None,
    }];
    let mut mission = make_mission_with_events(events);
    mission.squad = vec![99]; // merc 1 is NOT in the squad

    let vanguard = make_merc(1, MercArchetype::Vanguard, MercStatus::Ready);
    let mercs: Vec<&Mercenary> = vec![&vanguard];

    assert!(!can_use_archetype_resolution(&mission, 0, &mercs));
}

#[test]
fn archetype_resolution_not_available_invalid_event_index() {
    let mission = make_mission_with_events(vec![]);
    let mercs: Vec<&Mercenary> = vec![];
    assert!(!can_use_archetype_resolution(&mission, 0, &mercs));
}

#[test]
fn archetype_resolution_available_with_injured_merc() {
    // Injured mercs can still provide archetype resolution (only Lost blocks it)
    let events = vec![MissionEvent {
        trigger_at_percent: 25,
        event_type: EventType::FloodedPassage,
        resolved: false,
        resolution: None,
    }];
    let mut mission = make_mission_with_events(events);
    mission.squad = vec![1];

    let injured_arcanist = make_merc(1, MercArchetype::Arcanist, MercStatus::Injured);
    let mercs: Vec<&Mercenary> = vec![&injured_arcanist];

    assert!(can_use_archetype_resolution(&mission, 0, &mercs));
}

#[test]
fn archetype_resolution_works_when_one_of_multiple_mercs_qualifies() {
    let events = vec![MissionEvent {
        trigger_at_percent: 25,
        event_type: EventType::AncientDoor, // requires Scout
        resolved: false,
        resolution: None,
    }];
    let mut mission = make_mission_with_events(events);
    mission.squad = vec![1, 2, 3];

    let vanguard = make_merc(1, MercArchetype::Vanguard, MercStatus::Ready);
    let scout = make_merc(2, MercArchetype::Scout, MercStatus::Ready);
    let medic = make_merc(3, MercArchetype::Medic, MercStatus::Ready);
    let mercs: Vec<&Mercenary> = vec![&vanguard, &scout, &medic];

    assert!(can_use_archetype_resolution(&mission, 0, &mercs));
}
