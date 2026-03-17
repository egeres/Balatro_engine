/// Tests for GameState round-play integration.

use super::*;
use crate::card::ConsumableCard;
use crate::game::BalatroError;

#[test]
fn test_play_pair_through_game_state() {
    let mut gs = make_game();
    setup_round(
        &mut gs,
        vec![
            card(100, Rank::Ace, Suit::Spades),
            card(101, Rank::Ace, Suit::Hearts),
            card(102, Rank::Two, Suit::Clubs),
            card(103, Rank::Three, Suit::Diamonds),
            card(104, Rank::Four, Suit::Spades),
        ],
        5,
    );

    gs.select_card(0).unwrap();
    gs.select_card(1).unwrap();
    gs.play_hand().unwrap();

    // Pair of Aces = 64
    assert!(gs.score_accumulated >= 64.0);
    assert_eq!(gs.hands_remaining, 3);
}

#[test]
fn test_play_flush_through_game_state() {
    let mut gs = make_game();
    setup_round(
        &mut gs,
        vec![
            card(100, Rank::Ace, Suit::Spades),
            card(101, Rank::Three, Suit::Spades),
            card(102, Rank::Seven, Suit::Spades),
            card(103, Rank::Nine, Suit::Spades),
            card(104, Rank::Two, Suit::Spades),
        ],
        5,
    );

    for i in 0..5 {
        gs.select_card(i).unwrap();
    }
    gs.play_hand().unwrap();

    // Flush = 268
    assert!((gs.score_accumulated - 268.0).abs() < 1.0);
}

#[test]
fn test_hands_remaining_decrements_per_play() {
    let mut gs = make_game();
    setup_round(
        &mut gs,
        (0..52)
            .map(|i| card(i as u64, Rank::Ace, Suit::Spades))
            .collect(),
        8,
    );

    let before = gs.hands_remaining;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert_eq!(gs.hands_remaining, before - 1);
}

#[test]
fn test_discard_reduces_discards_remaining() {
    let mut gs = make_game();
    setup_round(
        &mut gs,
        (0..52)
            .map(|i| card(i as u64, Rank::Ace, Suit::Spades))
            .collect(),
        8,
    );

    let before = gs.discards_remaining;
    gs.select_card(0).unwrap();
    gs.discard_hand().unwrap();
    assert_eq!(gs.discards_remaining, before - 1);
}

#[test]
fn test_glass_card_does_not_always_break() {
    // Glass cards break 1/4 of the time; play 20 rounds, not all should break.
    let mut break_count = 0u32;
    for seed_n in 0..20u32 {
        let mut gs = GameState::new(
            DeckType::Blue,
            Stake::White,
            Some(format!("GLASSSEED{seed_n}")),
        );
        let mut glass_ace = card(200, Rank::Ace, Suit::Spades);
        glass_ace.enhancement = Enhancement::Glass;
        setup_round(&mut gs, vec![glass_ace, card(201, Rank::Two, Suit::Clubs)], 2);
        let initial_deck_len = gs.deck.len();
        gs.select_card(0).unwrap();
        gs.play_hand().unwrap();
        if gs.deck.len() < initial_deck_len {
            break_count += 1;
        }
    }
    assert!(break_count <= 15, "Glass broke {break_count}/20 times — too often");
    assert!(break_count >= 1, "Glass never broke across 20 attempts — likely broken logic");
}

#[test]
fn test_score_accumulates_across_multiple_plays() {
    let mut gs = make_game();
    setup_round(
        &mut gs,
        (0..20)
            .map(|i| card(i as u64, Rank::Ace, Suit::Spades))
            .collect(),
        8,
    );
    gs.score_goal = f64::MAX; // prevent auto-win after first hand

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    let after_first = gs.score_accumulated;
    assert!(after_first > 0.0);

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert!(gs.score_accumulated > after_first);
}

#[test]
fn test_joker_applies_to_every_hand_played() {
    let mut gs_with = make_game();
    let mut gs_without = make_game();

    let deck: Vec<CardInstance> = (0..10)
        .map(|i| card(i as u64, Rank::Ace, Suit::Spades))
        .collect();

    setup_round(&mut gs_with, deck.clone(), 5);
    gs_with.jokers.push(joker(999, JokerKind::Joker));

    setup_round(&mut gs_without, deck, 5);

    for gs in [&mut gs_with, &mut gs_without] {
        gs.select_card(0).unwrap();
        gs.play_hand().unwrap();
    }

    assert!(gs_with.score_accumulated > gs_without.score_accumulated);
}

// =========================================================
// swap_jokers
// =========================================================

