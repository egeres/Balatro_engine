/// Tests for common jokers: suit-based, hand-type bonuses, per-card bonuses.

use super::*;

// =========================================================
// Basic joker
// =========================================================

#[test]
fn test_basic_joker_adds_4_mult() {
    // High Card Ace + Joker: chips=16, mult=1+4=5 → 80
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::Joker)];
    let r = score(&played, &played, &jokers);
    assert_eq!(r.final_score as i64, 80);
}

#[test]
fn test_inactive_joker_has_no_effect() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut inactive = joker(0, JokerKind::Joker);
    inactive.active = false;
    let r = score(&played, &played, &[inactive]);
    // No +4 mult → chips=16, mult=1 → 16
    assert_eq!(r.final_score as i64, 16);
}

// =========================================================
// Suit jokers (per scoring card)
// =========================================================

#[test]
fn test_greedy_joker_fires_on_diamonds() {
    // GreedyJoker: +3 mult per Diamond card scored
    let played = vec![
        card(0, Rank::Ace, Suit::Diamonds),
        card(1, Rank::Ace, Suit::Spades),
    ];
    let jokers = vec![joker(0, JokerKind::GreedyJoker)];
    let r_with = score(&played, &played, &jokers);
    let r_without = score(&played, &played, &[]);
    // One Diamond → +3 mult → diff = 3 * chips
    let chips = r_without.final_chips;
    let expected_diff = chips * 3.0;
    assert!((r_with.final_score - r_without.final_score - expected_diff).abs() < 0.5);
}

#[test]
fn test_lusty_joker_fires_on_hearts() {
    // +3 mult per Heart in scoring cards
    let played = vec![card(0, Rank::Ace, Suit::Hearts)];
    let r = score(&played, &played, &[joker(0, JokerKind::LustyJoker)]);
    // HC: chips=16, mult=1+3=4 → 64
    assert_eq!(r.final_score as i64, 64);
}

#[test]
fn test_wrathful_joker_fires_on_spades() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::WrathfulJoker)]);
    // HC: chips=16, mult=1+3=4 → 64
    assert_eq!(r.final_score as i64, 64);
}

#[test]
fn test_gluttonous_joker_fires_on_clubs() {
    let played = vec![card(0, Rank::Ace, Suit::Clubs)];
    let r = score(&played, &played, &[joker(0, JokerKind::GluttonousJoker)]);
    assert_eq!(r.final_score as i64, 64);
}

#[test]
fn test_suit_joker_does_not_fire_on_wrong_suit() {
    // LustyJoker (Hearts) should not fire on Spades
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::LustyJoker)]);
    assert_eq!(r.final_score as i64, 16);
}

// =========================================================
// Rank jokers (per scoring card)
// =========================================================

#[test]
fn test_scholar_fires_on_ace() {
    // Scholar: +20 chips and +4 mult per Ace scored
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Scholar)]);
    // chips=16+20=36, mult=1+4=5 → 180
    assert_eq!(r.final_score as i64, 180);
}

#[test]
fn test_even_steven_fires_on_even_ranks() {
    // EvenSteven: +4 mult per even-ranked scoring card (2,4,6,8,10)
    let played = vec![card(0, Rank::Ten, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::EvenSteven)]);
    // HC: chips=5+10=15, mult=1+4=5 → 75
    assert_eq!(r.final_score as i64, 75);
}

#[test]
fn test_even_steven_does_not_fire_on_odd_rank() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::EvenSteven)]);
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_odd_todd_fires_on_odd_ranks() {
    // OddTodd: +31 chips per odd-ranked scoring card (A,3,5,7,9)
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::OddTodd)]);
    // HC: chips=5+11+31=47, mult=1 → 47
    assert_eq!(r.final_score as i64, 47);
}

#[test]
fn test_odd_todd_does_not_fire_on_even_rank() {
    let played = vec![card(0, Rank::Ten, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::OddTodd)]);
    assert_eq!(r.final_score as i64, 15);
}

