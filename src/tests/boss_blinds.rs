/// Comprehensive tests for all boss blind mechanics implemented in the engine.
///
/// Boss blinds are set via `GameState::boss_blind` and take effect either during
/// `begin_round()` (card debuffs), `play_hand()` (hand restrictions / money penalties),
/// `effective_hand_size()` (ThePsychic), or inside `score_hand()` (TheFlint).
///
/// What is covered here:
///
///   1.  Suit debuffs    — TheClub (♣), TheGoad (♠), TheHead (♥), TheWindow (♦):
///         All cards of the targeted suit have `debuffed = true` after `select_blind()`;
///         all other cards remain clean.
///
///   2.  Face-card debuffs — ThePlant, TheMark (identical effect):
///         J/Q/K cards are debuffed; 2–10 and Ace are unaffected.
///
///   3.  Debuff suppression — Luchador, Chicot jokers:
///         When either is active the blind cannot debuff any card.
///
///   4.  Boss-only debuffs — debuffs only fire when `current_blind == Boss`.
///         Small and Big blinds with a debuff boss set leave every card clean.
///
///   5.  Debuffed cards in scoring:
///         A debuffed card contributes 0 chips in Phase 2 but does not remove
///         the hand's base chips or mult.
///
///   6.  TheFlint — halves (floor) both base chips and base mult before card scoring.
///         e.g. Flush L1 chips 35→17, mult 4→2. HighCard mult 1→0 (score = 0).
///
///   7.  TheNeedle — only 1 hand may be played on a Boss round;
///         second attempt returns `Err(BalatroError::BossBlindEffect)`.
///         No restriction on Small/Big blinds.
///         Chip goal multiplier is 1× (not the standard 2×).
///
///   8.  TheTooth — deducts $1 per card played on a Boss round; no deduction elsewhere.
///
///   9.  ThePsychic — forces the hand draw to at least 5 cards during a Round
///         (uses `effective_hand_size().max(5)`).
///
///  10.  Chip goal multipliers:
///         TheWall 4×, VioletVessel 6×, TheNeedle 1×, all other bosses 2×.
///         Ante 1 base = 300; expected Boss goals tested against exact values.
///
///  11.  Dollar rewards on defeat:
///         Regular bosses (Antes 1–7) award $5.
///         Showdown bosses (CeruleanBell, VerdantLeaf, VioletVessel, AmberAcorn,
///         CrimsonHeart) award $8 regardless of ante.
///
///  12.  No-op bosses — TheOx, TheHook, TheMouth, TheFish, TheManacle, TheWall,
///         TheHouse, TheWheel, TheArm, TheWater, TheEye, TheSerpent, ThePillar, and
///         all showdown bosses have no effect on `score_hand()`.
///         score_hand() with these bosses == score_hand() with boss_blind = None.

use super::*;
use crate::game::{BalatroError, BlindKind};

// =========================================================
// Helpers
// =========================================================

/// Create a fresh game positioned to enter a Boss blind with the given boss active.
/// State is BlindSelect, current_blind is Boss. Call `gs.select_blind()` to start.
fn boss_select(boss: BossBlind) -> GameState {
    let mut gs = make_game();
    gs.boss_blind = Some(boss);
    gs.current_blind = BlindKind::Boss;
    gs
}

/// Call `score_hand` with a specific boss blind, no jokers, and default Level-1 hands.
fn score_with_boss(played: &[CardInstance], boss: BossBlind) -> crate::scoring::ScoreResult {
    score_hand(played, &[], &[], &default_hand_levels(), 3, 3, 0, 40, 52, Some(boss), 5, 0, 0)
}

/// Call `score_hand` with no boss blind — the baseline to compare against.
fn score_baseline(played: &[CardInstance]) -> crate::scoring::ScoreResult {
    score_hand(played, &[], &[], &default_hand_levels(), 3, 3, 0, 40, 52, None, 5, 0, 0)
}

