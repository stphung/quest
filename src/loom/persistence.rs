use super::types::LoomState;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn loom_save_path() -> io::Result<PathBuf> {
    Ok(crate::core::paths::get_quest_dir()?.join("loom.json"))
}

pub fn load_loom() -> LoomState {
    let path = match loom_save_path() {
        Ok(p) => p,
        Err(_) => return LoomState::new(),
    };
    load_loom_from_path(&path)
}

pub fn load_loom_from_path(path: &Path) -> LoomState {
    match fs::read_to_string(path) {
        Ok(json) => {
            let mut state: LoomState = serde_json::from_str(&json).unwrap_or_default();
            // Reset outdated saves — pattern definitions and features have changed.
            if state.persistent.version < super::types::LOOM_SAVE_VERSION {
                return LoomState::new();
            }
            sanitize_on_load(&mut state);
            state
        }
        Err(_) => LoomState::new(),
    }
}

/// Post-load sanitization: clamp NaN/Infinity floats, validate indices.
fn sanitize_on_load(state: &mut LoomState) {
    // ── Float sanitization ───────────────────────────────────────────────────
    for node in &mut state.persistent.nodes {
        if !node.buffer.is_finite() {
            node.buffer = 0.0;
        }
        if !node.buffer_capacity.is_finite() {
            node.buffer_capacity = node.base_rate * 10.0;
        }
        if !node.base_rate.is_finite() {
            node.base_rate = 25.0;
        }
        if !node.unlock_progress.is_finite() {
            node.unlock_progress = 0.0;
        }
        if !node.upgrade_remaining_secs.is_finite() {
            node.upgrade_remaining_secs = 0.0;
        }
    }
    for shuttle in &mut state.persistent.shuttles {
        if !shuttle.buffer.is_finite() {
            shuttle.buffer = 0.0;
        }
        if !shuttle.buffer_capacity.is_finite() {
            shuttle.buffer_capacity = 100.0;
        }
        if !shuttle.amount.is_finite() {
            shuttle.amount = 1.0;
        }
        if !shuttle.construction_secs_remaining.is_finite() {
            shuttle.construction_secs_remaining = 0.0;
        }
    }
    for pattern in &mut state.persistent.patterns {
        for req in &mut pattern.requirements {
            if !req.sustained_secs.is_finite() {
                req.sustained_secs = 0.0;
            }
            if !req.required_rate.is_finite() {
                req.required_rate = 0.0;
            }
            if !req.sustain_duration_secs.is_finite() {
                req.sustain_duration_secs = 0.0;
            }
        }
    }

    // ── Index sanitization ───────────────────────────────────────────────────
    // Clamp active_pattern to valid range.
    if !state.persistent.patterns.is_empty()
        && state.persistent.active_pattern >= state.persistent.patterns.len()
    {
        state.persistent.active_pattern = 0;
    }

    // Remove stale shuttle source references (idx >= shuttles.len()).
    let num_shuttles = state.persistent.shuttles.len();
    for shuttle in &mut state.persistent.shuttles {
        shuttle.sources_a.retain(
            |src| !matches!(src, super::types::LoomNodeRef::Shuttle(idx) if *idx >= num_shuttles),
        );
        shuttle.sources_b.retain(
            |src| !matches!(src, super::types::LoomNodeRef::Shuttle(idx) if *idx >= num_shuttles),
        );
    }
}

pub fn save_loom(loom: &LoomState) -> io::Result<()> {
    let path = loom_save_path()?;
    save_loom_to_path(loom, &path)
}

