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

// =========================================================
// The voucher slot is per ante, not per shop
// =========================================================

/// Beat the current blind with a trivially easy goal and step into the shop. The Boss ability is
/// nulled out so that whichever boss got rolled cannot interfere.
fn clear_a_blind(gs: &mut GameState) {
    gs.boss_blind = None;
    gs.select_blind().unwrap();
    gs.score_goal = 1.0;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert!(matches!(gs.state, GameStateKind::Shop));
}

#[test]
fn test_the_same_voucher_is_offered_all_ante() {
    let mut gs = GameState::new(DeckType::Red, Stake::White, Some("VOU1".to_string()));
    let ante_voucher = gs.shop_voucher.expect("a voucher is drawn at run start");

    clear_a_blind(&mut gs); // Small
    assert_eq!(gs.shop_voucher, Some(ante_voucher));
    gs.leave_shop().unwrap();

    clear_a_blind(&mut gs); // Big
    assert_eq!(gs.shop_voucher, Some(ante_voucher),
        "the voucher slot holds the same card all ante");
}

#[test]
fn test_the_voucher_slot_is_redrawn_when_the_boss_falls() {
    let mut gs = GameState::new(DeckType::Red, Stake::White, Some("VOU2".to_string()));

    // Take this ante's voucher, which empties the slot.
    clear_a_blind(&mut gs);
    gs.money = 100;
    gs.buy_voucher().unwrap();
    assert!(gs.shop_voucher.is_none());
    gs.leave_shop().unwrap();

    clear_a_blind(&mut gs); // Big — still empty, same ante
    assert!(gs.shop_voucher.is_none());
    gs.leave_shop().unwrap();

    clear_a_blind(&mut gs); // Boss — a new ante's voucher is drawn
    assert!(gs.shop_voucher.is_some(),
        "the Boss falling draws the next ante's voucher");
}

#[test]
fn test_a_bought_voucher_does_not_come_back_next_shop() {
    let mut gs = GameState::new(DeckType::Red, Stake::White, Some("VOU3".to_string()));
    clear_a_blind(&mut gs);
    gs.money = 100;
    let bought = gs.shop_voucher.unwrap();
    gs.buy_voucher().unwrap();

    gs.leave_shop().unwrap();
    clear_a_blind(&mut gs);

    assert!(gs.shop_voucher.is_none(),
        "buying the ante's voucher empties the slot until the next ante");
    assert!(gs.vouchers.contains(&bought));
}

#[test]
fn test_rerolling_cannot_fish_for_a_different_voucher() {
    let mut gs = GameState::new(DeckType::Red, Stake::White, Some("VOU4".to_string()));
    clear_a_blind(&mut gs);
    gs.money = 500;
    let offered = gs.shop_voucher;

    for _ in 0..8 {
        gs.reroll_shop().unwrap();
    }
    assert_eq!(gs.shop_voucher, offered);
}

// =========================================================
// Prices are computed once, from the base cost
// =========================================================

/// Put one known offer on the shelf so the price is unambiguous.
fn shop_with(gs: &mut GameState, item: crate::card::ShopItem, base: u32) {
    gs.state = GameStateKind::Shop;
    gs.shop_offers.clear();
    gs.shop_offers.push(crate::card::ShopOffer::new(item, base));
}

#[test]
fn test_a_discount_is_not_applied_twice_to_jokers() {
    // Blueprint is $10. Clearance Sale takes 25% off exactly once: floor((10 + 0.5) * 0.75) = 7.
    // Discounting the shelf price and then discounting again at the till gave $5.
    let mut gs = make_game();
    gs.vouchers.push(VoucherKind::ClearanceSale);
    let j = joker(1, JokerKind::Blueprint);
    shop_with(&mut gs, crate::card::ShopItem::Joker(j), JokerKind::Blueprint.base_cost());

    assert_eq!(gs.offer_price(0), Some(7));

    gs.money = 50;
    gs.buy_joker(0).unwrap();
    assert_eq!(gs.money, 43, "charged the sticker price, once");
}

