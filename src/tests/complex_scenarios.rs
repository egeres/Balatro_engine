/// Complex integration tests: elevated hand levels + 5 jokers, with fully worked calculations.
///
/// Each test documents every phase of scoring so the expected values are independently verifiable:
///
///   Phase 1  — calc_joker_before (hand-type chips/mult bonuses)
///   Phase 2  — per-scoring-card chips/mult/xmult + per-card joker effects
///   Phase 3  — held hand cards (Steel xmult, Baron, ShootTheMoon)
///   Phase 4  — calc_joker_main (flat chips/mult, xmult, joker editions)
///   Final    — chips * mult

use super::*;
use crate::scoring::score_hand;

// Helper: build a hand_levels map with one type overridden.
fn levels_with(ht: HandType, level: u32) -> std::collections::HashMap<HandType, HandLevelData> {
    let mut m = default_hand_levels();
    m.get_mut(&ht).unwrap().level = level;
    m
}

// Helper: score with custom levels and full parameter control.
fn score_levels(
    played: &[CardInstance],
    hand: &[CardInstance],
    jokers: &[JokerInstance],
    levels: &std::collections::HashMap<HandType, HandLevelData>,
) -> crate::scoring::ScoreResult {
    score_hand(played, hand, jokers, levels, 3, 3, 0, 40, 52, None, 5, 0)
}

// =========================================================
// Scenario 1: Level 3 Straight — CrazyJoker + DeviousJoker + Scholar + OddTodd + Joker
//
// Straight L3: base chips = 30 + 30×2 = 90, mult = 4 + 3×2 = 10
//
// Phase 1 (before):
//   CrazyJoker  → +12 mult   (mult = 22)
//   DeviousJoker → +100 chips (chips = 190)
//
// Phase 2 (A♠ 2♥ 3♦ 4♣ 5♠, all score):
//   A♠: +11 chips → 201; Scholar: +20 chips, +4 mult → 221, mult 26; OddTodd: +31 chips → 252
//   2♥: +2 chips → 254
//   3♦: +3 chips → 257; OddTodd: +31 chips → 288
//   4♣: +4 chips → 292
//   5♠: +5 chips → 297; OddTodd: +31 chips → 328
//
// Phase 4 (main):
//   Joker: +4 mult → mult = 30
//
// Final: 328 × 30 = 9840
// =========================================================

#[test]
fn test_scenario_straight_lvl3_five_jokers() {
    let played = vec![
        card(0, Rank::Ace,  Suit::Spades),
        card(1, Rank::Two,  Suit::Hearts),
        card(2, Rank::Three,Suit::Diamonds),
        card(3, Rank::Four, Suit::Clubs),
        card(4, Rank::Five, Suit::Spades),
    ];
    let jokers = vec![
        joker(0, JokerKind::CrazyJoker),
        joker(1, JokerKind::DeviousJoker),
        joker(2, JokerKind::Scholar),
        joker(3, JokerKind::OddTodd),
        joker(4, JokerKind::Joker),
    ];
    let levels = levels_with(HandType::Straight, 3);
    let r = score_levels(&played, &played, &jokers, &levels);

    assert_eq!(r.hand_type, HandType::Straight);
    assert_eq!(r.final_chips as i64, 328,  "chips mismatch");
    assert_eq!(r.final_mult  as i64, 30,   "mult mismatch");
    assert_eq!(r.final_score as i64, 9840, "score mismatch");
}

// =========================================================
// Scenario 2: Level 2 Flush (all Spades, non-consecutive) —
//   DrollJoker + CraftyJoker + WrathfulJoker + Arrowhead + GlassJoker(×2)
//
// Flush L2: base chips = 35 + 15×1 = 50, mult = 4 + 2×1 = 6
//
// Phase 1:
//   DrollJoker  → +10 mult  (mult = 16)
//   CraftyJoker → +80 chips (chips = 130)
//
// Phase 2 (2♠ 4♠ 7♠ 9♠ J♠, all score):
//   2♠: +2 chips → 132; WrathfulJoker: +3 mult → 19; Arrowhead: +50 chips → 182
//   4♠: +4       → 186;               +3 mult → 22;             +50       → 236
//   7♠: +7       → 243;               +3 mult → 25;             +50       → 293
//   9♠: +9       → 302;               +3 mult → 28;             +50       → 352
//   J♠: +10      → 362;               +3 mult → 31;             +50       → 412
//
// Phase 4:
//   GlassJoker (×2.0): mult × 2 = 62
//
// Final: 412 × 62 = 25544
// =========================================================

#[test]
fn test_scenario_flush_lvl2_five_jokers() {
    let played = vec![
        card(0, Rank::Two,  Suit::Spades),
        card(1, Rank::Four, Suit::Spades),
        card(2, Rank::Seven,Suit::Spades),
        card(3, Rank::Nine, Suit::Spades),
        card(4, Rank::Jack, Suit::Spades),
    ];
    let mut glass_j = joker(4, JokerKind::GlassJoker);
    glass_j.set_counter_f64("x_mult", 2.0);

    let jokers = vec![
        joker(0, JokerKind::DrollJoker),
        joker(1, JokerKind::CraftyJoker),
        joker(2, JokerKind::WrathfulJoker),
        joker(3, JokerKind::Arrowhead),
        glass_j,
    ];
    let levels = levels_with(HandType::Flush, 2);
    let r = score_levels(&played, &played, &jokers, &levels);

    assert_eq!(r.hand_type, HandType::Flush);
    assert_eq!(r.final_chips as i64, 412,   "chips mismatch");
    assert_eq!(r.final_mult  as i64, 62,    "mult mismatch");
    assert_eq!(r.final_score as i64, 25544, "score mismatch");
}

