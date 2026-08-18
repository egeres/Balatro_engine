/// Tests for basic hand type scoring (no jokers or enhancements).

use super::*;

#[test]
fn test_high_card_score() {
    // Single Ace: base 5 chips + 11 (Ace) = 16, mult 1 → 16
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::HighCard);
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_pair_score() {
    // Pair of Aces: base 10 + 11 + 11 = 32 chips, 2 mult → 64
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
    ];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::Pair);
    assert_eq!(r.final_score as i64, 64);
}

#[test]
fn test_two_pair_score() {
    // Aces + Kings: base 20 + 11+11+10+10 = 62 chips, 2 mult → 124
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
        card(2, Rank::King, Suit::Clubs),
        card(3, Rank::King, Suit::Diamonds),
        card(4, Rank::Two, Suit::Spades),
    ];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::TwoPair);
    // 4 scoring cards: 2 Aces + 2 Kings; 2 is the kicker (not scored)
    // chips = 20 + 11 + 11 + 10 + 10 = 62; mult = 2 → 124
    assert_eq!(r.final_score as i64, 124);
}

#[test]
fn test_three_of_a_kind_score() {
    // Three Aces: base 30 + 11*3 = 63 chips, 3 mult → 189
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
        card(2, Rank::Ace, Suit::Clubs),
        card(3, Rank::Two, Suit::Diamonds),
        card(4, Rank::Three, Suit::Spades),
    ];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::ThreeOfAKind);
    assert_eq!(r.final_score as i64, 189);
}

#[test]
fn test_straight_score() {
    // 5-6-7-8-9 mixed suits: base 30 + 5+6+7+8+9=35 = 65 chips, 4 mult → 260
    let played = vec![
        card(0, Rank::Five, Suit::Spades),
        card(1, Rank::Six, Suit::Hearts),
        card(2, Rank::Seven, Suit::Clubs),
        card(3, Rank::Eight, Suit::Diamonds),
        card(4, Rank::Nine, Suit::Spades),
    ];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::Straight);
    assert_eq!(r.final_score as i64, 260);
}

#[test]
fn test_flush_score() {
    // A-3-7-9-2 all Spades: base 35 + 11+3+7+9+2=32 = 67 chips, 4 mult → 268
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Three, Suit::Spades),
        card(2, Rank::Seven, Suit::Spades),
        card(3, Rank::Nine, Suit::Spades),
        card(4, Rank::Two, Suit::Spades),
    ];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::Flush);
    assert_eq!(r.final_score as i64, 268);
}

#[test]
fn test_full_house_score() {
    // 3 Aces + 2 Kings: base 40 + 11*3+10*2=53 = 93 chips, 4 mult → 372
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
        card(2, Rank::Ace, Suit::Clubs),
        card(3, Rank::King, Suit::Spades),
        card(4, Rank::King, Suit::Hearts),
    ];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::FullHouse);
    assert_eq!(r.final_score as i64, 372);
}

#[test]
fn test_four_of_a_kind_score() {
    // 4 Aces + King kicker: base 60 + 11*4=44 = 104 chips, 7 mult → 728
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
        card(2, Rank::Ace, Suit::Clubs),
        card(3, Rank::Ace, Suit::Diamonds),
        card(4, Rank::King, Suit::Spades),
    ];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::FourOfAKind);
    assert_eq!(r.final_score as i64, 728);
}

#[test]
fn test_straight_flush_score() {
    // 5-6-7-8-9 all Spades: base 100 + 35 = 135 chips, 8 mult → 1080
    let played = vec![
        card(0, Rank::Five, Suit::Spades),
        card(1, Rank::Six, Suit::Spades),
        card(2, Rank::Seven, Suit::Spades),
        card(3, Rank::Eight, Suit::Spades),
        card(4, Rank::Nine, Suit::Spades),
    ];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::StraightFlush);
    assert_eq!(r.final_score as i64, 1080);
}

#[test]
fn test_five_of_a_kind_score() {
    // 5 Aces: base 120 + 11*5=55 = 175 chips, 12 mult → 2100
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
        card(2, Rank::Ace, Suit::Clubs),
        card(3, Rank::Ace, Suit::Diamonds),
        card(4, Rank::Ace, Suit::Spades),
    ];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::FiveOfAKind);
    assert_eq!(r.final_score as i64, 2100);
}

