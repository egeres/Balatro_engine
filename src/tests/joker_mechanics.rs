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

// =========================================================
// `before`-phase scaling: the upgrade counts towards the hand that triggered it
// =========================================================

/// Play `played_idx` from a controlled hand and return the resulting state + score.
fn play_with(
    deck: Vec<CardInstance>,
    hand_size: usize,
    jokers: Vec<JokerInstance>,
    select: &[usize],
) -> (GameState, crate::scoring::ScoreResult) {
    let mut gs = make_game();
    setup_round(&mut gs, deck, hand_size);
    gs.jokers = jokers;
    gs.score_goal = f64::MAX; // never end the round
    for &i in select {
        gs.select_card(i).unwrap();
    }
    let r = gs.play_hand().unwrap();
    (gs, r)
}

/// Green Joker gains +1 Mult under `context.before` (card.lua:3563), so the very first hand
/// already scores with +1.
#[test]
fn test_green_joker_upgrade_applies_to_the_same_hand() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades)];
    let (gs, r) = play_with(deck, 1, vec![joker(0, JokerKind::GreenJoker)], &[0]);
    assert_eq!(gs.jokers[0].get_counter_i64("mult"), 1);
    // HC: 16 chips, mult 1 + 1 = 2 → 32
    assert_eq!(r.final_mult as i64, 2);
}

/// Runner's +15 Chips lands before the hand scores.
#[test]
fn test_runner_upgrade_applies_to_the_same_hand() {
    let deck = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Three, Suit::Hearts),
        card(2, Rank::Four, Suit::Clubs),
        card(3, Rank::Five, Suit::Diamonds),
        card(4, Rank::Six, Suit::Spades),
    ];
    let (gs, r) = play_with(deck, 5, vec![joker(0, JokerKind::Runner)], &[0, 1, 2, 3, 4]);
    assert_eq!(gs.jokers[0].get_counter_i64("chips"), 15);
    // Straight: 30 base + 2+3+4+5+6 = 50, +15 from Runner = 65
    assert_eq!(r.final_chips as i64, 65);
}

/// Square Joker's +4 Chips lands before the hand scores.
#[test]
fn test_square_joker_upgrade_applies_to_the_same_hand() {
    let deck = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Two, Suit::Clubs),
        card(3, Rank::Two, Suit::Diamonds),
    ];
    let (gs, r) = play_with(deck, 4, vec![joker(0, JokerKind::SquareJoker)], &[0, 1, 2, 3]);
    assert_eq!(gs.jokers[0].get_counter_i64("chips"), 4);
    // FourOfAKind: 60 base + 2+2+2+2 = 68, +4 = 72
    assert_eq!(r.final_chips as i64, 72);
}

/// Spare Trousers' +2 Mult lands before the hand scores.
#[test]
fn test_spare_trousers_upgrade_applies_to_the_same_hand() {
    let deck = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
        card(3, Rank::Three, Suit::Diamonds),
    ];
    let (gs, r) = play_with(deck, 4, vec![joker(0, JokerKind::SpareTrousers)], &[0, 1, 2, 3]);
    assert_eq!(gs.jokers[0].get_counter_i64("mult"), 2);
    // TwoPair: mult 2 + 2 = 4
    assert_eq!(r.final_mult as i64, 4);
}

/// Vampire eats the enhancement *before* the card scores, so an eaten Glass card
/// does not give its X2 on the hand that fed Vampire.
#[test]
fn test_vampire_eats_the_enhancement_before_it_scores() {
    let mut glass = card(0, Rank::Ace, Suit::Spades);
    glass.enhancement = Enhancement::Glass;
    let (gs, r) = play_with(vec![glass], 1, vec![joker(0, JokerKind::Vampire)], &[0]);

    assert!((gs.jokers[0].get_counter_f64("x_mult") - 1.1).abs() < 1e-9);
    // HC 16 chips. Glass X2 is gone; Vampire's own X1.1 applies → mult 1.1
    assert!((r.final_mult - 1.1).abs() < 1e-9, "got {}", r.final_mult);
    assert_eq!(gs.deck[0].enhancement, Enhancement::None);
}

/// Ride the Bus resets before scoring when the hand contains a scoring face card.
#[test]
fn test_ride_the_bus_resets_before_scoring() {
    let deck = vec![card(0, Rank::King, Suit::Spades)];
    let mut j = joker(0, JokerKind::RideTheBus);
    j.set_counter_i64("mult", 7);
    let (gs, r) = play_with(deck, 1, vec![j], &[0]);
    assert_eq!(gs.jokers[0].get_counter_i64("mult"), 0);
    // Reset happens before scoring, so no +7 this hand. HC 15 chips, mult 1.
    assert_eq!(r.final_mult as i64, 1);
}