// =========================================================
// Scenario 3: Level 2 Full House — Joker + Scholar + Baron + ShootTheMoon + GlassJoker(×1.5)
//   Played: A♠ A♥ A♣ K♠ K♥  (all 5 score)
//   Held (not played): K♦  Q♦
//
// Full House L2: chips = 40 + 25×1 = 65, mult = 4 + 2×1 = 6
//
// Phase 1: nothing fires
//
// Phase 2 (A♠ A♥ A♣ K♠ K♥ all score):
//   A♠: +11 chips → 76;  Scholar: +20 chips, +4 mult → 96, mult 10
//   A♥: +11 → 107;       Scholar: +20, +4 mult → 127, mult 14
//   A♣: +11 → 138;       Scholar: +20, +4 mult → 158, mult 18
//   K♠: +10 → 168
//   K♥: +10 → 178
//
// Phase 3 (held K♦, Q♦):
//   K♦: Baron (King held): mult × 1.5 → 27
//   Q♦: ShootTheMoon (Queen held): +13 mult → 40
//
// Phase 4:
//   Joker:     +4 mult     → 44
//   GlassJoker (×1.5): × 1.5 → 66
//
// Final: 178 × 66 = 11748
// =========================================================

#[test]
fn test_scenario_full_house_lvl2_five_jokers_with_held_cards() {
    let played = vec![
        card(0, Rank::Ace,  Suit::Spades),
        card(1, Rank::Ace,  Suit::Hearts),
        card(2, Rank::Ace,  Suit::Clubs),
        card(3, Rank::King, Suit::Spades),
        card(4, Rank::King, Suit::Hearts),
    ];
    // Cards still held in hand (not played)
    let held = vec![
        card(5, Rank::King,  Suit::Diamonds),
        card(6, Rank::Queen, Suit::Diamonds),
    ];

    let mut glass_j = joker(4, JokerKind::GlassJoker);
    glass_j.set_counter_f64("x_mult", 1.5);

    let jokers = vec![
        joker(0, JokerKind::Joker),
        joker(1, JokerKind::Scholar),
        joker(2, JokerKind::Baron),
        joker(3, JokerKind::ShootTheMoon),
        glass_j,
    ];
    let levels = levels_with(HandType::FullHouse, 2);
    let r = score_levels(&played, &held, &jokers, &levels);

    assert_eq!(r.hand_type, HandType::FullHouse);
    assert_eq!(r.final_chips as i64, 178,   "chips mismatch");
    assert_eq!(r.final_mult  as i64, 66,    "mult mismatch");
    assert_eq!(r.final_score as i64, 11748, "score mismatch");
}

// =========================================================
// Scenario 4: Level 2 Four of a Kind (4 Kings) —
//   Joker + AbstractJoker + Hologram(×2) + Vampire(×1.5) + Triboulet
//   Played: K♠ K♥ K♣ K♦ Q♦   (Queens are kicker, do not score)
//
// 4oaK L2: chips = 60 + 30×1 = 90, mult = 7 + 3×1 = 10
//
// Phase 1: nothing
//
// Phase 2 (4 Kings score; Queen kicker does NOT score):
//   K♠: +10 chips → 100; Triboulet (King, scored): mult × 2 →  20
//   K♥: +10       → 110;                            mult × 2 →  40
//   K♣: +10       → 120;                            mult × 2 →  80
//   K♦: +10       → 130;                            mult × 2 → 160
//
// Phase 4 (5 jokers own count for AbstractJoker):
//   Joker:          +4 mult          →  164
//   AbstractJoker:  +3 × 5 = +15 mult → 179
//   Hologram (×2.0): mult × 2.0       → 358
//   Vampire  (×1.5): mult × 1.5       → 537
//   Triboulet: nothing in main
//
// Final: 130 × 537 = 69810
// =========================================================

#[test]
fn test_scenario_four_of_a_kind_lvl2_five_jokers() {
    let played = vec![
        card(0, Rank::King,  Suit::Spades),
        card(1, Rank::King,  Suit::Hearts),
        card(2, Rank::King,  Suit::Clubs),
        card(3, Rank::King,  Suit::Diamonds),
        card(4, Rank::Queen, Suit::Diamonds), // kicker — does not score
    ];

    let mut hologram = joker(2, JokerKind::Hologram);
    hologram.set_counter_f64("x_mult", 2.0);
    let mut vampire = joker(3, JokerKind::Vampire);
    vampire.set_counter_f64("x_mult", 1.5);

    let jokers = vec![
        joker(0, JokerKind::Joker),
        joker(1, JokerKind::AbstractJoker),
        hologram,
        vampire,
        joker(4, JokerKind::Triboulet),
    ];
    let levels = levels_with(HandType::FourOfAKind, 2);
    let r = score_levels(&played, &played, &jokers, &levels);

    assert_eq!(r.hand_type, HandType::FourOfAKind);
    assert_eq!(r.final_chips as i64, 130,   "chips mismatch");
    assert_eq!(r.final_mult  as i64, 537,   "mult mismatch");
    assert_eq!(r.final_score as i64, 69810, "score mismatch");
}

