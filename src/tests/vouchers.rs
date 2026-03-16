/// Tests for voucher effects via GameState.

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