#[test]
fn test_fibonacci_fires_on_fibonacci_ranks() {
    // Fibonacci: +8 mult per Fibonacci-rank scored card (A,2,3,5,8)
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Fibonacci)]);
    // HC: chips=16, mult=1+8=9 → 144
    assert_eq!(r.final_score as i64, 144);
}

#[test]
fn test_fibonacci_does_not_fire_on_non_fibonacci_rank() {
    let played = vec![card(0, Rank::Four, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Fibonacci)]);
    // HC: chips=5+4=9, mult=1 → 9
    assert_eq!(r.final_score as i64, 9);
}

// =========================================================
// Hand-type flat mult jokers (before scoring cards)
// =========================================================

#[test]
fn test_jolly_joker_fires_on_pair() {
    let played = vec![card(0, Rank::Two, Suit::Spades), card(1, Rank::Two, Suit::Hearts)];
    let r = score(&played, &played, &[joker(0, JokerKind::JollyJoker)]);
    // Pair base 10+2+2=14 chips, mult=2+8=10 → 140
    assert_eq!(r.hand_type, HandType::Pair);
    assert_eq!(r.final_score as i64, 140);
}

#[test]
fn test_jolly_joker_does_not_fire_on_high_card() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::JollyJoker)]);
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_zany_joker_fires_on_three_of_a_kind() {
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Two, Suit::Clubs),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::ZanyJoker)]);
    // 3oaK base 30+6=36 chips, mult=3+12=15 → 540
    assert_eq!(r.hand_type, HandType::ThreeOfAKind);
    assert_eq!(r.final_score as i64, 540);
}

#[test]
fn test_mad_joker_fires_on_two_pair() {
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
        card(3, Rank::Three, Suit::Diamonds),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::MadJoker)]);
    // TwoPair base 20+2+2+3+3=30 chips, mult=2+10=12 → 360
    assert_eq!(r.hand_type, HandType::TwoPair);
    assert_eq!(r.final_score as i64, 360);
}

#[test]
fn test_crazy_joker_fires_on_straight() {
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Diamonds),
        card(3, Rank::Four, Suit::Clubs),
        card(4, Rank::Five, Suit::Spades),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::CrazyJoker)]);
    // Straight base 30+11+2+3+4+5=55 chips, mult=4+12=16 → 880
    assert_eq!(r.hand_type, HandType::Straight);
    assert_eq!(r.final_score as i64, 880);
}

#[test]
fn test_droll_joker_fires_on_flush() {
    // Use non-consecutive Spades (not a StraightFlush) for a pure Flush
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Four, Suit::Spades),
        card(2, Rank::Seven, Suit::Spades),
        card(3, Rank::Nine, Suit::Spades),
        card(4, Rank::Jack, Suit::Spades),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::DrollJoker)]);
    // Flush base 35+2+4+7+9+10=67 chips, mult=4+10=14 → 938
    assert_eq!(r.hand_type, HandType::Flush);
    assert_eq!(r.final_score as i64, 938);
}

// =========================================================
// Hand-type chip jokers
// =========================================================

#[test]
fn test_sly_joker_fires_on_pair() {
    let played = vec![card(0, Rank::Two, Suit::Spades), card(1, Rank::Two, Suit::Hearts)];
    let r = score(&played, &played, &[joker(0, JokerKind::SlyJoker)]);
    // Pair base 10+2+2+50=64 chips, mult=2 → 128
    assert_eq!(r.final_score as i64, 128);
}

#[test]
fn test_wily_joker_fires_on_three_of_a_kind() {
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Two, Suit::Clubs),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::WilyJoker)]);
    // 3oaK base 30+6+100=136 chips, mult=3 → 408
    assert_eq!(r.final_score as i64, 408);
}

#[test]
fn test_clever_joker_fires_on_two_pair() {
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
        card(3, Rank::Three, Suit::Diamonds),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::CleverJoker)]);
    // TwoPair base 20+2+2+3+3+80=110 chips, mult=2 → 220
    assert_eq!(r.final_score as i64, 220);
}

#[test]
fn test_devious_joker_fires_on_straight() {
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Diamonds),
        card(3, Rank::Four, Suit::Clubs),
        card(4, Rank::Five, Suit::Spades),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::DeviousJoker)]);
    // Straight base 30+55+100=155 chips, mult=4 → 620
    assert_eq!(r.final_score as i64, 620);
}

