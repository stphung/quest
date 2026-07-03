//! Junction cards — computed display data for the roads out of a waypoint.
//!
//! The card is the whole choice surface of Act 2: final integer prices, the
//! stops a road is known (or rumored) to hold, its authored character, and
//! whether it can be sailed. The UI renders this struct; tests snapshot it.

use super::route::{self, Road, RoadTag, RumorSubject, WaypointId};
use super::voyage::{VoyageState, MINUTES_PER_DAY};

/// One stop line on a card. Rumor-revealed stops are marked so the UI can
/// show where the knowledge came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopLine {
    pub name: &'static str,
    pub revealed_by_rumor: bool,
}

/// A road as the junction shows it. Everything is already composed: prices
/// include the current trim, annotations include every relevant rumor.
#[derive(Debug, Clone, PartialEq)]
pub struct RoadCard {
    pub road: &'static Road,
    /// "Toward Saint Elm's Rest" — the card's header line.
    pub title: String,
    pub blurb: &'static str,
    /// Authored character tags, display-named ("hungry", "tolled"...).
    pub character: Vec<&'static str>,
    pub stops: Vec<StopLine>,
    /// "about 2 days" / "2–3 days" — never a multiplier, never a float.
    pub days_label: String,
    /// Final whole-leg price at the current trim. A whole number.
    pub provisions_price: u32,
    /// Rumor lines that attach to this road.
    pub annotations: Vec<&'static str>,
    /// A named threat is always on the card — roads keep no secrets they
    /// were authored to tell.
    pub threat: Option<&'static str>,
    pub affordable: bool,
    /// Unaffordable roads render locked — unless this is the cheapest way
    /// out, which is never locked (running dry means drifting, not
    /// stranding).
    pub selectable: bool,
}

/// Cards for every road out of `waypoint`, in authored order.
pub fn junction_cards(voyage: &VoyageState, waypoint: WaypointId) -> Vec<RoadCard> {
    route::roads_from(waypoint)
        .map(|road| road_card(voyage, road))
        .collect()
}

/// Cards for the waypoint the voyage currently holds at, if any.
pub fn current_junction_cards(voyage: &VoyageState) -> Vec<RoadCard> {
    voyage
        .current_waypoint()
        .map(|w| junction_cards(voyage, w))
        .unwrap_or_default()
}

fn road_card(voyage: &VoyageState, road: &'static Road) -> RoadCard {
    let mut stops: Vec<StopLine> = road
        .known_stops
        .iter()
        .map(|id| StopLine {
            name: route::waypoint(*id).name,
            revealed_by_rumor: false,
        })
        .collect();
    let mut annotations = Vec::new();

    for learned in &voyage.rumors {
        let def = route::rumor(learned.rumor);
        let attaches = match def.subject {
            RumorSubject::Road(rid) => rid == road.id,
            // A rumor about a place attaches wherever that place is a stop
            // or the road's destination.
            RumorSubject::Place(wid) => wid == road.to || road.known_stops.contains(&wid),
        };
        if !attaches {
            continue;
        }
        annotations.push(def.text);
        if let Some(stop) = def.reveals_stop {
            let name = route::waypoint(stop).name;
            if !stops.iter().any(|s| s.name == name) {
                stops.push(StopLine {
                    name,
                    revealed_by_rumor: true,
                });
            }
        }
    }

    RoadCard {
        road,
        title: format!("Toward {}", route::waypoint(road.to).name),
        blurb: road.blurb,
        character: road.character.iter().map(RoadTag::display_name).collect(),
        stops,
        days_label: days_label(road, voyage),
        provisions_price: voyage.road_price(road),
        annotations,
        threat: road.threat.map(|t| t.name),
        affordable: voyage.road_affordable(road),
        selectable: voyage.road_selectable(road),
    }
}

