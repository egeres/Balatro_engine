/// Tests for the tag system: tags are earned by skipping blinds and fire at various points.

use super::*;
use crate::card::{PackCard, ShopItem};
use crate::types::TagTrigger;

fn game_at_ante(ante: u32, seed: &str) -> GameState {
    let mut gs = GameState::new(DeckType::Red, Stake::White, Some(seed.to_string()));
    gs.ante = ante;
    gs
}

// =========================================================
// Earning tags
// =========================================================

#[test]
fn test_skipping_a_blind_grants_a_tag() {
    let mut gs = game_at_ante(1, "SKIP1");
    assert!(gs.tags.is_empty());
    let money_before = gs.money;
    gs.skip_blind().unwrap();
    assert_eq!(gs.skips_this_run, 1);
    // Either a tag queued up, or an immediate one paid out on the spot.
    assert!(!gs.tags.is_empty() || gs.money != money_before || !gs.jokers.is_empty());
}

#[test]
fn test_tags_respect_their_minimum_ante() {
    let mut gs = game_at_ante(1, "MINANTE");
    for _ in 0..200 {
        let t = gs.random_tag();
        assert!(t.min_ante() <= 1, "{:?} needs ante {}", t, t.min_ante());
    }
}

#[test]
fn test_ante_two_unlocks_more_tags() {
    let mut gs = game_at_ante(2, "MINANTE2");
    let mut seen = std::collections::HashSet::new();
    for _ in 0..600 {
        seen.insert(gs.random_tag());
    }
    assert!(seen.iter().any(|t| t.min_ante() == 2), "ante-2 tags should be reachable");
}

// =========================================================
// Immediate tags
// =========================================================

#[test]
fn test_skip_tag_pays_five_per_skip() {
    let mut gs = game_at_ante(1, "SKIPTAG");
    gs.money = 0;
    gs.skips_this_run = 3;
    gs.gain_tag(TagKind::Skip);
    assert_eq!(gs.money, 15);
    assert!(gs.tags.is_empty(), "immediate tags are not queued");
}

#[test]
fn test_handy_tag_pays_per_hand_played() {
    let mut gs = game_at_ante(2, "HANDY");
    gs.money = 0;
    gs.hands_played_this_run = 7;
    gs.gain_tag(TagKind::Handy);
    assert_eq!(gs.money, 7);
}

#[test]
fn test_garbage_tag_pays_per_unused_discard() {
    let mut gs = game_at_ante(2, "GARBAGE");
    gs.money = 0;
    gs.unused_discards_this_run = 6;
    gs.gain_tag(TagKind::Garbage);
    assert_eq!(gs.money, 6);
}

#[test]
fn test_economy_tag_doubles_money_up_to_forty() {
    let mut gs = game_at_ante(1, "ECON");
    gs.money = 12;
    gs.gain_tag(TagKind::Economy);
    assert_eq!(gs.money, 24);

    gs.money = 100;
    gs.gain_tag(TagKind::Economy);
    assert_eq!(gs.money, 140, "the gain is capped at 40");
}

#[test]
fn test_orbital_tag_levels_a_hand_by_three() {
    let mut gs = game_at_ante(2, "ORBITAL");
    let before: u32 = gs.hand_levels.values().map(|h| h.level).sum();
    gs.gain_tag(TagKind::Orbital);
    let after: u32 = gs.hand_levels.values().map(|h| h.level).sum();
    assert_eq!(after, before + 3);
}

#[test]
fn test_top_up_tag_creates_two_common_jokers() {
    let mut gs = game_at_ante(2, "TOPUP");
    gs.gain_tag(TagKind::TopUp);
    assert_eq!(gs.jokers.len(), 2);
    assert!(gs.jokers.iter().all(|j| j.kind.rarity() == 1));
}

#[test]
fn test_top_up_tag_respects_joker_slots() {
    let mut gs = game_at_ante(2, "TOPUPFULL");
    gs.joker_slots = 1;
    gs.gain_tag(TagKind::TopUp);
    assert_eq!(gs.jokers.len(), 1);
}