/// Wee Joker gains +8 Chips per *scoring* 2 during card scoring, so the gain counts on the
/// hand that triggered it (card.lua:3083).
#[test]
fn test_wee_joker_upgrade_applies_to_the_same_hand() {
    let deck = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
    ];
    let (gs, r) = play_with(deck, 2, vec![joker(0, JokerKind::WeeJoker)], &[0, 1]);
    assert_eq!(gs.jokers[0].get_counter_i64("chips"), 16);
    // Pair: 10 base + 2 + 2 = 14, +16 from Wee = 30
    assert_eq!(r.final_chips as i64, 30);
}

/// Only *scoring* 2s count — a 2 left out of the scoring hand does nothing.
#[test]
fn test_wee_joker_ignores_non_scoring_twos() {
    let deck = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
        card(2, Rank::Two, Suit::Clubs), // kicker, does not score
    ];
    let (gs, _) = play_with(deck, 3, vec![joker(0, JokerKind::WeeJoker)], &[0, 1, 2]);
    assert_eq!(gs.jokers[0].get_counter_i64("chips"), 0);
}

/// Retriggers stack: Hack retriggers 2s, so each scoring 2 feeds Wee Joker twice.
#[test]
fn test_wee_joker_counts_retriggers() {
    let deck = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
    ];
    let (gs, _) = play_with(
        deck,
        2,
        vec![joker(0, JokerKind::WeeJoker), joker(1, JokerKind::Hack)],
        &[0, 1],
    );
    // 2 twos x 2 triggers each x 8 = 32
    assert_eq!(gs.jokers[0].get_counter_i64("chips"), 32);
}

// =========================================================
// Mime retriggers every held-in-hand effect
// =========================================================

#[test]
fn test_mime_retriggers_baron() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let hand = vec![card(1, Rank::King, Suit::Hearts)];
    let solo = score(&played, &hand, &[joker(0, JokerKind::Baron)]);
    let mimed = score(
        &played,
        &hand,
        &[joker(0, JokerKind::Baron), joker(1, JokerKind::Mime)],
    );
    assert!((solo.final_mult - 1.5).abs() < 1e-9);
    assert!((mimed.final_mult - 2.25).abs() < 1e-9, "got {}", mimed.final_mult);
}

#[test]
fn test_mime_retriggers_shoot_the_moon() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let hand = vec![card(1, Rank::Queen, Suit::Hearts)];
    let solo = score(&played, &hand, &[joker(0, JokerKind::ShootTheMoon)]);
    let mimed = score(
        &played,
        &hand,
        &[joker(0, JokerKind::ShootTheMoon), joker(1, JokerKind::Mime)],
    );
    // 1 + 13 = 14 solo; 1 + 13 + 13 = 27 with Mime
    assert_eq!(solo.final_mult as i64, 14);
    assert_eq!(mimed.final_mult as i64, 27);
}

#[test]
fn test_mime_still_retriggers_steel() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut steel = card(1, Rank::King, Suit::Hearts);
    steel.enhancement = Enhancement::Steel;
    let hand = vec![steel];
    let mimed = score(&played, &hand, &[joker(0, JokerKind::Mime)]);
    // 1 * 1.5 * 1.5 = 2.25
    assert!((mimed.final_mult - 2.25).abs() < 1e-9);
}

/// A held card that does nothing gains no repetition (state_events.lua:814).
#[test]
fn test_mime_does_not_retrigger_an_inert_held_card() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let hand = vec![card(1, Rank::Three, Suit::Hearts)];
    let r = score(&played, &hand, &[joker(0, JokerKind::Mime)]);
    assert_eq!(r.final_mult as i64, 1);
}

/// A Red Seal on a held card retriggers its held-in-hand effects too.
#[test]
fn test_red_seal_retriggers_a_held_steel_card() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut steel = card(1, Rank::King, Suit::Hearts);
    steel.enhancement = Enhancement::Steel;
    steel.seal = Seal::Red;
    let r = score(&played, &[steel], &[]);
    assert!((r.final_mult - 2.25).abs() < 1e-9);
}

/// Blueprint copying Mime adds another repetition.
#[test]
fn test_blueprint_copying_mime_adds_a_repetition() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let hand = vec![card(1, Rank::King, Suit::Hearts)];
    let r = score(
        &played,
        &hand,
        &[
            joker(0, JokerKind::Baron),
            joker(1, JokerKind::Blueprint),
            joker(2, JokerKind::Mime),
        ],
    );
    // Baron fires 3x (base + Mime + Blueprint-as-Mime); Blueprint also copies Mime for the
    // hand-card phase, which contributes no direct effect. 1.5^3 = 3.375
    assert!((r.final_mult - 3.375).abs() < 1e-9, "got {}", r.final_mult);
}

// =========================================================
// Hiker writes a permanent bonus rather than scoring chips
// =========================================================

#[test]
fn test_hiker_permanently_boosts_the_card_it_scored() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades)];
    let (gs, r) = play_with(deck, 1, vec![joker(0, JokerKind::Hiker)], &[0]);
    // Nothing extra this hand...
    assert_eq!(r.final_chips as i64, 16);
    // ...but the card is permanently worth 5 more.
    assert_eq!(gs.deck[0].extra_chips, 5);
}