/// Five mixed low Spades that form a Flush (2,3,4,5,7 — no straight).
/// Card chip total: 2+3+4+5+7 = 21.
fn flush_spades() -> Vec<CardInstance> {
    vec![
        card(0, Rank::Two,   Suit::Spades),
        card(1, Rank::Three, Suit::Spades),
        card(2, Rank::Four,  Suit::Spades),
        card(3, Rank::Five,  Suit::Spades),
        card(4, Rank::Seven, Suit::Spades),
    ]
}

// =========================================================
// 1. Suit debuff bosses
// =========================================================

#[test]
fn test_the_club_debuffs_all_clubs_in_deck() {
    let mut gs = boss_select(BossBlind::TheClub);
    gs.select_blind().unwrap();
    let all_clubs_debuffed = gs.deck.iter()
        .filter(|c| c.suit == Suit::Clubs)
        .all(|c| c.debuffed);
    assert!(all_clubs_debuffed, "TheClub: every Club in the deck should be debuffed");
}

#[test]
fn test_the_club_leaves_non_clubs_undebuffed() {
    let mut gs = boss_select(BossBlind::TheClub);
    gs.select_blind().unwrap();
    let non_clubs_clean = gs.deck.iter()
        .filter(|c| c.suit != Suit::Clubs)
        .all(|c| !c.debuffed);
    assert!(non_clubs_clean, "TheClub: no non-Club card should be debuffed");
}

#[test]
fn test_the_goad_debuffs_all_spades_in_deck() {
    let mut gs = boss_select(BossBlind::TheGoad);
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().filter(|c| c.suit == Suit::Spades).all(|c| c.debuffed),
        "TheGoad: every Spade should be debuffed"
    );
}

#[test]
fn test_the_goad_leaves_non_spades_undebuffed() {
    let mut gs = boss_select(BossBlind::TheGoad);
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().filter(|c| c.suit != Suit::Spades).all(|c| !c.debuffed),
        "TheGoad: no non-Spade card should be debuffed"
    );
}

#[test]
fn test_the_head_debuffs_all_hearts_in_deck() {
    let mut gs = boss_select(BossBlind::TheHead);
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().filter(|c| c.suit == Suit::Hearts).all(|c| c.debuffed),
        "TheHead: every Heart should be debuffed"
    );
}

#[test]
fn test_the_head_leaves_non_hearts_undebuffed() {
    let mut gs = boss_select(BossBlind::TheHead);
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().filter(|c| c.suit != Suit::Hearts).all(|c| !c.debuffed),
        "TheHead: no non-Heart card should be debuffed"
    );
}

#[test]
fn test_the_window_debuffs_all_diamonds_in_deck() {
    let mut gs = boss_select(BossBlind::TheWindow);
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().filter(|c| c.suit == Suit::Diamonds).all(|c| c.debuffed),
        "TheWindow: every Diamond should be debuffed"
    );
}

#[test]
fn test_the_window_leaves_non_diamonds_undebuffed() {
    let mut gs = boss_select(BossBlind::TheWindow);
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().filter(|c| c.suit != Suit::Diamonds).all(|c| !c.debuffed),
        "TheWindow: no non-Diamond card should be debuffed"
    );
}

// =========================================================
// 2. Face-card debuff bosses (ThePlant, TheMark)
// =========================================================

#[test]
fn test_the_plant_debuffs_all_face_cards() {
    let mut gs = boss_select(BossBlind::ThePlant);
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().filter(|c| c.rank.is_face()).all(|c| c.debuffed),
        "ThePlant: every J/Q/K should be debuffed"
    );
}

#[test]
fn test_the_plant_leaves_non_face_cards_undebuffed() {
    let mut gs = boss_select(BossBlind::ThePlant);
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().filter(|c| !c.rank.is_face()).all(|c| !c.debuffed),
        "ThePlant: 2–10 and Ace should not be debuffed"
    );
}

#[test]
fn test_the_mark_debuffs_all_face_cards() {
    let mut gs = boss_select(BossBlind::TheMark);
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().filter(|c| c.rank.is_face()).all(|c| c.debuffed),
        "TheMark: every J/Q/K should be debuffed"
    );
}

#[test]
fn test_the_mark_leaves_non_face_cards_undebuffed() {
    let mut gs = boss_select(BossBlind::TheMark);
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().filter(|c| !c.rank.is_face()).all(|c| !c.debuffed),
        "TheMark: 2–10 and Ace should not be debuffed"
    );
}

