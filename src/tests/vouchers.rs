/// Tests for voucher effects via GameState.
///
/// Also covers related card-enhancement and joker behaviors that require GameState round flow:
///   - Observatory voucher: X1.5 Mult per planet card used, stacks per use
///   - Gold Card enhancement: $3 per Gold card held in hand at end of round
///   - Campfire joker: resets x_mult to X1 when Boss Blind is defeated

use super::*;

// Helper: apply a voucher directly to a GameState
fn apply_voucher_to_game(voucher: VoucherKind) -> GameState {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.shop_voucher = Some(voucher);
    gs.money = 50; // plenty of money
    gs.buy_voucher().unwrap();
    gs
}

// =========================================================
// Consumable slot vouchers
// =========================================================

#[test]
fn test_crystal_ball_increases_consumable_slots() {
    let gs_before = make_game();
    let base = gs_before.consumable_slots;
    let gs = apply_voucher_to_game(VoucherKind::CrystalBall);
    assert_eq!(gs.consumable_slots, base + 1);
}

// =========================================================
// Hand count vouchers
// =========================================================

#[test]
fn test_grabber_increases_max_hands() {
    let gs_before = make_game();
    let base = gs_before.max_hands;
    let gs = apply_voucher_to_game(VoucherKind::Grabber);
    assert_eq!(gs.max_hands, base + 1);
}

#[test]
fn test_nacho_tong_increases_max_hands() {
    let gs_before = make_game();
    let base = gs_before.max_hands;
    let gs = apply_voucher_to_game(VoucherKind::NachoTong);
    assert_eq!(gs.max_hands, base + 1);
}

// =========================================================
// Discard count vouchers
// =========================================================

#[test]
fn test_wasteful_increases_max_discards() {
    let gs_before = make_game();
    let base = gs_before.max_discards;
    let gs = apply_voucher_to_game(VoucherKind::Wasteful);
    assert_eq!(gs.max_discards, base + 1);
}

#[test]
fn test_recyclomancy_increases_max_discards() {
    let gs_before = make_game();
    let base = gs_before.max_discards;
    let gs = apply_voucher_to_game(VoucherKind::Recyclomancy);
    assert_eq!(gs.max_discards, base + 1);
}

// =========================================================
// Interest cap vouchers
// =========================================================

#[test]
fn test_seed_money_increases_max_interest() {
    let gs_before = make_game();
    let base = gs_before.max_interest;
    let gs = apply_voucher_to_game(VoucherKind::SeedMoney);
    assert_eq!(gs.max_interest, base + 10);
}

#[test]
fn test_money_tree_increases_max_interest() {
    let gs_before = make_game();
    let base = gs_before.max_interest;
    let gs = apply_voucher_to_game(VoucherKind::MoneyTree);
    assert_eq!(gs.max_interest, base + 10);
}

// =========================================================
// Joker slot vouchers
// =========================================================

#[test]
fn test_blank_increases_joker_slots() {
    let gs_before = make_game();
    let base = gs_before.joker_slots;
    let gs = apply_voucher_to_game(VoucherKind::Blank);
    assert_eq!(gs.joker_slots, base + 1);
}

#[test]
fn test_antimatter_increases_joker_slots() {
    let gs_before = make_game();
    let base = gs_before.joker_slots;
    let gs = apply_voucher_to_game(VoucherKind::Antimatter);
    assert_eq!(gs.joker_slots, base + 1);
}

// =========================================================
// Hand size vouchers
// =========================================================

#[test]
fn test_paint_brush_increases_hand_size() {
    let gs_before = make_game();
    let base = gs_before.hand_size;
    let gs = apply_voucher_to_game(VoucherKind::PaintBrush);
    assert_eq!(gs.hand_size, base + 1);
}

#[test]
fn test_palette_increases_hand_size() {
    let gs_before = make_game();
    let base = gs_before.hand_size;
    let gs = apply_voucher_to_game(VoucherKind::Palette);
    assert_eq!(gs.hand_size, base + 1);
}

// =========================================================
// Reroll vouchers
// =========================================================

#[test]
fn test_directors_cut_gives_free_reroll() {
    let gs_before = make_game();
    let base = gs_before.free_rerolls;
    let gs = apply_voucher_to_game(VoucherKind::DirectorsCut);
    assert_eq!(gs.free_rerolls, base + 1);
}

