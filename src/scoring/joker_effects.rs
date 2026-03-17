use crate::card::{CardInstance, JokerInstance, HandLevelData};
use crate::types::*;
use std::collections::HashMap;
use super::ScoringContext;

pub struct JokerEffect {
    pub chips: i64,
    pub mult: i64,
    pub x_mult: f64,
    pub dollars: i32,
}

impl JokerEffect {
    fn new() -> Self {
        Self {
            chips: 0,
            mult: 0,
            x_mult: 1.0,
            dollars: 0,
        }
    }
}

pub(crate) fn count_retriggers(
    card_idx: usize,
    card: &CardInstance,
    jokers: &[JokerInstance],
    scoring_indices: &[usize],
    played_cards: &[CardInstance],
    hand_type: HandType,
    hands_remaining: u32,
) -> usize {
    let mut retriggers = 0usize;

    // Red seal: retrigger once
    if card.seal == Seal::Red {
        retriggers += 1;
    }

    // Dusk: retrigger all scoring cards on last hand (hands_remaining == 0)
    for joker in jokers.iter().filter(|j| j.active) {
        match joker.kind {
            JokerKind::Dusk if hands_remaining == 0 => {
                retriggers += 1;
            }
            JokerKind::Hack => {
                if matches!(
                    card.rank,
                    Rank::Two | Rank::Three | Rank::Four | Rank::Five
                ) {
                    retriggers += 1;
                }
            }
            JokerKind::SockAndBuskin => {
                if card.rank.is_face() {
                    retriggers += 1;
                }
            }
            JokerKind::HangingChad => {
                // First played card retriggers twice
                if scoring_indices.first() == Some(&card_idx) {
                    retriggers += 2;
                }
            }
            JokerKind::Seltzer => {
                // Retriggers all played cards for 10 hands, then destroyed
                let remaining = joker.get_counter_i64("hands");
                if remaining > 0 {
                    retriggers += 1;
                }
            }
            _ => {}
        }
    }

    retriggers
}

/// Effects triggered before individual card scoring (e.g. hand-level effects)
pub(crate) fn calc_joker_before(
    joker: &JokerInstance,
    hand_type: HandType,
    scoring_indices: &[usize],
    played_cards: &[CardInstance],
    hand_levels: &HashMap<HandType, HandLevelData>,
) -> JokerEffect {
    let mut effect = JokerEffect::new();
    match joker.kind {
        // These jokers trigger at the hand level (not per-card).
        // Wiki uses "contains", so they fire on any hand that includes the pattern.
        JokerKind::JollyJoker if hand_type.contains_pair() => {
            effect.mult += 8;
        }
        JokerKind::ZanyJoker if hand_type.contains_three_of_a_kind() => {
            effect.mult += 12;
        }
        JokerKind::MadJoker if hand_type.contains_two_pair() => {
            effect.mult += 10;
        }
        JokerKind::CrazyJoker if hand_type.contains_straight() => {
            effect.mult += 12;
        }
        JokerKind::DrollJoker if hand_type.contains_flush() => {
            effect.mult += 10;
        }
        JokerKind::SlyJoker if hand_type.contains_pair() => {
            effect.chips += 50;
        }
        JokerKind::WilyJoker if hand_type.contains_three_of_a_kind() => {
            effect.chips += 100;
        }
        JokerKind::CleverJoker if hand_type.contains_two_pair() => {
            effect.chips += 80;
        }
        JokerKind::DeviousJoker if hand_type.contains_straight() => {
            effect.chips += 100;
        }
        JokerKind::CraftyJoker if hand_type.contains_flush() => {
            effect.chips += 80;
        }
        _ => {}
    }
    effect
}

