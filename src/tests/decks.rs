/// Tests for deck-specific mechanics.

use super::*;

#[test]
fn test_green_deck_no_interest_on_win() {
    // Green deck: win_round gives $2/remaining hand + $1/remaining discard, no interest.
    let mut gs = GameState::new(DeckType::Green, Stake::White, Some("GREEN1".to_string()));
    setup_round(
        &mut gs,
        (0..52).map(|i| card(i as u64, Rank::Ace, Suit::Spades)).collect(),
        5,
    );
    gs.score_goal = 1.0; // trivially beat
    gs.money = 20;

    let hands_left = gs.hands_remaining;
    let discards_left = gs.discards_remaining;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    let expected_hands_bonus = 2 * (hands_left - 1) as i32;
    let expected_discards_bonus = discards_left as i32;
    let blind_reward = 3i32;

    let expected_money = 20 + blind_reward + expected_hands_bonus + expected_discards_bonus;
    assert_eq!(
        gs.money, expected_money,
        "Green deck money: got {}, expected {}",
        gs.money, expected_money
    );
}

#[test]
fn test_plasma_deck_balances_chips_and_mult() {
    // Plasma deck: final_score = avg(chips,mult)^2 instead of chips*mult.
    // Pair of Aces without Plasma: chips=32, mult=2 → 64
    // With Plasma: avg = (32+2)/2 = 17 → 17*17 = 289
    let mut gs = GameState::new(DeckType::Plasma, Stake::White, Some("PLASMA1".to_string()));
    setup_round(
        &mut gs,
        vec![
            card(100, Rank::Ace, Suit::Spades),
            card(101, Rank::Ace, Suit::Hearts),
            card(102, Rank::Two, Suit::Clubs),
        ],
        3,
    );
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.select_card(1).unwrap();
    gs.play_hand().unwrap();

    assert!(
        (gs.score_accumulated - 289.0).abs() < 1.0,
        "Plasma score: got {}, expected ~289",
        gs.score_accumulated
    );
}

#[test]
fn test_black_deck_has_one_fewer_hand() {
    let gs = GameState::new(DeckType::Black, Stake::White, Some("BLK".to_string()));
    let gs_blue = GameState::new(DeckType::Blue, Stake::White, Some("BLU".to_string()));
    // Black: 4 - 1 = 3 hands; Blue: 4 + 1 = 5 hands
    assert_eq!(gs.max_hands, 3);
    assert_eq!(gs_blue.max_hands, 5);
}

#[test]
fn test_abandoned_deck_has_no_face_cards() {
    let gs = GameState::new(DeckType::Abandoned, Stake::White, Some("ABN".to_string()));
    let has_face = gs.deck.iter().any(|c| c.rank.is_face());
    assert!(!has_face, "Abandoned deck should have no face cards");
    // 52 - 12 face cards (J,Q,K × 4 suits) = 40 cards
    assert_eq!(gs.deck.len(), 40);
}

#[test]
fn test_checkered_deck_has_only_spades_and_hearts() {
    let gs = GameState::new(DeckType::Checkered, Stake::White, Some("CHK".to_string()));
    for c in &gs.deck {
        assert!(
            c.suit == Suit::Spades || c.suit == Suit::Hearts,
            "Checkered deck card has unexpected suit: {:?}",
            c.suit
        );
    }
    // 26 Spades + 26 Hearts = 52 total
    assert_eq!(gs.deck.len(), 52);
}

#[test]
fn test_magic_deck_starts_with_crystal_ball_and_fools() {
    let gs = GameState::new(DeckType::Magic, Stake::White, Some("MAG".to_string()));
    assert!(gs.vouchers.contains(&VoucherKind::CrystalBall));
    let fool_count = gs.consumables.iter().filter(|c| {
        matches!(c, crate::card::ConsumableCard::Tarot(TarotCard::TheFool))
    }).count();
    assert_eq!(fool_count, 2, "Magic deck should start with 2 Fools");
}

#[test]
fn test_nebula_deck_starts_with_telescope() {
    let gs = GameState::new(DeckType::Nebula, Stake::White, Some("NEB".to_string()));
    assert!(gs.vouchers.contains(&VoucherKind::Telescope));
}

#[test]
fn test_zodiac_deck_starts_with_three_vouchers() {
    let gs = GameState::new(DeckType::Zodiac, Stake::White, Some("ZOD".to_string()));
    assert!(gs.vouchers.contains(&VoucherKind::TarotMerchant));
    assert!(gs.vouchers.contains(&VoucherKind::PlanetMerchant));
    assert!(gs.vouchers.contains(&VoucherKind::Overstock));
}

// =========================================================
// Blind scaling: stake tier and Plasma's ante_scaling
// =========================================================

fn ante1_small_goal(deck: DeckType, stake: Stake) -> f64 {
    let gs = GameState::new(deck, stake, Some("SCALING".to_string()));
    gs.get_blind_chip_goal()
}

#[test]
fn test_blind_sizes_scale_with_stake() {
    // misc_functions.lua:919 — three tables, selected by stake (game.lua:2053, :2056).
    let white = GameState::new(DeckType::Red, Stake::White, Some("S".to_string()));
    let green = GameState::new(DeckType::Red, Stake::Green, Some("S".to_string()));
    let purple = GameState::new(DeckType::Red, Stake::Purple, Some("S".to_string()));

    let goal = |gs: &GameState, ante: u32| {
        crate::game::get_base_blind_amount_scaled(ante, crate::game::blind_scaling_tier(gs.stake))
    };
    assert_eq!(goal(&white, 2), 800);
    assert_eq!(goal(&green, 2), 900);
    assert_eq!(goal(&purple, 2), 1000);
    assert_eq!(goal(&white, 8), 50000);
    assert_eq!(goal(&green, 8), 100000);
    assert_eq!(goal(&purple, 8), 200000);
}

#[test]
fn test_black_stake_uses_the_green_scaling_tier() {
    // Tiers are >=, so Black/Blue sit on tier 2 and Orange/Gold on tier 3.
    assert_eq!(crate::game::blind_scaling_tier(Stake::Black), 2);
    assert_eq!(crate::game::blind_scaling_tier(Stake::Blue), 2);
    assert_eq!(crate::game::blind_scaling_tier(Stake::Orange), 3);
    assert_eq!(crate::game::blind_scaling_tier(Stake::Gold), 3);
}

#[test]
fn test_plasma_deck_doubles_the_blind_requirement() {
    let plain = ante1_small_goal(DeckType::Red, Stake::White);
    let plasma = ante1_small_goal(DeckType::Plasma, Stake::White);
    assert_eq!(plasma, plain * 2.0);
}

// =========================================================
// Deck starting state
// =========================================================

#[test]
fn test_nebula_deck_trades_a_consumable_slot_for_telescope() {
    let plain = GameState::new(DeckType::Red, Stake::White, Some("N".to_string()));
    let nebula = GameState::new(DeckType::Nebula, Stake::White, Some("N".to_string()));
    assert!(nebula.vouchers.contains(&VoucherKind::Telescope));
    assert_eq!(nebula.consumable_slots, plain.consumable_slots - 1);
}

#[test]
fn test_ghost_deck_starts_with_a_hex() {
    let gs = GameState::new(DeckType::Ghost, Stake::White, Some("G".to_string()));
    assert_eq!(
        gs.consumables,
        vec![crate::card::ConsumableCard::Spectral(SpectralCard::Hex)]
    );
}