/// ThePlant and TheMark debuff exactly 12 face cards in a 52-card deck (3 ranks × 4 suits).
#[test]
fn test_the_plant_debuffs_exactly_12_cards() {
    let mut gs = boss_select(BossBlind::ThePlant);
    gs.select_blind().unwrap();
    let count = gs.deck.iter().filter(|c| c.debuffed).count();
    assert_eq!(count, 12, "ThePlant should debuff exactly 12 face cards (J/Q/K × 4 suits)");
}

#[test]
fn test_the_mark_debuffs_exactly_12_cards() {
    let mut gs = boss_select(BossBlind::TheMark);
    gs.select_blind().unwrap();
    let count = gs.deck.iter().filter(|c| c.debuffed).count();
    assert_eq!(count, 12, "TheMark should debuff exactly 12 face cards (J/Q/K × 4 suits)");
}

// =========================================================
// 3. Debuff suppression — Luchador and Chicot
// =========================================================

/// Luchador blocks TheClub from debuffing any card.
#[test]
fn test_luchador_disables_club_debuffs() {
    let mut gs = boss_select(BossBlind::TheClub);
    gs.jokers.push(joker(99, JokerKind::Luchador));
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().all(|c| !c.debuffed),
        "Luchador should suppress all TheClub debuffs"
    );
}

/// Chicot blocks TheGoad from debuffing any card.
#[test]
fn test_chicot_disables_goad_debuffs() {
    let mut gs = boss_select(BossBlind::TheGoad);
    gs.jokers.push(joker(99, JokerKind::Chicot));
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().all(|c| !c.debuffed),
        "Chicot should suppress all TheGoad debuffs"
    );
}

/// Luchador blocks ThePlant from debuffing face cards.
#[test]
fn test_luchador_disables_plant_debuffs() {
    let mut gs = boss_select(BossBlind::ThePlant);
    gs.jokers.push(joker(99, JokerKind::Luchador));
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().all(|c| !c.debuffed),
        "Luchador should suppress all ThePlant debuffs"
    );
}

/// Chicot blocks TheHead from debuffing Hearts.
#[test]
fn test_chicot_disables_head_debuffs() {
    let mut gs = boss_select(BossBlind::TheHead);
    gs.jokers.push(joker(99, JokerKind::Chicot));
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().all(|c| !c.debuffed),
        "Chicot should suppress all TheHead debuffs"
    );
}

/// Without a disabling joker, debuffs still apply normally.
#[test]
fn test_without_disabling_joker_debuffs_apply() {
    let mut gs = boss_select(BossBlind::TheWindow);
    // No Luchador or Chicot
    gs.select_blind().unwrap();
    let diamond_count = gs.deck.iter().filter(|c| c.suit == Suit::Diamonds).count();
    let debuffed_count = gs.deck.iter().filter(|c| c.debuffed).count();
    assert_eq!(
        debuffed_count, diamond_count,
        "without disabling joker, TheWindow should debuff all {} Diamonds", diamond_count
    );
}

// =========================================================
// 4. Debuffs only apply on Boss blind (not Small or Big)
// =========================================================

/// boss_blind = TheClub, but entering a Small blind → no debuffs applied.
#[test]
fn test_debuffs_do_not_apply_on_small_blind() {
    let mut gs = make_game();
    gs.boss_blind = Some(BossBlind::TheClub);
    // current_blind is Small by default
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().all(|c| !c.debuffed),
        "TheClub debuffs must not apply on Small blind"
    );
}

/// boss_blind = TheGoad, but entering a Big blind → no debuffs applied.
#[test]
fn test_debuffs_do_not_apply_on_big_blind() {
    let mut gs = make_game();
    gs.boss_blind = Some(BossBlind::TheGoad);
    gs.current_blind = BlindKind::Big;
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().all(|c| !c.debuffed),
        "TheGoad debuffs must not apply on Big blind"
    );
}

