/// Tests for stake-level mechanics.
///
/// Stakes (in ascending order): White=0, Red=1, Green=2, Black=3, Blue=4, Purple=5, Orange=6, Gold=7
///
/// What each stake unlocks:
///   - Red+:    Eternal stickers can appear on shop jokers (~5%)
///   - Green+:  Rental stickers can also appear on shop jokers (~5%)
///   - Black+:  -1 effective discard per round
///   - Blue+:   Perishable stickers can also appear on shop jokers (~5%)
///   - Purple+: Spectral packs appear in the shop
///   - Orange, Gold: same mechanical effects as Purple in the current engine

use super::*;
use crate::card::ShopItem;
use crate::types::PackKind;

// =========================================================
// Stake ordering
// =========================================================

#[test]
fn test_stake_ordering_is_correct() {
    assert!((Stake::White as u8) < (Stake::Red as u8));
    assert!((Stake::Red as u8) < (Stake::Green as u8));
    assert!((Stake::Green as u8) < (Stake::Black as u8));
    assert!((Stake::Black as u8) < (Stake::Blue as u8));
    assert!((Stake::Blue as u8) < (Stake::Purple as u8));
    assert!((Stake::Purple as u8) < (Stake::Orange as u8));
    assert!((Stake::Orange as u8) < (Stake::Gold as u8));
}

// =========================================================
// White stake: baseline — no stickers, full discards
// =========================================================

#[test]
fn test_white_stake_never_produces_stickered_jokers() {
    // White has an empty sticker pool; no sticker can ever be assigned regardless of RNG.
    let mut gs = GameState::new(DeckType::Blue, Stake::White, Some("WHITE_STICKER".to_string()));
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                assert!(!j.eternal,    "White stake should never produce eternal jokers");
                assert!(!j.rental,     "White stake should never produce rental jokers");
                assert!(!j.perishable, "White stake should never produce perishable jokers");
            }
        }
    }
}

#[test]
fn test_white_stake_effective_discards_equal_to_max() {
    let gs = GameState::new(DeckType::Blue, Stake::White, Some("WD".to_string()));
    assert_eq!(gs.effective_max_discards(), gs.max_discards,
        "White stake should not reduce discards");
}

#[test]
fn test_white_stake_shop_has_no_spectral_pack() {
    let mut gs = GameState::new(DeckType::Blue, Stake::White, Some("WHITE_SPEC".to_string()));
    gs.generate_shop();
    let has_spectral = gs.shop_offers.iter().any(|o| {
        matches!(o.kind, ShopItem::Pack(PackKind::SpectralPack))
    });
    assert!(!has_spectral, "White stake shop should not contain a SpectralPack");
}

// =========================================================
// Red stake: Eternal stickers only
// =========================================================

#[test]
fn test_red_stake_never_produces_rental_or_perishable_jokers() {
    // Red's sticker pool only contains Eternal (index 0). Rental and Perishable are
    // structurally impossible regardless of RNG outcome.
    let mut gs = GameState::new(DeckType::Blue, Stake::Red, Some("RED_RENTAL".to_string()));
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                assert!(!j.rental,     "Red stake should never produce rental jokers");
                assert!(!j.perishable, "Red stake should never produce perishable jokers");
            }
        }
    }
}

#[test]
fn test_red_stake_can_produce_eternal_jokers() {
    // ~5% per joker. Over 500 shops (≈1000 jokers) P(no eternal) < 10^-22.
    let mut gs = GameState::new(DeckType::Blue, Stake::Red, Some("RED_ETERNAL".to_string()));
    let mut found = false;
    for _ in 0..500 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                if j.eternal { found = true; }
            }
        }
        if found { break; }
    }
    assert!(found, "Red stake should eventually produce eternal jokers");
}

// =========================================================
// Green stake: Eternal + Rental stickers
// =========================================================

#[test]
fn test_green_stake_never_produces_perishable_jokers() {
    // Green's sticker pool is [Eternal, Rental]. Perishable requires Blue+.
    let mut gs = GameState::new(DeckType::Blue, Stake::Green, Some("GREEN_PERISH".to_string()));
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                assert!(!j.perishable, "Green stake should never produce perishable jokers");
            }
        }
    }
}

#[test]
fn test_green_stake_can_produce_rental_jokers() {
    // ~4.9% per joker for rental specifically. Over 500 shops P(none) < 10^-22.
    let mut gs = GameState::new(DeckType::Blue, Stake::Green, Some("GREEN_RENTAL".to_string()));
    let mut found = false;
    for _ in 0..500 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                if j.rental { found = true; }
            }
        }
        if found { break; }
    }
    assert!(found, "Green stake should eventually produce rental jokers");
}

// =========================================================
// Black stake: same stickers as Green, but -1 discard
// =========================================================

#[test]
fn test_black_stake_effective_discards_reduced_by_one() {
    let gs = GameState::new(DeckType::Blue, Stake::Black, Some("BLACK_DISC".to_string()));
    assert_eq!(
        gs.effective_max_discards(),
        gs.max_discards.saturating_sub(1),
        "Black stake should reduce effective discards by 1"
    );
}

