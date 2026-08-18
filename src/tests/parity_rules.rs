/// Rule systems that cut across many jokers: the contained-hand table, Pareidolia, Smeared
/// Joker, Wild/Stone suit matching, Splash, and The Psychic's rejected hands.

use super::*;
use crate::game::{BlindKind, GameStateKind};
use crate::hand_eval::evaluate_hand;

fn contains(cards: &[CardInstance], h: HandType) -> bool {
    evaluate_hand(cards, false, false, false, false).contained.contains(h)
}

/// A five-card Spade flush whose two lowest cards make a pair.
fn flush_holding_a_pair() -> Vec<CardInstance> {
    vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Spades),
        card(2, Rank::Five, Suit::Spades),
        card(3, Rank::Seven, Suit::Spades),
        card(4, Rank::Nine, Suit::Spades),
    ]
}

// =========================================================
// Contained hands: jokers ask what the hand holds, not what it is called
// =========================================================

#[test]
fn test_a_flush_containing_a_pair_contains_a_pair() {
    let eval = evaluate_hand(&flush_holding_a_pair(), false, false, false, false);
    assert_eq!(eval.hand_type, HandType::Flush);
    assert!(eval.contained.contains(HandType::Flush));
    assert!(eval.contained.contains(HandType::Pair));
    assert!(!eval.contained.contains(HandType::TwoPair));
}

#[test]
fn test_jolly_joker_fires_on_a_flush_that_holds_a_pair() {
    let played = flush_holding_a_pair();
    let base = score(&played, &[], &[]);
    let jolly = score(&played, &[], &[joker(0, JokerKind::JollyJoker)]);
    assert_eq!(jolly.final_mult - base.final_mult, 8.0,
        "Jolly Joker reads poker_hands Pair, which a Flush can hold");
}

#[test]
fn test_the_duo_fires_on_a_flush_that_holds_a_pair() {
    let played = flush_holding_a_pair();
    let base = score(&played, &[], &[]);
    let duo = score(&played, &[], &[joker(0, JokerKind::TheDuo)]);
    assert_eq!(duo.final_mult, base.final_mult * 2.0);
}

#[test]
fn test_sly_joker_fires_on_a_flush_that_holds_a_pair() {
    let played = flush_holding_a_pair();
    let base = score(&played, &[], &[]);
    let sly = score(&played, &[], &[joker(0, JokerKind::SlyJoker)]);
    assert_eq!(sly.final_chips - base.final_chips, 50.0);
}

#[test]
fn test_five_of_a_kind_contains_four_three_and_pair_but_not_two_pair() {
    // get_X_same matches an exact group size, so the containment comes from the cascade.
    let five = vec![
        card(0, Rank::Nine, Suit::Spades),
        card(1, Rank::Nine, Suit::Hearts),
        card(2, Rank::Nine, Suit::Clubs),
        card(3, Rank::Nine, Suit::Diamonds),
        card(4, Rank::Nine, Suit::Spades),
    ];
    assert!(contains(&five, HandType::FiveOfAKind));
    assert!(contains(&five, HandType::FourOfAKind));
    assert!(contains(&five, HandType::ThreeOfAKind));
    assert!(contains(&five, HandType::Pair));
    assert!(!contains(&five, HandType::TwoPair));
    assert!(!contains(&five, HandType::FullHouse));
}

#[test]
fn test_full_house_contains_two_pair() {
    let fh = vec![
        card(0, Rank::Nine, Suit::Spades),
        card(1, Rank::Nine, Suit::Hearts),
        card(2, Rank::Nine, Suit::Clubs),
        card(3, Rank::Four, Suit::Diamonds),
        card(4, Rank::Four, Suit::Spades),
    ];
    assert!(contains(&fh, HandType::FullHouse));
    assert!(contains(&fh, HandType::TwoPair));
    assert!(contains(&fh, HandType::ThreeOfAKind));
    assert!(contains(&fh, HandType::Pair));
    assert!(!contains(&fh, HandType::FourOfAKind));
}