#[test]
fn test_swap_jokers_changes_order() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Joker));
    gs.jokers.push(joker(1, JokerKind::AbstractJoker));
    gs.jokers.push(joker(2, JokerKind::HalfJoker));

    gs.swap_jokers(0, 2).unwrap();

    assert_eq!(gs.jokers[0].kind, JokerKind::HalfJoker);
    assert_eq!(gs.jokers[1].kind, JokerKind::AbstractJoker);
    assert_eq!(gs.jokers[2].kind, JokerKind::Joker);
}

#[test]
fn test_swap_jokers_adjacent() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Joker));
    gs.jokers.push(joker(1, JokerKind::GreenJoker));

    gs.swap_jokers(0, 1).unwrap();

    assert_eq!(gs.jokers[0].kind, JokerKind::GreenJoker);
    assert_eq!(gs.jokers[1].kind, JokerKind::Joker);
}

#[test]
fn test_swap_jokers_same_index_is_noop() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Joker));
    gs.jokers.push(joker(1, JokerKind::GreenJoker));

    gs.swap_jokers(1, 1).unwrap();

    assert_eq!(gs.jokers[0].kind, JokerKind::Joker);
    assert_eq!(gs.jokers[1].kind, JokerKind::GreenJoker);
}

#[test]
fn test_swap_jokers_out_of_range_first_index() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Joker));

    let err = gs.swap_jokers(5, 0).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(5, 1)));
}

#[test]
fn test_swap_jokers_out_of_range_second_index() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Joker));

    let err = gs.swap_jokers(0, 5).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(5, 1)));
}

#[test]
fn test_swap_jokers_empty_list_returns_error() {
    let mut gs = make_game();
    let err = gs.swap_jokers(0, 1).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(_, _)));
}

#[test]
fn test_swap_jokers_order_affects_blueprint_scoring() {
    // Blueprint copies the joker immediately to its right.
    // [Blueprint, Joker] → Blueprint copies Joker's +4 mult → total mult = 1+4+4 = 9, chips=16 → 144
    // After swapping to [Joker, Blueprint] → Blueprint has nothing to its right → no copy → mult=1+4=5 → 80
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers_before = vec![joker(0, JokerKind::Blueprint), joker(1, JokerKind::Joker)];
    let jokers_after  = vec![joker(1, JokerKind::Joker), joker(0, JokerKind::Blueprint)];

    let r_before = score(&played, &played, &jokers_before);
    let r_after  = score(&played, &played, &jokers_after);

    assert_eq!(r_before.final_score as i64, 144, "Blueprint + Joker should score 144");
    assert_eq!(r_after.final_score  as i64,  80, "Joker + Blueprint (nothing right) should score 80");
}

// =========================================================
// swap_consumables
// =========================================================

#[test]
fn test_swap_consumables_changes_order() {
    let mut gs = make_game();
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheFool));
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheMagician));
    gs.consumables.push(ConsumableCard::Planet(PlanetCard::Mercury));

    gs.swap_consumables(0, 2).unwrap();

    assert_eq!(gs.consumables[0], ConsumableCard::Planet(PlanetCard::Mercury));
    assert_eq!(gs.consumables[1], ConsumableCard::Tarot(TarotCard::TheMagician));
    assert_eq!(gs.consumables[2], ConsumableCard::Tarot(TarotCard::TheFool));
}

#[test]
fn test_swap_consumables_adjacent() {
    let mut gs = make_game();
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheSun));
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheMoon));

    gs.swap_consumables(0, 1).unwrap();

    assert_eq!(gs.consumables[0], ConsumableCard::Tarot(TarotCard::TheMoon));
    assert_eq!(gs.consumables[1], ConsumableCard::Tarot(TarotCard::TheSun));
}

#[test]
fn test_swap_consumables_same_index_is_noop() {
    let mut gs = make_game();
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheSun));
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheMoon));

    gs.swap_consumables(0, 0).unwrap();

    assert_eq!(gs.consumables[0], ConsumableCard::Tarot(TarotCard::TheSun));
    assert_eq!(gs.consumables[1], ConsumableCard::Tarot(TarotCard::TheMoon));
}

#[test]
fn test_swap_consumables_out_of_range_first_index() {
    let mut gs = make_game();
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheFool));

    let err = gs.swap_consumables(3, 0).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(3, 1)));
}

#[test]
fn test_swap_consumables_out_of_range_second_index() {
    let mut gs = make_game();
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheFool));

    let err = gs.swap_consumables(0, 3).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(3, 1)));
}

#[test]
fn test_swap_consumables_empty_list_returns_error() {
    let mut gs = make_game();
    let err = gs.swap_consumables(0, 1).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(_, _)));
}
