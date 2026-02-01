/// Scales ASCII art sprites vertically while preserving aspect ratio
///
/// Takes a multi-line ASCII sprite and scales it to a target height.
/// Uses sampling to maintain visual appearance at different sizes.

/// Scales a sprite to the target height
pub fn scale_sprite(sprite: &str, target_height: usize) -> Vec<String> {
    let lines: Vec<&str> = sprite.lines().collect();
    if lines.is_empty() {
        return vec![];
    }

    let source_height = lines.len();
    if target_height == source_height {
        return lines.iter().map(|s| s.to_string()).collect();
    }

    let mut scaled = Vec::new();

    for i in 0..target_height {
        // Sample from source using linear interpolation
        let source_index = (i * source_height) / target_height;
        let source_line = lines[source_index.min(source_height - 1)];
        scaled.push(source_line.to_string());
    }

    scaled
}

/// Applies depth shading to ASCII art using character density
pub fn apply_depth_shading(sprite: Vec<String>, depth: f64) -> Vec<String> {
    // Depth 0.0-0.3 = far (light chars)
    // Depth 0.3-0.7 = medium
    // Depth 0.7-1.0 = close (dark chars)

    let shading_map: &[(char, char)] = if depth < 0.3 {
        // Far away - lighten
        &[('█', '#'), ('#', '+'), ('+', ':'), ('*', '.'), ('●', '○')]
    } else if depth < 0.7 {
        // Medium distance - some lightening
        &[('█', '@'), ('●', '◆')]
    } else {
        // Close - keep dark/enhance
        &[(':', '+'), ('.', ':')]
    };

    sprite
        .iter()
        .map(|line| apply_char_map(line, shading_map))
        .collect()
}

fn apply_char_map(line: &str, char_map: &[(char, char)]) -> String {
    line.chars()
        .map(|c| {
            // Find first matching replacement
            for (from, to) in char_map {
                if c == *from {
                    return *to;
                }
            }
            c
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_sprite_same_size() {
        let sprite = "AB\nCD\nEF";
        let scaled = scale_sprite(sprite, 3);
        assert_eq!(scaled.len(), 3);
        assert_eq!(scaled[0], "AB");
        assert_eq!(scaled[1], "CD");
        assert_eq!(scaled[2], "EF");
    }

    #[test]
    fn test_scale_sprite_smaller() {
        let sprite = "Line1\nLine2\nLine3\nLine4";
        let scaled = scale_sprite(sprite, 2);
        assert_eq!(scaled.len(), 2);
        // Should sample lines 0 and 2
        assert_eq!(scaled[0], "Line1");
        assert_eq!(scaled[1], "Line3");
    }

    #[test]
    fn test_scale_sprite_larger() {
        let sprite = "A\nB";
        let scaled = scale_sprite(sprite, 4);
        assert_eq!(scaled.len(), 4);
        // Should duplicate lines
        assert_eq!(scaled[0], "A");
        assert_eq!(scaled[1], "A");
        assert_eq!(scaled[2], "B");
        assert_eq!(scaled[3], "B");
    }

    #[test]
    fn test_apply_depth_shading_far() {
        let sprite = vec!["███".to_string(), "●●●".to_string()];
        let shaded = apply_depth_shading(sprite, 0.2);
        assert_eq!(shaded[0], "###");
        assert_eq!(shaded[1], "○○○");
    }

    #[test]
    fn test_apply_depth_shading_close() {
        let sprite = vec!["...".to_string(), ":::".to_string()];
        let shaded = apply_depth_shading(sprite, 0.8);
        assert_eq!(shaded[0], ":::");
        assert_eq!(shaded[1], "+++");
    }
}