#[test]
fn test_crafty_joker_fires_on_flush() {
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Four, Suit::Spades),
        card(2, Rank::Seven, Suit::Spades),
        card(3, Rank::Nine, Suit::Spades),
        card(4, Rank::Jack, Suit::Spades),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::CraftyJoker)]);
    // Flush base 35+2+4+7+9+10+80=147 chips, mult=4 → 588
    assert_eq!(r.final_score as i64, 588);
}

// =========================================================
// Per-card jokers
// =========================================================

#[test]
fn test_scary_face_adds_chips_per_face_card() {
    let played = vec![card(0, Rank::Jack, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::ScaryFace)]);
    // HC: 5+10+30=45 chips, mult=1 → 45
    assert_eq!(r.final_score as i64, 45);
}

#[test]
fn test_scary_face_does_not_fire_on_non_face() {
    let played = vec![card(0, Rank::Two, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::ScaryFace)]);
    assert_eq!(r.final_score as i64, 7);
}

#[test]
fn test_walkie_talkie_fires_on_ten() {
    let played = vec![card(0, Rank::Ten, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::WalkieTalkie)]);
    // HC: 5+10+10=25 chips, mult=1+4=5 → 125
    assert_eq!(r.final_score as i64, 125);
}

#[test]
fn test_walkie_talkie_fires_on_four() {
    let played = vec![card(0, Rank::Four, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::WalkieTalkie)]);
    // HC: 5+4+10=19 chips, mult=1+4=5 → 95
    assert_eq!(r.final_score as i64, 95);
}

#[test]
fn test_walkie_talkie_does_not_fire_on_other_ranks() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::WalkieTalkie)]);
    // HC: 5+11=16, mult=1 → 16
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_smiley_face_fires_on_face_cards() {
    let played = vec![card(0, Rank::Queen, Suit::Hearts)];
    let r = score(&played, &played, &[joker(0, JokerKind::SmileyFace)]);
    // HC: 5+10=15 chips, mult=1+5=6 → 90
    assert_eq!(r.final_score as i64, 90);
}

#[test]
fn test_photograph_fires_on_first_face_card_only() {
    // Pair of Jacks: first Jack → x2 mult; second Jack → no extra x2
    let played = vec![
        card(0, Rank::Jack, Suit::Spades),
        card(1, Rank::Jack, Suit::Hearts),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::Photograph)]);
    // Pair: base 10+10+10=30 chips, mult=2. First Jack: *2 → mult=4. Score=30*4=120
    assert_eq!(r.hand_type, HandType::Pair);
    assert_eq!(r.final_score as i64, 120);
}

#[test]
fn test_hiker_adds_chips_per_scoring_card() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Hiker)]);
    // HC: 5+11+5=21 chips, mult=1 → 21
    assert_eq!(r.final_score as i64, 21);
}

#[test]
fn test_stone_joker_fires_on_stone_card() {
    let mut stone = card(0, Rank::Two, Suit::Spades);
    stone.enhancement = Enhancement::Stone;
    let played = vec![stone];
    // Pass empty hand to avoid double-counting the stone card in stone_count_in_deck
    let r = score(&played, &[], &[joker(0, JokerKind::StoneJoker)]);
    // HC base=5, Stone card chip_bonus=50, StoneJoker(1 stone in deck)=+25 → chips=80, mult=1 → 80
    assert_eq!(r.final_score as i64, 80);
}

#[test]
fn test_arrowhead_fires_on_spades() {
    let played = vec![card(0, Rank::Two, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Arrowhead)]);
    // HC: 5+2+50=57 chips, mult=1 → 57
    assert_eq!(r.final_score as i64, 57);
}

#[test]
fn test_onyx_agate_fires_on_clubs() {
    let played = vec![card(0, Rank::Two, Suit::Clubs)];
    let r = score(&played, &played, &[joker(0, JokerKind::OnyxAgate)]);
    // HC: 5+2=7 chips, mult=1+7=8 → 56
    assert_eq!(r.final_score as i64, 56);
}

