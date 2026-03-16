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
        // These jokers trigger at the hand level (not per-card)
        JokerKind::JollyJoker if hand_type == HandType::Pair => {
            effect.mult += 8;
        }
        JokerKind::ZanyJoker if hand_type == HandType::ThreeOfAKind => {
            effect.mult += 12;
        }
        JokerKind::MadJoker if hand_type == HandType::TwoPair => {
            effect.mult += 10;
        }
        JokerKind::CrazyJoker if hand_type == HandType::Straight => {
            effect.mult += 12;
        }
        JokerKind::DrollJoker if hand_type == HandType::Flush => {
            effect.mult += 10;
        }
        JokerKind::SlyJoker if hand_type == HandType::Pair => {
            effect.chips += 50;
        }
        JokerKind::WilyJoker if hand_type == HandType::ThreeOfAKind => {
            effect.chips += 100;
        }
        JokerKind::CleverJoker if hand_type == HandType::TwoPair => {
            effect.chips += 80;
        }
        JokerKind::DeviousJoker if hand_type == HandType::Straight => {
            effect.chips += 100;
        }
        JokerKind::CraftyJoker if hand_type == HandType::Flush => {
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
            if is_scoring && card.is_face(pareidolia) {
                // 1/2 chance to earn $2 (simulate as earning $1 on average)
                effect.dollars += 1; // simplified
            }
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
            if is_scoring && card.is_stone() {
                effect.chips += 25;
            }
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
            if is_scoring && card.effective_suits().contains(&Suit::Hearts) {
                // 1/2 chance x1.5 (simplified to always)
                effect.x_mult = 1.5;
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
            // +1 mult for each empty joker slot
            let empty_slots = ctx.joker_slot_count.saturating_sub(ctx.joker_count);
            effect.mult += empty_slots as i64;
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
            effect.chips += (ctx.hands_remaining as i64) * 30;
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
            // +4 mult for each card below starting deck size (52)
            let below = (ctx.total_deck_size as i64 - ctx.deck_cards_remaining as i64).max(0);
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
        JokerKind::SteelJoker => {
            // Count steel cards in full deck (not just hand)
            // simplified: count steel in hand
            let steel_count = hand.iter().filter(|c| c.enhancement == Enhancement::Steel).count();
            let x_per = 0.2_f64;
            if steel_count > 0 {
                effect.x_mult = 1.0 + x_per * steel_count as f64;
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
        JokerKind::TheDuo if hand_type == HandType::Pair => {
            effect.x_mult = 2.0;
        }
        JokerKind::TheTrio if hand_type == HandType::ThreeOfAKind => {
            effect.x_mult = 3.0;
        }
        JokerKind::TheFamily if hand_type == HandType::FourOfAKind => {
            effect.x_mult = 4.0;
        }
        JokerKind::TheOrder if hand_type == HandType::Straight => {
            effect.x_mult = 3.0;
        }
        JokerKind::TheTribe if hand_type == HandType::Flush => {
            effect.x_mult = 2.0;
        }
        JokerKind::CardSharp => {
            // x3 if this hand type was not played this round
            let played_this_round = ctx
                .hand_levels
                .get(&hand_type)
                .map(|h| h.played_this_round)
                .unwrap_or(0);
            if played_this_round == 0 {
                effect.x_mult = 3.0;
            }
        }
        JokerKind::SeeingDouble => {
            // x2 if hand contains a Club and another suit
            let has_club = played.iter().any(|c| c.effective_suits().contains(&Suit::Clubs));
            let has_non_club = played
                .iter()
                .any(|c| c.effective_suits().iter().any(|s| *s != Suit::Clubs));
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
            // x3 if 16+ enhanced cards in full deck (approximated by hand + scoring)
            let enhanced_in_hand = hand.iter().filter(|c| c.enhancement != Enhancement::None).count();
            let enhanced_in_play = played.iter().filter(|c| c.enhancement != Enhancement::None).count();
            if enhanced_in_hand + enhanced_in_play >= 8 {
                // simplified threshold
                effect.x_mult = 3.0;
            }
        }
        JokerKind::BaseballCard => {
            // x1.5 for each Uncommon joker
            // simplified: count all jokers / 2
            effect.x_mult = 1.0; // would need joker rarity info
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
            if hand_type == HandType::Pair {
                effect.x_mult = 3.0;
            }
        }
        JokerKind::GoldenTicket => {
            // +$1 per Gold enhancement card in scoring hand
            let gold_count = scoring_cards.iter()
                .filter(|&&i| played[i].enhancement == Enhancement::Gold)
                .count();
            effect.dollars += gold_count as i32;
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