#[test]
fn test_liquidation_halves_a_joker_once() {
    let mut gs = make_game();
    gs.vouchers.push(VoucherKind::Liquidation);
    let j = joker(1, JokerKind::Blueprint);
    shop_with(&mut gs, crate::card::ShopItem::Joker(j), JokerKind::Blueprint.base_cost());
    assert_eq!(gs.offer_price(0), Some(5), "floor(10.5 * 0.5)");
}

#[test]
fn test_a_couponed_offer_is_free_not_a_dollar() {
    // The override lands after the max(1) floor (card.lua:383).
    let mut gs = make_game();
    let j = joker(1, JokerKind::Blueprint);
    shop_with(&mut gs, crate::card::ShopItem::Joker(j), JokerKind::Blueprint.base_cost());
    gs.shop_offers[0].free = true;

    assert_eq!(gs.offer_price(0), Some(0));
    gs.money = 0;
    gs.buy_joker(0).unwrap();
    assert_eq!(gs.money, 0, "free means free");
}

#[test]
fn test_an_edition_surcharge_rides_on_the_base_cost() {
    let mut gs = make_game();
    let mut j = joker(1, JokerKind::Joker); // base $2
    j.edition = Edition::Polychrome;        // +$5
    shop_with(&mut gs, crate::card::ShopItem::Joker(j), JokerKind::Joker.base_cost());
    assert_eq!(gs.offer_price(0), Some(7), "floor(2 + 5 + 0.5)");
}

#[test]
fn test_a_coupon_beats_a_rental() {
    // set_cost applies rental then coupon, so the coupon wins (card.lua:381-383).
    let mut gs = make_game();
    let mut j = joker(1, JokerKind::Blueprint);
    j.rental = true;
    shop_with(&mut gs, crate::card::ShopItem::Joker(j), JokerKind::Blueprint.base_cost());
    assert_eq!(gs.offer_price(0), Some(1), "a rental is $1");

    gs.shop_offers[0].free = true;
    assert_eq!(gs.offer_price(0), Some(0), "unless it is couponed");
}

#[test]
fn test_buying_clearance_sale_reprices_what_is_still_on_the_shelf() {
    let mut gs = make_game();
    let j = joker(1, JokerKind::Blueprint);
    shop_with(&mut gs, crate::card::ShopItem::Joker(j), JokerKind::Blueprint.base_cost());
    assert_eq!(gs.offer_price(0), Some(10));

    gs.money = 50;
    gs.shop_voucher = Some(VoucherKind::ClearanceSale);
    gs.buy_voucher().unwrap();

    assert_eq!(gs.offer_price(0), Some(7), "prices follow the discount live");
}

// =========================================================
// Sell value follows the shop price, not the bare base cost
// =========================================================

#[test]
fn test_an_edition_raises_what_a_joker_sells_for() {
    // sell_cost is floor(cost/2), and cost includes the edition surcharge (card.lua:382).
    let plain = joker(1, JokerKind::Joker);
    let mut poly = joker(2, JokerKind::Joker);
    poly.edition = Edition::Polychrome;

    assert_eq!(plain.sell_value(0.0), 1, "floor(2.5) = 2, halved");
    assert_eq!(poly.sell_value(0.0), 3, "floor(2 + 5 + 0.5) = 7, halved");
}

#[test]
fn test_a_discount_lowers_what_your_board_is_worth() {
    let blueprint = joker(1, JokerKind::Blueprint); // base $10
    assert_eq!(blueprint.sell_value(0.0), 5);
    assert_eq!(blueprint.sell_value(25.0), 3, "floor(10.5 * 0.75) = 7, halved");
    assert_eq!(blueprint.sell_value(50.0), 2, "floor(10.5 * 0.5) = 5, halved");
}

#[test]
fn test_a_rental_sells_for_a_dollar() {
    let mut rented = joker(1, JokerKind::Blueprint);
    rented.rental = true;
    assert_eq!(rented.sell_value(0.0), 1, "a rental costs $1, so it sells for $1");
}

#[test]
fn test_the_egg_bonus_is_added_after_the_halving() {
    let mut egg = joker(1, JokerKind::Egg); // base $4 -> sells for 2
    assert_eq!(egg.sell_value(0.0), 2);
    egg.set_counter_i64("sell_bonus", 6);
    assert_eq!(egg.sell_value(0.0), 8, "extra_value is outside the max/floor");
}

