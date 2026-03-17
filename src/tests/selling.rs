/// Tests for selling jokers and consumables mid-game (during Round state).
///
/// sell_joker and sell_consumable have no state guard, so they are valid
/// at any point — including while a round is in progress.  These tests
/// verify that the mechanics work correctly outside the shop and that
/// downstream effects (scoring, Campfire, etc.) are reflected immediately.

use super::*;
use crate::card::ConsumableCard;
use crate::game::BalatroError;

// =========================================================
// sell_joker during Round state
// =========================================================

#[test]
fn test_sell_joker_mid_round_removes_it_from_list() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.jokers.push(joker(10, JokerKind::Joker));

    gs.sell_joker(0).unwrap();

    assert!(gs.jokers.is_empty(), "joker should be removed after selling");
}

#[test]
fn test_sell_joker_mid_round_pays_sell_value() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.jokers.push(joker(10, JokerKind::Joker)); // base_cost=2 → sell_value=1
    let before = gs.money;

    gs.sell_joker(0).unwrap();

    assert_eq!(gs.money, before + 1, "selling Joker (base_cost=2) should yield $1");
}

#[test]
fn test_sell_joker_mid_round_joker_no_longer_scores() {
    // [Joker, Joker]: mult = 1+4+4 = 9 → 144
    // Sell second Joker before playing → [Joker]: mult = 1+4 = 5 → 80
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.jokers.push(joker(10, JokerKind::Joker));
    gs.jokers.push(joker(11, JokerKind::Joker));

    gs.sell_joker(1).unwrap();

    gs.score_goal = f64::MAX;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    // chips=16, mult=1+4=5 → 80
    assert_eq!(gs.score_accumulated as i64, 80, "sold joker must not contribute to scoring");
}

#[test]
fn test_sell_joker_mid_round_reduces_abstract_joker_count() {
    // [AbstractJoker, Joker]: AJ gives +3×2=+6, Joker +4 → mult=11 → 176
    // Sell Joker before playing → [AbstractJoker]: +3×1=+3 → mult=4 → 64
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.jokers.push(joker(10, JokerKind::AbstractJoker));
    gs.jokers.push(joker(11, JokerKind::Joker));

    gs.sell_joker(1).unwrap(); // remove Joker; only AbstractJoker remains

    gs.score_goal = f64::MAX;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    // chips=16, mult=1+3=4 → 64
    assert_eq!(gs.score_accumulated as i64, 64,
        "AbstractJoker should count only 1 joker after the other is sold");
}

#[test]
fn test_sell_joker_mid_round_eternal_is_blocked() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    let mut eternal = joker(10, JokerKind::Joker);
    eternal.eternal = true;
    gs.jokers.push(eternal);

    let err = gs.sell_joker(0).unwrap_err();
    assert!(
        matches!(err, BalatroError::EternalCard),
        "selling an eternal joker mid-round must return EternalCard"
    );
    assert_eq!(gs.jokers.len(), 1, "eternal joker must remain after failed sell");
}

#[test]
fn test_sell_joker_mid_round_out_of_range_returns_error() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);

    let err = gs.sell_joker(0).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(0, 0)));
}

#[test]
fn test_campfire_gains_x_mult_when_joker_sold_mid_round() {
    // Campfire starts at x1.0; each sold joker adds +0.25.
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.jokers.push(joker(10, JokerKind::Campfire));
    gs.jokers.push(joker(11, JokerKind::Joker));

    gs.sell_joker(1).unwrap(); // sell Joker; Campfire should fire

    let x_mult = gs.jokers[0].get_counter_f64("x_mult");
    assert!(
        (x_mult - 1.25).abs() < 1e-9,
        "Campfire x_mult should be 1.25 after one joker sold, got {x_mult}"
    );
}

// =========================================================
// sell_consumable during Round state
// =========================================================

#[test]
fn test_sell_consumable_mid_round_removes_it() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheFool));

    gs.sell_consumable(0).unwrap();

    assert!(gs.consumables.is_empty(), "consumable should be removed after selling");
}

#[test]
fn test_sell_tarot_mid_round_pays_one_dollar() {
    // Tarot base_cost=3 → sell = (3/2).max(1) = 1
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheMagician));
    let before = gs.money;

    gs.sell_consumable(0).unwrap();

    assert_eq!(gs.money, before + 1, "selling a Tarot card should yield $1");
}

#[test]
fn test_sell_spectral_mid_round_pays_two_dollars() {
    // Spectral base_cost=4 → sell = (4/2).max(1) = 2
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.consumables.push(ConsumableCard::Spectral(SpectralCard::Familiar));
    let before = gs.money;

    gs.sell_consumable(0).unwrap();

    assert_eq!(gs.money, before + 2, "selling a Spectral card should yield $2");
}

#[test]
fn test_sell_consumable_mid_round_out_of_range_returns_error() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);

    let err = gs.sell_consumable(0).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(0, 0)));
}

#[test]
fn test_sell_multiple_consumables_mid_round() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheWorld));
    gs.consumables.push(ConsumableCard::Planet(PlanetCard::Jupiter));
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheSun));
    let before = gs.money;

    // Sell all three one by one (always index 0 as list shrinks)
    gs.sell_consumable(0).unwrap();
    gs.sell_consumable(0).unwrap();
    gs.sell_consumable(0).unwrap();

    assert!(gs.consumables.is_empty());
    // 3 cards × $1 each (all Tarot/Planet with base_cost=3)
    assert_eq!(gs.money, before + 3);
}

#[test]
fn test_sell_joker_and_consumable_mid_round_both_update_money() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.jokers.push(joker(10, JokerKind::Joker));         // sell_value = 1
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::Judgement)); // sell = 1
    let before = gs.money;

    gs.sell_joker(0).unwrap();
    gs.sell_consumable(0).unwrap();

    assert_eq!(gs.money, before + 2, "selling one joker and one tarot should yield $2 total");
}