/// Small integers on screen: the estimate is rounded to half-days and shown
/// as either "about N days" or a "N–M days" range.
fn days_label(road: &Road, voyage: &VoyageState) -> String {
    let est_days = f64::from(road.base_days) * voyage.trim.time_mult();
    let est_minutes = est_days * MINUTES_PER_DAY as f64;
    if est_minutes < MINUTES_PER_DAY as f64 * 0.75 {
        return "under a day".to_string();
    }
    let lo = est_days.floor() as u32;
    let hi = est_days.ceil() as u32;
    if lo == hi || est_days - f64::from(lo) < 0.25 {
        let n = if est_days - f64::from(lo) < 0.25 {
            lo
        } else {
            hi
        };
        if n <= 1 {
            "about a day".to_string()
        } else {
            format!("about {n} days")
        }
    } else if f64::from(hi) - est_days < 0.25 {
        format!("about {hi} days")
    } else {
        format!("{lo}\u{2013}{hi} days")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vessel::route::{RoadId, RumorId, ROUTE_START};
    use crate::vessel::voyage::{LearnedRumor, Trim};
    use chrono::{DateTime, Utc};

    fn t0() -> DateTime<Utc> {
        "2026-07-03T12:00:00Z".parse().unwrap()
    }

    fn voyage() -> VoyageState {
        VoyageState::begin("c".to_string(), 1, t0())
    }

    #[test]
    fn cards_come_in_authored_order_with_final_prices() {
        let v = voyage();
        // W2, the Shoal Markets junction: R2 (14) then R3 (16).
        let cards = junction_cards(&v, WaypointId(2));
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].road.id, RoadId(2));
        assert_eq!(cards[0].provisions_price, 14);
        assert_eq!(cards[1].provisions_price, 16);
        assert!(cards[0].affordable && cards[0].selectable);
        assert_eq!(cards[0].title, "Toward The Drowned Choir");
    }

    #[test]
    fn trim_composes_into_price_and_days() {
        let mut v = voyage();
        v.set_trim(Trim::Run);
        let cards = junction_cards(&v, WaypointId(2));
        // R2: 14 * 1.30 = 18.2 -> 18; 1.5 days * 0.8 = 1.2 days.
        assert_eq!(cards[0].provisions_price, 18);
        assert_eq!(cards[0].days_label, "about a day");

        v.set_trim(Trim::Quiet);
        let cards = junction_cards(&v, WaypointId(2));
        // 14 * 0.9 = 12.6 -> 13; 1.5 * 1.2 = 1.8 days.
        assert_eq!(cards[0].provisions_price, 13);
        assert_eq!(cards[0].days_label, "about 2 days");
    }

    #[test]
    fn unaffordable_locks_all_but_the_cheapest() {
        let mut v = voyage();
        v.provisions = 10.0; // below both W2 roads (14 and 16)
        let cards = junction_cards(&v, WaypointId(2));
        assert!(!cards[0].affordable && !cards[1].affordable);
        assert!(cards[0].selectable, "cheapest is never locked");
        assert!(!cards[1].selectable);
    }

    #[test]
    fn rumors_annotate_their_road_and_reveal_stops() {
        let mut v = voyage();
        // Rumor 1 is about road 2 and reveals Graywater Anchorage (W4).
        v.rumors.push(LearnedRumor {
            rumor: RumorId(1),
            learned_at: ROUTE_START,
        });
        let cards = junction_cards(&v, WaypointId(2));
        let card = &cards[0];
        assert_eq!(card.annotations.len(), 1);
        assert!(card.annotations[0].contains("Graywater"));
        assert!(card
            .stops
            .iter()
            .any(|s| s.name == "Graywater Anchorage" && s.revealed_by_rumor));
        // Known stops stay unmarked.
        assert!(card
            .stops
            .iter()
            .any(|s| s.name == "The Drowned Choir" && !s.revealed_by_rumor));
        // The other card is untouched.
        assert!(cards[1].annotations.is_empty());
    }

    #[test]
    fn threats_are_always_on_the_card() {
        let v = voyage();
        // W7 (Candlewick Light): R9 carries the Ossuary Warden.
        let cards = junction_cards(&v, WaypointId(7));
        let dark = cards.iter().find(|c| c.road.id == RoadId(9)).unwrap();
        assert_eq!(dark.threat, Some("the Ossuary Warden"));
        assert_eq!(dark.character, vec!["dark"]);
    }

    #[test]
    fn current_junction_cards_follow_the_voyage() {
        let v = voyage();
        let cards = current_junction_cards(&v);
        assert_eq!(cards.len(), 1, "the Last Harbor has one road out");
        assert_eq!(cards[0].road.id, RoadId(0));
    }
}
