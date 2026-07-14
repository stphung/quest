# Vessel / Act 2 — Delta

## ADDED Requirements

### Requirement: Voyage State Is Observed For Collection Milestones

The voyage loop SHALL surface crossing and era state to the achievement layer by idempotent observation — the voyage engine itself SHALL carry no achievement coupling. Each voyage frame SHALL report the live crossing's docked-waypoint set, hailed-pilgrim set, known-rumor count, and taken-refit count; each landfall SHALL additionally report the delivery's carried count, sea-day duration, and filled crew berths. Observations SHALL be safe to repeat (monotone within a crossing; unlock checks are one-time by the permanent-unlock contract) and SHALL occur only inside the voyage loop, preserving the invariant that no achievement path executes while the Act 2 kill-switch is off.

#### Scenario: Per-frame observation is idempotent

- **WHEN** the same voyage frame state is observed repeatedly
- **THEN** persistent unions and unlock state are unchanged after the first observation

#### Scenario: Offline-resolved progress is observed on return

- **WHEN** a crossing advances while the game is closed (auto-sail docks, hails, or rumor grants) and the player returns
- **THEN** the first frame's observation folds the resolved state into the unions and unlock checks

#### Scenario: No observation while dark

- **WHEN** the Act 2 kill-switch is off
- **THEN** the voyage loop is unreachable and no collection observation or unlock can occur
