/// Shop economics: what a reroll costs, what it actually replaces, and the polls that stock it.

use super::*;
use crate::game::GameStateKind;

/// A game sitting in a freshly stocked shop with plenty of money.
fn game_in_shop(seed: &str) -> GameState {
    let mut gs = GameState::new(DeckType::Red, Stake::White, Some(seed.to_string()));
    gs.select_blind().unwrap();
    gs.state = GameStateKind::Shop;
    gs.generate_shop();
    gs.money = 200;
    gs
}

// =========================================================
// Reroll price escalates, and keeps its escalation
// =========================================================

#[test]
fn test_reroll_price_climbs_by_a_dollar_each_time() {
    let mut gs = game_in_shop("REROLL1");
    assert_eq!(gs.reroll_cost, 5, "a shop opens at the base reroll price");

    gs.reroll_shop().unwrap();
    assert_eq!(gs.reroll_cost, 6);
    gs.reroll_shop().unwrap();
    assert_eq!(gs.reroll_cost, 7);
    gs.reroll_shop().unwrap();
    assert_eq!(gs.reroll_cost, 8);
}

#[test]
fn test_reroll_charges_the_escalated_price() {
    let mut gs = game_in_shop("REROLL2");
    gs.money = 100;

    gs.reroll_shop().unwrap();
    assert_eq!(gs.money, 95);
    gs.reroll_shop().unwrap();
    assert_eq!(gs.money, 89, "the second reroll costs $6");
    gs.reroll_shop().unwrap();
    assert_eq!(gs.money, 82, "the third costs $7");
}

#[test]
fn test_reroll_price_resets_when_a_new_round_begins() {
    let mut gs = game_in_shop("REROLL3");
    gs.reroll_shop().unwrap();
    gs.reroll_shop().unwrap();
    assert_eq!(gs.reroll_cost, 7);

    gs.leave_shop().unwrap();
    gs.select_blind().unwrap();
    assert_eq!(gs.reroll_cost, 5, "reroll_cost_increase is per round");
}

#[test]
fn test_a_free_reroll_does_not_raise_the_price() {
    let mut gs = GameState::new(DeckType::Red, Stake::White, Some("REROLL4".to_string()));
    gs.jokers.push(joker(1, JokerKind::ChaosTheClown));
    gs.select_blind().unwrap();
    gs.state = GameStateKind::Shop;
    gs.generate_shop();
    gs.money = 100;

    assert_eq!(gs.reroll_cost, 0, "a free reroll is free");
    gs.reroll_shop().unwrap();
    assert_eq!(gs.money, 100, "spending the free reroll costs nothing");
    assert_eq!(gs.reroll_cost, 5, "and it does not escalate the price");
}

#[test]
fn test_reroll_surplus_lowers_the_base_price() {
    let mut gs = game_in_shop("REROLL5");
    gs.vouchers.push(VoucherKind::RerollSurplus);
    gs.base_reroll_cost = 3;
    gs.reroll_cost_increase = 0;
    gs.recalculate_reroll_cost(true);

    assert_eq!(gs.reroll_cost, 3);
    gs.reroll_shop().unwrap();
    assert_eq!(gs.reroll_cost, 4, "the escalation stacks on top of the discounted base");
}

// =========================================================
// A reroll replaces the card slots and nothing else
// =========================================================

#[test]
fn test_reroll_leaves_the_booster_packs_alone() {
    let mut gs = game_in_shop("REROLLPACK");
    let packs_before: Vec<_> = gs
        .shop_offers
        .iter()
        .filter_map(|o| match o.kind {
            crate::card::ShopItem::Pack(p) => Some(p),
            _ => None,
        })
        .collect();
    assert_eq!(packs_before.len(), 2);

    gs.reroll_shop().unwrap();

    let packs_after: Vec<_> = gs
        .shop_offers
        .iter()
        .filter_map(|o| match o.kind {
            crate::card::ShopItem::Pack(p) => Some(p),
            _ => None,
        })
        .collect();
    assert_eq!(packs_before, packs_after, "rerolling must not reroll the boosters");
}