// =========================================================
// Stacking: two vouchers of same type stack
// =========================================================

#[test]
fn test_grabber_and_nacho_tong_stack() {
    let gs_before = make_game();
    let base = gs_before.max_hands;
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.money = 100;
    // Apply Grabber
    gs.shop_voucher = Some(VoucherKind::Grabber);
    gs.buy_voucher().unwrap();
    assert_eq!(gs.max_hands, base + 1);
    // Apply NachoTong
    gs.state = GameStateKind::Shop;
    gs.shop_voucher = Some(VoucherKind::NachoTong);
    gs.buy_voucher().unwrap();
    assert_eq!(gs.max_hands, base + 2);
}

// =========================================================
// Observatory voucher — X1.5 Mult per planet use for that hand
// =========================================================

/// Using a planet card with Observatory sets observatory_x_mult to X1.5.
#[test]
fn test_observatory_sets_x_mult_on_planet_use() {
    use crate::types::{PlanetCard, HandType};
    let mut gs = apply_voucher_to_game(VoucherKind::Observatory);
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Jupiter));
    gs.use_consumable(0, vec![]).unwrap();
    let flush_level = gs.hand_levels.get(&HandType::Flush).unwrap();
    assert!(
        (flush_level.observatory_x_mult - 1.5).abs() < 0.001,
        "Observatory: first planet use must set x_mult to 1.5, got {}",
        flush_level.observatory_x_mult
    );
}

/// Using two planet cards with Observatory stacks to X2.25 (1.5²).
#[test]
fn test_observatory_stacks_per_planet_use() {
    use crate::types::{PlanetCard, HandType};
    let mut gs = apply_voucher_to_game(VoucherKind::Observatory);
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Jupiter));
    gs.use_consumable(0, vec![]).unwrap();
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Jupiter));
    gs.use_consumable(0, vec![]).unwrap();
    let flush_level = gs.hand_levels.get(&HandType::Flush).unwrap();
    assert!(
        (flush_level.observatory_x_mult - 2.25).abs() < 0.001,
        "Observatory: two planet uses must give x_mult 2.25, got {}",
        flush_level.observatory_x_mult
    );
}

/// Without Observatory, planet use does not change observatory_x_mult.
#[test]
fn test_no_observatory_planet_does_not_set_x_mult() {
    use crate::types::{PlanetCard, HandType};
    let mut gs = make_game();
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Jupiter));
    gs.use_consumable(0, vec![]).unwrap();
    let flush_level = gs.hand_levels.get(&HandType::Flush).unwrap();
    assert!(
        (flush_level.observatory_x_mult - 1.0).abs() < 0.001,
        "Without Observatory, observatory_x_mult must stay 1.0"
    );
}

/// Observatory X1.5 actually increases Flush score during scoring.
#[test]
fn test_observatory_increases_flush_score() {
    use crate::types::{PlanetCard, HandType};
    // Jupiter levels Flush from L1→L2: chips = 35+15=50, mult = 4+2=6
    // Observatory X1.5 applied: mult = 6×1.5 = 9
    // 5 Spades 2-3-4-5-7: card chips = 2+3+4+5+7 = 21 → total chips = 71
    // score = 71 × 9 = 639
    //
    // Without Observatory (same planet used): mult = 6, score = 71 × 6 = 426
    let played = vec![
        card(0, Rank::Two,   Suit::Spades),
        card(1, Rank::Three, Suit::Spades),
        card(2, Rank::Four,  Suit::Spades),
        card(3, Rank::Five,  Suit::Spades),
        card(4, Rank::Seven, Suit::Spades),
    ];

    // Without Observatory: Jupiter just levels up (L2), score = 71×6 = 426
    let mut gs_no_obs = make_game();
    gs_no_obs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Jupiter));
    gs_no_obs.use_consumable(0, vec![]).unwrap();
    let result_no_obs = score_hand(
        &played, &[], &[], &gs_no_obs.hand_levels,
        3, 3, 0, 40, 52, None, 5, 0, 0,
    );
    assert_eq!(result_no_obs.final_score as i64, 426,
        "Without Observatory: L2 Flush score should be 426");

    // With Observatory: same planet use also gives X1.5, score = 71×9 = 639
    let mut gs = apply_voucher_to_game(VoucherKind::Observatory);
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Jupiter));
    gs.use_consumable(0, vec![]).unwrap();
    let result = score_hand(
        &played, &[], &[], &gs.hand_levels,
        3, 3, 0, 40, 52, None, 5, 0, 0,
    );
    assert_eq!(result.final_score as i64, 639,
        "Observatory should give X1.5 on Flush: expected 639, got {}",
        result.final_score as i64
    );
}