// =========================================================
// Double Tag
// =========================================================

#[test]
fn test_double_tag_copies_the_next_tag() {
    let mut gs = game_at_ante(1, "DOUBLE");
    gs.money = 0;
    gs.skips_this_run = 2;
    gs.gain_tag(TagKind::DoubleFun);
    assert_eq!(gs.tags, vec![TagKind::DoubleFun]);

    gs.gain_tag(TagKind::Skip);
    assert_eq!(gs.money, 20, "Skip Tag paid out twice");
    assert!(gs.tags.is_empty(), "the Double Tag is consumed");
}

#[test]
fn test_double_tag_does_not_copy_another_double_tag() {
    let mut gs = game_at_ante(1, "DOUBLE2");
    gs.gain_tag(TagKind::DoubleFun);
    gs.gain_tag(TagKind::DoubleFun);
    assert_eq!(gs.tags, vec![TagKind::DoubleFun, TagKind::DoubleFun]);
}

#[test]
fn test_double_tag_duplicates_a_queued_tag() {
    let mut gs = game_at_ante(1, "DOUBLE3");
    gs.gain_tag(TagKind::DoubleFun);
    gs.gain_tag(TagKind::Investment);
    assert_eq!(gs.tags, vec![TagKind::Investment, TagKind::Investment]);
}

// =========================================================
// Deferred tags
// =========================================================

#[test]
fn test_investment_tag_pays_after_the_boss_is_beaten() {
    let mut gs = game_at_ante(1, "INVEST");
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.current_blind = crate::game::BlindKind::Boss;
    // Which boss got rolled is irrelevant here, and some of them (The Psychic) would refuse a
    // one-card hand.
    gs.boss_blind = None;
    gs.money = 0;
    gs.tags.push(TagKind::Investment);
    gs.score_goal = 1.0;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert!(gs.money >= 25, "Investment Tag should pay 25, money = {}", gs.money);
    assert!(gs.tags.is_empty());
}

#[test]
fn test_juggle_tag_adds_hand_size_for_one_round() {
    let mut gs = game_at_ante(1, "JUGGLE");
    let base = gs.effective_hand_size();
    gs.tags.push(TagKind::Juggle);
    gs.select_blind().unwrap();
    assert_eq!(gs.effective_hand_size(), base + 3);
    assert!(gs.tags.is_empty());
}

#[test]
fn test_boss_tag_rerolls_the_boss_blind() {
    let mut gs = game_at_ante(1, "BOSSTAG");
    gs.tags.push(TagKind::Boss);
    gs.apply_blind_select_tags();
    assert!(gs.tags.is_empty());
    assert!(gs.boss_blind.is_some());
}

#[test]
fn test_pack_tags_queue_a_free_booster() {
    for (tag, expected) in [
        (TagKind::Charm, PackKind::ArcanaPackMega),
        (TagKind::Meteor, PackKind::CelestialPackMega),
        (TagKind::Standard, PackKind::StandardPackMega),
        (TagKind::Buffoon, PackKind::BuffoonPackMega),
        (TagKind::Ethereal, PackKind::SpectralPack),
    ] {
        let mut gs = game_at_ante(2, "PACKTAG");
        gs.tags.push(tag);
        gs.apply_blind_select_tags();
        assert_eq!(gs.pending_free_pack, Some(expected), "{:?}", tag);

        gs.open_pending_free_pack().unwrap();
        assert!(gs.current_pack.is_some());
        assert!(gs.pending_free_pack.is_none());
    }
}

#[test]
fn test_charm_tag_opens_a_pack_full_of_tarots() {
    let mut gs = game_at_ante(1, "CHARM");
    gs.tags.push(TagKind::Charm);
    gs.apply_blind_select_tags();
    gs.open_pending_free_pack().unwrap();
    let pack = gs.current_pack.as_ref().unwrap();
    assert!(pack.cards.iter().all(|c| matches!(
        c,
        PackCard::Consumable(crate::card::ConsumableCard::Tarot(_))
    )));
}

// =========================================================
// Shop tags
// =========================================================

