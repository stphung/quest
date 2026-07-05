//! Letters From Home (sub-project 6).
//!
//! Act 1's world does not vanish at launch — it writes. One letter waits at
//! every Chapter I/II arrival (delivered at ports, never on timers: nothing
//! is missable), sequenced so the correspondence quietly darkens. The
//! Threshold hands over the Last Letter; past the Last Lantern comes the
//! night the mail does not come, and after that the only lights are the
//! ones the Vessel carries.
//!
//! Act 1's runtime fate is design law here: frozen at launch, the letters
//! its only interface, and the Going-Dark closes even that.
//!
//! See `openspec/specs/vessel-act2/spec.md`.

use super::souls::SoulId;

/// The parcel tucked into every letter: small on purpose — letters are word
/// from home, not a second pantry.
pub const LETTER_PARCEL_PROVISIONS: f64 = 5.0;

/// One authored letter.
#[derive(Debug, Clone, Copy)]
pub struct LetterDef {
    pub sender: &'static str,
    pub text: &'static str,
    /// A postscript that only reads when its soul is aboard.
    pub postscript: Option<(SoulId, &'static str)>,
}

const fn letter(sender: &'static str, text: &'static str) -> LetterDef {
    LetterDef {
        sender,
        text,
        postscript: None,
    }
}

/// The sequence, in delivery order. Read together they tell the story of
/// home slowly banking its fires — foreshadowing without announcement.
pub const LETTERS: [LetterDef; 12] = [
    letter(
        "the Haven's new warden",
        "The rooms stand. I have kept your bunk made, which the committee \
         calls sentimental and I call readiness. The hearth draws well. \
         Everything you built is doing what you built it to do.",
    ),
    LetterDef {
        sender: "the Deep guild",
        text: "Layer 30 holds. Dues collected, ale drunk, nobody dead who \
               didn't sign for it. The lift you paid for runs sweet.",
        postscript: Some((
            SoulId(0),
            "P.S. Tell Torvald his chair is still his chair. We voted.",
        )),
    },
    letter(
        "the fisher-fleets",
        "Big run of silverback this month. We salted more than we can eat, \
         so some of it is in this box. The sea here still behaves, mostly, \
         which we take as your doing somehow.",
    ),
    LetterDef {
        sender: "the Loom-tenders",
        text: "The Loom hums your patterns without being asked, the way a \
               house settles in the shape of the family that left it. We \
               send the surplus. It wants sending; we can tell.",
        postscript: None,
    },
    LetterDef {
        sender: "the children of the harbor",
        text: "We drew your ship. Aldi made the sails too big and will not \
               be told. The teacher says the void is very large and we say \
               so is a whale and nobody is scared of THOSE anymore.",
        postscript: Some((
            SoulId(2),
            "P.S. Is Runa's skiff lonely? We visit it. It has a new pair \
             of hands now but we did ask it first.",
        )),
    },
    letter(
        "the Haven's new warden",
        "A fracture zone dimmed last week — the far one, past the salt \
         flats. The committee logged it as weather. I am telling you \
         because you would want the truth logged somewhere too.",
    ),
    letter(
        "the Deep guild",
        "Quieter month. Two crews retired, no new signings. The deep is \
         still the deep, but it listens more than it used to, if that \
         makes sense. Probably doesn't. Dues enclosed in salt cod.",
    ),
    letter(
        "the fisher-fleets",
        "The northern lanes went still — no fish, no weather, just still. \
         We fish south now and don't discuss it at table. Box is lighter \
         this month. Sorry. Catch what you can out there.",
    ),
    LetterDef {
        sender: "the Loom-tenders",
        text: "Three shuttles stopped. Not broken — finished, is the word \
               we've settled on. The patterns they carried are with you \
               now, we think. The Loom hums lower. It is not unhappy.",
        postscript: None,
    },
    letter(
        "the children of the harbor",
        "School moved into the Haven because the nights got long. We like \
         it. The warden lets us light one lantern each. Aldi asked if you \
         can see them from the void. Can you? Write back.",
    ),
    letter(
        "the Haven's new warden",
        "Half the harbor came up the hill this winter. We hold. I want the \
         record to show the design margins were generous and the builder \
         thought of us before we knew to think of ourselves.",
    ),
    letter(
        "the Loom-tenders",
        "The Loom gave us one more spool and went quiet. Not cold — quiet, \
         like a singer after the song. We wove the spool into blankets and \
         put the children in them. Whatever you are sailing toward, the \
         weave says you are nearly there. Trust the weave. We always did.",
    ),
];

/// Handed over at the Threshold, co-signed by everyone. It knows what it is.
pub const LAST_LETTER: LetterDef = LetterDef {
    sender: "everyone",
    text: "This one is signed by all of us — warden and guild, fleets and \
           tenders, and every child who could hold the pen. The postmaster \
           says this is the furthest a letter can go, so we are putting \
           everything in it: the harbor is lit, the blankets are warm, and \
           nobody here is sorry you went. Carry us the rest of the way. \
           You have carried us this far.",
    postscript: None,
};

/// The log's shortest entry.
pub const MAIL_FAILS_LOG: &str = "No letters. There will be no more letters.";

/// Sentinel indices for the letter event queue.
pub const LAST_LETTER_EVENT: u8 = 200;
pub const MAIL_FAILS_EVENT: u8 = 201;

pub fn letter_by_event(event: u8) -> Option<&'static LetterDef> {
    match event {
        LAST_LETTER_EVENT => Some(&LAST_LETTER),
        MAIL_FAILS_EVENT => None,
        i => LETTERS.get(i as usize),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twelve_letters_and_the_last_one() {
        assert_eq!(LETTERS.len(), 12);
        for l in &LETTERS {
            assert!(!l.sender.is_empty() && !l.text.is_empty());
        }
        assert!(!LAST_LETTER.text.is_empty());
    }

    #[test]
    fn postscripts_key_to_real_souls() {
        for l in &LETTERS {
            if let Some((soul, text)) = l.postscript {
                assert!((soul.0 as usize) < crate::vessel::souls::SOULS.len());
                assert!(!text.is_empty());
            }
        }
    }

    #[test]
    fn event_indices_resolve() {
        assert!(letter_by_event(0).is_some());
        assert!(letter_by_event(11).is_some());
        assert!(letter_by_event(12).is_none(), "the mail thins past twelve");
        assert!(letter_by_event(LAST_LETTER_EVENT).is_some());
        assert!(letter_by_event(MAIL_FAILS_EVENT).is_none());
    }
}