// =========================================================
// Scenario 5: Level 3 Flush Five (5×A♠) —
//   Joker + Scholar + OddTodd + GlassJoker(×2) + Hologram(×2)
//
// FlushFive L3: chips = 160 + 50×2 = 260, mult = 16 + 3×2 = 22
//
// Phase 1: nothing
//
// Phase 2 (all 5 Aces score, each identical):
//   Each A♠: +11 chips; Scholar: +20 chips, +4 mult; OddTodd: +31 chips
//   Per Ace: +62 chips, +4 mult  ×5 → +310 chips, +20 mult
//   After phase 2: chips = 260+310 = 570, mult = 22+20 = 42
//
// Phase 4:
//   Joker:           +4 mult  → 46
//   GlassJoker (×2): × 2      → 92
//   Hologram   (×2): × 2      → 184
//
// Final: 570 × 184 = 104880
// =========================================================

#[test]
fn test_scenario_flush_five_lvl3_five_jokers() {
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Spades),
        card(2, Rank::Ace, Suit::Spades),
        card(3, Rank::Ace, Suit::Spades),
        card(4, Rank::Ace, Suit::Spades),
    ];

    let mut glass_j = joker(3, JokerKind::GlassJoker);
    glass_j.set_counter_f64("x_mult", 2.0);
    let mut hologram = joker(4, JokerKind::Hologram);
    hologram.set_counter_f64("x_mult", 2.0);

    let jokers = vec![
        joker(0, JokerKind::Joker),
        joker(1, JokerKind::Scholar),
        joker(2, JokerKind::OddTodd),
        glass_j,
        hologram,
    ];
    let levels = levels_with(HandType::FlushFive, 3);
    let r = score_levels(&played, &played, &jokers, &levels);

    assert_eq!(r.hand_type, HandType::FlushFive);
    assert_eq!(r.final_chips as i64, 570,    "chips mismatch");
    assert_eq!(r.final_mult  as i64, 184,    "mult mismatch");
    assert_eq!(r.final_score as i64, 104880, "score mismatch");
}

// =========================================================
// Scenario 6: Level 2 Two Pair with Glass card + polychrome joker + 4 other jokers
//   Played: A♠(Glass) A♥ K♣ K♦ 2♠   (TwoPair: both pairs score; 2♠ is kicker)
//   Jokers: MadJoker + CleverJoker + Scholar + Joker(Polychrome) + GlassJoker(×1.5)
//
// TwoPair L2: chips = 20 + 20×1 = 40, mult = 2 + 1×1 = 3
//
// Phase 1:
//   MadJoker   → +10 mult  (mult = 13)
//   CleverJoker → +80 chips (chips = 120)
//
// Phase 2 (A♠ A♥ K♣ K♦ score; 2♠ kicker does NOT score):
//   A♠ (Glass):
//     chip_value = 11 → chips = 131
//     flat_mult_bonus = 0
//     x_mult_factor (Glass) = ×2 → mult = 26
//     Scholar (Ace): +20 chips, +4 mult → chips = 151, mult = 30
//   A♥:
//     +11 chips → 162
//     Scholar: +20, +4 mult → chips = 182, mult = 34
//   K♣:
//     +10 chips → 192
//   K♦:
//     +10 chips → 202
//
// Phase 4:
//   MadJoker:   nothing in main
//   CleverJoker: nothing in main
//   Scholar:    nothing in main
//   Joker (Polychrome): +4 mult → 38; then ×1.5 (poly) → 57
//   GlassJoker (×1.5): × 1.5 → 85.5 → 85 (truncated as i64)
//
// Final: 202 × 85.5 = 17271
// =========================================================

#[test]
fn test_scenario_two_pair_lvl2_glass_card_polychrome_joker() {
    let mut ace_glass = card(0, Rank::Ace, Suit::Spades);
    ace_glass.enhancement = Enhancement::Glass;

    let played = vec![
        ace_glass,
        card(1, Rank::Ace,  Suit::Hearts),
        card(2, Rank::King, Suit::Clubs),
        card(3, Rank::King, Suit::Diamonds),
        card(4, Rank::Two,  Suit::Spades), // kicker
    ];

    let mut poly_joker = joker(3, JokerKind::Joker);
    poly_joker.edition = Edition::Polychrome;
    let mut glass_j = joker(4, JokerKind::GlassJoker);
    glass_j.set_counter_f64("x_mult", 1.5);

    let jokers = vec![
        joker(0, JokerKind::MadJoker),
        joker(1, JokerKind::CleverJoker),
        joker(2, JokerKind::Scholar),
        poly_joker,
        glass_j,
    ];
    let levels = levels_with(HandType::TwoPair, 2);
    let r = score_levels(&played, &played, &jokers, &levels);

    assert_eq!(r.hand_type, HandType::TwoPair);
    assert_eq!(r.final_chips as i64, 202, "chips mismatch");
    // mult: 34 (after phase2) + 4 (Joker) = 38, ×1.5 (Polychrome) = 57, ×1.5 (GlassJoker) = 85.5
    assert!((r.final_mult - 85.5).abs() < 0.01, "mult mismatch: got {}", r.final_mult);
    assert!((r.final_score - 17271.0).abs() < 1.0, "score mismatch: got {}", r.final_score);
}