/// Entering a Boss blind resets any previously set debuffs before re-applying them.
/// (i.e. a non-debuff boss after a debuff boss clears old debuffs)
#[test]
fn test_boss_select_resets_previous_debuffs_to_clean() {
    // First boss round with TheClub: debuffs clubs
    let mut gs = boss_select(BossBlind::TheClub);
    gs.select_blind().unwrap();
    assert!(gs.deck.iter().any(|c| c.debuffed));

    // Simulate transitioning to next round (re-use the deck, set a non-debuffing boss)
    gs.boss_blind = Some(BossBlind::TheOx);
    gs.current_blind = BlindKind::Boss;
    gs.state = crate::game::GameStateKind::BlindSelect;
    gs.select_blind().unwrap();
    assert!(
        gs.deck.iter().all(|c| !c.debuffed),
        "after switching to a non-debuffing boss, old debuffs should be cleared"
    );
}

// =========================================================
// 5. Debuffed cards contribute 0 chips in scoring
// =========================================================

/// A debuffed Ace contributes 0 card chips: only the HighCard base (5) remains.
/// Normal Ace HighCard: chips = 5+11 = 16; debuffed: chips = 5+0 = 5.
#[test]
fn test_debuffed_card_contributes_zero_chips() {
    let normal = vec![card(0, Rank::Ace, Suit::Spades)];
    let r_normal = score_baseline(&normal);
    assert_eq!(r_normal.final_chips as i64, 16, "sanity: Ace HighCard chips = 5+11 = 16");

    let mut debuffed_ace = card(0, Rank::Ace, Suit::Spades);
    debuffed_ace.debuffed = true;
    let r_debuffed = score_baseline(&[debuffed_ace]);
    assert_eq!(
        r_debuffed.final_chips as i64, 5,
        "debuffed Ace: card chips = 0, total = base(5) only"
    );
}

/// Debuffed cards in a Pair: both 7s zeroed → only base Pair chips remain.
/// Normal Pair of 7s: chips=10+7+7=24; debuffed: chips=10+0+0=10.
#[test]
fn test_debuffed_pair_of_sevens_contributes_zero_card_chips() {
    let mut s1 = card(0, Rank::Seven, Suit::Spades);
    let mut s2 = card(1, Rank::Seven, Suit::Hearts);
    s1.debuffed = true;
    s2.debuffed = true;
    let r = score_baseline(&[s1, s2]);
    assert_eq!(r.final_chips as i64, 10, "both 7s debuffed: total chips = Pair base (10) only");
    assert_eq!(r.final_score as i64, 20, "Pair of debuffed 7s: score = 10 × 2 = 20");
}

/// Only the debuffed cards are suppressed; undebuffed cards in the same hand score normally.
/// Pair with one 7 debuffed and one clean 7: chips=10+0+7=17.
#[test]
fn test_only_debuffed_cards_are_suppressed() {
    let mut debuffed = card(0, Rank::Seven, Suit::Spades);
    debuffed.debuffed = true;
    let clean = card(1, Rank::Seven, Suit::Hearts);
    let r = score_baseline(&[debuffed, clean]);
    assert_eq!(r.final_chips as i64, 17, "only the debuffed 7 is suppressed; clean 7 scores normally");
}

/// Debuffed cards in the played hand do not generate dollar events (Gold Seal, Lucky).
/// Verify that a Gold-Sealed debuffed card earns $0 (since it's skipped entirely).
#[test]
fn test_debuffed_gold_seal_earns_no_dollars() {
    let mut gold_debuffed = card(0, Rank::Ace, Suit::Spades);
    gold_debuffed.seal = Seal::Gold;
    gold_debuffed.debuffed = true;
    let r = score_baseline(&[gold_debuffed]);
    assert_eq!(r.dollars_earned, 0, "debuffed Gold Seal card should earn $0");
}

// =========================================================
// 6. TheFlint — halves base chips and mult (floor)
// =========================================================

/// TheFlint reduces a Flush score from 224 to 76.
/// Without: chips=35+21=56, mult=4 → 224.
/// With: chips=floor(35/2)+21=17+21=38, mult=floor(4/2)=2 → 76.
#[test]
fn test_the_flint_reduces_flush_score() {
    let played = flush_spades();
    let r_flint    = score_with_boss(&played, BossBlind::TheFlint);
    let r_baseline = score_baseline(&played);
    assert_eq!(r_baseline.final_score as i64, 224, "baseline Flush (2–7♠) should score 224");
    assert_eq!(r_flint.final_score   as i64,  76, "TheFlint Flush should score 76");
}