#[test]
fn test_black_stake_never_produces_perishable_jokers() {
    // Black = 3, Blue = 4; Perishable threshold is Blue+.
    let mut gs = GameState::new(DeckType::Blue, Stake::Black, Some("BLACK_PERISH".to_string()));
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                assert!(!j.perishable, "Black stake should never produce perishable jokers");
            }
        }
    }
}

// =========================================================
// Blue stake: all sticker types possible
// =========================================================

#[test]
fn test_blue_stake_can_produce_perishable_jokers() {
    // Blue is the first stake where Perishable is available. ~5% per joker.
    let mut gs = GameState::new(DeckType::Blue, Stake::Blue, Some("BLUE_PERISH".to_string()));
    let mut found = false;
    for _ in 0..500 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                if j.perishable { found = true; }
            }
        }
        if found { break; }
    }
    assert!(found, "Blue stake should eventually produce perishable jokers");
}

#[test]
fn test_blue_stake_effective_discards_reduced_by_one() {
    // Blue >= Black, so the -1 discard penalty applies.
    let gs = GameState::new(DeckType::Blue, Stake::Blue, Some("BLUE_DISC".to_string()));
    assert_eq!(
        gs.effective_max_discards(),
        gs.max_discards.saturating_sub(1),
        "Blue stake (>= Black) should reduce effective discards by 1"
    );
}

// =========================================================
// Purple stake: spectral packs in shop
// =========================================================

#[test]
fn test_purple_stake_shop_contains_spectral_pack() {
    let mut gs = GameState::new(DeckType::Blue, Stake::Purple, Some("PURPLE_SPEC".to_string()));
    gs.generate_shop();
    let has_spectral = gs.shop_offers.iter().any(|o| {
        matches!(o.kind, ShopItem::Pack(PackKind::SpectralPack))
    });
    assert!(has_spectral, "Purple stake shop should always contain a SpectralPack");
}

#[test]
fn test_purple_stake_effective_discards_reduced_by_one() {
    let gs = GameState::new(DeckType::Blue, Stake::Purple, Some("PURPLE_DISC".to_string()));
    assert_eq!(
        gs.effective_max_discards(),
        gs.max_discards.saturating_sub(1),
        "Purple stake (>= Black) should reduce effective discards by 1"
    );
}

// =========================================================
// Orange stake
// =========================================================

#[test]
fn test_orange_stake_shop_contains_spectral_pack() {
    let mut gs = GameState::new(DeckType::Blue, Stake::Orange, Some("ORANGE_SPEC".to_string()));
    gs.generate_shop();
    let has_spectral = gs.shop_offers.iter().any(|o| {
        matches!(o.kind, ShopItem::Pack(PackKind::SpectralPack))
    });
    assert!(has_spectral, "Orange stake shop should contain a SpectralPack");
}

#[test]
fn test_orange_stake_effective_discards_reduced_by_one() {
    let gs = GameState::new(DeckType::Blue, Stake::Orange, Some("ORANGE_DISC".to_string()));
    assert_eq!(
        gs.effective_max_discards(),
        gs.max_discards.saturating_sub(1),
        "Orange stake (>= Black) should reduce effective discards by 1"
    );
}

// =========================================================
// Gold stake
// =========================================================

#[test]
fn test_gold_stake_shop_contains_spectral_pack() {
    let mut gs = GameState::new(DeckType::Blue, Stake::Gold, Some("GOLD_SPEC".to_string()));
    gs.generate_shop();
    let has_spectral = gs.shop_offers.iter().any(|o| {
        matches!(o.kind, ShopItem::Pack(PackKind::SpectralPack))
    });
    assert!(has_spectral, "Gold stake shop should contain a SpectralPack");
}

#[test]
fn test_gold_stake_effective_discards_reduced_by_one() {
    let gs = GameState::new(DeckType::Blue, Stake::Gold, Some("GOLD_DISC".to_string()));
    assert_eq!(
        gs.effective_max_discards(),
        gs.max_discards.saturating_sub(1),
        "Gold stake (>= Black) should reduce effective discards by 1"
    );
}

#[test]
fn test_gold_stake_can_produce_all_sticker_types() {
    // Gold >= Blue, so all three sticker types are possible.
    let mut gs = GameState::new(DeckType::Blue, Stake::Gold, Some("GOLD_STICKERS".to_string()));
    let (mut found_eternal, mut found_rental, mut found_perishable) = (false, false, false);
    for _ in 0..500 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                if j.eternal    { found_eternal = true; }
                if j.rental     { found_rental = true; }
                if j.perishable { found_perishable = true; }
            }
        }
        if found_eternal && found_rental && found_perishable { break; }
    }
    assert!(found_eternal,    "Gold stake should eventually produce eternal jokers");
    assert!(found_rental,     "Gold stake should eventually produce rental jokers");
    assert!(found_perishable, "Gold stake should eventually produce perishable jokers");
}