// =========================================================
// Scenario 7: Level 2 Three of a Kind (three 6s) —
//   ZanyJoker + WilyJoker + EvenSteven + OnyxAgate + Stuntman
//   Played: 6♣ 6♦ 6♠ K♥ 2♥  (6s score; K♥ and 2♥ are kickers)
//
// ThreeOfAKind L2: chips = 30 + 20×1 = 50, mult = 3 + 2×1 = 5
//
// Phase 1:
//   ZanyJoker  (ThreeOfAKind) → +12 mult   (mult = 17)
//   WilyJoker  (ThreeOfAKind) → +100 chips (chips = 150)
//
// Phase 2 (6♣ 6♦ 6♠ score; K♥ 2♥ do not):
//   6♣: +6 chips → 156; EvenSteven (6 even): +4 mult → 21; OnyxAgate (Club): +7 mult → 28
//   6♦: +6 chips → 162; EvenSteven (6 even): +4 mult → 32
//   6♠: +6 chips → 168; EvenSteven (6 even): +4 mult → 36
//
// Phase 4:
//   Stuntman: +250 chips → 418
//
// Final: 418 × 36 = 15048
// =========================================================

#[test]
fn test_scenario_three_of_a_kind_lvl2_five_jokers() {
    let played = vec![
        card(0, Rank::Six,  Suit::Clubs),
        card(1, Rank::Six,  Suit::Diamonds),
        card(2, Rank::Six,  Suit::Spades),
        card(3, Rank::King, Suit::Hearts), // kicker
        card(4, Rank::Two,  Suit::Hearts), // kicker
    ];
    let jokers = vec![
        joker(0, JokerKind::ZanyJoker),
        joker(1, JokerKind::WilyJoker),
        joker(2, JokerKind::EvenSteven),
        joker(3, JokerKind::OnyxAgate),
        joker(4, JokerKind::Stuntman),
    ];
    let levels = levels_with(HandType::ThreeOfAKind, 2);
    let r = score_levels(&played, &played, &jokers, &levels);

    assert_eq!(r.hand_type, HandType::ThreeOfAKind);
    assert_eq!(r.final_chips as i64, 418,   "chips mismatch");
    assert_eq!(r.final_mult  as i64, 36,    "mult mismatch");
    assert_eq!(r.final_score as i64, 15048, "score mismatch");
}

// =========================================================
// Scenario 8: Level 3 Pair (2 cards played) —
//   JollyJoker + SlyJoker + HalfJoker + LustyJoker + Fibonacci
//   Played: A♥ A♦  (only 2 cards → HalfJoker fires)
//
// Pair L3: chips = 10 + 15×2 = 40, mult = 2 + 1×2 = 4
//
// Phase 1:
//   JollyJoker (Pair) → +8 mult   (mult = 12)
//   SlyJoker   (Pair) → +50 chips (chips = 90)
//
// Phase 2 (A♥, A♦ both score):
//   A♥: +11 chips → 101; LustyJoker (Heart): +3 mult → 15; Fibonacci (Ace): +8 mult → 23
//   A♦: +11 chips → 112; Fibonacci  (Ace):   +8 mult → 31
//
// Phase 4:
//   HalfJoker (played.len() = 2 ≤ 3): +20 mult → 51
//
// Final: 112 × 51 = 5712
// =========================================================

#[test]
fn test_scenario_pair_lvl3_halfjoker_lusty_fibonacci() {
    let played = vec![
        card(0, Rank::Ace, Suit::Hearts),
        card(1, Rank::Ace, Suit::Diamonds),
    ];
    let jokers = vec![
        joker(0, JokerKind::JollyJoker),
        joker(1, JokerKind::SlyJoker),
        joker(2, JokerKind::HalfJoker),
        joker(3, JokerKind::LustyJoker),
        joker(4, JokerKind::Fibonacci),
    ];
    let levels = levels_with(HandType::Pair, 3);
    let r = score_levels(&played, &played, &jokers, &levels);

    assert_eq!(r.hand_type, HandType::Pair);
    assert_eq!(r.final_chips as i64, 112,  "chips mismatch");
    assert_eq!(r.final_mult  as i64, 51,   "mult mismatch");
    assert_eq!(r.final_score as i64, 5712, "score mismatch");
}

// =========================================================
// Scenario 9: Level 2 Flush (all Clubs) —
//   GluttonousJoker + Blackboard + Banner + Bull + Bootstraps
//   Played: 2♣ 5♣ 8♣ J♣ K♣  (all Clubs → Blackboard fires)
//   Context: hands_remaining = 2, money = $15
//
// Flush L2: chips = 35 + 15×1 = 50, mult = 4 + 2×1 = 6
//
// Phase 1: nothing
//
// Phase 2 (all 5 score):
//   2♣: +2 chips → 52; GluttonousJoker (Club): +3 mult →  9
//   5♣: +5 chips → 57; GluttonousJoker:         +3 mult → 12
//   8♣: +8 chips → 65; GluttonousJoker:         +3 mult → 15
//   J♣: +10      → 75; GluttonousJoker:         +3 mult → 18
//   K♣: +10      → 85; GluttonousJoker:         +3 mult → 21
//
// Phase 4:
//   GluttonousJoker: no main effect
//   Blackboard (all hand = Clubs): × 3 mult → 63
//   Banner (hands_remaining = 2): +2×30 = +60 chips → 145
//   Bull   (money = $15):         +2×15 = +30 chips → 175
//   Bootstraps (money = $15, $15/$5 = 3): +2×3 = +6 mult → 69
//
// Final: 175 × 69 = 12075
// =========================================================