/// TheFlint correctly floors Flush base chips: 35 → 17 (floor(35/2)).
#[test]
fn test_the_flint_floors_flush_base_chips() {
    let played = flush_spades();
    let r = score_with_boss(&played, BossBlind::TheFlint);
    // final_chips = 17 (base after floor) + 21 (cards) = 38
    assert_eq!(r.final_chips as i64, 38, "TheFlint: Flush chips should be floor(35/2)+21 = 38");
}

/// TheFlint halves Flush base mult: 4 → 2 (floor(4/2) = 2 exactly).
#[test]
fn test_the_flint_halves_flush_base_mult() {
    let played = flush_spades();
    let r = score_with_boss(&played, BossBlind::TheFlint);
    assert_eq!(r.final_mult as i64, 2, "TheFlint: Flush mult should be floor(4/2) = 2");
}

/// TheFlint with Pair of Aces: chips=floor(10/2)+22=5+22=27, mult=floor(2/2)=1 → 27.
#[test]
fn test_the_flint_pair_of_aces() {
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
    ];
    let r = score_with_boss(&played, BossBlind::TheFlint);
    assert_eq!(r.final_chips as i64,  27, "TheFlint Pair of Aces: chips = floor(10/2)+11+11 = 27");
    assert_eq!(r.final_mult  as i64,   1, "TheFlint Pair of Aces: mult  = floor(2/2) = 1");
    assert_eq!(r.final_score as i64,  27, "TheFlint Pair of Aces: score = 27 × 1 = 27");
}

/// TheFlint floors HighCard mult (1) to 0, making the final score 0.
/// floor(1/2) = floor(0.5) = 0 → chips × 0 = 0.
#[test]
fn test_the_flint_zeroes_high_card_score() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score_with_boss(&played, BossBlind::TheFlint);
    assert_eq!(
        r.final_mult  as i64, 0,
        "TheFlint: HighCard base mult=1 → floor(1/2)=0"
    );
    assert_eq!(
        r.final_score as i64, 0,
        "TheFlint: score = chips × 0 = 0"
    );
}

/// TheFlint has no effect on scoring when it is NOT the active boss blind.
#[test]
fn test_the_flint_effect_absent_without_boss() {
    let played = flush_spades();
    let r_flint    = score_with_boss(&played, BossBlind::TheFlint);
    let r_baseline = score_baseline(&played);
    assert!(
        r_flint.final_score < r_baseline.final_score,
        "TheFlint score ({}) must be lower than baseline ({})",
        r_flint.final_score, r_baseline.final_score
    );
}

// =========================================================
// 7. TheNeedle — one-hand limit
// =========================================================

/// The first hand during a TheNeedle Boss blind succeeds normally.
#[test]
fn test_the_needle_allows_first_hand() {
    let mut gs = boss_select(BossBlind::TheNeedle);
    gs.select_blind().unwrap();
    gs.score_goal = f64::MAX; // prevent win_round from firing
    gs.select_card(0).unwrap();
    assert!(gs.play_hand().is_ok(), "TheNeedle: first hand must succeed");
}

/// Attempting a second hand during a TheNeedle Boss blind returns BossBlindEffect.
#[test]
fn test_the_needle_blocks_second_hand_on_boss() {
    let mut gs = boss_select(BossBlind::TheNeedle);
    gs.select_blind().unwrap();
    gs.score_goal = f64::MAX;
    // First hand succeeds
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    // Second hand must fail
    gs.select_card(0).unwrap();
    let result = gs.play_hand();
    assert!(
        matches!(result, Err(BalatroError::BossBlindEffect(_))),
        "TheNeedle: second hand must return BossBlindEffect, got {:?}", result
    );
}