#[test]
fn test_bloodstone_fires_on_hearts() {
    // Bloodstone is pre-rolled in round.rs; test with extra_x_mult=1.5 pre-set on the card
    let mut hearts_card = card(0, Rank::Two, Suit::Hearts);
    hearts_card.extra_x_mult = 1.5;
    let played = vec![hearts_card];
    let r = score(&played, &played, &[joker(0, JokerKind::Bloodstone)]);
    // HC: 5+2=7 chips, mult=1*1.5=1.5 → 10.5 → 10
    assert_eq!(r.final_score as i64, 10);
}

#[test]
fn test_bloodstone_does_not_fire_without_pre_roll() {
    // Without pre-rolling (extra_x_mult=1.0 default), Bloodstone has no effect
    let played = vec![card(0, Rank::Two, Suit::Hearts)];
    let r = score(&played, &played, &[joker(0, JokerKind::Bloodstone)]);
    // HC: 5+2=7 chips, mult=1 (no x_mult) → 7
    assert_eq!(r.final_score as i64, 7);
}

#[test]
fn test_bloodstone_can_trigger_via_round() {
    // Run many trials: Bloodstone has 1/2 chance to trigger per scoring Hearts card
    let mut triggered = false;
    for trial in 0..30 {
        let seed = format!("bloodstone_{}", trial);
        let mut gs = crate::game::GameState::new(DeckType::Blue, Stake::White, Some(seed));
        let hearts_card = card(0, Rank::Two, Suit::Hearts);
        setup_round(&mut gs, vec![hearts_card], 1);
        gs.jokers.push(joker(1, JokerKind::Bloodstone));
        gs.score_goal = 1.0;
        gs.select_card(0).unwrap();
        let result = gs.play_hand().unwrap();
        // Without Bloodstone trigger: 7 chips * 1 mult = 7; with trigger: 7 * 1.5 → 10
        if result.final_score as i64 >= 10 {
            triggered = true;
            break;
        }
    }
    assert!(triggered, "Bloodstone should trigger at least once in 30 trials");
}

#[test]
fn test_rough_gem_earns_dollar_on_diamonds() {
    let played = vec![card(0, Rank::Two, Suit::Diamonds)];
    let r = score(&played, &played, &[joker(0, JokerKind::RoughGem)]);
    assert_eq!(r.dollars_earned, 1);
}

#[test]
fn test_business_card_can_earn_dollars_on_face_card() {
    // BusinessCard has 1/2 chance to earn $2 per scoring face card; handled in round.rs
    let mut earned = false;
    for trial in 0..50 {
        let seed = format!("business_{}", trial);
        let mut gs = crate::game::GameState::new(DeckType::Blue, Stake::White, Some(seed));
        let king = card(0, Rank::King, Suit::Spades);
        setup_round(&mut gs, vec![king], 1);
        gs.jokers.push(joker(1, JokerKind::BusinessCard));
        gs.score_goal = 1.0;
        gs.money = 0;
        gs.select_card(0).unwrap();
        gs.play_hand().unwrap();
        // Blind reward $3 + interest $0 + possibly $2 from BusinessCard = $5
        if gs.money >= 5 {
            earned = true;
            break;
        }
    }
    assert!(earned, "BusinessCard should earn $2 per face card scored at least once in 50 trials");
}

// =========================================================
// Held-in-hand jokers
// =========================================================

#[test]
fn test_baron_fires_on_king_held_in_hand() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let hand = vec![card(1, Rank::King, Suit::Hearts)];
    let r = score(&played, &hand, &[joker(0, JokerKind::Baron)]);
    // HC: 5+11=16 chips, mult=1*1.5=1.5 → 24
    assert_eq!(r.final_score as i64, 24);
}

#[test]
fn test_baron_does_not_fire_on_played_king() {
    // King is played (not held) → Baron should not fire
    let played = vec![card(0, Rank::King, Suit::Spades)];
    let r = score(&played, &[], &[joker(0, JokerKind::Baron)]);
    // HC: 5+10=15 chips, mult=1 → 15
    assert_eq!(r.final_score as i64, 15);
}

