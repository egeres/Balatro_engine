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
mod planet_cards;
mod rare_jokers;
mod spectral_cards;
mod stickers;
mod tarot_cards;
mod vouchers;

// =========================================================
// Shared Helpers (pub so submodules can import via use super::*)
// =========================================================

use crate::card::{CardInstance, HandLevelData, JokerInstance};
use crate::game::{GameState, GameStateKind};
use crate::scoring::score_hand;
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

/// Score with sensible defaults for unused parameters.
pub fn score(
    played: &[CardInstance],
    hand: &[CardInstance],
    jokers: &[JokerInstance],
) -> crate::scoring::ScoreResult {
    score_hand(
        played,
        hand,
        jokers,
        &default_hand_levels(),
        3,    // hands_remaining
        3,    // discards_remaining
        0,    // money
        40,   // deck_remaining
        52,   // total_deck
        None, // boss_blind
        5,    // joker_slot_count
        0,    // tarot_cards_used
    )
}

/// Score with full parameter control.
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
    score_hand(
        played,
        hand,
        jokers,
        &default_hand_levels(),
        hands_remaining,
        discards_remaining,
        money,
        deck_remaining,
        total_deck,
        None,
        joker_slot_count,
        tarot_cards_used,
    )
}

pub fn make_game() -> GameState {
    GameState::new(DeckType::Blue, Stake::White, Some("TESTROUND".to_string()))
}

/// Set up a game in Round state with a controlled hand.
/// Clears the deck and replaces it with `deck_cards`, sets hand to first `hand_size` indices.
pub fn setup_round(gs: &mut GameState, deck_cards: Vec<CardInstance>, hand_size: usize) {
    gs.state = GameStateKind::Round;
    gs.score_accumulated = 0.0;
    gs.hands_remaining = 4;
    gs.discards_remaining = 3;
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