/// TheNeedle imposes no hand-count restriction on Small or Big blinds.
#[test]
fn test_the_needle_no_restriction_on_small_blind() {
    let mut gs = make_game();
    gs.boss_blind = Some(BossBlind::TheNeedle);
    // Small blind (default)
    gs.select_blind().unwrap();
    gs.score_goal = f64::MAX;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    // Second hand should not be restricted
    gs.select_card(0).unwrap();
    assert!(
        gs.play_hand().is_ok(),
        "TheNeedle must not restrict hands on a Small blind"
    );
}

#[test]
fn test_the_needle_no_restriction_on_big_blind() {
    let mut gs = make_game();
    gs.boss_blind = Some(BossBlind::TheNeedle);
    gs.current_blind = BlindKind::Big;
    gs.select_blind().unwrap();
    gs.score_goal = f64::MAX;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    gs.select_card(0).unwrap();
    assert!(
        gs.play_hand().is_ok(),
        "TheNeedle must not restrict hands on a Big blind"
    );
}

// =========================================================
// 8. TheTooth — -$1 per card played on Boss blind
// =========================================================

/// Playing 1 card against TheTooth deducts exactly $1.
#[test]
fn test_the_tooth_deducts_1_dollar_for_1_card() {
    let mut gs = boss_select(BossBlind::TheTooth);
    gs.select_blind().unwrap();
    gs.score_goal = f64::MAX;
    let money_before = gs.money;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert_eq!(gs.money, money_before - 1, "TheTooth: 1 card played → -$1");
}

/// Playing 5 cards against TheTooth deducts exactly $5.
#[test]
fn test_the_tooth_deducts_5_dollars_for_5_cards() {
    let mut gs = boss_select(BossBlind::TheTooth);
    gs.select_blind().unwrap();
    gs.score_goal = f64::MAX;
    let money_before = gs.money;
    for i in 0..5 {
        gs.select_card(i).unwrap();
    }
    gs.play_hand().unwrap();
    assert_eq!(gs.money, money_before - 5, "TheTooth: 5 cards played → -$5");
}

/// TheTooth deducts money based on the exact number of cards in that hand play.
#[test]
fn test_the_tooth_deducts_3_dollars_for_3_cards() {
    let mut gs = boss_select(BossBlind::TheTooth);
    gs.select_blind().unwrap();
    gs.score_goal = f64::MAX;
    let money_before = gs.money;
    for i in 0..3 {
        gs.select_card(i).unwrap();
    }
    gs.play_hand().unwrap();
    assert_eq!(gs.money, money_before - 3, "TheTooth: 3 cards played → -$3");
}

/// TheTooth does NOT deduct money when the blind is Small (only Boss blind).
#[test]
fn test_the_tooth_no_deduction_on_small_blind() {
    let mut gs = make_game();
    gs.boss_blind = Some(BossBlind::TheTooth);
    // Small blind — not Boss
    gs.select_blind().unwrap();
    gs.score_goal = f64::MAX;
    let money_before = gs.money;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert_eq!(gs.money, money_before, "TheTooth must not deduct money on a Small blind");
}

/// TheTooth does NOT deduct money when the blind is Big.
#[test]
fn test_the_tooth_no_deduction_on_big_blind() {
    let mut gs = make_game();
    gs.boss_blind = Some(BossBlind::TheTooth);
    gs.current_blind = BlindKind::Big;
    gs.select_blind().unwrap();
    gs.score_goal = f64::MAX;
    let money_before = gs.money;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert_eq!(gs.money, money_before, "TheTooth must not deduct money on a Big blind");
}

// =========================================================
// 9. ThePsychic — forces ≥5 card hand draw
// =========================================================

/// ThePsychic forces the hand to contain at least 5 cards, even when hand_size < 5.
#[test]
fn test_the_psychic_forces_minimum_5_card_hand() {
    let mut gs = boss_select(BossBlind::ThePsychic);
    gs.hand_size = 3; // explicitly below 5
    gs.select_blind().unwrap();
    assert!(
        gs.hand.len() >= 5,
        "ThePsychic should force hand size to at least 5; got {}", gs.hand.len()
    );
}