#[test]
fn test_flush_five_score() {
    // 5 Aces all Spades: base 160 + 55 = 215 chips, 16 mult → 3440
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Spades),
        card(2, Rank::Ace, Suit::Spades),
        card(3, Rank::Ace, Suit::Spades),
        card(4, Rank::Ace, Suit::Spades),
    ];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::FlushFive);
    assert_eq!(r.final_score as i64, 3440);
}

#[test]
fn test_flush_house_score() {
    // 3 Aces + 2 Kings all Spades: base 140 + 53 = 193 chips, 14 mult → 2702
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Spades),
        card(2, Rank::Ace, Suit::Spades),
        card(3, Rank::King, Suit::Spades),
        card(4, Rank::King, Suit::Spades),
    ];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::FlushHouse);
    assert_eq!(r.final_score as i64, 2702);
}

#[test]
fn test_hand_levels_upgrade_increases_base_score() {
    // Level 2 Pair should have higher base chips/mult than level 1
    let mut levels = default_hand_levels();
    let l1_chips = levels[&HandType::Pair].chips(HandType::Pair);
    let l1_mult = levels[&HandType::Pair].mult(HandType::Pair);

    levels.get_mut(&HandType::Pair).unwrap().level = 2;
    let l2_chips = levels[&HandType::Pair].chips(HandType::Pair);
    let l2_mult = levels[&HandType::Pair].mult(HandType::Pair);

    assert!(l2_chips > l1_chips, "Level 2 Pair should have more chips");
    assert!(l2_mult > l1_mult, "Level 2 Pair should have more mult");
}

// =========================================================
// Scoring-card ordering
// =========================================================

/// Balatro sorts the scoring hand left-to-right before scoring (state_events.lua:600).
/// Grouping by rank otherwise leaves the order at the mercy of HashMap iteration.
#[test]
fn test_scoring_indices_are_in_played_order() {
    // Two pair: the lower pair sits to the left of the higher pair.
    let played = vec![
        card(0, Rank::Three, Suit::Spades),
        card(1, Rank::King, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
        card(3, Rank::King, Suit::Diamonds),
        card(4, Rank::Nine, Suit::Spades),
    ];
    let r = crate::hand_eval::evaluate_hand(&played, false, false, false, false);
    assert_eq!(r.hand_type, HandType::TwoPair);
    assert_eq!(r.scoring_indices, vec![0, 1, 2, 3]);
}

/// Hanging Chad retriggers `scoring_hand[1]` — the leftmost scoring card, not whichever
/// card the rank grouping happened to emit first.
#[test]
fn test_hanging_chad_retriggers_the_leftmost_scoring_card() {
    // 3♠ K♥ 3♣ K♦ 9♠ → two pair; leftmost scoring card is the 3♠ (3 chips).
    let played = vec![
        card(0, Rank::Three, Suit::Spades),
        card(1, Rank::King, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
        card(3, Rank::King, Suit::Diamonds),
        card(4, Rank::Nine, Suit::Spades),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::HangingChad)]);
    // TwoPair: 20 base + 3 + 10 + 3 + 10 = 46, plus the 3♠ scored twice more (+6) = 52
    assert_eq!(r.final_chips as i64, 52);
}

/// Photograph fires on the first *face* card in played order.
#[test]
fn test_photograph_fires_on_the_leftmost_face_card() {
    let played = vec![
        card(0, Rank::Queen, Suit::Spades),
        card(1, Rank::Queen, Suit::Hearts),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::Photograph)]);
    // Pair: 10 + 10 + 10 = 30 chips, mult 2 x2 (one Photograph trigger) = 4 → 120
    assert_eq!(r.final_score as i64, 120);
}

/// A hand of nothing but Stone cards is still a High Card, and `contained` has to say so.
///
/// Stone cards take no part in deciding the hand type, so an all-Stone hand leaves the
/// evaluator with nothing to read — but it still names the hand High Card, and the hand that
/// was played must be among the hands the cards contain. The two are computed by separate
/// functions, which is exactly how they came to disagree.
#[test]
fn test_an_all_stone_hand_contains_the_high_card_it_reports() {
    let mut played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Seven, Suit::Hearts),
        card(2, Rank::King, Suit::Clubs),
    ];
    for c in played.iter_mut() {
        c.enhancement = Enhancement::Stone;
    }
    let r = crate::hand_eval::evaluate_hand(&played, false, false, false, false);
    assert_eq!(r.hand_type, HandType::HighCard);
    assert!(
        r.contained.contains(HandType::HighCard),
        "an all-Stone hand reported High Card but contained no hand at all"
    );
    // Every Stone card still scores.
    assert_eq!(r.scoring_indices, vec![0, 1, 2]);
}
