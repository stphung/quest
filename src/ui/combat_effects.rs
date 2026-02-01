use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

#[derive(Debug, Clone)]
pub struct VisualEffect {
    pub effect_type: EffectType,
    pub lifetime: f64,
    pub max_lifetime: f64,
}

#[derive(Debug, Clone)]
pub enum EffectType {
    DamageNumber { value: u32, is_crit: bool },
    AttackFlash,
    HitImpact,
}

impl VisualEffect {
    pub fn new(effect_type: EffectType, max_lifetime: f64) -> Self {
        Self {
            effect_type,
            lifetime: 0.0,
            max_lifetime,
        }
    }

    pub fn update(&mut self, delta: f64) -> bool {
        self.lifetime += delta;
        self.lifetime <= self.max_lifetime
    }

    pub fn is_active(&self) -> bool {
        self.lifetime <= self.max_lifetime
    }

    pub fn render(&self) -> Option<Line<'static>> {
        match &self.effect_type {
            EffectType::DamageNumber { value, is_crit } => {
                let progress = self.lifetime / self.max_lifetime;
                if progress > 0.8 {
                    // Fade out
                    None
                } else {
                    let style = if *is_crit {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let text = if *is_crit {
                        format!("╔═══════╗\n║ CRIT! ║\n║  {}  ║\n╚═══════╝", value)
                    } else {
                        format!("{}", value)
                    };

                    Some(Line::from(Span::styled(text, style)))
                }
            }
            EffectType::AttackFlash => {
                if self.lifetime < 0.1 {
                    Some(Line::from(Span::styled(
                        "⚔".repeat(20),
                        Style::default().fg(Color::Yellow),
                    )))
                } else {
                    None
                }
            }
            EffectType::HitImpact => {
                let frame = (self.lifetime * 10.0) as usize % 3;
                let impact_char = match frame {
                    0 => "*!@#$%",
                    1 => "@#$%^&",
                    _ => "#$%^*!",
                };
                Some(Line::from(Span::styled(
                    impact_char,
                    Style::default().fg(Color::Red),
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_creation() {
        let effect = VisualEffect::new(EffectType::DamageNumber { value: 50, is_crit: false }, 1.0);
        assert!(effect.is_active());
        assert_eq!(effect.lifetime, 0.0);
    }

    #[test]
    fn test_effect_update() {
        let mut effect = VisualEffect::new(EffectType::AttackFlash, 0.2);
        assert!(effect.update(0.1)); // Still active
        assert!(effect.update(0.1)); // Should be done
        assert!(!effect.update(0.1)); // No longer active
    }

    #[test]
    fn test_damage_number_render() {
        let effect = VisualEffect::new(EffectType::DamageNumber { value: 42, is_crit: false }, 1.0);
        let rendered = effect.render();
        assert!(rendered.is_some());
    }
}