/// ThePsychic does not reduce a hand that is already larger than 5.
/// Blue deck default hand_size = 8 → hand should stay at 8.
#[test]
fn test_the_psychic_does_not_reduce_large_hand() {
    let mut gs = boss_select(BossBlind::ThePsychic);
    // Default Blue deck hand_size = 8
    gs.select_blind().unwrap();
    assert_eq!(
        gs.hand.len(), 8,
        "ThePsychic must not reduce hand below the normal hand_size (8)"
    );
}

/// Without ThePsychic, hand_size = 3 draws exactly 3 cards.
#[test]
fn test_without_the_psychic_hand_size_3_draws_3() {
    let mut gs = boss_select(BossBlind::TheOx); // non-psychic boss
    gs.hand_size = 3;
    gs.select_blind().unwrap();
    assert_eq!(
        gs.hand.len(), 3,
        "without ThePsychic, hand_size=3 should draw exactly 3 cards"
    );
}

// =========================================================
// 10. Chip goal multipliers
// =========================================================
// Ante 1 base = 300. Boss goal = base × multiplier.

/// TheWall uses a 4× chip multiplier → goal = 300 × 4 = 1200.
#[test]
fn test_the_wall_chip_goal_is_4x() {
    let mut gs = boss_select(BossBlind::TheWall);
    assert_eq!(gs.get_blind_chip_goal() as i64, 1200, "TheWall: 300 × 4 = 1200");
}

/// TheNeedle uses a 1× chip multiplier → goal = 300 × 1 = 300.
#[test]
fn test_the_needle_chip_goal_is_1x() {
    let mut gs = boss_select(BossBlind::TheNeedle);
    assert_eq!(gs.get_blind_chip_goal() as i64, 300, "TheNeedle: 300 × 1 = 300");
}

/// VioletVessel uses a 6× chip multiplier → goal = 300 × 6 = 1800.
#[test]
fn test_violet_vessel_chip_goal_is_6x() {
    let mut gs = boss_select(BossBlind::VioletVessel);
    assert_eq!(gs.get_blind_chip_goal() as i64, 1800, "VioletVessel: 300 × 6 = 1800");
}

/// All other boss blinds use the standard 2× chip multiplier → goal = 300 × 2 = 600.
#[test]
fn test_all_standard_bosses_have_2x_chip_goal() {
    let standard_bosses = [
        BossBlind::TheOx, BossBlind::TheHook, BossBlind::TheMouth, BossBlind::TheFish,
        BossBlind::TheClub, BossBlind::TheManacle, BossBlind::TheTooth, BossBlind::TheHouse,
        BossBlind::TheMark, BossBlind::TheWheel, BossBlind::TheArm, BossBlind::ThePsychic,
        BossBlind::TheGoad, BossBlind::TheWater, BossBlind::TheEye, BossBlind::ThePlant,
        BossBlind::TheHead, BossBlind::TheWindow, BossBlind::TheSerpent, BossBlind::ThePillar,
        BossBlind::TheFlint, BossBlind::CeruleanBell, BossBlind::VerdantLeaf,
        BossBlind::AmberAcorn, BossBlind::CrimsonHeart,
    ];
    for boss in standard_bosses {
        let mut gs = boss_select(boss);
        assert_eq!(
            gs.get_blind_chip_goal() as i64, 600,
            "{:?} should have 2× chip goal (600)", boss
        );
    }
}

// =========================================================
// 11. Boss blind dollar rewards
// =========================================================
// Interest formula: floor((money_before + reward) / 5), capped at max_interest/5 = 5.
// make_game() money = 4; dollars_earned from scoring = 0 for simple hands.

/// Helper: win a boss blind and return the money delta (reward + interest).
fn win_boss_and_get_money_delta(boss: BossBlind) -> i32 {
    let mut gs = boss_select(boss);
    gs.select_blind().unwrap();
    gs.score_goal = 1.0;
    let money_before = gs.money;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    gs.money - money_before
}

/// Regular boss blinds (non-showdown) award $5 on defeat.
#[test]
fn test_regular_boss_blind_awards_5_dollars() {
    // TheOx is a representative regular boss
    let money_before = 4_i32;
    let reward = 5_i32;
    let interest = (money_before + reward) / 5; // 9/5 = 1
    let expected_delta = reward + interest;       // 5 + 1 = 6
    assert_eq!(
        win_boss_and_get_money_delta(BossBlind::TheOx), expected_delta,
        "regular boss should award $5 + $1 interest = $6 delta"
    );
}

