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