/// Per-card joker effects (individual = true context)
pub(crate) fn calc_joker_individual(
    joker: &JokerInstance,
    card_idx: usize,
    card: &CardInstance,
    hand_type: HandType,
    scoring_indices: &[usize],
    played_cards: &[CardInstance],
    hand_cards: &[CardInstance],
    pareidolia: bool,
) -> JokerEffect {
    let mut effect = JokerEffect::new();
    let is_scoring = scoring_indices.contains(&card_idx);

    match joker.kind {
        JokerKind::GreedyJoker => {
            if is_scoring && card.effective_suits().contains(&Suit::Diamonds) {
                effect.mult += 3;
            }
        }
        JokerKind::LustyJoker => {
            if is_scoring && card.effective_suits().contains(&Suit::Hearts) {
                effect.mult += 3;
            }
        }
        JokerKind::WrathfulJoker => {
            if is_scoring && card.effective_suits().contains(&Suit::Spades) {
                effect.mult += 3;
            }
        }
        JokerKind::GluttonousJoker => {
            if is_scoring && card.effective_suits().contains(&Suit::Clubs) {
                effect.mult += 3;
            }
        }
        JokerKind::ScaryFace => {
            if is_scoring && card.is_face(pareidolia) {
                effect.chips += 30;
            }
        }
        JokerKind::Fibonacci => {
            if is_scoring && card.rank.is_fibonacci() {
                effect.mult += 8;
            }
        }
        JokerKind::EvenSteven => {
            if is_scoring && card.rank.is_even() {
                effect.mult += 4;
            }
        }
        JokerKind::OddTodd => {
            if is_scoring && card.rank.is_odd() {
                effect.chips += 31;
            }
        }
        JokerKind::Scholar => {
            if is_scoring && card.rank == Rank::Ace {
                effect.chips += 20;
                effect.mult += 4;
            }
        }
        JokerKind::BusinessCard => {
            // Handled in game loop (round.rs) with proper 1/2 probability roll
        }
        JokerKind::Photograph => {
            // First face card scored this hand gives x2 mult
            if is_scoring && card.is_face(pareidolia) {
                let first_face_idx = scoring_indices
                    .iter()
                    .find(|&&i| played_cards[i].is_face(pareidolia));
                if first_face_idx == Some(&card_idx) {
                    effect.x_mult = 2.0;
                }
            }
        }
        JokerKind::WalkieTalkie => {
            if is_scoring && (card.rank == Rank::Ten || card.rank == Rank::Four) {
                effect.chips += 10;
                effect.mult += 4;
            }
        }
        JokerKind::SmileyFace => {
            if is_scoring && card.is_face(pareidolia) {
                effect.mult += 5;
            }
        }
        JokerKind::StoneJoker => {
            // Handled in calc_joker_main (counts all Stone cards in full deck)
        }
        JokerKind::Hiker => {
            if is_scoring {
                effect.chips += 5; // adds +5 chips permanently (tracked separately)
            }
        }
        JokerKind::Arrowhead => {
            if is_scoring && card.effective_suits().contains(&Suit::Spades) {
                effect.chips += 50;
            }
        }
        JokerKind::OnyxAgate => {
            if is_scoring && card.effective_suits().contains(&Suit::Clubs) {
                effect.mult += 7;
            }
        }
        JokerKind::Bloodstone => {
            // Pre-rolled in round.rs: extra_x_mult is set to 1.5 if triggered (1/2 chance)
            if is_scoring && card.effective_suits().contains(&Suit::Hearts) && card.extra_x_mult > 1.0 {
                effect.x_mult = card.extra_x_mult;
            }
        }
        JokerKind::RoughGem => {
            if is_scoring && card.effective_suits().contains(&Suit::Diamonds) {
                effect.dollars += 1;
            }
        }
        JokerKind::ShootTheMoon | JokerKind::Baron => {
            // Handled in calc_joker_hand_card (fires for held cards, not played cards)
        }
        JokerKind::Triboulet => {
            if is_scoring && (card.rank == Rank::King || card.rank == Rank::Queen) {
                effect.x_mult = 2.0;
            }
        }
        // Canio is handled in calc_joker_main (not per-card)

        JokerKind::TheIdol => {
            if is_scoring {
                // The Idol: x2 for the specific rank/suit combo
                // (tracked per counter)
                let rank_match = joker.counters.get("rank").and_then(|v| v.as_str())
                    == Some(&format!("{:?}", card.rank));
                let suit_match = joker.counters.get("suit").and_then(|v| v.as_str())
                    == Some(&format!("{:?}", card.suit));
                if rank_match && suit_match {
                    effect.x_mult = 2.0;
                }
            }
        }
        _ => {}
    }
    effect
}

/// Per-hand-card effects (for cards held in hand, not played)
pub(crate) fn calc_joker_hand_card(
    joker: &JokerInstance,
    card: &CardInstance,
    hand_type: HandType,
    scoring_indices: &[usize],
    played_cards: &[CardInstance],
    hand_cards: &[CardInstance],
) -> JokerEffect {
    let mut effect = JokerEffect::new();
    match joker.kind {
        JokerKind::Blackboard => {
            // All cards in hand are Spades or Clubs → x3 mult
            // This is evaluated as a whole in calc_joker_main
        }
        JokerKind::Baron => {
            // x1.5 mult for each King held in hand (not played)
            if card.rank == Rank::King && !card.debuffed {
                effect.x_mult = 1.5;
            }
        }
        JokerKind::ShootTheMoon => {
            // +13 mult for each Queen held in hand (not played)
            if card.rank == Rank::Queen && !card.debuffed {
                effect.mult += 13;
            }
        }
        JokerKind::ReservedParking => {
            // Handled in game loop (round.rs) with proper 1/2 probability roll
        }
        _ => {}
    }
    effect
}

