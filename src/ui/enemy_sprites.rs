/// Enemy sprite templates for 3D rendering
pub struct EnemySprite {
    pub base_art: &'static str,
    #[allow(dead_code)]
    pub width: usize,
    #[allow(dead_code)]
    pub height: usize,
}

impl EnemySprite {
    pub const fn new(art: &'static str, width: usize, height: usize) -> Self {
        Self {
            base_art: art,
            width,
            height,
        }
    }
}

// Sprite templates (10 lines tall base)

pub const SPRITE_ORC: EnemySprite = EnemySprite::new(
    r"      ╱╲
     ╱██╲
    │████│
   ╱██████╲
  │ ●    ● │
  │   ▼    │
  ╰────────╯
   ││    ││
   ││    ││
   ╰╯    ╰╯   ",
    14,
    10,
);

pub const SPRITE_TROLL: EnemySprite = EnemySprite::new(
    r"     ╱██╲
    ╱████╲
   ╱██████╲
  ╱████████╲
  │●      ●│
  │   ▼▼   │
  │ ╱────╲ │
  ╰────────╯
   ││    ││
   ╰╯    ╰╯   ",
    14,
    10,
);

pub const SPRITE_DRAKE: EnemySprite = EnemySprite::new(
    r"   ╱╲    ╱╲
  ╱  ╲  ╱  ╲
 ╱   ████   ╲
╱   ██████   ╲
│   ◆    ◆   │
│     ▼▼     │
╰─┬────────┬─╯
  │  ╱╲╱╲  │
  └──────┘
    ╰╯  ╰╯    ",
    14,
    10,
);

pub const SPRITE_BEAST: EnemySprite = EnemySprite::new(
    r"   ╱╲  ╱╲
  ╱  ╲╱  ╲
 ╱  ████  ╲
╱  ██████  ╲
│  ●    ●  │
│    ▼▼    │
╰──┬────┬──╯
   │    │
   ╰╯  ╰╯
              ",
    14,
    10,
);

pub const SPRITE_HORROR: EnemySprite = EnemySprite::new(
    r"     ╱██╲
    ╱ ██ ╲
   ╱ ████ ╲
  │  ●  ●  │
  │    ●   │
  │  ◆ ◆   │
  │ ╱───╲  │
  ╰───────╯
   ╱ ╲ ╱ ╲
  ╰───╯───╯   ",
    14,
    10,
);

pub const SPRITE_CRUSHER: EnemySprite = EnemySprite::new(
    r"   ═══════
    ╱█████╲
   ╱███████╲
  │ ●     ● │
  │    ▼    │
  │  ╱═══╲  │
  ╰─────────╯
   ║│    │║
   ║│    │║
   ╰╯    ╰╯   ",
    14,
    10,
);

/// Gets the appropriate sprite template for an enemy name
pub fn get_sprite_for_enemy(enemy_name: &str) -> &'static EnemySprite {
    let name_lower = enemy_name.to_lowercase();

    if name_lower.contains("orc") {
        &SPRITE_ORC
    } else if name_lower.contains("troll") {
        &SPRITE_TROLL
    } else if name_lower.contains("drake") {
        &SPRITE_DRAKE
    } else if name_lower.contains("beast") || name_lower.contains("fiend") {
        &SPRITE_BEAST
    } else if name_lower.contains("horror") || name_lower.contains("terror") {
        &SPRITE_HORROR
    } else if name_lower.contains("crusher")
        || name_lower.contains("render")
        || name_lower.contains("maw")
    {
        &SPRITE_CRUSHER
    } else {
        // Default to generic beast
        &SPRITE_BEAST
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sprite_for_orc() {
        let sprite = get_sprite_for_enemy("Grizzled Orc");
        assert_eq!(sprite.height, 10);
        assert!(sprite.base_art.contains("●"));
    }

    #[test]
    fn test_get_sprite_for_drake() {
        let sprite = get_sprite_for_enemy("Dark Drake");
        assert_eq!(sprite.height, 10);
        assert!(sprite.base_art.contains("◆"));
    }

    #[test]
    fn test_get_sprite_default() {
        let sprite = get_sprite_for_enemy("Unknown Monster");
        assert_eq!(sprite.height, 10);
    }

    #[test]
    fn test_sprite_dimensions() {
        assert_eq!(SPRITE_ORC.height, 10);
        assert_eq!(SPRITE_TROLL.height, 10);
        assert_eq!(SPRITE_DRAKE.height, 10);
    }
}