#[test]
fn test_flush_five_contains_a_flush_but_not_a_straight() {
    let ff = vec![
        card(0, Rank::Nine, Suit::Spades),
        card(1, Rank::Nine, Suit::Spades),
        card(2, Rank::Nine, Suit::Spades),
        card(3, Rank::Nine, Suit::Spades),
        card(4, Rank::Nine, Suit::Spades),
    ];
    assert!(contains(&ff, HandType::FlushFive));
    assert!(contains(&ff, HandType::Flush));
    assert!(!contains(&ff, HandType::Straight));
    assert!(!contains(&ff, HandType::StraightFlush));
}

#[test]
fn test_runner_scales_on_a_straight_inside_a_bigger_hand() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Five, Suit::Spades),
        card(1, Rank::Six, Suit::Spades),
        card(2, Rank::Seven, Suit::Spades),
        card(3, Rank::Eight, Suit::Spades),
        card(4, Rank::Nine, Suit::Spades),
    ];
    setup_round(&mut gs, cards, 5);
    gs.jokers.push(joker(1, JokerKind::Runner));
    gs.score_goal = f64::MAX;

    for i in 0..5 { gs.select_card(i).unwrap(); }
    let r = gs.play_hand().unwrap();

    assert_eq!(r.hand_type, HandType::StraightFlush);
    let runner = gs.jokers.iter().find(|j| j.kind == JokerKind::Runner).unwrap();
    assert_eq!(runner.get_counter_i64("chips"), 15,
        "a Straight Flush contains a Straight, so Runner scales");
}

// =========================================================
// Splash: every played card scores, for every hand type
// =========================================================

#[test]
fn test_splash_scores_the_odd_card_out_of_a_four_fingers_flush() {
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Three, Suit::Spades),
        card(2, Rank::Four, Suit::Spades),
        card(3, Rank::Five, Suit::Spades),
        card(4, Rank::King, Suit::Hearts),
    ];
    let jokers = vec![joker(0, JokerKind::FourFingers), joker(1, JokerKind::Splash)];
    let r = score(&played, &[], &jokers);
    assert_eq!(r.scoring_card_indices.len(), 5,
        "Splash overwrites the whole scoring hand, whatever the hand type");
}

#[test]
fn test_splash_scores_every_card_of_a_full_house() {
    let played = vec![
        card(0, Rank::Nine, Suit::Spades),
        card(1, Rank::Nine, Suit::Hearts),
        card(2, Rank::Nine, Suit::Clubs),
        card(3, Rank::Four, Suit::Diamonds),
        card(4, Rank::Four, Suit::Spades),
    ];
    let r = score(&played, &[], &[joker(0, JokerKind::Splash)]);
    assert_eq!(r.scoring_card_indices.len(), 5);
}

#[test]
fn test_splash_still_scores_the_kicker_of_a_pair() {
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Nine, Suit::Clubs),
    ];
    let r = score(&played, &[], &[joker(0, JokerKind::Splash)]);
    assert_eq!(r.scoring_card_indices, vec![0, 1, 2]);
}

// =========================================================
// Pareidolia turns every card into a face card
// =========================================================

#[test]
fn test_pareidolia_makes_sock_and_buskin_retrigger_everything() {
    let played = vec![card(0, Rank::Two, Suit::Spades)];
    let without = score(&played, &[], &[joker(0, JokerKind::SockAndBuskin)]);
    let with = score(&played, &[], &[
        joker(0, JokerKind::Pareidolia),
        joker(1, JokerKind::SockAndBuskin),
    ]);
    // High Card: 5 chips base, plus 2 per trigger of the played card.
    assert_eq!(without.final_chips, 7.0);
    assert_eq!(with.final_chips, 9.0, "Sock and Buskin should retrigger the 2 under Pareidolia");
}

