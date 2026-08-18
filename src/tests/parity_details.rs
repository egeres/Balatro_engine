/// Smaller parity details: which events reach Hologram and Campfire, boss blinds answering to
/// Chicot, The Fish's prepped flag, and jokers that have to act once per copy.

use super::*;
use crate::game::{BlindKind, GameStateKind};

fn hologram_xmult(gs: &GameState) -> f64 {
    gs.jokers
        .iter()
        .find(|j| j.kind == JokerKind::Hologram)
        .map(|j| j.get_counter_f64("x_mult"))
        .unwrap_or(0.0)
}

// =========================================================
// Hologram sees every playing card added to the deck
// =========================================================

#[test]
fn test_hologram_counts_marble_jokers_stone_card() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Hologram));
    gs.jokers.push(joker(2, JokerKind::MarbleJoker));

    gs.select_blind().unwrap();

    assert_eq!(hologram_xmult(&gs), 1.25, "Marble Joker calls playing_card_joker_effects");
}

#[test]
fn test_hologram_counts_certificates_card() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Hologram));
    gs.jokers.push(joker(2, JokerKind::Certificate));

    gs.select_blind().unwrap();

    assert_eq!(hologram_xmult(&gs), 1.25);
}

#[test]
fn test_hologram_counts_the_dna_copy() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.jokers.push(joker(1, JokerKind::Hologram));
    gs.jokers.push(joker(2, JokerKind::Dna));
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    assert_eq!(hologram_xmult(&gs), 1.25, "DNA reports playing_cards_created");
}

#[test]
fn test_hologram_counts_a_whole_batch_of_spectral_cards() {
    // Incantation adds four at once, and Hologram scales with the batch size.
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.jokers.push(joker(1, JokerKind::Hologram));
    gs.consumable_slots = 5;
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::Incantation));

    gs.use_consumable(0, vec![]).unwrap();

    assert_eq!(hologram_xmult(&gs), 2.0, "four cards at 0.25 each");
}

// =========================================================
// Campfire counts every card sold
// =========================================================

#[test]
fn test_campfire_grows_when_a_consumable_is_sold() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.jokers.push(joker(1, JokerKind::Campfire));
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheFool));

    gs.sell_consumable(0).unwrap();

    assert_eq!(gs.jokers[0].get_counter_f64("x_mult"), 1.25,
        "context.selling_card covers consumables, not just jokers");
}

#[test]
fn test_campfire_still_grows_when_a_joker_is_sold() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.jokers.push(joker(1, JokerKind::Campfire));
    gs.jokers.push(joker(2, JokerKind::Joker));

    gs.sell_joker(1).unwrap();

    assert_eq!(gs.jokers[0].get_counter_f64("x_mult"), 1.25);
}

// =========================================================
// Boss effects answer to Chicot
// =========================================================

#[test]
fn test_chicot_turns_off_the_tooth() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::TheTooth);
    gs.jokers.push(joker(1, JokerKind::Chicot));
    gs.money = 10;
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    assert_eq!(gs.money, 10, "a disabled The Tooth charges nothing");
}

#[test]
fn test_the_tooth_still_charges_when_it_is_on() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::TheTooth);
    gs.money = 10;
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    assert_eq!(gs.money, 9, "-$1 per card played");
}

#[test]
fn test_chicot_turns_off_the_needle() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::King, Suit::Hearts),
    ];
    setup_round(&mut gs, cards, 2);
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::TheNeedle);
    gs.jokers.push(joker(1, JokerKind::Chicot));
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    gs.select_card(0).unwrap();
    assert!(gs.play_hand().is_ok(), "a disabled The Needle allows a second hand");
}

// =========================================================
// The Fish only hides what a play drew
// =========================================================

#[test]
fn test_the_fish_hides_cards_drawn_after_a_play() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::King, Suit::Hearts),
        card(2, Rank::Queen, Suit::Clubs),
    ];
    setup_round(&mut gs, cards, 1);
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::TheFish);
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    assert!(gs.hand.iter().any(|&di| gs.deck[di].face_down),
        "cards drawn after a play are hidden");
}

#[test]
fn test_the_fish_leaves_cards_drawn_after_a_discard_face_up() {
    // `prepped` is only set by press_play, so a discard's replacements stay visible.
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::King, Suit::Hearts),
        card(2, Rank::Queen, Suit::Clubs),
    ];
    setup_round(&mut gs, cards, 1);
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::TheFish);
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.discard_hand().unwrap();

    assert!(gs.hand.iter().all(|&di| !gs.deck[di].face_down),
        "a discard does not prep The Fish");
}

// =========================================================
// Jokers that must act once per copy
// =========================================================

#[test]
fn test_two_turtle_beans_each_shrink() {
    let mut gs = make_game();
    gs.joker_slots = 5;
    gs.jokers.push(joker(1, JokerKind::TurtleBean));
    gs.jokers.push(joker(2, JokerKind::TurtleBean));

    gs.select_blind().unwrap();

    for j in gs.jokers.iter().filter(|j| j.kind == JokerKind::TurtleBean) {
        assert_eq!(j.get_counter_i64("h_size"), 4,
            "each Turtle Bean shrinks for itself, not just the first");
    }
}

#[test]
fn test_two_to_do_lists_each_get_a_target() {
    let mut gs = make_game();
    gs.joker_slots = 5;
    gs.jokers.push(joker(1, JokerKind::ToDoList));
    gs.jokers.push(joker(2, JokerKind::ToDoList));

    gs.select_blind().unwrap();

    // Both must hold a target; with one roll each they are independent draws.
    for j in gs.jokers.iter().filter(|j| j.kind == JokerKind::ToDoList) {
        assert!(j.counters.get("hand_type").and_then(|v| v.as_str()).is_some());
    }
}

// =========================================================
// Joker Stencil counts itself, and needs a slot to be empty
// =========================================================

#[test]
fn test_joker_stencil_counts_itself_as_an_empty_slot() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::JokerStencil)];
    // 5 slots, 1 joker → 4 empty, +1 for the Stencil itself = X5.
    let r = score(&played, &[], &jokers);
    assert_eq!(r.final_mult, 5.0);
}

#[test]
fn test_joker_stencil_does_nothing_with_a_full_board() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::JokerStencil)];
    let levels = default_hand_levels();
    let mut inputs = crate::scoring::ScoreInputs::new(&played, &[], &jokers, &levels);
    inputs.joker_slot_count = 1; // no empty slot at all
    let r = crate::scoring::score_hand(inputs);
    assert_eq!(r.final_mult, 1.0, "card.lua:3966 gates the whole effect on an empty slot");
}

// =========================================================
// The Hanged Man destroys at most two
// =========================================================

#[test]
fn test_the_hanged_man_destroys_at_most_two_cards() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::King, Suit::Hearts),
        card(2, Rank::Queen, Suit::Clubs),
        card(3, Rank::Jack, Suit::Diamonds),
    ];
    setup_round(&mut gs, cards, 4);
    gs.consumable_slots = 5;
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheHangedMan));

    gs.use_consumable(0, vec![0, 1, 2, 3]).unwrap();

    assert_eq!(gs.deck.len(), 2, "The Hanged Man destroys up to 2, however many are selected");
}
