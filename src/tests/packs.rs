/// Tests for booster pack contents and pick rules.
///
/// Wiki reference: https://balatrowiki.org/w/Booster_Packs
///   Arcana/Celestial/Standard: Normal=3 cards/1 pick, Jumbo=5/1, Mega=5/2
///   Buffoon/Spectral:          Normal=2 cards/1 pick, Jumbo=4/1, Mega=4/2

use super::*;
use crate::card::ShopItem;

// =========================================================
// cards_shown correctness
// =========================================================

#[test]
fn test_arcana_pack_shows_3_cards() {
    assert_eq!(PackKind::ArcanaPack.cards_shown(), 3);
}

#[test]
fn test_arcana_jumbo_shows_5_cards() {
    assert_eq!(PackKind::ArcanaPackJumbo.cards_shown(), 5);
}

#[test]
fn test_arcana_mega_shows_5_cards() {
    assert_eq!(PackKind::ArcanaPackMega.cards_shown(), 5);
}

#[test]
fn test_celestial_pack_shows_3_cards() {
    assert_eq!(PackKind::CelestialPack.cards_shown(), 3);
}

#[test]
fn test_standard_pack_shows_3_cards() {
    assert_eq!(PackKind::StandardPack.cards_shown(), 3);
}

#[test]
fn test_buffoon_pack_shows_2_cards() {
    assert_eq!(PackKind::BuffoonPack.cards_shown(), 2);
}

#[test]
fn test_buffoon_jumbo_shows_4_cards() {
    assert_eq!(PackKind::BuffoonPackJumbo.cards_shown(), 4);
}

#[test]
fn test_buffoon_mega_shows_4_cards() {
    assert_eq!(PackKind::BuffoonPackMega.cards_shown(), 4);
}

#[test]
fn test_spectral_pack_shows_2_cards() {
    assert_eq!(PackKind::SpectralPack.cards_shown(), 2);
}

#[test]
fn test_spectral_jumbo_shows_4_cards() {
    assert_eq!(PackKind::SpectralPackJumbo.cards_shown(), 4);
}

#[test]
fn test_spectral_mega_shows_4_cards() {
    assert_eq!(PackKind::SpectralPackMega.cards_shown(), 4);
}

// =========================================================
// picks_allowed correctness
// =========================================================

#[test]
fn test_normal_packs_allow_1_pick() {
    assert_eq!(PackKind::ArcanaPack.picks_allowed(), 1);
    assert_eq!(PackKind::CelestialPack.picks_allowed(), 1);
    assert_eq!(PackKind::StandardPack.picks_allowed(), 1);
    assert_eq!(PackKind::BuffoonPack.picks_allowed(), 1);
    assert_eq!(PackKind::SpectralPack.picks_allowed(), 1);
}

#[test]
fn test_jumbo_packs_allow_1_pick() {
    assert_eq!(PackKind::ArcanaPackJumbo.picks_allowed(), 1);
    assert_eq!(PackKind::CelestialPackJumbo.picks_allowed(), 1);
    assert_eq!(PackKind::StandardPackJumbo.picks_allowed(), 1);
    assert_eq!(PackKind::BuffoonPackJumbo.picks_allowed(), 1);
    assert_eq!(PackKind::SpectralPackJumbo.picks_allowed(), 1);
}

#[test]
fn test_mega_packs_allow_2_picks() {
    assert_eq!(PackKind::ArcanaPackMega.picks_allowed(), 2);
    assert_eq!(PackKind::CelestialPackMega.picks_allowed(), 2);
    assert_eq!(PackKind::StandardPackMega.picks_allowed(), 2);
    assert_eq!(PackKind::BuffoonPackMega.picks_allowed(), 2);
    assert_eq!(PackKind::SpectralPackMega.picks_allowed(), 2);
}

// =========================================================
// Pack generation produces correct number of cards
// =========================================================