/// Main joker effects (evaluated once per hand)
pub(crate) fn calc_joker_main(joker: &JokerInstance, ctx: &ScoringContext) -> JokerEffect {
    let mut effect = JokerEffect::new();
    let hand_type = ctx.hand_type;
    let scoring_cards = ctx.scoring_cards;
    let played = ctx.played_cards;
    let hand = ctx.hand_cards;

    match joker.kind {
        JokerKind::Joker => {
            effect.mult += 4;
        }
        JokerKind::JokerStencil => {
            // X1 mult for each empty joker slot (multiplicative)
            let empty_slots = ctx.joker_slot_count.saturating_sub(ctx.joker_count);
            effect.x_mult = 1.0 + empty_slots as f64;
        }
        JokerKind::AbstractJoker => {
            effect.mult += (ctx.joker_count as i64) * 3;
        }
        JokerKind::HalfJoker => {
            if played.len() <= 3 {
                effect.mult += 20;
            }
        }
        JokerKind::Banner => {
            effect.chips += (ctx.discards_remaining as i64) * 30;
        }
        JokerKind::MysticSummit => {
            if ctx.discards_remaining == 0 {
                effect.mult += 15;
            }
        }
        JokerKind::Supernova => {
            let plays = ctx
                .hand_levels
                .get(&hand_type)
                .map(|h| h.played)
                .unwrap_or(0);
            effect.mult += plays as i64;
        }
        JokerKind::RideTheBus => {
            let mult = joker.get_counter_i64("mult");
            effect.mult += mult;
        }
        JokerKind::BlueJoker => {
            // +2 chips per remaining card in deck
            effect.chips += (ctx.deck_cards_remaining as i64) * 2;
        }
        JokerKind::Erosion => {
            // +4 mult for each card permanently removed from starting deck (52)
            let below = (52i64 - ctx.total_deck_size as i64).max(0);
            effect.mult += below * 4;
        }
        JokerKind::Misprint => {
            // Random mult 0-23
            effect.mult += 11; // simplified: use average
        }
        JokerKind::GreenJoker => {
            let mult = joker.get_counter_i64("mult");
            effect.mult += mult;
        }
        JokerKind::SpareTrousers => {
            let mult = joker.get_counter_i64("mult");
            effect.mult += mult;
        }
        JokerKind::FlashCard => {
            let mult = joker.get_counter_i64("mult");
            effect.mult += mult;
        }
        JokerKind::CeremonialDagger => {
            let mult = joker.get_counter_i64("mult");
            effect.mult += mult;
        }
        JokerKind::Popcorn => {
            let mult = joker.get_counter_i64("mult");
            effect.mult += mult;
        }
        JokerKind::Swashbuckler => {
            let mult = joker.get_counter_i64("mult");
            effect.mult += mult;
        }
        JokerKind::Runner => {
            let chips = joker.get_counter_i64("chips");
            effect.chips += chips;
        }
        JokerKind::IceCream => {
            let chips = joker.get_counter_i64("chips");
            effect.chips += chips;
        }
        JokerKind::SquareJoker => {
            let chips = joker.get_counter_i64("chips");
            effect.chips += chips;
        }
        JokerKind::WeeJoker => {
            let chips = joker.get_counter_i64("chips");
            effect.chips += chips;
        }
        JokerKind::Castle => {
            let chips = joker.get_counter_i64("chips");
            effect.chips += chips;
        }
        JokerKind::Hologram => {
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::Vampire => {
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::Obelisk => {
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::LuckyCat => {
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::Constellation => {
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::GlassJoker => {
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::Ramen => {
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::HitTheRoad => {
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::Madness => {
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::Campfire => {
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::Yorick => {
            // Gains +1 Xmult every 23 cards discarded — counter updated in discard_hand
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::Canio => {
            // Gains +1 Xmult each time a face card is destroyed — counter updated in game
            let xmult = joker.get_counter_f64("x_mult");
            if xmult > 1.0 {
                effect.x_mult = xmult;
            }
        }
        JokerKind::StoneJoker => {
            // +25 Chips per Stone card in the full deck
            effect.chips += ctx.stone_count_in_deck as i64 * 25;
        }
        JokerKind::SteelJoker => {
            // +X0.2 Mult per Steel card in the full deck
            let steel_count = ctx.steel_count_in_deck;
            if steel_count > 0 {
                effect.x_mult = 1.0 + 0.2 * steel_count as f64;
            }
        }
        JokerKind::Blackboard => {
            // All cards in hand are Spades or Clubs
            let all_dark = hand.iter().all(|c| {
                c.effective_suits().iter().any(|s| matches!(s, Suit::Spades | Suit::Clubs))
            });
            if all_dark {
                effect.x_mult = 3.0;
            }
        }
        JokerKind::Throwback => {
            // +0.25 Xmult per blind skipped
            let skips = joker.get_counter_i64("skips");
            if skips > 0 {
                effect.x_mult = 1.0 + 0.25 * skips as f64;
            }
        }
        JokerKind::Bootstraps => {
            // +2 mult per $5 above $0
            let multiples = (ctx.money / 5).max(0);
            effect.mult += multiples as i64 * 2;
        }
        JokerKind::Bull => {
            // +2 chips per $1 held
            effect.chips += (ctx.money.max(0) as i64) * 2;
        }
        JokerKind::ToTheMoon => {
            // +1 interest per $5 (essentially tracked at end of round)
            // For scoring: no chip/mult effect
        }
        JokerKind::FortuneTeller => {
            // +1 mult per tarot card used this run
            effect.mult += ctx.tarot_cards_used as i64;
        }
        JokerKind::TheDuo if hand_type.contains_pair() => {
            effect.x_mult = 2.0;
        }
        JokerKind::TheTrio if hand_type.contains_three_of_a_kind() => {
            effect.x_mult = 3.0;
        }
        JokerKind::TheFamily if hand_type.contains_four_of_a_kind() => {
            effect.x_mult = 4.0;
        }
        JokerKind::TheOrder if hand_type.contains_straight() => {
            effect.x_mult = 3.0;
        }
        JokerKind::TheTribe if hand_type.contains_flush() => {
            effect.x_mult = 2.0;
        }
        JokerKind::CardSharp => {
            // x3 if this hand type was already played this round
            let played_this_round = ctx
                .hand_levels
                .get(&hand_type)
                .map(|h| h.played_this_round)
                .unwrap_or(0);
            if played_this_round > 0 {
                effect.x_mult = 3.0;
            }
        }
        JokerKind::SeeingDouble => {
            // x2 if a SCORING Club card and a SCORING card of any other suit exist
            let has_club = scoring_cards.iter().any(|&i| played[i].effective_suits().contains(&Suit::Clubs));
            let has_non_club = scoring_cards
                .iter()
                .any(|&i| played[i].effective_suits().iter().any(|s| *s != Suit::Clubs));
            if has_club && has_non_club {
                effect.x_mult = 2.0;
            }
        }
        JokerKind::AncientJoker => {
            // x1.5 for each scoring card of the designated suit
            let suit_str = joker
                .counters
                .get("suit")
                .and_then(|v| v.as_str())
                .unwrap_or("Hearts");
            let target_suit = match suit_str {
                "Spades" => Suit::Spades,
                "Hearts" => Suit::Hearts,
                "Clubs" => Suit::Clubs,
                "Diamonds" => Suit::Diamonds,
                _ => Suit::Hearts,
            };
            for &idx in scoring_cards {
                if played[idx].effective_suits().contains(&target_suit) {
                    effect.x_mult *= 1.5;
                }
            }
        }
        JokerKind::FlowerPot => {
            // x3 if scoring hand has all 4 suits
            let suits_present: std::collections::HashSet<&Suit> = scoring_cards
                .iter()
                .flat_map(|&i| {
                    let card = &played[i];
                    card.effective_suits().clone().into_iter().collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
                .iter()
                .collect();
            // simplified: check all 4 suits present
            let has_all_4 = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds]
                .iter()
                .all(|s| scoring_cards.iter().any(|&i| played[i].effective_suits().contains(s)));
            if has_all_4 {
                effect.x_mult = 3.0;
            }
        }
        JokerKind::DriversLicense => {
            // x3 if 16+ enhanced cards in full deck
            if ctx.enhanced_count_in_deck >= 16 {
                effect.x_mult = 3.0;
            }
        }
        JokerKind::RedCard => {
            // +3 mult per blind skipped (counter incremented in skip_blind)
            let mult = joker.get_counter_i64("mult");
            effect.mult += mult;
        }
        JokerKind::RaisedFist => {
            // Adds double the rank value of the lowest card held in hand to Mult
            if let Some(min_rank) = hand.iter()
                .map(|c| c.base_chip_value())
                .filter(|&v| v > 0)
                .min()
            {
                effect.mult += min_rank * 2;
            }
        }
        JokerKind::BaseballCard => {
            // x1.5 for each Uncommon joker (rarity == 2)
            let uncommon_count = ctx.jokers.iter()
                .filter(|j| j.active && j.kind.rarity() == 2)
                .count();
            if uncommon_count > 0 {
                effect.x_mult = 1.5_f64.powi(uncommon_count as i32);
            }
        }
        JokerKind::ToDoList => {
            // $4 if the played hand matches the tracked hand type
            let target_str = joker.counters.get("hand_type")
                .and_then(|v| v.as_str())
                .unwrap_or("HighCard");
            let target = match target_str {
                "Pair" => HandType::Pair,
                "TwoPair" => HandType::TwoPair,
                "ThreeOfAKind" => HandType::ThreeOfAKind,
                "Straight" => HandType::Straight,
                "Flush" => HandType::Flush,
                "FullHouse" => HandType::FullHouse,
                "FourOfAKind" => HandType::FourOfAKind,
                "StraightFlush" => HandType::StraightFlush,
                "FiveOfAKind" => HandType::FiveOfAKind,
                "FlushHouse" => HandType::FlushHouse,
                "FlushFive" => HandType::FlushFive,
                _ => HandType::HighCard,
            };
            if hand_type == target {
                effect.dollars += 4;
            }
        }
        JokerKind::Acrobat => {
            // x3 mult on last hand of round
            if ctx.hands_remaining == 0 {
                effect.x_mult = 3.0;
            }
        }
        JokerKind::Stuntman => {
            // +250 chips, -2 hand size (hand size tracked separately)
            effect.chips += 250;
        }
        JokerKind::SquareJoker => {
            // Already tracked in counter
        }
        JokerKind::Cloud9 => {
            // $1 per 9 in full deck at end of round (tracked separately)
        }
        JokerKind::MidasMask => {
            // Face cards become Gold cards when scored (effect on cards, not scoring)
        }
        JokerKind::Luchador => {
            // Disables current boss blind effect
        }
        JokerKind::Vagabond => {
            // Create Tarot if $4 or less (tracked at hand play time)
        }
        JokerKind::GrosMichel => {
            effect.mult += 15;
        }
        JokerKind::Cavendish => {
            // Unconditional X3 Mult (no hand-type condition per wiki)
            effect.x_mult = 3.0;
        }
        JokerKind::GoldenTicket => {
            // +$4 per Gold enhancement card in scoring hand
            let gold_count = scoring_cards.iter()
                .filter(|&&i| played[i].enhancement == Enhancement::Gold)
                .count();
            effect.dollars += gold_count as i32 * 4;
        }
        JokerKind::LoyaltyCard => {
            // x4 mult every 6 hands played (triggered on 6th, 12th, 18th hand)
            let total_played: u32 = ctx.hand_levels.values().map(|h| h.played).sum();
            if total_played > 0 && (total_played % 6) == 5 {
                effect.x_mult = 4.0;
            }
        }
        JokerKind::Blueprint => {
            // Copy the joker immediately to the right
            let pos = ctx.jokers.iter().position(|j| j as *const JokerInstance == joker as *const JokerInstance);
            if let Some(p) = pos {
                if p + 1 < ctx.jokers.len() {
                    let next = &ctx.jokers[p + 1];
                    if next.active && !matches!(next.kind, JokerKind::Blueprint | JokerKind::Brainstorm) {
                        let copied = calc_joker_main(next, ctx);
                        effect.chips += copied.chips;
                        effect.mult += copied.mult;
                        effect.x_mult *= copied.x_mult;
                        effect.dollars += copied.dollars;
                    }
                }
            }
        }
        JokerKind::Brainstorm => {
            // Copy the leftmost joker (that is not Blueprint or Brainstorm)
            if let Some(first) = ctx.jokers.iter().find(|&j| {
                j.active
                && !matches!(j.kind, JokerKind::Blueprint | JokerKind::Brainstorm)
                && (j as *const JokerInstance != joker as *const JokerInstance)
            }) {
                let copied = calc_joker_main(first, ctx);
                effect.chips += copied.chips;
                effect.mult += copied.mult;
                effect.x_mult *= copied.x_mult;
                effect.dollars += copied.dollars;
            }
        }
        _ => {}
    }

    effect
}

