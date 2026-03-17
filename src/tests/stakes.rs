/// Tests for stake-level mechanics.
///
/// Stakes (in ascending order): White=0, Red=1, Green=2, Black=3, Blue=4, Purple=5, Orange=6, Gold=7
///
/// What each stake changes (cumulative):
///   White:   base difficulty, no modifiers
///   Red:     Small Blind gives no cash reward ($0 instead of $3)
///   Green:   required score scales faster each Ante (not directly testable here)
///   Black:   30% chance Jokers in shops have Eternal sticker
///   Blue:    -1 effective discard per round
///   Purple:  score scales even faster; Spectral Packs appear in shops
///   Orange:  30% chance Jokers in shops have Perishable sticker (on top of Eternal)
///   Gold:    30% chance Jokers in shops have Rental sticker (independent of Eternal/Perishable)
///
/// Eternal and Perishable are mutually exclusive; Rental can combine with either.

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
// White stake: baseline — no stickers, full discards, $3 small blind
// =========================================================

#[test]
fn test_white_stake_never_produces_stickered_jokers() {
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
// Red stake: Small Blind gives no reward; still no stickers
// =========================================================

#[test]
fn test_red_stake_never_produces_stickered_jokers() {
    // Stickers only begin at Black stake.
    let mut gs = GameState::new(DeckType::Blue, Stake::Red, Some("RED_STICKER".to_string()));
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                assert!(!j.eternal,    "Red stake should never produce eternal jokers");
                assert!(!j.rental,     "Red stake should never produce rental jokers");
                assert!(!j.perishable, "Red stake should never produce perishable jokers");
            }
        }
    }
}

#[test]
fn test_red_stake_effective_discards_equal_to_max() {
    let gs = GameState::new(DeckType::Blue, Stake::Red, Some("RED_DISC".to_string()));
    assert_eq!(gs.effective_max_discards(), gs.max_discards,
        "Red stake should not reduce discards (that begins at Blue)");
}

// =========================================================
// Green stake: still no stickers
// =========================================================

#[test]
fn test_green_stake_never_produces_stickered_jokers() {
    // Stickers only begin at Black stake.
    let mut gs = GameState::new(DeckType::Blue, Stake::Green, Some("GREEN_STICKER".to_string()));
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                assert!(!j.eternal,    "Green stake should never produce eternal jokers");
                assert!(!j.rental,     "Green stake should never produce rental jokers");
                assert!(!j.perishable, "Green stake should never produce perishable jokers");
            }
        }
    }
}

#[test]
fn test_green_stake_effective_discards_equal_to_max() {
    let gs = GameState::new(DeckType::Blue, Stake::Green, Some("GREEN_DISC".to_string()));
    assert_eq!(gs.effective_max_discards(), gs.max_discards,
        "Green stake should not reduce discards (that begins at Blue)");
}

// =========================================================
// Black stake: Eternal stickers only; discards NOT yet reduced
// =========================================================

#[test]
fn test_black_stake_can_produce_eternal_jokers() {
    // ~30% per joker. Over 50 shops (~100 jokers) P(no eternal) < (0.70)^100 ≈ 10^-15.
    let mut gs = GameState::new(DeckType::Blue, Stake::Black, Some("BLACK_ETERNAL".to_string()));
    let mut found = false;
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                if j.eternal { found = true; }
            }
        }
        if found { break; }
    }
    assert!(found, "Black stake should eventually produce eternal jokers");
}

#[test]
fn test_black_stake_never_produces_rental_or_perishable_jokers() {
    // Only Eternal is in the pool at Black; Perishable needs Orange+, Rental needs Gold+.
    let mut gs = GameState::new(DeckType::Blue, Stake::Black, Some("BLACK_NO_RP".to_string()));
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                assert!(!j.rental,     "Black stake should never produce rental jokers");
                assert!(!j.perishable, "Black stake should never produce perishable jokers");
            }
        }
    }
}

#[test]
fn test_black_stake_effective_discards_equal_to_max() {
    // The -1 discard penalty starts at Blue (stake 5), not Black (stake 4).
    let gs = GameState::new(DeckType::Blue, Stake::Black, Some("BLACK_DISC".to_string()));
    assert_eq!(
        gs.effective_max_discards(),
        gs.max_discards,
        "Black stake should NOT reduce discards; that begins at Blue"
    );
}