/// CeruleanBell (showdown) awards $8 on defeat.
#[test]
fn test_cerulean_bell_awards_8_dollars() {
    let money_before = 4_i32;
    let reward = 8_i32;
    let interest = (money_before + reward) / 5; // 12/5 = 2
    let expected_delta = reward + interest;
    assert_eq!(
        win_boss_and_get_money_delta(BossBlind::CeruleanBell), expected_delta,
        "CeruleanBell should award $8 + $2 interest = $10 delta"
    );
}

/// VerdantLeaf (showdown) awards $8 on defeat.
#[test]
fn test_verdant_leaf_awards_8_dollars() {
    let money_before = 4_i32;
    let reward = 8_i32;
    let interest = (money_before + reward) / 5;
    assert_eq!(
        win_boss_and_get_money_delta(BossBlind::VerdantLeaf), reward + interest
    );
}

/// VioletVessel (showdown) awards $8 on defeat.
#[test]
fn test_violet_vessel_awards_8_dollars() {
    let money_before = 4_i32;
    let reward = 8_i32;
    let interest = (money_before + reward) / 5;
    assert_eq!(
        win_boss_and_get_money_delta(BossBlind::VioletVessel), reward + interest
    );
}

/// AmberAcorn (showdown) awards $8 on defeat.
#[test]
fn test_amber_acorn_awards_8_dollars() {
    let money_before = 4_i32;
    let reward = 8_i32;
    let interest = (money_before + reward) / 5;
    assert_eq!(
        win_boss_and_get_money_delta(BossBlind::AmberAcorn), reward + interest
    );
}

/// CrimsonHeart (showdown) awards $8 on defeat.
#[test]
fn test_crimson_heart_awards_8_dollars() {
    let money_before = 4_i32;
    let reward = 8_i32;
    let interest = (money_before + reward) / 5;
    assert_eq!(
        win_boss_and_get_money_delta(BossBlind::CrimsonHeart), reward + interest
    );
}

/// Showdown bosses award $3 more than regular bosses (8 − 5 = 3).
#[test]
fn test_showdown_bosses_award_3_more_than_regular() {
    let delta_regular  = win_boss_and_get_money_delta(BossBlind::TheOx);
    let delta_showdown = win_boss_and_get_money_delta(BossBlind::CeruleanBell);
    // Interest differs too: regular 9/5=1, showdown 12/5=2, so delta showdown = 10, regular = 6
    assert_eq!(
        delta_showdown - delta_regular, 4,
        "showdown boss should net $4 more than a regular boss (3 extra reward + 1 extra interest)"
    );
}

// =========================================================
// 12. No-op bosses — score_hand result unchanged
// =========================================================
// Passing these bosses to score_hand must produce the same final_score as None.
// (Only TheFlint modifies score_hand output; all others pass through `_ => {}`.)

#[test]
fn test_the_ox_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheOx).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_hook_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheHook).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_mouth_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheMouth).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_fish_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheFish).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_manacle_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheManacle).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_wall_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheWall).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_house_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheHouse).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_wheel_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheWheel).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_arm_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheArm).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_water_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheWater).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_eye_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheEye).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_serpent_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheSerpent).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_pillar_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::ThePillar).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_needle_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheNeedle).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_the_tooth_does_not_modify_score_calculation() {
    // TheTooth deducts money in play_hand(), not inside score_hand() — score result unchanged.
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::TheTooth).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_cerulean_bell_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::CeruleanBell).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_verdant_leaf_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::VerdantLeaf).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_violet_vessel_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::VioletVessel).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_amber_acorn_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::AmberAcorn).final_score, score_baseline(&p).final_score);
}

#[test]
fn test_crimson_heart_does_not_modify_score() {
    let p = flush_spades();
    assert_eq!(score_with_boss(&p, BossBlind::CrimsonHeart).final_score, score_baseline(&p).final_score);
}
