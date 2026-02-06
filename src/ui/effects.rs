//! Combat visual effects using tachyonfx
//!
//! Provides shader-like effects for enemy sprites during combat:
//! - Hit flash when enemy takes damage
//! - Critical hit with more intense flash
//! - Death dissolve effect
//! - Boss entrance effect

use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use std::time::Instant;
use tachyonfx::{fx, Duration as TfxDuration, Effect, Interpolation, Shader};

/// Types of combat effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatEffectType {
    /// Brief white flash when enemy is hit
    EnemyHit,
    /// Intense yellow flash for critical hits
    CriticalHit,
    /// Enemy fades/dissolves on death
    EnemyDeath,
    /// Dramatic entrance for bosses
    BossEntrance,
    /// Subtle idle breathing/pulse
    IdlePulse,
}

/// Manages active visual effects for combat
pub struct CombatEffectManager {
    /// Currently active effect (we only run one at a time for simplicity)
    active_effect: Option<Effect>,
    /// When the current effect started
    effect_start: Option<Instant>,
    /// Type of current effect (for debugging/logging)
    effect_type: Option<CombatEffectType>,
    /// Last frame time for delta calculation
    last_frame: Instant,
}

impl std::fmt::Debug for CombatEffectManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CombatEffectManager")
            .field("effect_type", &self.effect_type)
            .field("is_active", &self.active_effect.is_some())
            .finish()
    }
}

impl Clone for CombatEffectManager {
    fn clone(&self) -> Self {
        // Effects are transient, so cloning returns a fresh manager
        Self::new()
    }
}

impl Default for CombatEffectManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CombatEffectManager {
    pub fn new() -> Self {
        Self {
            active_effect: None,
            effect_start: None,
            effect_type: None,
            last_frame: Instant::now(),
        }
    }

    /// Trigger a new combat effect
    pub fn trigger(&mut self, effect_type: CombatEffectType, _area: Rect) {
        let effect = match effect_type {
            CombatEffectType::EnemyHit => create_hit_flash(),
            CombatEffectType::CriticalHit => create_crit_flash(),
            CombatEffectType::EnemyDeath => create_death_dissolve(),
            CombatEffectType::BossEntrance => create_boss_entrance(),
            CombatEffectType::IdlePulse => create_idle_pulse(),
        };

        self.active_effect = Some(effect);
        self.effect_start = Some(Instant::now());
        self.effect_type = Some(effect_type);
    }

    /// Check if any effect is currently active
    pub fn is_active(&self) -> bool {
        self.active_effect.is_some()
    }

    /// Get the current effect type
    #[allow(dead_code)]
    pub fn current_type(&self) -> Option<CombatEffectType> {
        self.effect_type
    }

    /// Process effects and apply to the buffer
    /// Returns true if an effect was applied
    pub fn process(&mut self, buf: &mut Buffer, area: Rect) -> bool {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.last_frame = now;

        if let Some(effect) = &mut self.active_effect {
            // Convert std::time::Duration to tachyonfx::Duration
            let tfx_delta = TfxDuration::from(delta);
            effect.process(tfx_delta, buf, area);

            // Check if effect is done
            if effect.done() {
                self.active_effect = None;
                self.effect_start = None;
                self.effect_type = None;
                return false;
            }
            return true;
        }

        false
    }

    /// Clear all effects
    pub fn clear(&mut self) {
        self.active_effect = None;
        self.effect_start = None;
        self.effect_type = None;
    }
}

/// Creates a brief white flash effect for normal hits
fn create_hit_flash() -> Effect {
    // Flash to white, then fade back (150ms total)
    fx::sequence(&[
        fx::fade_to_fg(Color::White, (75, Interpolation::Linear)),
        fx::fade_to_fg(Color::Red, (75, Interpolation::Linear)),
    ])
}

/// Creates an intense yellow/white flash for critical hits
fn create_crit_flash() -> Effect {
    // More dramatic: flash yellow -> white -> back (300ms)
    fx::sequence(&[
        fx::fade_to_fg(Color::Yellow, (50, Interpolation::QuadOut)),
        fx::fade_to_fg(Color::White, (100, Interpolation::QuadOut)),
        fx::fade_to_fg(Color::Yellow, (50, Interpolation::Linear)),
        fx::fade_to_fg(Color::Red, (100, Interpolation::Linear)),
    ])
}

/// Creates a dissolve/fade effect for enemy death
fn create_death_dissolve() -> Effect {
    // Dissolve effect: shift through colors and fade out (500ms)
    fx::sequence(&[
        fx::fade_to_fg(Color::DarkGray, (200, Interpolation::QuadIn)),
        fx::dissolve((300, Interpolation::QuadIn)),
    ])
}

/// Creates a dramatic entrance effect for bosses
fn create_boss_entrance() -> Effect {
    // Boss entrance: pulse red dramatically (800ms)
    fx::sequence(&[
        fx::fade_to_fg(Color::Black, (0, Interpolation::Linear)),
        fx::fade_to_fg(Color::Rgb(139, 0, 0), (200, Interpolation::QuadOut)), // DarkRed
        fx::fade_to_fg(Color::Red, (200, Interpolation::QuadIn)),
        fx::fade_to_fg(Color::Rgb(255, 99, 71), (200, Interpolation::QuadOut)), // LightRed/Tomato
        fx::fade_to_fg(Color::Red, (200, Interpolation::Linear)),
    ])
}

/// Creates a subtle idle pulse effect
fn create_idle_pulse() -> Effect {
    // Gentle pulse between red shades (2000ms, loops)
    fx::ping_pong(fx::fade_to_fg(
        Color::Rgb(180, 0, 0),
        (1000, Interpolation::SineInOut),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_manager_creation() {
        let manager = CombatEffectManager::new();
        assert!(!manager.is_active());
    }

    #[test]
    fn test_trigger_effect() {
        let mut manager = CombatEffectManager::new();
        let area = Rect::new(0, 0, 20, 10);

        manager.trigger(CombatEffectType::EnemyHit, area);
        assert!(manager.is_active());
    }

    #[test]
    fn test_clear_effects() {
        let mut manager = CombatEffectManager::new();
        let area = Rect::new(0, 0, 20, 10);

        manager.trigger(CombatEffectType::EnemyHit, area);
        manager.clear();
        assert!(!manager.is_active());
    }
}
