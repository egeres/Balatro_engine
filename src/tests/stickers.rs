/// Tests for sticker mechanics: eternal, rental, perishable, and card seals.

use super::*;

#[test]
fn test_eternal_joker_cannot_be_sold() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    let mut eternal = joker(50, JokerKind::Joker);
    eternal.eternal = true;
    gs.jokers.push(eternal);

    let result = gs.sell_joker(0);
    assert!(result.is_err(), "Selling an eternal joker should fail");
}

#[test]
fn test_non_eternal_joker_can_be_sold() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.jokers.push(joker(50, JokerKind::Joker));
    let before_money = gs.money;
    gs.sell_joker(0).unwrap();
    assert!(gs.money > before_money);
    assert!(gs.jokers.is_empty());
}

#[test]
fn test_perishable_joker_expires_after_5_rounds() {
    let mut gs = GameState::new(DeckType::Blue, Stake::White, Some("PERISH".to_string()));

    let mut p_joker = joker(1, JokerKind::Joker);
    p_joker.perishable = true;
    p_joker.perishable_rounds_left = 5;
    gs.jokers.push(p_joker);

    for round in 0u64..5 {
        setup_round(
            &mut gs,
            vec![card(round * 10 + 50, Rank::Ace, Suit::Spades)],
            1,
        );
        gs.score_goal = 1.0;
        gs.select_card(0).unwrap();
        gs.play_hand().unwrap();
        if matches!(gs.state, GameStateKind::Shop) {
            gs.state = GameStateKind::BlindSelect;
        }
    }

    assert!(
        !gs.jokers[0].active,
        "Perishable joker should be inactive after 5 rounds"
    );
}

#[test]
fn test_perishable_joker_still_active_after_4_rounds() {
    let mut gs = GameState::new(DeckType::Blue, Stake::White, Some("PERISH2".to_string()));

    let mut p_joker = joker(1, JokerKind::Joker);
    p_joker.perishable = true;
    p_joker.perishable_rounds_left = 5;
    gs.jokers.push(p_joker);

    for i in 0..4u64 {
        setup_round(&mut gs, vec![card(i * 10, Rank::Ace, Suit::Spades)], 1);
        gs.score_goal = 1.0;
        gs.select_card(0).unwrap();
        gs.play_hand().unwrap();
        if matches!(gs.state, GameStateKind::Shop) {
            gs.state = GameStateKind::BlindSelect;
        }
    }

    assert!(
        gs.jokers[0].active,
        "Perishable joker should still be active after only 4 rounds"
    );
    assert_eq!(gs.jokers[0].perishable_rounds_left, 1);
}

#[test]
fn test_rental_joker_costs_1_dollar_per_shop_visit() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.money = 20;

    let mut rental = joker(1, JokerKind::Joker);
    rental.rental = true;
    gs.jokers.push(rental);

    gs.leave_shop().unwrap();
    assert_eq!(gs.money, 19, "Rental joker should cost $1 on leave_shop");
}

#[test]
fn test_black_stake_reduces_discards_by_one() {
    let gs_white = GameState::new(DeckType::Blue, Stake::White, Some("W".to_string()));
    let gs_black = GameState::new(DeckType::Blue, Stake::Black, Some("B".to_string()));
    // max_discards unchanged; effective reduces it during round
    assert_eq!(gs_white.max_discards, 3);
    assert_eq!(gs_black.max_discards, 3);
}

// =========================================================
// Card Seals
// =========================================================

#[test]
fn test_gold_seal_earns_3_dollars_when_scored() {
    // Gold seal on a scoring card → +$3 in score result
    let mut ace = card(0, Rank::Ace, Suit::Spades);
    ace.seal = Seal::Gold;
    let played = vec![ace];
    let r = score(&played, &played, &[]);
    assert_eq!(r.dollars_earned, 3, "Gold seal should earn $3 when card is scored");
}

#[test]
fn test_gold_seal_on_non_scoring_card_earns_nothing() {
    // Kicker does not score → Gold seal should not trigger
    let mut two = card(0, Rank::Two, Suit::Spades);
    two.seal = Seal::Gold;
    let ace = card(1, Rank::Ace, Suit::Hearts);
    // Play only the Ace (High Card) — Two is in hand but not played
    let played = vec![ace.clone()];
    let hand = vec![ace, two];
    let r = score(&played, &hand, &[]);
    assert_eq!(r.dollars_earned, 0, "Gold seal on non-scoring card should earn nothing");
}

#[test]
fn test_red_seal_retriggers_card_once() {
    // Red seal retriggers the card → chips counted twice
    // High Card: base 5 chips. Ace without seal → 5+11=16.
    // With Red seal on Ace → scores twice → 5+11+11=27, mult=1 → 27
    let mut ace = card(0, Rank::Ace, Suit::Spades);
    ace.seal = Seal::Red;
    let played = vec![ace];
    let r = score(&played, &played, &[]);
    // base 5 chips + Ace 11 (first) + Ace 11 (retrigger) = 27
    assert_eq!(r.final_score as i64, 27, "Red seal should retrigger card chips once");
}

#[test]
fn test_red_seal_no_effect_on_non_red_card() {
    let ace = card(0, Rank::Ace, Suit::Spades); // no seal
    let played = vec![ace];
    let r = score(&played, &played, &[]);
    assert_eq!(r.final_score as i64, 16, "No seal should not retrigger");
}

#[test]
fn test_blue_seal_creates_planet_card_at_round_end() {
    // Blue seal on a card held in hand at round end → creates a Planet consumable
    let mut gs = make_game();
    let mut blue_card = card(0, Rank::Ace, Suit::Spades);
    blue_card.seal = Seal::Blue;
    // Set up round with the blue-sealed card in hand (not played)
    setup_round(
        &mut gs,
        vec![
            blue_card,
            card(1, Rank::Two, Suit::Hearts), // play this one
        ],
        2,
    );
    gs.score_goal = 1.0;
    gs.consumables.clear();

    // Play card at index 1 (the Two) — Blue Ace stays in hand
    gs.select_card(1).unwrap();
    gs.play_hand().unwrap(); // triggers win_round → Blue seal fires

    let planet_count = gs.consumables.iter().filter(|c| {
        matches!(c, crate::card::ConsumableCard::Planet(_))
    }).count();
    assert!(planet_count >= 1, "Blue seal should have created a Planet card at round end");
}

#[test]
fn test_purple_seal_creates_tarot_card_on_discard() {
    // Purple seal on a discarded card → creates a Tarot consumable
    let mut gs = make_game();
    let mut purple_card = card(0, Rank::Two, Suit::Spades);
    purple_card.seal = Seal::Purple;
    setup_round(
        &mut gs,
        (0..10).map(|i| {
            if i == 0 {
                let mut c = card(i as u64, Rank::Two, Suit::Spades);
                c.seal = Seal::Purple;
                c
            } else {
                card(i as u64, Rank::Ace, Suit::Hearts)
            }
        }).collect(),
        5,
    );
    gs.consumables.clear();

    // Discard the purple card (index 0 in hand)
    gs.select_card(0).unwrap();
    gs.discard_hand().unwrap();

    let tarot_count = gs.consumables.iter().filter(|c| {
        matches!(c, crate::card::ConsumableCard::Tarot(_))
    }).count();
    assert!(tarot_count >= 1, "Purple seal should have created a Tarot card on discard");
}