#[test]
fn test_shoot_the_moon_fires_on_queen_held_in_hand() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let hand = vec![card(1, Rank::Queen, Suit::Hearts)];
    let r = score(&played, &hand, &[joker(0, JokerKind::ShootTheMoon)]);
    // HC: 5+11=16 chips, mult=1+13=14 → 224
    assert_eq!(r.final_score as i64, 224);
}

#[test]
fn test_shoot_the_moon_does_not_fire_on_played_queen() {
    // Queen is played → should not fire
    let played = vec![card(0, Rank::Queen, Suit::Spades)];
    let r = score(&played, &[], &[joker(0, JokerKind::ShootTheMoon)]);
    // HC: 5+10=15 chips, mult=1 → 15
    assert_eq!(r.final_score as i64, 15);
}

// =========================================================
// Legendary/special per-card jokers
// =========================================================

#[test]
fn test_triboulet_fires_on_scored_king() {
    let played = vec![card(0, Rank::King, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Triboulet)]);
    // HC: 5+10=15 chips, mult=1*2=2 → 30
    assert_eq!(r.final_score as i64, 30);
}

#[test]
fn test_triboulet_fires_on_scored_queen() {
    let played = vec![card(0, Rank::Queen, Suit::Hearts)];
    let r = score(&played, &played, &[joker(0, JokerKind::Triboulet)]);
    assert_eq!(r.final_score as i64, 30);
}

#[test]
fn test_triboulet_stacks_on_pair_of_kings() {
    let played = vec![
        card(0, Rank::King, Suit::Spades),
        card(1, Rank::King, Suit::Hearts),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::Triboulet)]);
    // Pair: base 10+10+10=30 chips, mult=2. Each King → *2: 2*2*2=8. Score=30*8=240
    assert_eq!(r.final_score as i64, 240);
}

#[test]
fn test_the_idol_fires_on_matching_rank_and_suit() {
    let played = vec![card(0, Rank::Two, Suit::Spades)];
    let mut idol = joker(0, JokerKind::TheIdol);
    idol.counters.insert("rank".to_string(), serde_json::json!("Two"));
    idol.counters.insert("suit".to_string(), serde_json::json!("Spades"));
    let r = score(&played, &played, &[idol]);
    // HC: 5+2=7 chips, mult=1*2=2 → 14
    assert_eq!(r.final_score as i64, 14);
}

#[test]
fn test_the_idol_does_not_fire_on_wrong_suit() {
    let played = vec![card(0, Rank::Two, Suit::Hearts)];
    let mut idol = joker(0, JokerKind::TheIdol);
    idol.counters.insert("rank".to_string(), serde_json::json!("Two"));
    idol.counters.insert("suit".to_string(), serde_json::json!("Spades"));
    let r = score(&played, &played, &[idol]);
    assert_eq!(r.final_score as i64, 7);
}

// =========================================================
// Context-aware jokers
// =========================================================

#[test]
fn test_banner_adds_chips_per_discard_remaining() {
    // Banner: +30 chips per remaining Discard (default 3 passed to score()) → +90 chips
    // High Card Ace: chips = 16 + 90 = 106, mult = 1 → 106
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Banner)]);
    assert_eq!(r.final_score as i64, 106);
}

#[test]
fn test_half_joker_activates_with_3_or_fewer_cards() {
    // HalfJoker: +20 mult when ≤3 cards played. Play 1 Ace → +20 mult
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::HalfJoker)]);
    // chips=16, mult=1+20=21 → 336
    assert_eq!(r.final_score as i64, 336);
}

#[test]
fn test_half_joker_does_not_activate_with_5_cards() {
    let played = vec![
        card(0, Rank::Five, Suit::Spades),
        card(1, Rank::Six, Suit::Hearts),
        card(2, Rank::Seven, Suit::Clubs),
        card(3, Rank::Eight, Suit::Diamonds),
        card(4, Rank::Nine, Suit::Spades),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::HalfJoker)]);
    // Straight score without HalfJoker: 260
    assert_eq!(r.final_score as i64, 260);
}