#[test]
fn test_scenario_flush_lvl2_blackboard_money_jokers() {
    let played = vec![
        card(0, Rank::Two,  Suit::Clubs),
        card(1, Rank::Five, Suit::Clubs),
        card(2, Rank::Eight,Suit::Clubs),
        card(3, Rank::Jack, Suit::Clubs),
        card(4, Rank::King, Suit::Clubs),
    ];
    let jokers = vec![
        joker(0, JokerKind::GluttonousJoker),
        joker(1, JokerKind::Blackboard),
        joker(2, JokerKind::Banner),
        joker(3, JokerKind::Bull),
        joker(4, JokerKind::Bootstraps),
    ];
    let levels = levels_with(HandType::Flush, 2);
    let r = score_hand(
        &played, &played, &jokers, &levels,
        2,   // hands_remaining
        3,   // discards_remaining
        15,  // money
        40,  // deck_remaining
        52,  // total_deck
        None,
        5,   // joker_slot_count
        0,   // tarot_cards_used
    );

    assert_eq!(r.hand_type, HandType::Flush);
    assert_eq!(r.final_chips as i64, 175,   "chips mismatch");
    assert_eq!(r.final_mult  as i64, 69,    "mult mismatch");
    assert_eq!(r.final_score as i64, 12075, "score mismatch");
}

// =========================================================
// Scenario 10: Level 2 High Card (A♦ only scores) —
//   BlueJoker + FortuneTeller + GreedyJoker + Runner(150) + IceCream(100)
//   Context: deck_remaining = 30, tarot_cards_used = 8
//
// HighCard L2: chips = 5 + 10×1 = 15, mult = 1 + 1×1 = 2
//
// Phase 1: nothing
//
// Phase 2 (only A♦ scores):
//   A♦: +11 chips → 26; GreedyJoker (Diamond): +3 mult → 5
//
// Phase 4:
//   BlueJoker     (deck = 30): +2×30 = +60 chips → 86
//   FortuneTeller (tarots = 8): +8 mult → 13
//   GreedyJoker:  no main effect
//   Runner        (counter chips = 150): +150 chips → 236
//   IceCream      (counter chips = 100): +100 chips → 336
//
// Final: 336 × 13 = 4368
// =========================================================

#[test]
fn test_scenario_high_card_lvl2_deck_and_economy_jokers() {
    let played = vec![
        card(0, Rank::Ace,  Suit::Diamonds),
        card(1, Rank::Four, Suit::Clubs),
        card(2, Rank::Nine, Suit::Hearts),
        card(3, Rank::Jack, Suit::Spades),
        card(4, Rank::Two,  Suit::Hearts),
    ];

    let mut runner = joker(3, JokerKind::Runner);
    runner.set_counter_i64("chips", 150);
    let mut ice_cream = joker(4, JokerKind::IceCream);
    ice_cream.set_counter_i64("chips", 100);

    let jokers = vec![
        joker(0, JokerKind::BlueJoker),
        joker(1, JokerKind::FortuneTeller),
        joker(2, JokerKind::GreedyJoker),
        runner,
        ice_cream,
    ];
    let levels = levels_with(HandType::HighCard, 2);
    let r = score_hand(
        &played, &played, &jokers, &levels,
        3,   // hands_remaining
        3,   // discards_remaining
        0,   // money
        30,  // deck_remaining
        52,  // total_deck
        None,
        5,   // joker_slot_count
        8,   // tarot_cards_used
    );

    assert_eq!(r.hand_type, HandType::HighCard);
    assert_eq!(r.final_chips as i64, 336,  "chips mismatch");
    assert_eq!(r.final_mult  as i64, 13,   "mult mismatch");
    assert_eq!(r.final_score as i64, 4368, "score mismatch");
}

// =========================================================
// Scenario 11: Level 2 Straight (mixed suits) —
//   WalkieTalkie + Swashbuckler(mult=12) + SpareTrousers(mult=8) + TheOrder(×3) + Erosion
//   Played: 4♣ 5♦ 6♥ 7♠ 8♦
//   Context: deck_remaining = 42, total_deck = 52  (10 cards below full → Erosion +40 mult)
//
// Straight L2: chips = 30 + 30×1 = 60, mult = 4 + 3×1 = 7
//
// Phase 1: nothing
//
// Phase 2 (all 5 score):
//   4♣: +4 chips → 64; WalkieTalkie (rank 4): +10 chips → 74, +4 mult → 11
//   5♦: +5  → 79
//   6♥: +6  → 85
//   7♠: +7  → 92
//   8♦: +8  → 100
//
// Phase 4:
//   WalkieTalkie:       no main effect
//   Swashbuckler (mult counter = 12): +12 mult → 23
//   SpareTrousers (mult counter =  8): +8 mult → 31
//   TheOrder   (Straight): ×3 mult → 93
//   Erosion    (total 52 − remaining 42 = 10 below): +4×10 = +40 mult → 133
//
// Final: 100 × 133 = 13300
// =========================================================