#[test]
fn test_reroll_leaves_the_voucher_alone() {
    let mut gs = game_in_shop("REROLLVOU");
    let voucher_before = gs.shop_voucher;
    assert!(voucher_before.is_some());

    for _ in 0..5 {
        gs.reroll_shop().unwrap();
    }

    assert_eq!(gs.shop_voucher, voucher_before,
        "the voucher slot is not rerollable, so it cannot be fished for");
}

#[test]
fn test_reroll_replaces_the_card_slots() {
    let mut gs = game_in_shop("REROLLCARD");
    let card_slots = |gs: &GameState| {
        gs.shop_offers
            .iter()
            .filter(|o| !matches!(o.kind, crate::card::ShopItem::Pack(_)))
            .count()
    };
    let before = card_slots(&gs);
    assert!(before > 0);

    gs.reroll_shop().unwrap();
    assert_eq!(card_slots(&gs), before, "the same number of card slots come back");
    assert!(gs.shop_offers.iter().all(|o| !o.sold), "and they are fresh, not sold");
}

// =========================================================
// edition_rate reaches the jokers
// =========================================================

#[test]
fn test_glow_up_makes_editioned_shop_jokers_far_more_common() {
    let count_editioned = |edition_rate: f64| {
        let mut n = 0;
        for seed in 0..400 {
            let mut gs = GameState::new(DeckType::Red, Stake::White, Some(format!("ED{}", seed)));
            gs.edition_rate = edition_rate;
            if let Some(j) = gs.generate_random_joker() {
                if j.edition != Edition::None {
                    n += 1;
                }
            }
        }
        n
    };

    let plain = count_editioned(1.0);
    let glowed = count_editioned(4.0);
    assert!(glowed > plain * 2,
        "Glow Up should roughly quadruple editioned jokers, got {} vs {}", plain, glowed);
}

#[test]
fn test_hone_raises_the_edition_rate_when_redeemed() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.money = 50;
    gs.shop_voucher = Some(VoucherKind::Hone);
    gs.buy_voucher().unwrap();
    assert_eq!(gs.edition_rate, 2.0);

    gs.money = 50;
    gs.shop_voucher = Some(VoucherKind::GlowUp);
    gs.buy_voucher().unwrap();
    assert_eq!(gs.edition_rate, 4.0);
}

// =========================================================
// Stake stickers come from one roll
// =========================================================

#[test]
fn test_perishables_are_not_starved_by_eternals() {
    // A single poll decides both, so at Gold stake each lands near 30% of eligible jokers —
    // two independent rolls would have left perishable at 21%.
    let mut eternal = 0;
    let mut perishable = 0;
    let mut total = 0;
    for seed in 0..600 {
        let mut gs = GameState::new(DeckType::Red, Stake::Gold, Some(format!("ST{}", seed)));
        if let Some(j) = gs.generate_random_joker() {
            if !j.kind.eternal_compat() || !j.kind.perishable_compat() {
                continue;
            }
            total += 1;
            if j.eternal { eternal += 1; }
            if j.perishable { perishable += 1; }
        }
    }
    assert!(total > 300, "sanity: enough eligible jokers, got {}", total);
    let e = eternal as f64 / total as f64;
    let p = perishable as f64 / total as f64;
    assert!((0.22..0.38).contains(&e), "eternal rate {:.3} should sit near 0.30", e);
    assert!((0.22..0.38).contains(&p), "perishable rate {:.3} should sit near 0.30", p);
}

#[test]
fn test_a_joker_never_gets_both_stake_stickers() {
    for seed in 0..300 {
        let mut gs = GameState::new(DeckType::Red, Stake::Gold, Some(format!("BOTH{}", seed)));
        if let Some(j) = gs.generate_random_joker() {
            assert!(!(j.eternal && j.perishable),
                "{:?} got both stickers from one poll", j.kind);
        }
    }
}