#[test]
fn test_mystic_summit_fires_only_at_zero_discards() {
    // With discards_remaining=3, no bonus
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::MysticSummit)]);
    assert_eq!(r.final_score as i64, 16);

    // With 0 discards remaining → fires
    let r2 = score_hand(
        &played,
        &played,
        &[joker(0, JokerKind::MysticSummit)],
        &default_hand_levels(),
        3,
        0,
        0,
        40,
        52,
        None,
        5,
        0,
        0,
        0,
    );
    // chips=16, mult=1+15=16 → 256
    assert_eq!(r2.final_score as i64, 256);
}

#[test]
fn test_supernova_adds_mult_equal_to_times_played() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Supernova)]);
    // played=0 → +0 mult, chips=16, mult=1 → 16
    assert_eq!(r.final_score as i64, 16);

    let mut levels = default_hand_levels();
    levels.get_mut(&HandType::HighCard).unwrap().played = 5;
    let r2 = score_hand(
        &played,
        &played,
        &[joker(0, JokerKind::Supernova)],
        &levels,
        3,
        3,
        0,
        40,
        52,
        None,
        5,
        0,
        0,
        0,
    );
    // chips=16, mult=1+5=6 → 96
    assert_eq!(r2.final_score as i64, 96);
}

#[test]
fn test_abstract_joker_scales_with_joker_count() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];

    // 1 joker (the abstract joker itself): +3 mult
    let r1 = score(&played, &played, &[joker(0, JokerKind::AbstractJoker)]);
    // chips=16, mult=1+3=4 → 64
    assert_eq!(r1.final_score as i64, 64);

    // 3 jokers: +9 mult
    let jokers3 = vec![
        joker(0, JokerKind::AbstractJoker),
        joker(1, JokerKind::Joker),
        joker(2, JokerKind::Joker),
    ];
    let r3 = score(&played, &played, &jokers3);
    // AbstractJoker: +9 mult; Joker×2: +8 mult → mult = 1+9+8=18; chips=16 → 288
    assert_eq!(r3.final_score as i64, 288);
}

// =========================================================
// Flat-mult hand-level jokers
// =========================================================

#[test]
fn test_the_duo_fires_on_pair() {
    let played = vec![card(0, Rank::Ace, Suit::Spades), card(1, Rank::Ace, Suit::Hearts)];
    let r = score(&played, &played, &[joker(0, JokerKind::TheDuo)]);
    // Pair Aces: 10+11+11=32 chips, mult=2*2=4 → 128
    assert_eq!(r.final_score as i64, 128);
}

#[test]
fn test_the_trio_fires_on_three_of_a_kind() {
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Two, Suit::Clubs),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::TheTrio)]);
    // 3oaK: 36 chips, mult=3*3=9 → 324
    assert_eq!(r.final_score as i64, 324);
}

#[test]
fn test_the_family_fires_on_four_of_a_kind() {
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Two, Suit::Clubs),
        card(3, Rank::Two, Suit::Diamonds),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::TheFamily)]);
    // FourOfAKind base 60+2*4=68 chips, mult=7*4=28 → 1904
    assert_eq!(r.hand_type, HandType::FourOfAKind);
    assert_eq!(r.final_score as i64, 1904);
}

#[test]
fn test_the_order_fires_on_straight() {
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Diamonds),
        card(3, Rank::Four, Suit::Clubs),
        card(4, Rank::Five, Suit::Spades),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::TheOrder)]);
    // Straight: 55 chips, mult=4*3=12 → 660
    assert_eq!(r.final_score as i64, 660);
}

#[test]
fn test_the_tribe_fires_on_flush() {
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Four, Suit::Spades),
        card(2, Rank::Seven, Suit::Spades),
        card(3, Rank::Nine, Suit::Spades),
        card(4, Rank::Jack, Suit::Spades),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::TheTribe)]);
    // Flush base 35+2+4+7+9+10=67 chips, mult=4*2=8 → 536
    assert_eq!(r.final_score as i64, 536);
}

// =========================================================
// Other common jokers
// =========================================================

