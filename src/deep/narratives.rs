//! Per-layer, per-mission-type narrative flavor text for The Deep.
//!
//! Each of the 30 layers × 8 mission types has a unique 1–2 sentence
//! expedition log entry displayed in the mission detail panel.
//! Construction missions have per-infrastructure narratives (Outpost,
//! SupplyCache, Watchtower, Bridge).

use super::types::{Infrastructure, MissionType};

/// Returns atmospheric narrative text for a given layer and mission type.
///
/// Used in the mission detail panel below the "Layer N — Tier" heading.
/// Returns an empty string for out-of-range layers.
pub fn layer_narrative(layer: u32, mission_type: &MissionType) -> &'static str {
    // Map mission type to a numeric key for compact matching.
    // Each Construction variant gets its own key for unique narratives.
    let key = match mission_type {
        MissionType::SupplyRun => 0,
        MissionType::Recon => 1,
        MissionType::Expedition => 2,
        MissionType::Breakthrough => 3,
        MissionType::Construction(Infrastructure::Outpost) => 4,
        MissionType::Construction(Infrastructure::SupplyCache) => 5,
        MissionType::Construction(Infrastructure::Watchtower) => 6,
        MissionType::Construction(Infrastructure::Bridge) => 7,
        MissionType::GatewayExpedition => 8,
    };

    match (layer, key) {
        // ── The Shallows (Layers 1-3) ──────────────────────────────────────

        (1, 0) => "Ration caches behind a collapsed barracks door. Mostly intact.",
        (1, 1) => "Cold draft through narrow limestone. The walls were carved, not bored.",
        (1, 2) => "A guard station, sealed from the inside. Someone didn't want out.",
        (1, 3) => "Whatever held this entrance was deliberate. The lock is on the wrong side.",
        (1, 4) => "Chokepoint at the barracks junction. We stake our colors here.",
        (1, 5) => "Stone alcove behind the mess hall. Dry, hidden, holds three weeks of kit.",
        (1, 6) => "Guard post above the main corridor. Every approach visible from the bench.",
        (1, 7) => "Gap between sections \u{2014} old drainage trench. Boards and rope, serviceable.",

        (2, 0) => "Old kit bags behind a cave-in. Half the food is still good.",
        (2, 1) => "Patrol routes carved into a stone tablet. Someone mapped this before us.",
        (2, 2) => "A sealed armory. The weapons inside are older than any war we know.",
        (2, 3) => "The passage narrows to a single gate. Something died on this side of it.",
        (2, 4) => "Corner where two patrol routes cross. Whoever holds it controls the block.",
        (2, 5) => "Collapsed storage room, one wall intact. Enough dry space for a season.",
        (2, 6) => "Elevated platform above the armory gate. Clear sight down both forks.",
        (2, 7) => "Sinkhole bisects the eastern passage. We span it with salvaged timber.",

        (3, 0) => "Supply crates stacked behind a ritual warning. We take what we need.",
        (3, 1) => "The inscriptions change here. These aren't orders \u{2014} they're warnings.",
        (3, 2) => "The deeper rooms are hotter. Something below is generating heat.",
        (3, 3) => "A wall of fused stone blocks the descent. Not collapse \u{2014} deliberate sealing.",
        (3, 4) => "Narrow approach, single entry. We fortify what the old garrison left behind.",
        (3, 5) => "Ritual crates left untouched. We clear the sigils and fill them with rations.",
        (3, 6) => "Stone shelf overlooking the descent. Cold air rises \u{2014} anything below moves slow.",
        (3, 7) => "The floor drops six feet where the sealing wall cracked. We bridge the break.",

        // ── The Warrens (Layers 4-7) ───────────────────────────────────────

        (4, 0) => "Abandoned camp gear beside fungal growth. The food is gone; the tools are not.",
        (4, 1) => "The walls are no longer cut stone. Something organic took hold here.",
        (4, 2) => "A cluster of collapsed living quarters. Families lived in these tunnels.",
        (4, 3) => "The passage is alive with root tendrils. They weren't here last week.",
        (4, 4) => "Root tendrils have threaded through the mortar joints. We cut them back; they return by morning.",
        (4, 5) => "Old camp stores cached here, but fungal bloom has split the crates. We reline with oilskin.",
        (4, 6) => "The vantage we need is behind a wall of pale growth. Hacking through it releases spores.",
        (4, 7) => "The gap is narrow but the floor is soft with mycelium. The bridge footing keeps sinking.",

        (5, 0) => "Carved dolls arranged in a ring near a cold fire pit. One set of prints.",
        (5, 1) => "Archive tablets reference a place called the Wellspring. No coordinates.",
        (5, 2) => "The rumbles are closer now. The floor transmits vibration every few hours.",
        (5, 3) => "A guardian of the old occupation. It knows these tunnels better than we do.",
        (5, 4) => "The foundation logs drew root contact overnight. The position holds; the wood will not.",
        (5, 5) => "Dry storage is compromised at every candidate site. Fungal spore counts are unacceptable.",
        (5, 6) => "The growth here is thick enough to block sightlines. We burn a corridor and call it a post.",
        (5, 7) => "The carved dolls were arranged across the crossing point. The squad moved them. The prints returned.",

        (6, 0) => "An archive cache wrapped in wax cloth. The tablets inside are legible.",
        (6, 1) => "Tool marks shift from picks to ritual implements. The work changed hands here.",
        (6, 2) => "Something displaced an entire section of tunnel. Moved, not collapsed.",
        (6, 3) => "The Wellspring references point down. The guardian at this depth is not human.",
        (6, 4) => "Outpost stakes driven through root mats rather than soil. The tendrils reroute around the posts.",
        (6, 5) => "A sealed archive alcove converted to cache use. The tablets inside describe a Wellspring tributary.",
        (6, 6) => "The watchtower column had to be rebuilt twice. Something below keeps shifting the load-bearing floor.",
        (6, 7) => "The crossing spans a growth-filled crevice. Whatever is below the fungus, the bridge does not touch it.",

        (7, 0) => "Rations from the last camp, still sealed. They packed to leave \u{2014} they never did.",
        (7, 1) => "The Overseer's body twitches when we pass. The scouts mark every twitch on the map.",
        (7, 2) => "Something below answered the rumble. Your squad pushed toward it before the sound died.",
        (7, 3) => "The Wellspring tablets end here. Whatever was below is what the Overseer was guarding.",
        (7, 4) => "The Overseer's post stood here once. We build over its outline. The twitching stops while we work.",
        (7, 5) => "Supply stocks must stay elevated. Floor contact rots provisions within a day at this depth.",
        (7, 6) => "The only clear sightline runs directly over where the Overseer fell. The squad avoids looking down.",
        (7, 7) => "The bridge crosses the last mapped gap before the Wellspring descent. We do not name what is below it.",

        // ── The Hollows (Layers 8-12) ──────────────────────────────────────

        (8, 0) => "Crystal formations block the old supply route. Harvest what you can before the light shifts.",
        (8, 1) => "Stalker nests crown the upper walls. Three exits \u{2014} and one route the stalkers avoid.",
        (8, 2) => "The blackwater seam runs deep. Anything in the water moves toward noise.",
        (8, 3) => "The walls pulse once as your squad enters. They pulse again, faster, when blades are drawn.",
        (8, 4) => "The outpost rises where three tunnel mouths meet. The walls pulse approval \u{2014} or warning.",
        (8, 5) => "The cache seals tight against the spore clouds. Your medic marks the rotation schedule in shifts.",
        (8, 6) => "The watchtower stands at the crystal spire's base. The refracted light shows things that aren't moving.",
        (8, 7) => "The blackwater channel splits the approach. Your engineers lay the bridge before the current learns them.",

        (9, 0) => "The Echo was sorting supplies when we arrived. It kept working until we touched the crates.",
        (9, 1) => "Your Arcanist won't say what the light is. She marks the source and says don't return.",
        (9, 2) => "The spore cloud releases when approached. Your medic says it's targeted.",
        (9, 3) => "The guardian didn't attack. It recorded. Your squad killed it reading its own log.",
        (9, 4) => "The Echo helped dig the foundation. Your squad stopped watching after the third hour.",
        (9, 5) => "Provisions stowed. The spores don't penetrate the seal \u{2014} whatever they're looking for isn't inside.",
        (9, 6) => "The tower's sightlines cover the Echo circuit exactly. Your Arcanist says this is coincidence.",
        (9, 7) => "The gap was cut clean, too wide for accident. Something made sure nothing crosses without a bridge.",

        (10, 0) => "Supply cache from a previous expedition \u{2014} not yours. The log ends mid-sentence.",
        (10, 1) => "Echoes walk the same route every six hours. Your scouts time their passes to six minutes.",
        (10, 2) => "Crystal nodes shatter under pressure. The shards track movement.",
        (10, 3) => "The rhythm in the walls stopped when the guardian fell. The silence is worse than the pulse.",
        (10, 4) => "The outpost sits where the old camp was. The stone already knows the shape of walls.",
        (10, 5) => "The cache node clicks twice as the seal drops. The crystal shards outside orient toward the sound.",
        (10, 6) => "The tower sees four tunnels. One of them the scouts report empty \u{2014} it's never empty twice.",
        (10, 7) => "The bridge span holds. The blackwater slows beneath it, then resumes, as if deciding.",

        (11, 0) => "The blackwater tastes of iron and old ash. The filtration cache two layers up makes it safe.",
        (11, 1) => "Three routes forward \u{2014} two of them walked recently. The third shows no prints at all.",
        (11, 2) => "The stalkers don't hunt. They herd. Your medic figures this out after the second ambush.",
        (11, 3) => "The guardian's hands were shaped for something other than fighting. You fought it anyway.",
        (11, 4) => "The walls pulse faster during construction. Slower once the outpost roof goes up.",
        (11, 5) => "The cache seals over a dry ledge the stalkers avoid. Your quartermaster doesn't ask why.",
        (11, 6) => "The watchtower sightline catches the herd route before the next push. Two ambushes avoided in one draw.",
        (11, 7) => "The gap reeks of blackwater and old iron. Your engineers breathe through cloth and don't speak.",

        (12, 0) => "The deep caches are full. Whatever came before you stockpiled, then left without taking anything.",
        (12, 1) => "Past this point, the light doesn't move with the crystals. The memory is fresh.",
        (12, 2) => "The walls stopped pulsing at the advance post. The silence follows your squad out.",
        (12, 3) => "The final guardian recognized the Warband emblem. It lowered its arms first. You didn't.",
        (12, 4) => "The outpost foundation disturbs something under the stone. It doesn't pulse \u{2014} it listens.",
        (12, 5) => "The stockpiles left behind fill the cache without modification. The previous expedition knew the dimensions.",
        (12, 6) => "The tower sees where the light doesn't move. Your scouts won't say what they observed on the first watch.",
        (12, 7) => "The final crossing required. The stone on both sides leans slightly inward \u{2014} the span holds them apart.",

        // ── The Sunken Reach (Layers 13-18) ────────────────────────────────

        (13, 0) => "Salvage the outer antechambers before the tide returns. The rust lines show how high it gets.",
        (13, 1) => "Chart the flooded corridors and locate the sealed inner gates. Note which ones face inward.",
        (13, 2) => "Push into the entry vaults. The water is waist-deep. Sentinels are still at their posts.",
        (13, 3) => "The gate guardian holds a drowned post. It has not moved in centuries. Force it to.",
        (13, 4) => "Secure the high vault above the tide line. Drive anchors before the water returns.",
        (13, 5) => "Pack dry stores into the sealed antechamber. The rust line marks how high the water rises.",
        (13, 6) => "The entry corridors branch in seven directions. Plant the watch post where three of them cross.",
        (13, 7) => "The inner gate is twenty meters across flooded stone. Bridge it. Armor doesn't swim.",

        (14, 0) => "The cleared sections hold more than supplies \u{2014} caches the original occupants left behind.",
        (14, 1) => "Map the seal array and record the glyph sequence. They match marks on the god items.",
        (14, 2) => "Advance past the first seal wall. Something on the other side is louder than it should be.",
        (14, 3) => "Shatter the outer containment barrier. You're past the first lock.",
        (14, 4) => "The seal array makes a defensible perimeter. Set the outpost inside the glyph ring.",
        (14, 5) => "Cache provisions in the dry hollow behind the first seal wall. The glyphs hold the water back.",
        (14, 6) => "The watchtower faces the seal array. Log every shift in the glyph light.",
        (14, 7) => "The flooded passage splits the cleared section. Bridge it or lose half the corridor to the tide.",

        (15, 0) => "Strip the outer halls quickly. The pressure shifts without warning.",
        (15, 1) => "Survey the pressure anomaly zones. The water holds still in chambers it should fill.",
        (15, 2) => "Cross the pressure threshold. The water moves toward the seals, not away.",
        (15, 3) => "Break the middle lock. The pressure change on the other side will be immediate.",
        (15, 4) => "Find the pressure-stable room and build there. The anomaly zones will collapse anything else.",
        (15, 5) => "The dry shelf above the still water holds. Pack the cache before the pressure shifts again.",
        (15, 6) => "The watch post overlooks two pressure zones. Mark the waterline hourly \u{2014} it moves.",
        (15, 7) => "The gap between anomaly chambers floods without warning. Span it now, while the water holds still.",

        (16, 0) => "Retrieve the stockpiles from the throne antechambers. Leave the throne room alone for now.",
        (16, 1) => "Locate the Drowned King's chamber and record the orientation. The throne faces downward.",
        (16, 2) => "Secure the throne level corridors. The formations are ceremonial \u{2014} but the swords are real.",
        (16, 3) => "The throne room guardian has held this post since the sealing. Displace it.",
        (16, 4) => "The dry landing above the throne level is the only ground worth holding. Take it.",
        (16, 5) => "The antechamber stores are accessible from above. Cache provisions where the King's court once ate.",
        (16, 6) => "The throne sits below. The watch post sits above it. Do not let the formations out of sight.",
        (16, 7) => "The throne level is two flooded drops from the antechambers. Bridge both. We don't kneel to wade.",

        (17, 0) => "The sub-throne caches are intact. Whatever was sealed here was expected to be retrieved.",
        (17, 1) => "Chart the containment layer geometry. The barriers all face the same direction.",
        (17, 2) => "Fight through the inner containment ring. These guardians held something in, not out.",
        (17, 3) => "The inner seal is the last layer before the deep chambers. Crack it.",
        (17, 4) => "The containment ring has one dry node. Build the outpost there \u{2014} the barriers face away from it.",
        (17, 5) => "Pack the sub-throne hollow. Whatever was sealed here expected these stores to be collected.",
        (17, 6) => "The inner ring faces inward. The watchtower watches the same direction.",
        (17, 7) => "The final shaft before the deep chambers is flooded at high tide. Bridge the low-water gap.",

        (18, 0) => "Final salvage before the descent. The structure changes here. The mercs can feel it.",
        (18, 1) => "Scout the boundary between the Reach and what lies beyond. The seals here are older.",
        (18, 2) => "Clear the final Reach chambers. Something below has been aware of you since Layer 14.",
        (18, 3) => "Breach the deepest seal. Beyond it, a kingdom older than the world still waits.",
        (18, 4) => "Last solid ground before the descent. Post holds here or there is no fallback.",
        (18, 5) => "Cache the final provisions at the boundary. If the mercs return from below, they'll need them.",
        (18, 6) => "The oldest seals glow faintly. The watch post reads their light and notes when it changes.",
        (18, 7) => "The passage down begins as a flooded drop. Bridge it. Whatever is below has waited long enough.",

        // ── The Abyss (Layers 19-25) ───────────────────────────────────────

        (19, 0) => "The cached supplies were exactly where the maps said. The maps were written after this run.",
        (19, 1) => "Routes charted. The echoes arrive half a second early \u{2014} the stone knows you're coming.",
        (19, 2) => "Three contacts, two cleared. The third wasn't hostile. It was watching.",
        (19, 3) => "The guardian fell. Before it died, Sarn heard it say his name. He's never been this deep.",
        (19, 4) => "Outpost set. The crew found a foundation already in place. Same dimensions. Same cut.",
        (19, 5) => "Cache stocked. The shelving was already assembled \u{2014} bolts torqued, labels face-out.",
        (19, 6) => "Tower placed. The sightlines cover corridors that weren't there when the scouts ran them.",
        (19, 7) => "Bridge laid. The far anchor point was dressed stone. The gap may not have existed yesterday.",

        (20, 0) => "Returned with full packs. The field log shows three days. The squad was down eight hours.",
        (20, 1) => "Mapped the eastern corridor twice. Both charts are accurate. They don't match.",
        (20, 2) => "The hostiles at this depth \u{2014} their gear is ours. Different serial marks, same wear.",
        (20, 3) => "Layer cleared. Davan's axe came back two inches shorter. The edge has never been better.",
        (20, 4) => "Forward post established. Renn's survey notes appear in the log three hours before she wrote them.",
        (20, 5) => "Depot complete. The preservation chalk marks on the crates match Davan's handwriting. He denies it.",
        (20, 6) => "Watchtower complete. The observer reports the same corridor six times. It's a different corridor each time.",
        (20, 7) => "Bridge complete. Both ends rest on stone. The stone was not continuous when the survey began.",

        (21, 0) => "Cache run complete. Lira swears she's done this exact run before. No prior record.",
        (21, 1) => "Scouted the northern galleries. Tomas recognized the damage patterns. First time at this depth.",
        (21, 2) => "Heavy contact. Renn's blade came back with a nick she couldn't explain.",
        (21, 3) => "Guardian down. The squad's orders echoed back ten seconds before they were given.",
        (21, 4) => "Outpost raised. The crew argued about wall placement for an hour. The walls were already where they decided.",
        (21, 5) => "Supply depot finished. The inventory count was accurate before the count was taken.",
        (21, 6) => "Tower footing set. The stone accepted the anchor bolts without drilling \u{2014} the holes were already there.",
        (21, 7) => "Span installed. Tomas recognized the far ledge. He has no record of ever standing on it.",

        (22, 0) => "Salvage complete. The supplies were stacked facing the route home. Nobody here to do it.",
        (22, 1) => "The passage mapped itself \u{2014} walls shifted between sweeps, always to shorten the route.",
        (22, 2) => "No hostiles. Whatever held this ground is gone. The dust says it left before we arrived.",
        (22, 3) => "The guardian recognized the squad. It waited until we were set before it moved.",
        (22, 4) => "Post established. The chalk line for the perimeter was already drawn. Still wet.",
        (22, 5) => "Cache built. The ration rotation schedule posted inside matches the one Mira drafted on the way down.",
        (22, 6) => "Observation post operational. The log book inside has one prior entry. The handwriting is Lira's.",
        (22, 7) => "Crossing complete. The bridge held weight before the final pin was driven. It simply decided to hold.",

        (23, 0) => "Mira found her own handwriting on a crate seal. She's never packed a crate in her life.",
        (23, 1) => "The deeper corridors are lit \u{2014} faintly, from no source. It's enough to read by.",
        (23, 2) => "Three engagements. The last enemy watched the squad pull back. Then it left the way we came in.",
        (23, 3) => "The moment the guardian fell, every wound log cleared. Davan still has the scars.",
        (23, 4) => "Outpost complete. The reinforcement bracing was cut to length before the squad measured the spans.",
        (23, 5) => "Depot anchored. Whatever the crew stores here has been here before. The dust rings prove it.",
        (23, 6) => "Watchtower stands. The second shift observer says he's already filed a report on what he's about to see.",
        (23, 7) => "Bridge finished. The gap it spans is narrower than the span. The stone moved to meet the beam.",

        (24, 0) => "Cache retrieval complete. The supplies were fresh. The ration stamps are dated six weeks from now.",
        (24, 1) => "Final scouting depth before the Wellspring tier. The passage ahead breathes. Literally.",
        (24, 2) => "The hostiles weren't holding ground \u{2014} they were blocking something beyond.",
        (24, 3) => "The Abyss opens deeper. Whatever waits at Layer 25 has been aware since Layer 19.",
        (24, 4) => "Forward base complete. The dedications scratched into the lintel are names from the current roster.",
        (24, 5) => "Supply cache finished. The manifest was pre-written. The items match. The ink isn't dry.",
        (24, 6) => "Tower complete. Two of the three sightlines look into corridors that don't exist yet.",
        (24, 7) => "Bridge complete. The stone is warm underfoot. The crew crossed before the mortar had time to cure.",

        (25, 0) => "The supplies were already moved. Something arranged them first.",
        (25, 1) => "She walked six hours. The timer says three minutes. Map is blank.",
        (25, 2) => "The walls remember your last push. Defenders form from old stone.",
        (25, 3) => "The guardian simply decides you are not allowed. Your mercs disagree.",
        (25, 4) => "The outpost is there. No one is certain it was built. The crew remembers working. The work remembers itself.",
        (25, 5) => "The cache exists because it was needed. The Wellspring tier doesn't let useful things fail to appear.",
        (25, 6) => "The watchtower watches. What it sees, it does not share. The log stays blank. The observer looks content.",
        (25, 7) => "The bridge is there because the crew believed the other side existed. It does. It always did.",

        // ── The Void (Layers 26-30) ────────────────────────────────────────

        (26, 0) => "No ground here in any physical sense. The crates rest on certainty.",
        (26, 1) => "The scout returns knowing, not having seen. The void told her directly.",
        (26, 2) => "The resistance is not creatures. It is doubt. Wavering mercs vanish.",
        (26, 3) => "The breakthrough is believing the next layer exists. Your mercs believe.",
        (26, 4) => "The outpost assembles itself. Your mercs stood where it should be.",
        (26, 5) => "The cache holds what your mercs remember packing. It may hold more.",
        (26, 6) => "The watchtower rises because someone must watch. It rises toward nothing.",
        (26, 7) => "No gap to span. The bridge insists otherwise. Your mercs let it.",

        (27, 0) => "Rations come back full. Your mercs ate. The Medic notes no one looks fed.",
        (27, 1) => "The path she charts has always existed. It appears on older maps.",
        (27, 2) => "Injuries seal before the Medic kneels. The void refuses diminishment.",
        (27, 3) => "The void expected this. It steps aside like a door left unlocked.",
        (27, 4) => "The position is claimed by occupying it. No hammer required.",
        (27, 5) => "The provisions exist because the next push needs them. They appear.",
        (27, 6) => "She watches what isn't there. Her report describes it exactly.",
        (27, 7) => "The bridge connects two points the void had separated on purpose.",

        (28, 0) => "No incident. Your mercs feel watched \u{2014} not with hostility. With patience.",
        (28, 1) => "The void catalogs her as she catalogs it. The notes are not hers.",
        (28, 2) => "Nothing opposes you. Something studies you. It takes detailed notes.",
        (28, 3) => "Layer 28 yields. Whatever watches here has decided in your favor.",
        (28, 4) => "The outpost holds. Whatever studies you marks its coordinates too.",
        (28, 5) => "The cache is full. The observer notes the count. So do you.",
        (28, 6) => "The tower stands where observation already occurred. A formality.",
        (28, 7) => "The gap closes before the bridge finishes. It was already closed.",

        (29, 0) => "Provisions left by no one. Enough for the final push. Not one more.",
        (29, 1) => "The Gateway is visible from here. The scout says it always was.",
        (29, 2) => "The void clears a path \u{2014} not submission. Preparation. It waited.",
        (29, 3) => "Something in the deep exhales. Your mercs feel it in their chests.",
        (29, 4) => "Final position before the Gateway. The void acknowledges the choice.",
        (29, 5) => "Enough provisions for one more layer. No more appear. None needed.",
        (29, 6) => "The tower sees the Gateway clearly. The scout confirms what she felt.",
        (29, 7) => "The last bridge. It spans the threshold between preparation and arrival.",

        (30, 0) => "Ritual now. Your mercs move through the void like they belong.",
        (30, 1) => "She sees the Gateway from Layer 30. She returns changed.",
        (30, 2) => "Nothing here will stop you. Everything here has watched you arrive.",
        (30, 3) => "The void parts. The Gateway stands. The Origin Wound opens.",
        (30, 4) => "The outpost at the end of everything. Your mercs occupy it in silence.",
        (30, 5) => "The cache is left stocked. For whoever comes after, or no one.",
        (30, 6) => "The watchtower faces the Gateway. Nothing watches back. Nothing needs to.",
        (30, 7) => "The bridge reaches the Origin Wound. The void did not resist. It agreed.",
        (30, 8) => "The Gateway opens. Beyond it is not a place. It is why places exist.",

        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NON_CONSTRUCTION: &[MissionType] = &[
        MissionType::SupplyRun,
        MissionType::Recon,
        MissionType::Expedition,
        MissionType::Breakthrough,
        MissionType::GatewayExpedition,
    ];

    const CONSTRUCTION: &[MissionType] = &[
        MissionType::Construction(Infrastructure::Outpost),
        MissionType::Construction(Infrastructure::SupplyCache),
        MissionType::Construction(Infrastructure::Watchtower),
        MissionType::Construction(Infrastructure::Bridge),
    ];

    #[test]
    fn every_layer_has_narratives_for_all_mission_types_except_gateway_expedition() {
        for layer in 1..=30u32 {
            for mission_type in NON_CONSTRUCTION.iter().chain(CONSTRUCTION.iter()) {
                if *mission_type == MissionType::GatewayExpedition {
                    continue;
                }
                let text = layer_narrative(layer, mission_type);
                assert!(
                    !text.is_empty(),
                    "expected narrative for layer {layer}, mission {mission_type:?}"
                );
            }
        }
    }

    #[test]
    fn layer_30_has_gateway_expedition_narrative() {
        let text = layer_narrative(30, &MissionType::GatewayExpedition);
        assert!(!text.is_empty());
        assert!(text.contains("Gateway opens"));
    }

    #[test]
    fn gateway_expedition_narrative_is_empty_below_layer_30() {
        for layer in 1..30u32 {
            assert_eq!(layer_narrative(layer, &MissionType::GatewayExpedition), "");
        }
    }

    #[test]
    fn out_of_range_layer_returns_empty_string() {
        assert_eq!(layer_narrative(0, &MissionType::SupplyRun), "");
        assert_eq!(layer_narrative(31, &MissionType::SupplyRun), "");
        assert_eq!(layer_narrative(100, &MissionType::Breakthrough), "");
    }

    #[test]
    fn specific_known_narratives_match_expected_text() {
        assert_eq!(
            layer_narrative(1, &MissionType::SupplyRun),
            "Ration caches behind a collapsed barracks door. Mostly intact."
        );
        assert_eq!(
            layer_narrative(30, &MissionType::Construction(Infrastructure::Bridge)),
            "The bridge reaches the Origin Wound. The void did not resist. It agreed."
        );
    }
}
