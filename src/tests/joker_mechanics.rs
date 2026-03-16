/// Tests for gameplay-modifying jokers: FourFingers, Shortcut, Smeared, Splash, Mime,
/// Hack, SockAndBuskin, HangingChad, Pareidolia.

use super::*;

// =========================================================
// Hand evaluation modifiers
// =========================================================

#[test]
fn test_four_fingers_enables_4_card_flush() {
    // 4 Spades that don't form a straight → Flush with FourFingers
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Three, Suit::Spades),
        card(2, Rank::Seven, Suit::Spades),
        card(3, Rank::Jack, Suit::Spades),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::FourFingers)]);
    assert_eq!(r.hand_type, HandType::Flush);
    // Flush: base 35+11+3+7+10=66 chips, mult=4 → 264
    assert_eq!(r.final_score as i64, 264);
}

#[test]
fn test_four_fingers_enables_4_card_straight() {
    // A-2-3-4 mixed suits → Straight with FourFingers, not without
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Diamonds),
        card(3, Rank::Four, Suit::Clubs),
    ];
    let without = score(&played, &played, &[]);
    let with_ff = score(&played, &played, &[joker(0, JokerKind::FourFingers)]);
    assert_ne!(without.hand_type, HandType::Straight);
    assert_eq!(with_ff.hand_type, HandType::Straight);
}

#[test]
fn test_shortcut_enables_gapped_straight() {
    // 2-3-4-6-7: gap between 4 and 6 (diff=2). Not a straight normally; is with Shortcut.
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Three, Suit::Hearts),
        card(2, Rank::Four, Suit::Diamonds),
        card(3, Rank::Six, Suit::Clubs),
        card(4, Rank::Seven, Suit::Spades),
    ];
    let without = score(&played, &played, &[]);
    let with_sc = score(&played, &played, &[joker(0, JokerKind::Shortcut)]);
    assert_ne!(without.hand_type, HandType::Straight, "Should not be Straight without Shortcut");
    assert_eq!(with_sc.hand_type, HandType::Straight, "Should be Straight with Shortcut");
}

#[test]
fn test_smeared_joker_enables_flush_across_equivalent_suits() {
    // SmearedJoker: Hearts = Diamonds for flush purposes.
    // 3 Hearts + 2 Diamonds → flush with Smeared, not without.
    let played = vec![
        card(0, Rank::Ace, Suit::Hearts),
        card(1, Rank::Three, Suit::Hearts),
        card(2, Rank::Seven, Suit::Hearts),
        card(3, Rank::Nine, Suit::Diamonds),
        card(4, Rank::Jack, Suit::Diamonds),
    ];
    let without = score(&played, &played, &[]);
    let with_smeared = score(&played, &played, &[joker(0, JokerKind::SmearedJoker)]);
    assert_ne!(without.hand_type, HandType::Flush, "Mixed suits should not be Flush without Smeared");
    assert_eq!(with_smeared.hand_type, HandType::Flush, "Hearts+Diamonds should be Flush with Smeared");
}

#[test]
fn test_smeared_joker_enables_flush_spades_clubs() {
    // Spades = Clubs with Smeared
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Four, Suit::Spades),
        card(2, Rank::Seven, Suit::Spades),
        card(3, Rank::Nine, Suit::Clubs),
        card(4, Rank::Jack, Suit::Clubs),
    ];
    let without = score(&played, &played, &[]);
    let with_smeared = score(&played, &played, &[joker(0, JokerKind::SmearedJoker)]);
    assert_ne!(without.hand_type, HandType::Flush);
    assert_eq!(with_smeared.hand_type, HandType::Flush);
}

// =========================================================
// Scoring set modifiers
// =========================================================

#[test]
fn test_splash_makes_all_played_cards_score() {
    // Pair 2♠2♥ + kicker 3♣. Without Splash: 3♣ doesn't score.
    // With Splash: all 3 cards score.
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    let without = score(&played, &played, &[]);
    let with_splash = score(&played, &played, &[joker(0, JokerKind::Splash)]);
    // Without: Pair chips=10+2+2=14, mult=2 → 28
    assert_eq!(without.final_score as i64, 28);
    // With Splash: chips=10+2+2+3=17, mult=2 → 34
    assert_eq!(with_splash.final_score as i64, 34);
}

// =========================================================
// Retrigger jokers
// =========================================================

#[test]
fn test_hack_retriggers_two_through_five() {
    let played = vec![card(0, Rank::Two, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Hack)]);
    // 2♠ scores twice: base 5 + 2+2=9, mult=1 → 9
    assert_eq!(r.final_score as i64, 9);
}

#[test]
fn test_hack_does_not_retrigger_high_ranks() {
    let played = vec![card(0, Rank::King, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Hack)]);
    assert_eq!(r.final_score as i64, 15);
}

#[test]
fn test_sock_and_buskin_retriggers_face_cards() {
    let played = vec![card(0, Rank::Jack, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::SockAndBuskin)]);
    // Jack scores twice: 5+10+10=25, mult=1 → 25
    assert_eq!(r.final_score as i64, 25);
}

#[test]
fn test_hanging_chad_retriggers_first_card_twice() {
    let played = vec![card(0, Rank::Two, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::HangingChad)]);
    // 2♠ scores 3×: 5+2+2+2=11, mult=1 → 11
    assert_eq!(r.final_score as i64, 11);
}

#[test]
fn test_mime_retriggers_steel_card_held_in_hand() {
    // Steel card held: normally x1.5; with Mime it retriggers → x1.5*x1.5=x2.25
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut steel = card(1, Rank::Two, Suit::Hearts);
    steel.enhancement = Enhancement::Steel;
    let hand = vec![steel];
    let r_without = score(&played, &hand, &[]);
    let r_with = score(&played, &hand, &[joker(0, JokerKind::Mime)]);
    // Without Mime: 16*1.5=24; With Mime: 16*2.25=36
    assert_eq!(r_without.final_score as i64, 24);
    assert_eq!(r_with.final_score as i64, 36);
}

// =========================================================
// Face-card detection modifier
// =========================================================

#[test]
fn test_pareidolia_makes_non_face_cards_count_as_face() {
    // Without Pareidolia, 2♠ is not face → ScaryFace does not fire
    // With Pareidolia, all cards are face → ScaryFace fires
    let played = vec![card(0, Rank::Two, Suit::Spades)];
    let without = score(&played, &played, &[joker(0, JokerKind::ScaryFace)]);
    let with_par = score(&played, &played, &[joker(0, JokerKind::Pareidolia), joker(1, JokerKind::ScaryFace)]);
    // Without: HC 5+2=7 chips, mult=1 → 7
    assert_eq!(without.final_score as i64, 7);
    // With Pareidolia: HC 5+2+30=37 chips, mult=1 → 37
    assert_eq!(with_par.final_score as i64, 37);
}