#[test]
fn test_ancient_joker_multiplies_per_designated_suit_in_scoring() {
    // Default suit is Hearts; Pair 2♥2♦ → 1 Heart in scoring → x1.5
    let played = vec![
        card(0, Rank::Two, Suit::Hearts),
        card(1, Rank::Two, Suit::Diamonds),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::AncientJoker)]);
    // Pair: 10+2+2=14 chips, mult=2*1.5=3 → 42
    assert_eq!(r.final_score as i64, 42);
}

#[test]
fn test_joker_stencil_adds_mult_per_empty_joker_slot() {
    // 1 JokerStencil in 5 slots → 4 empty → +4 mult... no wait, Stencil is x(empty+stencil)
    // Actually: x1 per each stencil * (empty slots + stencil count)?
    // Let's verify: 1 stencil in 5-slot → x(1+4) or x1 per empty counting itself?
    // From tests.rs: joker_slot_count=5, 1 joker → result 80
    // HC chips=16, mult=1. If stencil gives x5 → 80. Correct: x(joker_slot_count) = x5
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::JokerStencil)]);
    // HC: 16 chips, mult=1*5=5 → 80
    assert_eq!(r.final_score as i64, 80);
}

#[test]
fn test_blackboard_fires_when_all_hand_cards_are_spades_or_clubs() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Blackboard)]);
    // all Spades hand → x3 → 16*3=48
    assert_eq!(r.final_score as i64, 48);
}

#[test]
fn test_blackboard_does_not_fire_with_mixed_suits_in_hand() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let hand = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts), // Hearts breaks Blackboard
    ];
    let r = score(&played, &hand, &[joker(0, JokerKind::Blackboard)]);
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_seeing_double_fires_with_club_and_non_club_in_played() {
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Clubs),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::SeeingDouble)]);
    // HC: only Ace scores → 16 chips, then SeeingDouble x2 → mult=2 → 32
    assert_eq!(r.final_score as i64, 32);
}

#[test]
fn test_seeing_double_does_not_fire_with_only_clubs() {
    let played = vec![card(0, Rank::Ace, Suit::Clubs), card(1, Rank::Two, Suit::Clubs)];
    let r = score(&played, &played, &[joker(0, JokerKind::SeeingDouble)]);
    // HC: Ace scores → 16 chips, no x2 → 16
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_flower_pot_fires_with_all_four_suits_in_scoring_cards() {
    // Two Pair 2♠2♥3♦3♣: all 4 suits in scoring cards
    let played = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Diamonds),
        card(3, Rank::Three, Suit::Clubs),
    ];
    let r = score(&played, &played, &[joker(0, JokerKind::FlowerPot)]);
    // TwoPair: base 20+2+2+3+3=30 chips, mult=2*3=6 → 180
    assert_eq!(r.hand_type, HandType::TwoPair);
    assert_eq!(r.final_score as i64, 180);
}

#[test]
fn test_card_sharp_does_not_fire_on_first_play_this_round() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::CardSharp)]);
    // HC: 16 chips, mult=1 (no X3 — hand not yet played this round) → 16
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_card_sharp_fires_when_hand_type_already_played_this_round() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut levels = default_hand_levels();
    levels.get_mut(&HandType::HighCard).unwrap().played_this_round = 1;
    let r = score_hand(&played, &played, &[joker(0, JokerKind::CardSharp)], &levels, 3, 3, 0, 40, 52, None, 5, 0, 0, 0);
    // HC: 16 chips, mult=1*3=3 → 48 (X3 because HighCard already played this round)
    assert_eq!(r.final_score as i64, 48);
}

#[test]
fn test_bootstraps_adds_mult_per_5_dollars() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    // money=10 → 2 lots of $5 → +4 mult
    let r = score_full(&played, &played, &[joker(0, JokerKind::Bootstraps)], 3, 3, 10, 40, 52, 5, 0);
    // HC: 16*(1+4)=80
    assert_eq!(r.final_score as i64, 80);
}

#[test]
fn test_bull_adds_chips_per_dollar_held() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    // money=5 → +10 chips
    let r = score_full(&played, &played, &[joker(0, JokerKind::Bull)], 3, 3, 5, 40, 52, 5, 0);
    // HC: 16+10=26, mult=1 → 26
    assert_eq!(r.final_score as i64, 26);
}