#[test]
fn test_scenario_straight_lvl2_walkietalkie_order_erosion() {
    let played = vec![
        card(0, Rank::Four, Suit::Clubs),
        card(1, Rank::Five, Suit::Diamonds),
        card(2, Rank::Six,  Suit::Hearts),
        card(3, Rank::Seven,Suit::Spades),
        card(4, Rank::Eight,Suit::Diamonds),
    ];

    let mut swash = joker(1, JokerKind::Swashbuckler);
    swash.set_counter_i64("mult", 12);
    let mut spare = joker(2, JokerKind::SpareTrousers);
    spare.set_counter_i64("mult", 8);

    let jokers = vec![
        joker(0, JokerKind::WalkieTalkie),
        swash,
        spare,
        joker(3, JokerKind::TheOrder),
        joker(4, JokerKind::Erosion),
    ];
    let levels = levels_with(HandType::Straight, 2);
    let r = score_hand(
        &played, &played, &jokers, &levels,
        3,   // hands_remaining
        3,   // discards_remaining
        0,   // money
        42,  // deck_remaining
        52,  // total_deck
        None,
        5,   // joker_slot_count
        0,   // tarot_cards_used
    );

    assert_eq!(r.hand_type, HandType::Straight);
    assert_eq!(r.final_chips as i64, 100,   "chips mismatch");
    assert_eq!(r.final_mult  as i64, 133,   "mult mismatch");
    assert_eq!(r.final_score as i64, 13300, "score mismatch");
}

// =========================================================
// Scenario 12: Level 3 Flush House — 9 jokers (3 Negative edition) — LAST HAND
//
// Game context:
//   Vouchers:  Blank + Antimatter (+2 joker slots) plus 3 Negative jokers (+3 slots) → 10 slots
//   Money:     $0 (all spent)
//   Hands:     0 remaining — this IS the final hand of the round
//
// Jokers (9 total):
//   1. Acrobat              — ×3 mult on last hand (hands_remaining = 0)
//   2. Canio          (×4)  — counter x_mult = 4.0
//   3. Campfire       (×2)  — counter x_mult = 2.0
//   4. Yorick         (×1.5)— counter x_mult = 1.5
//   5. CeremonialDagger(+20)— counter mult = 20
//   6. FlashCard      (+16) — counter mult = 16
//   7. RideTheBus [Neg](+10)— counter mult = 10  (Negative edition — grants +1 joker slot)
//   8. GreenJoker  [Neg](+12)— counter mult = 12 (Negative edition — grants +1 joker slot)
//   9. Popcorn     [Neg](+6) — counter mult = 6  (Negative edition — depleted; grants +1 slot)
//
// Flush House L3: chips = 140 + 40×2 = 220, mult = 14 + 4×2 = 22
//
// Phase 1: nothing
//
// Phase 2 (A♠×3 + K♠×2, all 5 score):
//   3 × Ace  (+11 each): +33 chips → 253
//   2 × King (+10 each): +20 chips → 273
//
// Phase 4 (joker editions after each main effect; Negative = +0 chips/mult/xmult):
//   Acrobat  (hands=0):      ×3     → mult = 66
//   Canio    (xmult=4.0):    ×4     → mult = 264
//   Campfire (xmult=2.0):    ×2     → mult = 528
//   Yorick   (xmult=1.5):    ×1.5   → mult = 792
//   CeremonialDagger (+20):  +20    → mult = 812
//   FlashCard        (+16):  +16    → mult = 828
//   RideTheBus [Neg] (+10):  +10    → mult = 838  ; Negative edition → no bonus
//   GreenJoker [Neg] (+12):  +12    → mult = 850  ; Negative edition → no bonus
//   Popcorn    [Neg] (+6):   +6     → mult = 856  ; Negative edition → no bonus
//
// Final: 273 × 856 = 233688
// =========================================================

#[test]
fn test_scenario_flushhouse_lvl3_nine_jokers_last_hand() {
    let played = vec![
        card(0, Rank::Ace,  Suit::Spades),
        card(1, Rank::Ace,  Suit::Spades),
        card(2, Rank::Ace,  Suit::Spades),
        card(3, Rank::King, Suit::Spades),
        card(4, Rank::King, Suit::Spades),
    ];

    let mut canio = joker(1, JokerKind::Canio);
    canio.set_counter_f64("x_mult", 4.0);

    let mut campfire = joker(2, JokerKind::Campfire);
    campfire.set_counter_f64("x_mult", 2.0);

    let mut yorick = joker(3, JokerKind::Yorick);
    yorick.set_counter_f64("x_mult", 1.5);

    let mut ceremonial = joker(4, JokerKind::CeremonialDagger);
    ceremonial.set_counter_i64("mult", 20);

    let mut flash = joker(5, JokerKind::FlashCard);
    flash.set_counter_i64("mult", 16);

    // Negative-edition jokers — each grants +1 joker slot (no scoring bonus in this engine)
    let mut ride = joker(6, JokerKind::RideTheBus);
    ride.edition = Edition::Negative;
    ride.set_counter_i64("mult", 10);

    let mut green = joker(7, JokerKind::GreenJoker);
    green.edition = Edition::Negative;
    green.set_counter_i64("mult", 12);

    let mut popcorn = joker(8, JokerKind::Popcorn);
    popcorn.edition = Edition::Negative;
    popcorn.set_counter_i64("mult", 6); // depleted from base 20

    let jokers = vec![
        joker(0, JokerKind::Acrobat),
        canio, campfire, yorick, ceremonial, flash, ride, green, popcorn,
    ];

    let levels = levels_with(HandType::FlushHouse, 3);
    let r = score_hand(
        &played, &played, &jokers, &levels,
        0,   // hands_remaining — final hand of the round, Acrobat fires
        3,   // discards_remaining
        0,   // money
        52,  // deck_remaining
        52,  // total_deck
        None,
        9,   // joker_slot_count (Blank + Antimatter vouchers + 3 Negative jokers)
        0,   // tarot_cards_used
    );

    assert_eq!(r.hand_type, HandType::FlushHouse);
    assert_eq!(r.final_chips as i64, 273,    "chips mismatch");
    assert_eq!(r.final_mult  as i64, 856,    "mult mismatch");
    assert_eq!(r.final_score as i64, 233688, "score mismatch");
}