// =========================================================
// Blue stake: Eternal stickers + -1 discard; still no Perishable/Rental
// =========================================================

#[test]
fn test_blue_stake_can_produce_eternal_jokers() {
    let mut gs = GameState::new(DeckType::Blue, Stake::Blue, Some("BLUE_ETERNAL".to_string()));
    let mut found = false;
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                if j.eternal { found = true; }
            }
        }
        if found { break; }
    }
    assert!(found, "Blue stake should produce eternal jokers (inherited from Black+)");
}

#[test]
fn test_blue_stake_never_produces_rental_or_perishable_jokers() {
    let mut gs = GameState::new(DeckType::Blue, Stake::Blue, Some("BLUE_NO_RP".to_string()));
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                assert!(!j.rental,     "Blue stake should never produce rental jokers");
                assert!(!j.perishable, "Blue stake should never produce perishable jokers");
            }
        }
    }
}

#[test]
fn test_blue_stake_effective_discards_reduced_by_one() {
    let gs = GameState::new(DeckType::Blue, Stake::Blue, Some("BLUE_DISC".to_string()));
    assert_eq!(
        gs.effective_max_discards(),
        gs.max_discards.saturating_sub(1),
        "Blue stake should reduce effective discards by 1"
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
        "Purple stake (>= Blue) should reduce effective discards by 1"
    );
}

// =========================================================
// Orange stake: Eternal + Perishable stickers; still no Rental
// =========================================================

#[test]
fn test_orange_stake_can_produce_eternal_jokers() {
    let mut gs = GameState::new(DeckType::Blue, Stake::Orange, Some("ORANGE_ETERNAL".to_string()));
    let mut found = false;
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind { if j.eternal { found = true; } }
        }
        if found { break; }
    }
    assert!(found, "Orange stake should produce eternal jokers (inherited from Black+)");
}

#[test]
fn test_orange_stake_can_produce_perishable_jokers() {
    // ~30% per joker. Over 200 shops P(none) < (0.70)^400 ≈ 10^-56.
    let mut gs = GameState::new(DeckType::Blue, Stake::Orange, Some("ORANGE_PERISH".to_string()));
    let mut found = false;
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind { if j.perishable { found = true; } }
        }
        if found { break; }
    }
    assert!(found, "Orange stake should eventually produce perishable jokers");
}

#[test]
fn test_orange_stake_never_produces_rental_jokers() {
    // Rental only unlocks at Gold stake.
    let mut gs = GameState::new(DeckType::Blue, Stake::Orange, Some("ORANGE_NO_RENT".to_string()));
    for _ in 0..200 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                assert!(!j.rental, "Orange stake should never produce rental jokers");
            }
        }
    }
}

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
        "Orange stake (>= Blue) should reduce effective discards by 1"
    );
}

// =========================================================
// Gold stake: all three sticker types possible; Rental is independent
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
        "Gold stake (>= Blue) should reduce effective discards by 1"
    );
}

#[test]
fn test_gold_stake_can_produce_all_sticker_types() {
    // Each sticker type has ~30% chance per joker; all three should appear within 200 shops.
    let mut gs = GameState::new(DeckType::Blue, Stake::Gold, Some("GOLD_STICKERS".to_string()));
    let (mut found_eternal, mut found_rental, mut found_perishable) = (false, false, false);
    for _ in 0..200 {
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

#[test]
fn test_gold_stake_rental_can_coexist_with_eternal() {
    // Rental is independent of Eternal/Perishable; a joker can be both Rental and Eternal.
    let mut gs = GameState::new(DeckType::Blue, Stake::Gold, Some("GOLD_RENT_ETERNAL".to_string()));
    let mut found = false;
    for _ in 0..500 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                if j.eternal && j.rental { found = true; }
            }
        }
        if found { break; }
    }
    assert!(found, "Gold stake should eventually produce a joker that is both Eternal and Rental");
}

#[test]
fn test_eternal_and_perishable_never_coexist() {
    // The engine must never assign both Eternal and Perishable to the same joker.
    let mut gs = GameState::new(DeckType::Blue, Stake::Gold, Some("NO_ETERNAL_PERISH".to_string()));
    for _ in 0..500 {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            if let ShopItem::Joker(j) = &offer.kind {
                assert!(
                    !(j.eternal && j.perishable),
                    "A joker must not be both Eternal and Perishable"
                );
            }
        }
    }
}