#[test]
fn test_coupon_tag_makes_the_shop_free() {
    let mut gs = game_at_ante(1, "COUPON");
    gs.tags.push(TagKind::Coupon);
    gs.state = GameStateKind::Shop;
    gs.generate_shop();
    assert!(gs.shop_offers.iter().all(|o| o.price == 0));
    assert!(gs.tags.is_empty());
}

#[test]
fn test_d6_tag_makes_rerolls_free() {
    let mut gs = game_at_ante(1, "D6");
    gs.tags.push(TagKind::D6);
    gs.state = GameStateKind::Shop;
    gs.generate_shop();
    assert_eq!(gs.reroll_cost, 0);

    gs.money = 0;
    gs.reroll_shop().unwrap();
    gs.reroll_shop().unwrap();
    assert_eq!(gs.money, 0, "D6 covers every reroll in the shop");
}

#[test]
fn test_voucher_tag_adds_a_second_voucher() {
    let mut gs = game_at_ante(1, "VTAG");
    gs.tags.push(TagKind::Voucher);
    gs.state = GameStateKind::Shop;
    gs.generate_shop();
    let vouchers = gs
        .shop_offers
        .iter()
        .filter(|o| matches!(o.kind, ShopItem::Voucher(_)))
        .count();
    assert_eq!(vouchers, 1, "the extra voucher is stocked as a shop offer");
    assert!(gs.shop_voucher.is_some(), "alongside the usual one");
}

#[test]
fn test_rarity_tags_guarantee_a_joker_of_that_rarity() {
    for (tag, rarity) in [(TagKind::Uncommon, 2u8), (TagKind::Rare, 3u8)] {
        let mut gs = game_at_ante(1, "RARITY");
        gs.tags.push(tag);
        gs.state = GameStateKind::Shop;
        gs.generate_shop();
        let found = gs.shop_offers.iter().any(|o| match &o.kind {
            ShopItem::Joker(j) => j.kind.rarity() == rarity,
            _ => false,
        });
        assert!(found, "{:?} should guarantee a rarity-{} joker", tag, rarity);
    }
}

#[test]
fn test_edition_tags_stamp_a_free_joker() {
    for (tag, edition) in [
        (TagKind::Foil, Edition::Foil),
        (TagKind::Holographic, Edition::Holographic),
        (TagKind::Polychrome, Edition::Polychrome),
        (TagKind::Negative, Edition::Negative),
    ] {
        let mut gs = game_at_ante(2, "EDITION");
        gs.tags.push(tag);
        gs.state = GameStateKind::Shop;
        gs.generate_shop();
        let offer = gs
            .shop_offers
            .iter()
            .find(|o| matches!(&o.kind, ShopItem::Joker(j) if j.edition == edition));
        let offer = offer.unwrap_or_else(|| panic!("{:?} should stamp a joker", tag));
        assert_eq!(offer.price, 0, "{:?} hands the joker over for free", tag);
    }
}

#[test]
fn test_shop_tags_do_not_carry_into_the_next_shop() {
    let mut gs = game_at_ante(1, "CARRY");
    gs.tags.push(TagKind::Coupon);
    gs.state = GameStateKind::Shop;
    gs.generate_shop();
    assert!(gs.shop_offers.iter().all(|o| o.price == 0));

    gs.leave_shop().unwrap();
    gs.state = GameStateKind::Shop;
    gs.generate_shop();
    assert!(gs.shop_offers.iter().any(|o| o.price > 0), "the next shop charges again");
}

// =========================================================
// Metadata sanity
// =========================================================

#[test]
fn test_every_tag_has_a_trigger_and_a_name() {
    for t in TagKind::ALL {
        assert!(!t.display_name().is_empty());
        let _ = t.trigger();
    }
    assert_eq!(TagKind::Skip.trigger(), TagTrigger::Immediate);
    assert_eq!(TagKind::Investment.trigger(), TagTrigger::BossDefeated);
    assert_eq!(TagKind::DoubleFun.trigger(), TagTrigger::CopyNextTag);
}