#[test]
fn test_blue_joker_adds_chips_per_card_remaining_in_deck() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    // deck_remaining=10 → +20 chips
    let r = score_full(&played, &played, &[joker(0, JokerKind::BlueJoker)], 3, 3, 0, 10, 52, 5, 0);
    // HC: 16+20=36, mult=1 → 36
    assert_eq!(r.final_score as i64, 36);
}

#[test]
fn test_steel_joker_scales_with_steel_cards_in_hand() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut steel_card = card(1, Rank::Two, Suit::Hearts);
    steel_card.enhancement = Enhancement::Steel;
    let hand = vec![steel_card];
    let r = score(&played, &hand, &[joker(0, JokerKind::SteelJoker)]);
    // Steel in hand: phase 3 → mult*=1.5; SteelJoker: mult*=1.2. HC: 16*(1.5*1.2)=28.8 → 28
    assert_eq!(r.final_score as i64, 28);
}

#[test]
fn test_stuntman_adds_250_chips() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Stuntman)]);
    // HC: 5+11+250=266, mult=1 → 266
    assert_eq!(r.final_score as i64, 266);
}

#[test]
fn test_acrobat_fires_only_on_last_hand() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score_full(&played, &played, &[joker(0, JokerKind::Acrobat)], 0, 3, 0, 40, 52, 5, 0);
    // hands_remaining=0 → x3 mult → 16*3=48
    assert_eq!(r.final_score as i64, 48);
}

#[test]
fn test_acrobat_does_not_fire_with_hands_remaining() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score_full(&played, &played, &[joker(0, JokerKind::Acrobat)], 2, 3, 0, 40, 52, 5, 0);
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_fortune_teller_adds_mult_per_tarot_used() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    // 5 tarots used → +5 mult
    let r = score_full(&played, &played, &[joker(0, JokerKind::FortuneTeller)], 3, 3, 0, 40, 52, 5, 5);
    // HC: 16*(1+5)=96
    assert_eq!(r.final_score as i64, 96);
}

#[test]
fn test_drivers_license_fires_with_enough_enhanced_cards() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let hand: Vec<CardInstance> = (0..8).map(|i| {
        let mut c = card(i + 10, Rank::Two, Suit::Hearts);
        c.enhancement = Enhancement::Bonus;
        c
    }).collect();
    let r = score(&played, &hand, &[joker(0, JokerKind::DriversLicense)]);
    // x3 → HC: 16*3=48
    assert_eq!(r.final_score as i64, 48);
}

#[test]
fn test_drivers_license_does_not_fire_below_threshold() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let hand: Vec<CardInstance> = (0..2).map(|i| {
        let mut c = card(i + 10, Rank::Two, Suit::Hearts);
        c.enhancement = Enhancement::Bonus;
        c
    }).collect();
    let r = score(&played, &hand, &[joker(0, JokerKind::DriversLicense)]);
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_erosion_fires_when_deck_is_smaller_than_starting_size() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    // total_deck=42 → 52-42=10 cards permanently removed → +40 mult
    let r = score_full(&played, &played, &[joker(0, JokerKind::Erosion)], 3, 3, 0, 40, 42, 5, 0);
    // HC: 16*(1+40)=656
    assert_eq!(r.final_score as i64, 656);
}

#[test]
fn test_erosion_does_not_fire_when_deck_is_full() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    // total_deck=52 → no cards removed → +0 mult
    let r = score_full(&played, &played, &[joker(0, JokerKind::Erosion)], 3, 3, 0, 40, 52, 5, 0);
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_misprint_adds_flat_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Misprint)]);
    // +11 mult → HC: 16*(1+11)=192
    assert_eq!(r.final_score as i64, 192);
}

#[test]
fn test_throwback_scales_with_blind_skips() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Throwback);
    j.set_counter_i64("skips", 4);
    let r = score(&played, &played, &[j]);
    // x(1+0.25*4)=x2 → HC: 16*2=32
    assert_eq!(r.final_score as i64, 32);
}
