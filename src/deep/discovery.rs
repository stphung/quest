use super::types::TheDeepState;

/// Activate The Deep for the player.
///
/// Sets `state.discovered = true`. This is used by the debug menu — the caller
/// is responsible for deciding whether to persist the change.
pub fn discover_deep(state: &mut TheDeepState) {
    state.discovered = true;
}

/// Returns whether The Deep has been discovered.
pub fn is_deep_discovered(state: &TheDeepState) -> bool {
    state.discovered
}