// =========================================================
// Gold Card enhancement — $3 per card held in hand at round end
// =========================================================

/// Gold Card enhancement pays $3 per Gold card held in hand when a round is won.
#[test]
fn test_gold_card_enhancement_pays_3_dollars_at_round_end() {
    let mut gs = make_game();
    // Build a round with 2 Gold cards and 1 normal card
    let mut gold1 = card(1, Rank::Ace, Suit::Spades);
    gold1.enhancement = Enhancement::Gold;
    let mut gold2 = card(2, Rank::King, Suit::Spades);
    gold2.enhancement = Enhancement::Gold;
    let normal = card(3, Rank::Two, Suit::Spades);
    setup_round(&mut gs, vec![gold1, gold2, normal], 3);
    // Win the round immediately (score_goal = 0 would trigger win on any play,
    // but we use the GameState's field directly)
    // Play the normal card to trigger score; set goal to 1 chip
    gs.score_goal = 1.0;
    gs.select_card(2).unwrap(); // index 2 = normal card (Two of Spades)
    let money_before = gs.money;
    gs.play_hand().unwrap();
    // After win_round: 2 Gold cards in hand → +$6; also interest and blind reward
    // Baseline: blind reward for Small = $3 (White stake), interest on (4+3)/5=1
    // gold: +$6
    // We only care that the delta includes the $6 from Gold cards
    let delta = gs.money - money_before;
    assert!(delta >= 6, "Gold Card should pay $3 per Gold card held; expected delta ≥ $6, got {}", delta);
}

/// Gold Card enhancement does NOT pay if the card is debuffed.
#[test]
fn test_gold_card_debuffed_does_not_pay() {
    let mut gs = make_game();
    let mut gold = card(1, Rank::Ace, Suit::Spades);
    gold.enhancement = Enhancement::Gold;
    gold.debuffed = true;
    let trigger = card(2, Rank::Two, Suit::Spades);
    setup_round(&mut gs, vec![gold, trigger], 2);
    gs.score_goal = 1.0;
    gs.select_card(1).unwrap(); // play the normal card
    let money_before = gs.money;
    gs.play_hand().unwrap();
    // Delta should NOT include $3 from the debuffed Gold card
    // Blind reward ($3 White Small) + interest (4+3)/5 = 1 → delta = 4
    // If Gold paid despite debuff, delta would be 7
    let delta = gs.money - money_before;
    assert!(delta < 7, "Debuffed Gold card must not pay $3; got delta {}", delta);
}

// =========================================================
// Campfire joker — resets x_mult to X1 on Boss Blind defeat
// =========================================================

/// Campfire x_mult is reset to X1 when a Boss Blind is won.
#[test]
fn test_campfire_resets_x_mult_on_boss_blind_defeat() {
    use crate::game::BlindKind;
    let mut gs = make_game();
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(BossBlind::TheOx);
    let mut campfire = joker(1, JokerKind::Campfire);
    campfire.set_counter_f64("x_mult", 3.5); // boosted from selling cards
    gs.jokers.push(campfire);
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.score_goal = 1.0;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    // Round won → Campfire x_mult must be reset to 1.0
    assert!(
        (gs.jokers[0].get_counter_f64("x_mult") - 1.0).abs() < 0.001,
        "Campfire must reset to X1 after Boss Blind defeat"
    );
}

/// Campfire x_mult is NOT reset when a Small or Big Blind is won.
#[test]
fn test_campfire_not_reset_on_small_blind_win() {
    let mut gs = make_game();
    // default: Small blind
    let mut campfire = joker(1, JokerKind::Campfire);
    campfire.set_counter_f64("x_mult", 2.5);
    gs.jokers.push(campfire);
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.score_goal = 1.0;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert!(
        (gs.jokers[0].get_counter_f64("x_mult") - 2.5).abs() < 0.001,
        "Campfire must NOT reset on Small Blind win"
    );
}