#[test]
fn test_pareidolia_keeps_ride_the_bus_from_ever_scaling() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Two, Suit::Spades)], 1);
    gs.jokers.push(joker(1, JokerKind::Pareidolia));
    gs.jokers.push(joker(2, JokerKind::RideTheBus));
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    let bus = gs.jokers.iter().find(|j| j.kind == JokerKind::RideTheBus).unwrap();
    assert_eq!(bus.get_counter_i64("mult"), 0,
        "every card is a face card under Pareidolia, so Ride the Bus resets");
}

#[test]
fn test_pareidolia_lets_midas_mask_gild_everything() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Two, Suit::Spades)], 1);
    gs.jokers.push(joker(1, JokerKind::Pareidolia));
    gs.jokers.push(joker(2, JokerKind::MidasMask));
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    assert_eq!(gs.deck[0].enhancement, Enhancement::Gold);
}

#[test]
fn test_pareidolia_debuffs_the_whole_deck_under_the_plant() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Pareidolia));
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::ThePlant);

    gs.select_blind().unwrap();

    assert!(gs.deck.iter().all(|c| c.debuffed),
        "The Plant calls is_face(true), which Pareidolia answers for every card");
}

#[test]
fn test_pareidolia_makes_faceless_joker_pay_on_any_three_discards() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Three, Suit::Hearts),
        card(2, Rank::Four, Suit::Clubs),
    ];
    setup_round(&mut gs, cards, 3);
    gs.jokers.push(joker(1, JokerKind::Pareidolia));
    gs.jokers.push(joker(2, JokerKind::FacelessJoker));
    gs.money = 0;

    for i in 0..3 { gs.select_card(i).unwrap(); }
    gs.discard_hand().unwrap();

    assert_eq!(gs.money, 5, "three face cards under Pareidolia should pay Faceless Joker");
}

// =========================================================
// Smeared Joker merges the colours for every suit check
// =========================================================

#[test]
fn test_smeared_joker_lets_a_diamond_feed_lusty_joker() {
    let played = vec![card(0, Rank::Two, Suit::Diamonds)];
    let without = score(&played, &[], &[joker(0, JokerKind::LustyJoker)]);
    let with = score(&played, &[], &[
        joker(0, JokerKind::SmearedJoker),
        joker(1, JokerKind::LustyJoker),
    ]);
    assert_eq!(without.final_mult, 1.0);
    assert_eq!(with.final_mult, 4.0, "Smeared makes Diamonds count as Hearts everywhere");
}

#[test]
fn test_smeared_joker_lets_a_club_feed_arrowhead() {
    let played = vec![card(0, Rank::Two, Suit::Clubs)];
    let without = score(&played, &[], &[joker(0, JokerKind::Arrowhead)]);
    let with = score(&played, &[], &[
        joker(0, JokerKind::SmearedJoker),
        joker(1, JokerKind::Arrowhead),
    ]);
    assert_eq!(with.final_chips - without.final_chips, 50.0);
}

#[test]
fn test_smeared_joker_widens_the_club_blind_to_spades() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::SmearedJoker));
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::TheClub);

    gs.select_blind().unwrap();

    assert!(gs.deck.iter().filter(|c| c.suit == Suit::Spades).all(|c| c.debuffed),
        "debuff_card goes through is_suit, which Smeared widens to the whole colour");
}

// =========================================================
// Wild and Stone cards under is_suit
// =========================================================

#[test]
fn test_a_wild_card_is_debuffed_by_every_suit_blind() {
    let mut gs = make_game();
    gs.deck[0].suit = Suit::Hearts;
    gs.deck[0].enhancement = Enhancement::Wild;
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::TheClub);

    gs.select_blind().unwrap();

    assert!(gs.deck[0].debuffed, "a Wild Card counts as every suit, Clubs included");
}

#[test]
fn test_a_stone_card_is_immune_to_suit_blinds() {
    let mut gs = make_game();
    gs.deck[0].suit = Suit::Clubs;
    gs.deck[0].enhancement = Enhancement::Stone;
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::TheClub);

    gs.select_blind().unwrap();

    assert!(!gs.deck[0].debuffed, "a Stone card has no suit for is_suit to match");
}

