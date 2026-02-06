//! Demo of advanced TUI sprite rendering techniques
//! Run with: cargo run --example sprite_demo

use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // Clear screen and hide cursor
    print!("\x1b[2J\x1b[H\x1b[?25l");
    io::stdout().flush().unwrap();

    println!("\x1b[1;36m═══════════════════════════════════════════════════════════════\x1b[0m");
    println!("\x1b[1;36m              QUEST SPRITE RENDERING DEMO\x1b[0m");
    println!("\x1b[1;36m═══════════════════════════════════════════════════════════════\x1b[0m\n");

    // 1. Current ASCII Art Style
    println!("\x1b[1;33m▎ 1. CURRENT STYLE (Box Drawing)\x1b[0m\n");
    println!("\x1b[31m        ╱╲    ╱╲");
    println!("       ╱  ╲  ╱  ╲");
    println!("      ╱   ████   ╲");
    println!("     ╱   ██████   ╲");
    println!("     │   ◆    ◆   │");
    println!("     │     ▼▼     │");
    println!("     ╰─┬────────┬─╯");
    println!("       │  ╱╲╱╲  │");
    println!("       └────────┘");
    println!("         ╰╯  ╰╯\x1b[0m");
    println!("\n   \x1b[33mFire Drake\x1b[0m\n");

    sleep(Duration::from_secs(2));

    // 2. Half-Block Pixel Art
    println!("\x1b[1;33m▎ 2. HALF-BLOCK PIXEL ART (2x vertical resolution)\x1b[0m\n");

    // Orc in half-blocks with shading
    println!("   \x1b[38;2;60;60;60m░░░░░░\x1b[38;2;100;60;40m▄▄▄▄▄▄\x1b[38;2;60;60;60m░░░░░░\x1b[0m");
    println!("   \x1b[38;2;60;60;60m░░░░\x1b[38;2;100;60;40m▄\x1b[38;2;140;90;60m██████\x1b[38;2;100;60;40m▄\x1b[38;2;60;60;60m░░░░\x1b[0m");
    println!("   \x1b[38;2;60;60;60m░░░\x1b[38;2;140;90;60m██\x1b[38;2;180;120;80m██████\x1b[38;2;140;90;60m██\x1b[38;2;60;60;60m░░░\x1b[0m");
    println!("   \x1b[38;2;60;60;60m░░\x1b[38;2;140;90;60m██\x1b[38;2;255;50;50m▀▀\x1b[38;2;180;120;80m████\x1b[38;2;255;50;50m▀▀\x1b[38;2;140;90;60m██\x1b[38;2;60;60;60m░░\x1b[0m");
    println!("   \x1b[38;2;60;60;60m░░\x1b[38;2;140;90;60m████\x1b[38;2;40;40;40m▄▄▄▄\x1b[38;2;140;90;60m████\x1b[38;2;60;60;60m░░\x1b[0m");
    println!("   \x1b[38;2;60;60;60m░░░\x1b[38;2;100;60;40m▀\x1b[38;2;140;90;60m████████\x1b[38;2;100;60;40m▀\x1b[38;2;60;60;60m░░░\x1b[0m");
    println!("   \x1b[38;2;60;60;60m░░░░\x1b[38;2;80;50;30m██\x1b[38;2;60;60;60m░░░░\x1b[38;2;80;50;30m██\x1b[38;2;60;60;60m░░░░\x1b[0m");
    println!("   \x1b[38;2;60;60;60m░░░░\x1b[38;2;80;50;30m▀▀\x1b[38;2;60;60;60m░░░░\x1b[38;2;80;50;30m▀▀\x1b[38;2;60;60;60m░░░░\x1b[0m");
    println!("\n   \x1b[38;2;200;150;100mWarrior Orc\x1b[0m\n");

    sleep(Duration::from_secs(2));

    // Dragon with gradients
    println!("\x1b[1;33m▎ 3. GRADIENT SHADING (True Color)\x1b[0m\n");

    println!("       \x1b[38;2;100;0;0m▄\x1b[38;2;150;20;0m█\x1b[38;2;200;50;0m█\x1b[38;2;255;100;0m█\x1b[38;2;255;150;50m█\x1b[38;2;200;50;0m█\x1b[38;2;150;20;0m█\x1b[38;2;100;0;0m▄\x1b[0m");
    println!("     \x1b[38;2;100;0;0m▄\x1b[38;2;200;50;0m█\x1b[38;2;255;100;0m█\x1b[38;2;255;150;50m████\x1b[38;2;255;100;0m█\x1b[38;2;200;50;0m█\x1b[38;2;100;0;0m▄\x1b[0m");
    println!("    \x1b[38;2;150;20;0m█\x1b[38;2;255;100;0m██\x1b[38;2;255;255;100m◆\x1b[38;2;255;150;50m████\x1b[38;2;255;255;100m◆\x1b[38;2;255;100;0m██\x1b[38;2;150;20;0m█\x1b[0m");
    println!("    \x1b[38;2;200;50;0m██\x1b[38;2;255;100;0m████████\x1b[38;2;200;50;0m██\x1b[0m");
    println!("     \x1b[38;2;150;20;0m▀\x1b[38;2;200;50;0m██\x1b[38;2;255;100;0m████\x1b[38;2;200;50;0m██\x1b[38;2;150;20;0m▀\x1b[0m");
    println!("       \x1b[38;2;100;0;0m▀▀\x1b[38;2;150;20;0m████\x1b[38;2;100;0;0m▀▀\x1b[0m");
    println!("\n   \x1b[38;2;255;100;0mInferno Drake\x1b[0m\n");

    sleep(Duration::from_secs(2));

    // 4. Braille high-res
    println!("\x1b[1;33m▎ 4. BRAILLE DOT MATRIX (8 dots per cell)\x1b[0m\n");

    // Skull in braille - each character is 2x4 dots
    println!("   \x1b[37m⠀⠀⠀⣀⣤⣶⣶⣶⣶⣤⣀⠀⠀⠀\x1b[0m");
    println!("   \x1b[37m⠀⠀⣴⣿⣿⣿⣿⣿⣿⣿⣿⣷⡀⠀\x1b[0m");
    println!("   \x1b[37m⠀⣼⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣧⠀\x1b[0m");
    println!("   \x1b[37m⠀⣿⣿\x1b[31m⣿⣿\x1b[37m⣿⣿⣿\x1b[31m⣿⣿\x1b[37m⣿⣿⠀\x1b[0m");
    println!("   \x1b[37m⠀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠀\x1b[0m");
    println!("   \x1b[37m⠀⢿⣿⣿⣿\x1b[90m⣤⣤⣤⣤\x1b[37m⣿⣿⡿⠀\x1b[0m");
    println!("   \x1b[37m⠀⠈⢿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠁⠀\x1b[0m");
    println!("   \x1b[37m⠀⠀⠀⠙⠻⠿⠿⠿⠿⠟⠋⠀⠀⠀\x1b[0m");
    println!("\n   \x1b[37mAncient Lich\x1b[0m\n");

    sleep(Duration::from_secs(2));

    // 5. HP-based coloring
    println!("\x1b[1;33m▎ 5. HP-BASED COLOR TINTING\x1b[0m\n");

    let sprite = [
        "    ▄████▄    ",
        "   ████████   ",
        "   ██◆◆████   ",
        "   ████████   ",
        "    ▀████▀    ",
    ];

    // 100% HP - Green
    println!("   \x1b[1;32m100% HP:\x1b[0m");
    for line in &sprite {
        println!("   \x1b[38;2;100;255;100m{}\x1b[0m", line);
    }

    // 50% HP - Yellow
    println!("\n   \x1b[1;33m50% HP:\x1b[0m");
    for line in &sprite {
        println!("   \x1b[38;2;255;255;100m{}\x1b[0m", line);
    }

    // 25% HP - Orange
    println!("\n   \x1b[1;38;2;255;165;0m25% HP:\x1b[0m");
    for line in &sprite {
        println!("   \x1b[38;2;255;140;50m{}\x1b[0m", line);
    }

    // 10% HP - Red + pulsing would happen in real impl
    println!("\n   \x1b[1;31m10% HP:\x1b[0m");
    for line in &sprite {
        println!("   \x1b[38;2;255;50;50m{}\x1b[0m", line);
    }

    sleep(Duration::from_secs(2));

    // 6. Animation demo
    println!("\n\x1b[1;33m▎ 6. IDLE ANIMATION (breathing cycle)\x1b[0m\n");

    let frames = [
        // Frame 1 - neutral
        vec![
            "      ▄████▄      ",
            "     ████████     ",
            "     ██\x1b[33m◆\x1b[31m  \x1b[33m◆\x1b[31m██     ",
            "     ████████     ",
            "      ▀████▀      ",
            "       ████       ",
            "      ██  ██      ",
        ],
        // Frame 2 - inhale
        vec![
            "      ▄████▄      ",
            "    ██████████    ",
            "    ███\x1b[33m◆\x1b[31m  \x1b[33m◆\x1b[31m███    ",
            "    ██████████    ",
            "     ▀██████▀     ",
            "       ████       ",
            "      ██  ██      ",
        ],
        // Frame 3 - exhale
        vec![
            "      ▄████▄      ",
            "     ████████     ",
            "     ██\x1b[33m◆\x1b[31m  \x1b[33m◆\x1b[31m██     ",
            "     ████████     ",
            "      ▀████▀      ",
            "       ████       ",
            "      ██  ██      ",
        ],
        // Frame 4 - slight movement
        vec![
            "       ▄████▄     ",
            "      ████████    ",
            "      ██\x1b[33m◆\x1b[31m  \x1b[33m◆\x1b[31m██    ",
            "      ████████    ",
            "       ▀████▀     ",
            "        ████      ",
            "       ██  ██     ",
        ],
    ];

    // Animate for a few cycles
    for _ in 0..3 {
        for frame in &frames {
            // Move cursor up to redraw
            print!("\x1b[8A");
            for line in frame {
                println!("   \x1b[31m{}\x1b[0m", line);
            }
            io::stdout().flush().unwrap();
            sleep(Duration::from_millis(250));
        }
    }

    println!("\n   \x1b[33mBlood Ogre\x1b[0m (animated)\n");

    sleep(Duration::from_secs(1));

    // 7. Hit effect demo
    println!("\x1b[1;33m▎ 7. HIT FLASH EFFECT\x1b[0m\n");

    let normal_sprite = [
        "      \x1b[31m▄████▄\x1b[0m      ",
        "     \x1b[31m████████\x1b[0m     ",
        "     \x1b[31m██\x1b[33m◆\x1b[31m  \x1b[33m◆\x1b[31m██\x1b[0m     ",
        "     \x1b[31m████████\x1b[0m     ",
        "      \x1b[31m▀████▀\x1b[0m      ",
    ];

    let flash_sprite = [
        "      \x1b[97;47m▄████▄\x1b[0m      ",
        "     \x1b[97;47m████████\x1b[0m     ",
        "     \x1b[97;47m██◆  ◆██\x1b[0m     ",
        "     \x1b[97;47m████████\x1b[0m     ",
        "      \x1b[97;47m▀████▀\x1b[0m      ",
    ];

    for _ in 0..3 {
        // Normal
        print!("\x1b[6A");
        for line in &normal_sprite {
            println!("   {}", line);
        }
        io::stdout().flush().unwrap();
        sleep(Duration::from_millis(400));

        // Flash (simulates hit)
        print!("\x1b[5A");
        println!("   \x1b[1;33m-15 DMG!\x1b[0m");
        for line in &flash_sprite {
            println!("   {}", line);
        }
        io::stdout().flush().unwrap();
        sleep(Duration::from_millis(100));
    }

    println!();
    sleep(Duration::from_secs(1));

    // 8. Particle effects
    println!("\n\x1b[1;33m▎ 8. PARTICLE EFFECTS (damage sparks)\x1b[0m\n");

    let base = [
        "      ▄████▄      ",
        "     ████████     ",
        "     ██◆  ◆██     ",
        "     ████████     ",
        "      ▀████▀      ",
    ];

    let particles = [
        ["  ✦", "    *", " ·", "      ", "   "],
        ["", "  ✧  ", "     *", "  ·", ""],
        ["", "", "    ✦", "  * ", "    ·"],
        ["", "", "", "   ✧", "  *  "],
    ];

    for p in &particles {
        print!("\x1b[6A");
        for (i, line) in base.iter().enumerate() {
            println!("   \x1b[31m{}\x1b[33m{}\x1b[0m", line, p[i]);
        }
        io::stdout().flush().unwrap();
        sleep(Duration::from_millis(150));
    }

    println!();
    sleep(Duration::from_secs(1));

    // 9. Boss entrance
    println!("\n\x1b[1;33m▎ 9. BOSS ENTRANCE ANIMATION\x1b[0m\n");

    // Stage 1: darkness with eyes
    println!("   \x1b[90m░░░░░░░░░░░░░░░░░░\x1b[0m");
    println!("   \x1b[90m░░░░░░░░░░░░░░░░░░\x1b[0m");
    println!("   \x1b[90m░░░░░\x1b[31m◆\x1b[90m░░░░░░\x1b[31m◆\x1b[90m░░░░░\x1b[0m");
    println!("   \x1b[90m░░░░░░░░░░░░░░░░░░\x1b[0m");
    println!("   \x1b[90m░░░░░░░░░░░░░░░░░░\x1b[0m");
    println!("   \x1b[90m░░░░░░░░░░░░░░░░░░\x1b[0m");
    io::stdout().flush().unwrap();
    sleep(Duration::from_secs(1));

    // Stage 2: silhouette
    print!("\x1b[6A");
    println!("   \x1b[90m░░░░░\x1b[30;40m▄██████▄\x1b[90m░░░░░\x1b[0m");
    println!("   \x1b[90m░░░\x1b[30;40m▄████████████▄\x1b[90m░░░\x1b[0m");
    println!("   \x1b[90m░░\x1b[30;40m█████\x1b[31m◆\x1b[30;40m████\x1b[31m◆\x1b[30;40m█████\x1b[90m░░\x1b[0m");
    println!("   \x1b[90m░░\x1b[30;40m██████████████\x1b[90m░░\x1b[0m");
    println!("   \x1b[90m░░░\x1b[30;40m▀██████████▀\x1b[90m░░░\x1b[0m");
    println!("   \x1b[90m░░░░░\x1b[30;40m████████\x1b[90m░░░░░\x1b[0m");
    io::stdout().flush().unwrap();
    sleep(Duration::from_secs(1));

    // Stage 3: full reveal
    print!("\x1b[6A");
    println!("       \x1b[38;2;128;0;128m▄██████▄\x1b[0m       ");
    println!("     \x1b[38;2;148;0;148m▄████████████▄\x1b[0m     ");
    println!("    \x1b[38;2;168;0;168m█████\x1b[38;2;255;0;255m◆\x1b[38;2;168;0;168m████\x1b[38;2;255;0;255m◆\x1b[38;2;168;0;168m█████\x1b[0m    ");
    println!("    \x1b[38;2;148;0;148m██████████████\x1b[0m    ");
    println!("     \x1b[38;2;128;0;128m▀██████████▀\x1b[0m     ");
    println!("       \x1b[38;2;108;0;108m████████\x1b[0m       ");
    println!("\n   \x1b[1;38;2;255;0;255m☠ VOID EMPEROR ☠\x1b[0m\n");

    sleep(Duration::from_secs(2));

    // Show cursor and end
    print!("\x1b[?25h");
    println!("\x1b[1;36m═══════════════════════════════════════════════════════════════\x1b[0m");
    println!("\x1b[1;32m Demo complete! Press Ctrl+C to exit.\x1b[0m");
    println!("\x1b[1;36m═══════════════════════════════════════════════════════════════\x1b[0m");

    // Keep alive so user can see final state
    loop {
        sleep(Duration::from_secs(1));
    }
}