#[test]
fn test_hiker_bonus_shows_up_on_the_next_play() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.jokers = vec![joker(0, JokerKind::Hiker)];
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    let first = gs.play_hand().unwrap();
    assert_eq!(first.final_chips as i64, 16);

    // Replay the same (now boosted) card.
    gs.hand = vec![0];
    gs.select_card(0).unwrap();
    let second = gs.play_hand().unwrap();
    assert_eq!(second.final_chips as i64, 21, "the +5 should be baked into the card");
    assert_eq!(gs.deck[0].extra_chips, 10);
}

#[test]
fn test_hiker_counts_each_retrigger() {
    // Hack retriggers the 2, so Hiker fires twice on it.
    let deck = vec![card(0, Rank::Two, Suit::Spades)];
    let (gs, _) = play_with(
        deck,
        1,
        vec![joker(0, JokerKind::Hiker), joker(1, JokerKind::Hack)],
        &[0],
    );
    assert_eq!(gs.deck[0].extra_chips, 10);
}

/// Balatro writes Hiker's perma_bonus mid-scoring, so a retriggered card scores the boosted
/// value on its later triggers (card.lua:3067).
#[test]
fn test_hiker_bonus_compounds_within_a_single_hand() {
    // Hack retriggers the 2, so it scores twice: once at 2 chips, once at 2 + 5.
    let deck = vec![card(0, Rank::Two, Suit::Spades)];
    let (gs, r) = play_with(
        deck,
        1,
        vec![joker(0, JokerKind::Hiker), joker(1, JokerKind::Hack)],
        &[0],
    );
    // High Card base 5, then 2 + 7 from the two triggers = 14
    assert_eq!(r.final_chips as i64, 14);
    assert_eq!(gs.deck[0].extra_chips, 10);
}

#[test]
fn test_hiker_without_retriggers_adds_nothing_to_this_hand() {
    let deck = vec![card(0, Rank::Two, Suit::Spades)];
    let (gs, r) = play_with(deck, 1, vec![joker(0, JokerKind::Hiker)], &[0]);
    // Single trigger: 5 + 2, the bonus lands after.
    assert_eq!(r.final_chips as i64, 7);
    assert_eq!(gs.deck[0].extra_chips, 5);
}

// =========================================================
// The hand type is locked in before jokers touch the cards
// =========================================================

/// Balatro decides the hand at the top of evaluate_play (state_events.lua:572), before the
/// `before` pass. Vampire eating a Wild Card's enhancement must not retroactively break the
/// flush that Wild Card was completing.
#[test]
fn test_vampire_eating_a_wild_card_does_not_break_the_flush() {
    let mut wild = card(4, Rank::Nine, Suit::Hearts);
    wild.enhancement = Enhancement::Wild;
    let deck = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Four, Suit::Spades),
        card(2, Rank::Six, Suit::Spades),
        card(3, Rank::Eight, Suit::Spades),
        wild,
    ];
    let (gs, r) = play_with(deck, 5, vec![joker(0, JokerKind::Vampire)], &[0, 1, 2, 3, 4]);

    assert_eq!(r.hand_type, HandType::Flush, "the flush was locked in before Vampire ate it");
    assert_eq!(gs.deck[4].enhancement, Enhancement::None, "the Wild is still consumed");
    assert!((gs.jokers[0].get_counter_f64("x_mult") - 1.1).abs() < 1e-9);
    // Flush: 35 base + 2 + 4 + 6 + 8 + 9 = 64 chips
    assert_eq!(r.final_chips as i64, 64);
}

/// Without Vampire the same hand is a plain flush, so the guard above is not hiding a change
/// in how the hand is read.
#[test]
fn test_wild_card_flush_scores_the_same_without_vampire() {
    let mut wild = card(4, Rank::Nine, Suit::Hearts);
    wild.enhancement = Enhancement::Wild;
    let deck = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Four, Suit::Spades),
        card(2, Rank::Six, Suit::Spades),
        card(3, Rank::Eight, Suit::Spades),
        wild,
    ];
    let (_, r) = play_with(deck, 5, vec![], &[0, 1, 2, 3, 4]);
    assert_eq!(r.hand_type, HandType::Flush);
    assert_eq!(r.final_chips as i64, 64);
}

/// Midas Mask gilding face cards mid-pass likewise cannot change what is being scored.
#[test]
fn test_midas_mask_does_not_change_the_scored_hand() {
    let deck = vec![
        card(0, Rank::King, Suit::Spades),
        card(1, Rank::King, Suit::Hearts),
    ];
    let (gs, r) = play_with(deck, 2, vec![joker(0, JokerKind::MidasMask)], &[0, 1]);
    assert_eq!(r.hand_type, HandType::Pair);
    assert_eq!(gs.deck[0].enhancement, Enhancement::Gold);
    assert_eq!(gs.deck[1].enhancement, Enhancement::Gold);
}