#[test]
fn test_a_stone_card_does_not_feed_suit_jokers() {
    let mut stone = card(0, Rank::Two, Suit::Clubs);
    stone.enhancement = Enhancement::Stone;
    let r = score(&[stone], &[], &[joker(0, JokerKind::GluttonousJoker)]);
    assert_eq!(r.final_mult, 1.0, "a Stone card is suitless, so Gluttonous Joker sees nothing");
}

#[test]
fn test_a_stone_card_in_hand_breaks_blackboard() {
    let played = vec![card(0, Rank::Two, Suit::Spades)];
    let mut stone = card(1, Rank::Ace, Suit::Spades);
    stone.enhancement = Enhancement::Stone;

    let clean = score(&played, &[card(2, Rank::King, Suit::Clubs)], &[joker(0, JokerKind::Blackboard)]);
    let stoned = score(&played, &[stone], &[joker(0, JokerKind::Blackboard)]);

    assert_eq!(clean.final_mult, 3.0);
    assert_eq!(stoned.final_mult, 1.0, "is_suit is false for a Stone card, which breaks Blackboard");
}

// =========================================================
// The Psychic: a hand of fewer than 5 cards is played but scores nothing
// =========================================================

#[test]
fn test_the_psychic_rejects_a_short_hand_without_blocking_it() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
        card(2, Rank::Two, Suit::Clubs),
        card(3, Rank::Three, Suit::Clubs),
        card(4, Rank::Four, Suit::Clubs),
        card(5, Rank::Five, Suit::Clubs),
    ];
    setup_round(&mut gs, cards, 6);
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::ThePsychic);
    gs.score_goal = f64::MAX;
    let hands_before = gs.hands_remaining;

    gs.select_card(0).unwrap();
    gs.select_card(1).unwrap();
    let r = gs.play_hand().expect("the hand is played, not blocked");

    assert_eq!(r.final_score, 0.0, "a rejected hand scores nothing");
    assert_eq!(gs.score_accumulated, 0.0);
    assert_eq!(gs.hands_remaining, hands_before - 1, "it still costs a hand");
}

#[test]
fn test_the_psychic_scores_a_five_card_hand_normally() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Three, Suit::Hearts),
        card(2, Rank::Four, Suit::Clubs),
        card(3, Rank::Five, Suit::Clubs),
        card(4, Rank::Seven, Suit::Diamonds),
    ];
    setup_round(&mut gs, cards, 5);
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::ThePsychic);
    gs.score_goal = f64::MAX;

    for i in 0..5 { gs.select_card(i).unwrap(); }
    let r = gs.play_hand().unwrap();
    assert!(r.final_score > 0.0);
}

#[test]
fn test_the_psychic_short_hand_skips_the_joker_phase() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::ThePsychic);
    gs.jokers.push(joker(1, JokerKind::GreenJoker));
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    let green = gs.jokers.iter().find(|j| j.kind == JokerKind::GreenJoker).unwrap();
    assert_eq!(green.get_counter_i64("mult"), 0,
        "evaluate_play skips the whole joker block when the blind debuffs the hand");
}

#[test]
fn test_chicot_turns_the_psychic_off() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::ThePsychic);
    gs.jokers.push(joker(1, JokerKind::Chicot));
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    let r = gs.play_hand().unwrap();
    assert!(r.final_score > 0.0, "a disabled blind debuffs nothing");
}

#[test]
fn test_the_psychic_only_applies_during_its_own_blind() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.current_blind = BlindKind::Small;
    gs.boss_blind = Some(BossBlind::ThePsychic);
    gs.score_goal = f64::MAX;
    assert!(matches!(gs.state, GameStateKind::Round));

    gs.select_card(0).unwrap();
    let r = gs.play_hand().unwrap();
    assert!(r.final_score > 0.0, "The Psychic only applies during its own Boss blind");
}