#[test]
fn test_buffoon_pack_generates_2_cards() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.shop_offers = vec![ShopItem::Pack(PackKind::BuffoonPack)]
        .into_iter()
        .map(|kind| crate::card::ShopOffer { kind, price: 4, sold: false })
        .collect();
    gs.money = 10;
    gs.joker_slots = 10;
    gs.buy_pack(0).unwrap();
    assert_eq!(
        gs.current_pack.as_ref().unwrap().cards.len(), 2,
        "Buffoon Normal pack should generate 2 cards"
    );
}

#[test]
fn test_buffoon_jumbo_pack_generates_4_cards() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.shop_offers = vec![ShopItem::Pack(PackKind::BuffoonPackJumbo)]
        .into_iter()
        .map(|kind| crate::card::ShopOffer { kind, price: 6, sold: false })
        .collect();
    gs.money = 10;
    gs.joker_slots = 10;
    gs.buy_pack(0).unwrap();
    assert_eq!(
        gs.current_pack.as_ref().unwrap().cards.len(), 4,
        "Buffoon Jumbo pack should generate 4 cards"
    );
}

#[test]
fn test_spectral_pack_generates_2_cards() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.shop_offers = vec![ShopItem::Pack(PackKind::SpectralPack)]
        .into_iter()
        .map(|kind| crate::card::ShopOffer { kind, price: 4, sold: false })
        .collect();
    gs.money = 10;
    gs.buy_pack(0).unwrap();
    assert_eq!(
        gs.current_pack.as_ref().unwrap().cards.len(), 2,
        "Spectral Normal pack should generate 2 cards"
    );
}

#[test]
fn test_arcana_pack_generates_3_cards() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.shop_offers = vec![ShopItem::Pack(PackKind::ArcanaPack)]
        .into_iter()
        .map(|kind| crate::card::ShopOffer { kind, price: 4, sold: false })
        .collect();
    gs.money = 10;
    gs.buy_pack(0).unwrap();
    assert_eq!(
        gs.current_pack.as_ref().unwrap().cards.len(), 3,
        "Arcana Normal pack should generate 3 cards"
    );
}

#[test]
fn test_arcana_jumbo_pack_generates_5_cards() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.shop_offers = vec![ShopItem::Pack(PackKind::ArcanaPackJumbo)]
        .into_iter()
        .map(|kind| crate::card::ShopOffer { kind, price: 6, sold: false })
        .collect();
    gs.money = 10;
    gs.buy_pack(0).unwrap();
    assert_eq!(
        gs.current_pack.as_ref().unwrap().cards.len(), 5,
        "Arcana Jumbo pack should generate 5 cards"
    );
}

// =========================================================
// No double-counting planet/tarot usage when picked from pack
// =========================================================

#[test]
fn test_planet_card_picked_from_pack_not_counted_until_used() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.shop_offers = vec![ShopItem::Pack(PackKind::CelestialPack)]
        .into_iter()
        .map(|kind| crate::card::ShopOffer { kind, price: 4, sold: false })
        .collect();
    gs.money = 10;
    gs.buy_pack(0).unwrap();

    let initial_count = gs.planet_cards_used;
    // Pick the first card from the pack (a planet)
    gs.take_pack_card(0).unwrap();
    // planet_cards_used should NOT increment on pick — only on use
    assert_eq!(gs.planet_cards_used, initial_count,
        "planet_cards_used should not increment when picking from pack, only when using");
}

#[test]
fn test_tarot_card_picked_from_pack_not_counted_until_used() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.shop_offers = vec![ShopItem::Pack(PackKind::ArcanaPack)]
        .into_iter()
        .map(|kind| crate::card::ShopOffer { kind, price: 4, sold: false })
        .collect();
    gs.money = 10;
    gs.buy_pack(0).unwrap();

    let initial_count = gs.tarot_cards_used;
    gs.take_pack_card(0).unwrap();
    assert_eq!(gs.tarot_cards_used, initial_count,
        "tarot_cards_used should not increment when picking from pack, only when using");
}