// =========================================================
// Scenario 13: Level 3 Four of a Kind (4 Queens) — 8 jokers (2 Negative edition)
//
// Game context:
//   Vouchers:  Blank (+1 joker slot) plus 2 Negative jokers (+2 slots) → 8 slots
//   Money:     $0
//   Played:    Q♠ Q♥ Q♣ Q♦ 2♠  (four Queens score; 2♠ is the kicker)
//
// Jokers (8 total):
//   1. TheFamily              — ×4 xmult for Four of a Kind
//   2. Photograph             — ×2 xmult on FIRST face card scored
//   3. ScaryFace              — +30 chips per face card scored
//   4. SmileyFace             — +5 mult per face card scored
//   5. Castle          (+100) — counter chips = 100
//   6. WeeJoker        (+80)  — counter chips = 80
//   7. LuckyCat  [Neg] (×2.5) — counter x_mult = 2.5 (Negative edition → +1 joker slot)
//   8. Obelisk   [Neg] (×2.0) — counter x_mult = 2.0 (Negative edition → +1 joker slot)
//
// Four of a Kind L3: chips = 60 + 30×2 = 120, mult = 7 + 3×2 = 13
//
// Phase 1: nothing
//
// Phase 2 (Q♠ Q♥ Q♣ Q♦ score; 2♠ kicker does NOT score):
//   Q♠ (first face):
//     +10 chips → 130
//     Photograph (first face):  ×2 mult → 26
//     ScaryFace:                +30 chips → 160; SmileyFace: +5 mult → 31
//   Q♥: +10 → 170; ScaryFace: +30 → 200; SmileyFace: +5 → 36
//   Q♣: +10 → 210; ScaryFace: +30 → 240; SmileyFace: +5 → 41
//   Q♦: +10 → 250; ScaryFace: +30 → 280; SmileyFace: +5 → 46
//
// Phase 4:
//   TheFamily  (FourOfAKind): ×4     → mult = 184
//   Castle     (+100 chips):  +100   → chips = 380
//   WeeJoker   (+80  chips):  +80    → chips = 460
//   LuckyCat   [Neg] (×2.5):  ×2.5   → mult = 460  ; Negative edition → no bonus
//   Obelisk    [Neg] (×2.0):  ×2.0   → mult = 920  ; Negative edition → no bonus
//
// Final: 460 × 920 = 423200
// =========================================================

#[test]
fn test_scenario_four_of_a_kind_lvl3_eight_jokers_face_avalanche() {
    let played = vec![
        card(0, Rank::Queen, Suit::Spades),
        card(1, Rank::Queen, Suit::Hearts),
        card(2, Rank::Queen, Suit::Clubs),
        card(3, Rank::Queen, Suit::Diamonds),
        card(4, Rank::Two,   Suit::Spades), // kicker
    ];

    let mut castle = joker(4, JokerKind::Castle);
    castle.set_counter_i64("chips", 100);

    let mut wee = joker(5, JokerKind::WeeJoker);
    wee.set_counter_i64("chips", 80);

    let mut lucky = joker(6, JokerKind::LuckyCat);
    lucky.edition = Edition::Negative;
    lucky.set_counter_f64("x_mult", 2.5);

    let mut obelisk = joker(7, JokerKind::Obelisk);
    obelisk.edition = Edition::Negative;
    obelisk.set_counter_f64("x_mult", 2.0);

    let jokers = vec![
        joker(0, JokerKind::TheFamily),
        joker(1, JokerKind::Photograph),
        joker(2, JokerKind::ScaryFace),
        joker(3, JokerKind::SmileyFace),
        castle, wee, lucky, obelisk,
    ];

    let levels = levels_with(HandType::FourOfAKind, 3);
    let r = score_hand(
        &played, &played, &jokers, &levels,
        3,   // hands_remaining
        3,   // discards_remaining
        0,   // money
        40,  // deck_remaining
        52,  // total_deck
        None,
        8,   // joker_slot_count (Blank voucher + 2 Negative jokers)
        0,   // tarot_cards_used
    );

    assert_eq!(r.hand_type, HandType::FourOfAKind);
    assert_eq!(r.final_chips as i64, 460,    "chips mismatch");
    assert_eq!(r.final_mult  as i64, 920,    "mult mismatch");
    assert_eq!(r.final_score as i64, 423200, "score mismatch");
}

