/// Test suite for the Balatro game engine.
/// Organized by mechanic:
///   - hand_types:       Basic hand type scoring
///   - card_effects:     Card enhancement and edition effects
///   - common_jokers:    Per-card and hand-type jokers
///   - rare_jokers:      Counter-based and scaling jokers
///   - joker_mechanics:  Gameplay-modifying jokers (FourFingers, Splash, etc.)
///   - tarot_cards:      Tarot card application via GameState
///   - vouchers:         Voucher effects via GameState
///   - decks:            Deck-specific mechanics
///   - stickers:         Sticker mechanics (eternal, rental, perishable)
///   - gamestate:        GameState round-play integration

mod boss_blinds;
mod card_effects;
mod common_jokers;
mod complex_scenarios;
mod decks;
mod gamestate;
mod hand_types;
mod joker_mechanics;
mod long_runs;
mod misc_jokers;
mod packs;
mod planet_cards;
mod rare_jokers;
mod spectral_cards;
mod selling;
mod stakes;
mod stickers;
mod tags;
mod tarot_cards;
mod vouchers;

// =========================================================
// Shared Helpers (pub so submodules can import via use super::*)
// =========================================================

use crate::card::{CardInstance, HandLevelData, JokerInstance};
use crate::game::{GameState, GameStateKind};
use crate::scoring::{score_hand, RoundTargets, ScoreInputs};
use crate::types::*;
use std::collections::HashMap;

pub fn card(id: u64, rank: Rank, suit: Suit) -> CardInstance {
    CardInstance::new(id, rank, suit)
}

pub fn joker(id: u64, kind: JokerKind) -> JokerInstance {
    JokerInstance::new(id, kind, Edition::None)
}

pub fn default_hand_levels() -> HashMap<HandType, HandLevelData> {
    let hand_types = [
        HandType::HighCard,
        HandType::Pair,
        HandType::TwoPair,
        HandType::ThreeOfAKind,
        HandType::Straight,
        HandType::Flush,
        HandType::FullHouse,
        HandType::FourOfAKind,
        HandType::StraightFlush,
        HandType::FiveOfAKind,
        HandType::FlushHouse,
        HandType::FlushFive,
    ];
    let mut m = HashMap::new();
    for ht in hand_types {
        m.insert(ht, HandLevelData::new(true));
    }
    m
}

/// Counts of the enhancements in play, as `score_hand` expects them.
fn deck_enhancement_counts(
    played: &[CardInstance],
    hand: &[CardInstance],
) -> (usize, usize, usize) {
    let all = || played.iter().chain(hand.iter());
    (
        all().filter(|c| c.enhancement == Enhancement::Steel).count(),
        all().filter(|c| c.is_stone()).count(),
        all().filter(|c| c.enhancement != Enhancement::None).count(),
    )
}

/// Build inputs with the counts derived from the cards, the way a real round would.
pub fn inputs<'a>(
    played: &'a [CardInstance],
    hand: &'a [CardInstance],
    jokers: &'a [JokerInstance],
    levels: &'a HashMap<HandType, HandLevelData>,
) -> ScoreInputs<'a> {
    let (steel, stone, enhanced) = deck_enhancement_counts(played, hand);
    let mut i = ScoreInputs::new(played, hand, jokers, levels);
    i.hands_remaining = 3;
    i.discards_remaining = 3;
    i.deck_cards_remaining = 40;
    i.steel_count_in_deck = steel;
    i.stone_count_in_deck = stone;
    i.enhanced_count_in_deck = enhanced;
    i
}

/// Score with sensible defaults for unused parameters.
pub fn score(
    played: &[CardInstance],
    hand: &[CardInstance],
    jokers: &[JokerInstance],
) -> crate::scoring::ScoreResult {
    let levels = default_hand_levels();
    score_hand(inputs(played, hand, jokers, &levels))
}

/// Score against explicit hand levels.
pub fn score_levels(
    played: &[CardInstance],
    hand: &[CardInstance],
    jokers: &[JokerInstance],
    levels: &HashMap<HandType, HandLevelData>,
) -> crate::scoring::ScoreResult {
    score_hand(inputs(played, hand, jokers, levels))
}

/// Score with the round-wide joker targets (The Idol, Ancient Joker) set explicitly.
pub fn score_with_targets(
    played: &[CardInstance],
    hand: &[CardInstance],
    jokers: &[JokerInstance],
    targets: RoundTargets,
) -> crate::scoring::ScoreResult {
    let levels = default_hand_levels();
    let mut i = inputs(played, hand, jokers, &levels);
    i.round_targets = targets;
    score_hand(i)
}

/// Score with full parameter control.
#[allow(clippy::too_many_arguments)]
pub fn score_full(
    played: &[CardInstance],
    hand: &[CardInstance],
    jokers: &[JokerInstance],
    hands_remaining: u32,
    discards_remaining: u32,
    money: i32,
    deck_remaining: usize,
    total_deck: usize,
    joker_slot_count: usize,
    tarot_cards_used: u32,
) -> crate::scoring::ScoreResult {
    let levels = default_hand_levels();
    let mut i = inputs(played, hand, jokers, &levels);
    i.hands_remaining = hands_remaining;
    i.discards_remaining = discards_remaining;
    i.money = money;
    i.deck_cards_remaining = deck_remaining;
    i.total_deck_size = total_deck;
    i.joker_slot_count = joker_slot_count;
    i.tarot_cards_used = tarot_cards_used;
    score_hand(i)
}

pub fn make_game() -> GameState {
    GameState::new(DeckType::Blue, Stake::White, Some("TESTROUND".to_string()))
}

/// Set up a game in Round state with a controlled hand.
/// Clears the deck and replaces it with `deck_cards`, sets hand to first `hand_size` indices.
pub fn setup_round(gs: &mut GameState, deck_cards: Vec<CardInstance>, hand_size: usize) {
    gs.state = GameStateKind::Round;
    gs.score_accumulated = 0.0;
    // Start at a full complement so "first hand of the round" checks (DNA, Sixth Sense) behave.
    gs.hands_remaining = gs.effective_max_hands();
    gs.discards_remaining = gs.effective_max_discards();
    gs.selected_indices.clear();
    gs.hand.clear();
    gs.draw_pile.clear();
    gs.discard_pile.clear();
    gs.deck = deck_cards;
    for i in 0..hand_size.min(gs.deck.len()) {
        gs.hand.push(i);
    }
    for i in hand_size..gs.deck.len() {
        gs.draw_pile.push(i);
    }
}
