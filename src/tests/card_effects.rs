/// Tests for card enhancements and card/joker editions.

use super::*;

// =========================================================
// Card Enhancement Effects
// =========================================================

#[test]
fn test_mult_enhancement_adds_flat_mult() {
    // Pair of Aces; second Ace has Mult enhancement (+4 mult when scoring)
    // chips = 10 + 11 + 11 = 32; mult = 2 + 4 = 6 → 192
    let mut ace_mult = card(1, Rank::Ace, Suit::Hearts);
    ace_mult.enhancement = Enhancement::Mult;
    let played = vec![card(0, Rank::Ace, Suit::Spades), ace_mult];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::Pair);
    assert_eq!(r.final_score as i64, 192);
}

#[test]
fn test_bonus_enhancement_adds_chips() {
    // Single Ace (Bonus): base 5 + 11(rank) + 30(bonus) = 46, mult 1 → 46
    let mut ace = card(0, Rank::Ace, Suit::Spades);
    ace.enhancement = Enhancement::Bonus;
    let played = vec![ace];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::HighCard);
    assert_eq!(r.final_score as i64, 46);
}

#[test]
fn test_glass_enhancement_doubles_mult() {
    // Single Ace (Glass): chips = 5 + 11 = 16, mult = 1 * 2 = 2 → 32
    let mut ace = card(0, Rank::Ace, Suit::Spades);
    ace.enhancement = Enhancement::Glass;
    let played = vec![ace];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::HighCard);
    assert_eq!(r.final_score as i64, 32);
}

#[test]
fn test_stone_enhancement_gives_50_chips_and_always_scores() {
    // Stone card always scores 50 chips regardless of hand context
    // High Card, stone played alone: base 5 + 50(stone) = 55, mult 1 → 55
    let mut stone = card(0, Rank::Two, Suit::Spades);
    stone.enhancement = Enhancement::Stone;
    let played = vec![stone];
    let r = score(&played, &played, &[]);
    assert_eq!(r.final_score as i64, 55);
}

#[test]
fn test_steel_enhancement_held_in_hand_boosts_mult() {
    // Play single Ace; hold Steel King in hand → mult *= 1.5
    // chips = 5 + 11 = 16, mult = 1 * 1.5 = 1.5 → 24
    let ace = card(0, Rank::Ace, Suit::Spades);
    let mut steel_king = card(1, Rank::King, Suit::Spades);
    steel_king.enhancement = Enhancement::Steel;
    let played = vec![ace.clone()];
    let hand = vec![ace, steel_king];
    let r = score(&played, &hand, &[]);
    assert_eq!(r.hand_type, HandType::HighCard);
    assert!((r.final_score - 24.0).abs() < 0.01);
}

#[test]
fn test_two_steel_cards_held_stack_multiplicatively() {
    // Play Ace; hold 2 Steel Kings → mult = 1 * 1.5 * 1.5 = 2.25 → 16 * 2.25 = 36
    let ace = card(0, Rank::Ace, Suit::Spades);
    let mut steel1 = card(1, Rank::King, Suit::Spades);
    let mut steel2 = card(2, Rank::King, Suit::Hearts);
    steel1.enhancement = Enhancement::Steel;
    steel2.enhancement = Enhancement::Steel;
    let played = vec![ace.clone()];
    let hand = vec![ace, steel1, steel2];
    let r = score(&played, &hand, &[]);
    assert!((r.final_score - 36.0).abs() < 0.01);
}

#[test]
fn test_debuffed_card_contributes_zero_chips() {
    // Debuffed Ace: contributes 0 chips, no enhancement effects
    // High Card with only a debuffed card → base 5 chips, 1 mult → 5
    let mut ace = card(0, Rank::Ace, Suit::Spades);
    ace.debuffed = true;
    let played = vec![ace];
    let r = score(&played, &played, &[]);
    assert_eq!(r.hand_type, HandType::HighCard);
    assert_eq!(r.final_score as i64, 5);
}

// =========================================================
// Card Edition Effects
// =========================================================

#[test]
fn test_foil_edition_adds_50_chips() {
    // Foil Ace in High Card: base 5 + 11(rank) + 50(foil) = 66, mult 1 → 66
    let mut ace = card(0, Rank::Ace, Suit::Spades);
    ace.edition = Edition::Foil;
    let played = vec![ace];
    let r = score(&played, &played, &[]);
    assert_eq!(r.final_score as i64, 66);
}

#[test]
fn test_holographic_edition_adds_10_mult() {
    // Holo Ace in High Card: chips = 5+11=16, mult = 1+10=11 → 176
    let mut ace = card(0, Rank::Ace, Suit::Spades);
    ace.edition = Edition::Holographic;
    let played = vec![ace];
    let r = score(&played, &played, &[]);
    assert_eq!(r.final_score as i64, 176);
}

#[test]
fn test_polychrome_edition_multiplies_mult() {
    // Poly Ace in High Card: chips = 16, mult = 1 * 1.5 = 1.5 → 24
    let mut ace = card(0, Rank::Ace, Suit::Spades);
    ace.edition = Edition::Polychrome;
    let played = vec![ace];
    let r = score(&played, &played, &[]);
    assert!((r.final_score - 24.0).abs() < 0.01);
}

#[test]
fn test_foil_and_mult_enhancement_stack() {
    // Mult enhancement: +4 mult. Foil edition: +50 chips.
    // chips = 5 + 11 + 50 = 66, mult = 1 + 4 = 5 → 330
    let mut ace = card(0, Rank::Ace, Suit::Spades);
    ace.enhancement = Enhancement::Mult;
    ace.edition = Edition::Foil;
    let played = vec![ace];
    let r = score(&played, &played, &[]);
    assert_eq!(r.final_score as i64, 330);
}

// =========================================================
// Joker Edition Effects
// =========================================================

#[test]
fn test_joker_polychrome_edition_x15_mult() {
    // Polychrome Joker: +4 mult from Joker then ×1.5 from edition
    // High Card Ace: chips=16, mult=(1+4)*1.5=7.5 → 120
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut poly_joker = joker(0, JokerKind::Joker);
    poly_joker.edition = Edition::Polychrome;
    let jokers = vec![poly_joker];
    let r = score(&played, &played, &jokers);
    assert!((r.final_score - 120.0).abs() < 0.01);
}

#[test]
fn test_joker_foil_edition_adds_50_chips() {
    // Foil Joker: +50 chips from edition
    // High Card Ace: chips=16+50=66, mult=1+4(Joker)=5 → 330
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut foil_joker = joker(0, JokerKind::Joker);
    foil_joker.edition = Edition::Foil;
    let jokers = vec![foil_joker];
    let r = score(&played, &played, &jokers);
    assert_eq!(r.final_score as i64, 330);
}

#[test]
fn test_joker_holographic_edition_adds_10_mult() {
    // Holographic Joker: +10 mult from edition
    // High Card Ace: chips=16, mult=1+4(Joker)+10(holo)=15 → 240
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut holo_joker = joker(0, JokerKind::Joker);
    holo_joker.edition = Edition::Holographic;
    let jokers = vec![holo_joker];
    let r = score(&played, &played, &jokers);
    assert_eq!(r.final_score as i64, 240);
}
