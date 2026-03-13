//! Shard Fusion challenge — 2048-style tile-merging puzzle.

mod logic;
mod types;

pub use logic::{
    apply_slide, check_game_over, process_input, spawn_tile, start_shard_fusion_game,
    tick_shard_fusion, Direction, ShardFusionInput,
};
pub use types::{
    ShardFusionAnimState, ShardFusionDifficulty, ShardFusionGame, ShardFusionResult, TileMove,
    FLASH_TICKS, SLIDE_TICKS,
};

impl_apply_game_result! {
    fn apply_shard_fusion_result;
    variant: ShardFusion;
    result_body: |result, _state, _reward| {
        use ShardFusionResult::*;
        match result {
            Win => (true, ""),
            Loss => (false, "The shards refuse to fuse further."),
        }
    }
    game_type: crate::achievements::MinigameType::ShardFusion;
    icon: "\u{25C6}";
    win_message: "Fusion achieved!";
}