pub fn save_loom_to_path(loom: &LoomState, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(loom)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loom::types::LoomArchetype;

    #[test]
    fn test_loom_save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("loom.json");

        let mut state = LoomState::new();
        state.persistent.discovered = true;
        state.persistent.archetype = Some(LoomArchetype::BurnBright);

        save_loom_to_path(&state, &path).unwrap();
        let loaded = load_loom_from_path(&path);

        assert!(loaded.persistent.discovered);
        assert_eq!(loaded.persistent.archetype, Some(LoomArchetype::BurnBright));
        assert_eq!(loaded.persistent.nodes.len(), 6);
    }

    #[test]
    fn test_loom_load_missing_file_returns_default() {
        let loaded = load_loom_from_path(Path::new("/nonexistent/loom.json"));
        assert!(!loaded.persistent.discovered);
    }

    #[test]
    fn test_loom_load_corrupt_json_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("loom.json");
        fs::write(&path, "{{not valid json").unwrap();

        let loaded = load_loom_from_path(&path);
        assert!(!loaded.persistent.discovered);
        assert_eq!(loaded.persistent.nodes.len(), 6);
    }

    #[test]
    fn test_sanitize_clamps_nan_node_fields() {
        let mut state = LoomState::new();
        state.persistent.nodes[0].buffer = f64::NAN;
        state.persistent.nodes[1].buffer = f64::INFINITY;
        state.persistent.nodes[2].buffer_capacity = f64::NAN;
        state.persistent.nodes[3].unlock_progress = f64::NAN;
        state.persistent.nodes[4].upgrade_remaining_secs = f64::INFINITY;
        state.persistent.nodes[5].base_rate = f64::NAN;

        sanitize_on_load(&mut state);

        assert_eq!(state.persistent.nodes[0].buffer, 0.0);
        assert_eq!(state.persistent.nodes[1].buffer, 0.0);
        assert!(state.persistent.nodes[2].buffer_capacity.is_finite());
        assert_eq!(state.persistent.nodes[3].unlock_progress, 0.0);
        assert_eq!(state.persistent.nodes[4].upgrade_remaining_secs, 0.0);
        assert_eq!(state.persistent.nodes[5].base_rate, 25.0);
    }

    #[test]
    fn test_sanitize_clamps_nan_shuttle_fields() {
        let mut state = LoomState::new();
        use crate::loom::types::*;
        let mut shuttle = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![],
            vec![],
        );
        shuttle.amount = f64::NAN;
        shuttle.construction_secs_remaining = f64::INFINITY;
        shuttle.buffer = f64::NAN;
        shuttle.buffer_capacity = f64::NAN;
        state.persistent.shuttles.push(shuttle);

        sanitize_on_load(&mut state);

        assert_eq!(state.persistent.shuttles[0].amount, 1.0);
        assert_eq!(
            state.persistent.shuttles[0].construction_secs_remaining,
            0.0
        );
        assert_eq!(state.persistent.shuttles[0].buffer, 0.0);
        assert_eq!(state.persistent.shuttles[0].buffer_capacity, 100.0);
    }

    #[test]
    fn test_sanitize_clamps_nan_pattern_requirements() {
        let mut state = LoomState::new();
        crate::loom::discovery::complete_discovery(&mut state);

        state.persistent.patterns[0].requirements[0].sustained_secs = f64::NAN;
        state.persistent.patterns[0].requirements[0].required_rate = f64::INFINITY;
        state.persistent.patterns[0].requirements[0].sustain_duration_secs = f64::NAN;

        sanitize_on_load(&mut state);

        let req = &state.persistent.patterns[0].requirements[0];
        assert_eq!(req.sustained_secs, 0.0);
        assert_eq!(req.required_rate, 0.0);
        assert_eq!(req.sustain_duration_secs, 0.0);
    }

    #[test]
    fn test_sanitize_clamps_active_pattern_out_of_bounds() {
        let mut state = LoomState::new();
        crate::loom::discovery::complete_discovery(&mut state);
        state.persistent.active_pattern = 999;

        sanitize_on_load(&mut state);

        assert_eq!(state.persistent.active_pattern, 0);
    }

    #[test]
    fn test_sanitize_removes_stale_shuttle_source_refs() {
        let mut state = LoomState::new();
        use crate::loom::types::*;
        // One shuttle referencing Shuttle(5) which doesn't exist
        let mut shuttle = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![
                LoomNodeRef::Extractor(NodeId::EmberSpindle),
                LoomNodeRef::Shuttle(5),
            ],
            vec![LoomNodeRef::Shuttle(99)],
        );
        shuttle.under_construction = false;
        state.persistent.shuttles.push(shuttle);

        sanitize_on_load(&mut state);

        // Stale Shuttle refs should be removed, Extractor refs kept
        assert_eq!(
            state.persistent.shuttles[0].sources_a,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)]
        );
        assert!(state.persistent.shuttles[0].sources_b.is_empty());
    }

    #[test]
    fn test_old_save_without_version_resets_to_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("loom.json");

        // Simulate an old save: discovered, has patterns, no version field.
        // JSON without "version" → deserializes as version 0.
        fs::write(
            &path,
            r#"{"persistent":{"discovered":true,"patterns":[{"index":0,"name":"Old","requirements":[],"completed":true}]}}"#,
        )
        .unwrap();

        let loaded = load_loom_from_path(&path);

        // Old save should be reset — discovered is false, no patterns.
        assert!(!loaded.persistent.discovered, "Old save should reset");
        assert!(
            loaded.persistent.patterns.is_empty(),
            "Patterns should be empty after reset"
        );
        assert_eq!(
            loaded.persistent.version,
            crate::loom::types::LOOM_SAVE_VERSION
        );
    }

    #[test]
    fn test_current_version_save_loads_normally() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("loom.json");

        let mut state = LoomState::new();
        state.persistent.discovered = true;

        save_loom_to_path(&state, &path).unwrap();
        let loaded = load_loom_from_path(&path);

        assert!(
            loaded.persistent.discovered,
            "Current version save should load normally"
        );
        assert_eq!(
            loaded.persistent.version,
            crate::loom::types::LOOM_SAVE_VERSION
        );
    }
}
