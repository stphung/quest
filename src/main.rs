mod constants;

fn main() {
    println!("Idle RPG - Coming Soon");

    // Verify constants are imported and accessible
    println!("Tick interval: {}ms", constants::TICK_INTERVAL_MS);
    println!("Base XP per tick: {}", constants::BASE_XP_PER_TICK);
}