// =========================================================
// Scenario 14: Level 3 Five of a Kind (5×8, mixed suits) — 9 jokers (3 Negative edition)
//   All discards spent → MysticSummit fires; 4 blind skips → Throwback ×2; CardSharp ×3
//
// Game context:
//   Vouchers:  Blank + Antimatter (+2 slots) plus 3 Negative jokers (+3 slots) → 10 slots
//   Money:     $0 (all spent on upgrades)
//   Discards:  0 remaining (Wasteful + Recyclomancy used — MysticSummit fires)
//   Blinds:    4 skipped this run (Throwback scales up)
//
// Jokers (9 total):
//   1. Hiker                     — +5 chips per scoring card played
//   2. SquareJoker        (+120) — counter chips = 120
//   3. Madness     [Neg]  (×2.5) — counter x_mult = 2.5 (Negative edition → +1 slot)
//   4. Throwback   [Neg]  (×2.0) — 4 blind skips: 1+0.25×4=2.0 (Negative edition → +1 slot)
//   5. HitTheRoad  [Neg]  (×1.5) — counter x_mult = 1.5 (Negative edition → +1 slot)
//   6. Constellation      (×3.0) — counter x_mult = 3.0
//   7. Ramen              (×2.0) — default counter x_mult = 2.0
//   8. CardSharp          (×3.0) — ×3 because FiveOfAKind not yet played this round
//   9. MysticSummit       (+15)  — +15 mult when discards_remaining = 0
//
// Five of a Kind L3: chips = 120 + 35×2 = 190, mult = 12 + 3×2 = 18
//
// Phase 1: nothing
//
// Phase 2 (all 5×8 score, mixed suits):
//   Each 8♥: +8 chips (rank) + 5 chips (Hiker) = +13 per card × 5 = +65
//   After Phase 2: chips = 255, mult = 18
//
// Phase 4 (x_mult applied immediately per joker; Negative edition → no extra bonus):
//   Hiker:          no main effect
//   SquareJoker:    +120 chips      → chips = 375
//   Madness [Neg]:  ×2.5            → mult = 45   ; edition → no bonus
//   Throwback [Neg]:×2.0            → mult = 90   ; edition → no bonus
//   HitTheRoad [Neg]:×1.5           → mult = 135  ; edition → no bonus
//   Constellation:  ×3.0            → mult = 405
//   Ramen:          ×2.0            → mult = 810
//   CardSharp:      ×3.0            → mult = 2430
//   MysticSummit (discards=0): +15  → mult = 2445
//
// Final: 375 × 2445 = 916875
// =========================================================

#[test]
fn test_scenario_five_of_a_kind_lvl3_nine_jokers_zero_discards() {
    let played = vec![
        card(0, Rank::Eight, Suit::Spades),
        card(1, Rank::Eight, Suit::Hearts),
        card(2, Rank::Eight, Suit::Clubs),
        card(3, Rank::Eight, Suit::Diamonds),
        card(4, Rank::Eight, Suit::Spades),
    ];

    let mut square = joker(1, JokerKind::SquareJoker);
    square.set_counter_i64("chips", 120);

    let mut madness = joker(2, JokerKind::Madness);
    madness.edition = Edition::Negative;
    madness.set_counter_f64("x_mult", 2.5);

    let mut throwback = joker(3, JokerKind::Throwback);
    throwback.edition = Edition::Negative;
    throwback.set_counter_i64("skips", 4); // 1 + 0.25×4 = ×2.0

    let mut hitroad = joker(4, JokerKind::HitTheRoad);
    hitroad.edition = Edition::Negative;
    hitroad.set_counter_f64("x_mult", 1.5);

    let mut constellation = joker(5, JokerKind::Constellation);
    constellation.set_counter_f64("x_mult", 3.0);

    // Ramen starts at x2.0 by default — no explicit set needed
    let ramen = joker(6, JokerKind::Ramen);

    // CardSharp fires ×3 because FiveOfAKind played_this_round = 0 (never played this round)
    let cardsharp = joker(7, JokerKind::CardSharp);

    let jokers = vec![
        joker(0, JokerKind::Hiker),
        square, madness, throwback, hitroad, constellation, ramen, cardsharp,
        joker(8, JokerKind::MysticSummit),
    ];

    let levels = levels_with(HandType::FiveOfAKind, 3);
    let r = score_hand(
        &played, &played, &jokers, &levels,
        3,   // hands_remaining
        0,   // discards_remaining — all used (Wasteful + Recyclomancy vouchers), MysticSummit fires
        0,   // money
        40,  // deck_remaining
        52,  // total_deck
        None,
        9,   // joker_slot_count (Blank + Antimatter + 3 Negative jokers)
        0,   // tarot_cards_used
    );

    assert_eq!(r.hand_type, HandType::FiveOfAKind);
    assert_eq!(r.final_chips as i64, 375,    "chips mismatch");
    assert_eq!(r.final_mult  as i64, 2445,   "mult mismatch");
    assert_eq!(r.final_score as i64, 916875, "score mismatch");
}