#[test]
fn test_selling_pays_the_discounted_value() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.vouchers.push(VoucherKind::Liquidation);
    gs.jokers.push(joker(1, JokerKind::Blueprint));
    gs.money = 0;

    assert_eq!(gs.joker_sell_value(0), Some(2));
    gs.sell_joker(0).unwrap();
    assert_eq!(gs.money, 2, "Liquidation halves what the board is worth too");
}

#[test]
fn test_swashbuckler_reads_the_discounted_sell_values() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![
        joker(0, JokerKind::Swashbuckler),
        joker(1, JokerKind::Blueprint), // sells for 5, or 2 under Liquidation
    ];
    let levels = default_hand_levels();

    let mut plain = crate::scoring::ScoreInputs::new(&played, &[], &jokers, &levels);
    plain.discount_percent = 0.0;
    let plain = crate::scoring::score_hand(plain);

    let mut cheap = crate::scoring::ScoreInputs::new(&played, &[], &jokers, &levels);
    cheap.discount_percent = 50.0;
    let cheap = crate::scoring::score_hand(cheap);

    assert_eq!(plain.final_mult - cheap.final_mult, 3.0,
        "Swashbuckler is worth less when the board is worth less");
}

// =========================================================
// End-of-round payouts
// =========================================================
// `evaluate_round` (state_events.lua:1135) lays out every row in one synchronous pass and only
// then lets the dollars land, so interest is worked out from the balance the round *ended* on.

/// Win the Small blind having played `hands_used` of the deck's hands, and report the money
/// gained. Starting balance is forced to `money` so interest is predictable.
fn small_blind_payout(money: i32, hands_used: u32) -> i32 {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.money = money;
    gs.score_goal = f64::MAX;

    // Burn the hands we are not saving, then win on the last one.
    for _ in 1..hands_used {
        gs.hand = vec![0];
        gs.select_card(0).unwrap();
        gs.play_hand().unwrap();
    }
    gs.score_goal = 1.0;
    gs.hand = vec![0];
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    gs.money - money
}

/// Every unused hand pays $1 on every deck — `money_per_hand or 1` (state_events.lua:1165).
/// Only a Challenge ever switches it off.
#[test]
fn test_every_unused_hand_pays_a_dollar() {
    // Blue Deck, five hands. $3 Small blind reward, $0 interest at $4.
    assert_eq!(small_blind_payout(4, 1), 3 + 4, "four hands left over");
    assert_eq!(small_blind_payout(4, 3), 3 + 2, "two hands left over");
    assert_eq!(small_blind_payout(4, 5), 3, "no hands left over, no bonus");
}

/// The Green Deck raises the rate to $2, adds $1 per unused discard, and gives up interest
/// (game.lua:631).
#[test]
fn test_green_deck_pays_more_per_hand_and_earns_no_interest() {
    let mut gs = GameState::new(DeckType::Green, Stake::White, Some("GREENPAY".to_string()));
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.money = 50; // enough that interest would be very visible
    gs.score_goal = 1.0;
    let hands = gs.hands_remaining;
    let discards = gs.discards_remaining;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    let expected = 3 + 2 * (hands - 1) as i32 + discards as i32;
    assert_eq!(gs.money - 50, expected, "Green Deck: $2 a hand, $1 a discard, no interest");
}

/// Interest is charged on the balance the round ended on, before the round's own payouts.
#[test]
fn test_interest_ignores_the_rewards_it_is_paid_alongside() {
    // At $4 the player is one dollar short of an interest step. The $3 blind reward and the
    // $4 of unused hands would push the balance past $5, but interest is already settled.
    assert_eq!(small_blind_payout(4, 1), 3 + 4, "no interest at $4");
    // At $5 exactly one step is due.
    assert_eq!(small_blind_payout(5, 1), 3 + 4 + 1);
    // At $9 still one step — $10 would be two, and the payouts do not count towards it.
    assert_eq!(small_blind_payout(9, 1), 3 + 4 + 1);
    assert_eq!(small_blind_payout(10, 1), 3 + 4 + 2);
}
