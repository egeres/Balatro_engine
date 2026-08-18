/// Tests for voucher effects via GameState.
///
/// Also covers related card-enhancement and joker behaviors that require GameState round flow:
///   - Observatory voucher: X1.5 Mult per *held* planet card of the scored hand
///   - Gold Card enhancement: $3 per Gold card held in hand at end of round
///   - Campfire joker: resets x_mult to X1 when Boss Blind is defeated

use super::*;
use crate::card::{PackCard, ShopItem};

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
fn test_seed_money_raises_the_interest_cap_to_50() {
    let gs = apply_voucher_to_game(VoucherKind::SeedMoney);
    assert_eq!(gs.max_interest, 50);
}

#[test]
fn test_money_tree_raises_the_interest_cap_to_100() {
    let gs = apply_voucher_to_game(VoucherKind::MoneyTree);
    assert_eq!(gs.max_interest, 100);
}

// =========================================================
// Joker slot vouchers
// =========================================================

#[test]
fn test_blank_does_nothing() {
    let gs_before = make_game();
    let base = gs_before.joker_slots;
    let gs = apply_voucher_to_game(VoucherKind::Blank);
    assert_eq!(gs.joker_slots, base, "Blank is a pure unlock stepping stone");
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
fn test_directors_cut_unlocks_one_boss_reroll_per_ante() {
    let mut gs = make_game();
    gs.money = 100;

    // Without the voucher there is nothing to reroll with.
    assert!(gs.reroll_boss_blind().is_err());

    gs.vouchers.push(VoucherKind::DirectorsCut);
    gs.reroll_boss_blind().unwrap();
    assert_eq!(gs.money, 90, "a Boss reroll costs $10");
    assert!(gs.reroll_boss_blind().is_err(), "only one per ante");

    // Retcon lifts the per-ante limit.
    gs.vouchers.push(VoucherKind::Retcon);
    gs.reroll_boss_blind().unwrap();
    gs.reroll_boss_blind().unwrap();
    assert_eq!(gs.money, 70);
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
// Observatory voucher — X1.5 Mult per *held* Planet card of the scored hand
// =========================================================
// The voucher pays for Planets you are still holding, not for Planets you have spent
// (card.lua:2293 reads `self.ability.consumeable.hand_type` off a card sitting in the consumable
// area). Using the card levels the hand and takes the X1.5 away with it.

/// A held Jupiter multiplies a Flush by X1.5 once Observatory is redeemed.
#[test]
fn test_observatory_x_mult_comes_from_a_held_planet() {
    use crate::types::PlanetCard;
    // Flush L1: chips 35, mult 4. 5 Spades 2-3-4-5-7 add 21 chips → 56 chips.
    // Held Jupiter under Observatory: mult 4 × 1.5 = 6 → 336.
    let played = flush_2_3_4_5_7();

    let mut gs = apply_voucher_to_game(VoucherKind::Observatory);
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Jupiter).into());
    assert_eq!(score_with_held_planets(&gs, &played).final_score as i64, 336);

    // Without the voucher the same held card does nothing.
    let mut plain = make_game();
    plain.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Jupiter).into());
    assert_eq!(score_with_held_planets(&plain, &played).final_score as i64, 224,
        "a held Planet pays nothing without Observatory");
}

/// Two held Jupiters stack to X2.25.
#[test]
fn test_observatory_stacks_per_held_planet() {
    use crate::types::PlanetCard;
    let played = flush_2_3_4_5_7();
    let mut gs = apply_voucher_to_game(VoucherKind::Observatory);
    for _ in 0..2 {
        gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Jupiter).into());
    }
    // 56 chips × (4 × 1.5 × 1.5 = 9) = 504
    assert_eq!(score_with_held_planets(&gs, &played).final_score as i64, 504);
}

/// A held Planet only pays for its own hand type.
#[test]
fn test_observatory_ignores_a_planet_for_another_hand() {
    use crate::types::PlanetCard;
    let played = flush_2_3_4_5_7();
    let mut gs = apply_voucher_to_game(VoucherKind::Observatory);
    // Mercury levels Pair, not Flush.
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Mercury).into());
    assert_eq!(score_with_held_planets(&gs, &played).final_score as i64, 224);
}

/// Spending the Planet levels the hand and gives the X1.5 up.
#[test]
fn test_observatory_x_mult_is_lost_when_the_planet_is_used() {
    use crate::types::PlanetCard;
    let played = flush_2_3_4_5_7();
    let mut gs = apply_voucher_to_game(VoucherKind::Observatory);
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Jupiter).into());
    gs.use_consumable(0, vec![]).unwrap();
    assert!(gs.consumables.is_empty());
    // Flush is now L2 (chips 50, mult 6) and nothing is held: 71 × 6 = 426.
    assert_eq!(score_with_held_planets(&gs, &played).final_score as i64, 426,
        "a spent Planet leaves a level behind, not a multiplier");
}

/// A plain 2-3-4-5-7 of Spades — 56 chips at Flush level 1.
fn flush_2_3_4_5_7() -> Vec<CardInstance> {
    vec![
        card(0, Rank::Two,   Suit::Spades),
        card(1, Rank::Three, Suit::Spades),
        card(2, Rank::Four,  Suit::Spades),
        card(3, Rank::Five,  Suit::Spades),
        card(4, Rank::Seven, Suit::Spades),
    ]
}

/// Score `played` against the game's hand levels and whatever Planets it is holding.
fn score_with_held_planets(gs: &GameState, played: &[CardInstance]) -> crate::scoring::ScoreResult {
    let observatory: Vec<HandType> = match gs.has_voucher(VoucherKind::Observatory) {
        true => gs.consumables.iter().filter_map(|c| match c.card {
            crate::card::ConsumableCard::Planet(p) => Some(p.hand_type()),
            _ => None,
        }).collect(),
        false => Vec::new(),
    };
    let jokers = [];
    let mut si = ScoreInputs::new(played, &[], &jokers, &gs.hand_levels);
    si.hands_remaining = 3;
    si.discards_remaining = 3;
    si.deck_cards_remaining = 40;
    si.observatory_planets = &observatory;
    score_hand(si)
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
    // The same round twice, differing only in whether the card left in hand carries a debuffed
    // Gold enhancement. Comparing the two deltas keeps the test about Gold rather than about
    // whatever the blind reward and interest happen to come to.
    let round_delta = |gold: bool| {
        let mut gs = make_game();
        let mut held = card(1, Rank::Ace, Suit::Spades);
        if gold {
            held.enhancement = Enhancement::Gold;
            held.debuffed = true;
        }
        let trigger = card(2, Rank::Two, Suit::Spades);
        setup_round(&mut gs, vec![held, trigger], 2);
        gs.score_goal = 1.0;
        gs.select_card(1).unwrap(); // play the plain card, keep the Gold one in hand
        let money_before = gs.money;
        gs.play_hand().unwrap();
        gs.money - money_before
    };
    assert_eq!(
        round_delta(true), round_delta(false),
        "a debuffed Gold card held at end of round must pay nothing"
    );
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

// =========================================================
// Shop composition and the rate vouchers
// =========================================================

fn shop_item_types(gs: &mut GameState, rounds: usize) -> std::collections::HashMap<&'static str, usize> {
    let mut counts = std::collections::HashMap::new();
    for _ in 0..rounds {
        gs.generate_shop();
        for offer in &gs.shop_offers {
            let key = match &offer.kind {
                ShopItem::Joker(_) => "joker",
                ShopItem::Consumable(crate::card::ConsumableCard::Tarot(_)) => "tarot",
                ShopItem::Consumable(crate::card::ConsumableCard::Planet(_)) => "planet",
                ShopItem::Consumable(crate::card::ConsumableCard::Spectral(_)) => "spectral",
                ShopItem::PlayingCard(_) => "playing_card",
                ShopItem::Pack(_) => "pack",
                ShopItem::Voucher(_) => "voucher",
            };
            *counts.entry(key).or_insert(0) += 1;
        }
    }
    counts
}

#[test]
fn test_shop_stocks_consumables_alongside_jokers() {
    let mut gs = make_game();
    let counts = shop_item_types(&mut gs, 300);
    assert!(counts.get("joker").copied().unwrap_or(0) > 0);
    assert!(counts.get("tarot").copied().unwrap_or(0) > 0, "shop should stock tarots");
    assert!(counts.get("planet").copied().unwrap_or(0) > 0, "shop should stock planets");
}

#[test]
fn test_shop_offers_two_booster_packs() {
    let mut gs = make_game();
    gs.generate_shop();
    let packs = gs.shop_offers.iter().filter(|o| matches!(o.kind, ShopItem::Pack(_))).count();
    assert_eq!(packs, 2);
}

#[test]
fn test_jokers_dominate_the_default_card_slots() {
    // 20 / 4 / 4 weighting (game.lua:1901).
    let mut gs = make_game();
    let counts = shop_item_types(&mut gs, 400);
    let jokers = counts.get("joker").copied().unwrap_or(0) as f64;
    let tarots = counts.get("tarot").copied().unwrap_or(0) as f64;
    assert!(jokers > tarots * 3.0, "jokers {} tarots {}", jokers, tarots);
}

#[test]
fn test_no_playing_cards_or_spectrals_in_the_shop_by_default() {
    let mut gs = make_game();
    let counts = shop_item_types(&mut gs, 200);
    assert_eq!(counts.get("playing_card").copied().unwrap_or(0), 0);
    assert_eq!(counts.get("spectral").copied().unwrap_or(0), 0);
}

#[test]
fn test_magic_trick_puts_playing_cards_in_the_shop() {
    let mut gs = make_game();
    gs.vouchers.push(VoucherKind::MagicTrick);
    gs.playing_card_rate = 4.0;
    let counts = shop_item_types(&mut gs, 200);
    assert!(counts.get("playing_card").copied().unwrap_or(0) > 0);
}

#[test]
fn test_ghost_deck_puts_spectrals_in_the_shop() {
    let mut gs = GameState::new(DeckType::Ghost, Stake::White, Some("GHOSTSHOP".to_string()));
    let counts = shop_item_types(&mut gs, 400);
    assert!(counts.get("spectral").copied().unwrap_or(0) > 0);
}

#[test]
fn test_tarot_merchant_raises_the_tarot_rate() {
    let gs = apply_voucher_to_game(VoucherKind::TarotMerchant);
    assert!((gs.tarot_rate - 9.6).abs() < 1e-9);
    let gs = apply_voucher_to_game(VoucherKind::TarotTycoon);
    assert_eq!(gs.tarot_rate, 32.0);
}

#[test]
fn test_reroll_vouchers_cut_the_reroll_price() {
    let base = make_game().base_reroll_cost;
    let gs = apply_voucher_to_game(VoucherKind::RerollSurplus);
    assert_eq!(gs.base_reroll_cost, base - 2);
}

#[test]
fn test_hone_and_glow_up_scale_the_edition_rate() {
    assert_eq!(make_game().edition_rate, 1.0);
    assert_eq!(apply_voucher_to_game(VoucherKind::Hone).edition_rate, 2.0);
    assert_eq!(apply_voucher_to_game(VoucherKind::GlowUp).edition_rate, 4.0);
}

#[test]
fn test_hieroglyph_and_petroglyph_trade_an_ante_for_resources() {
    let mut gs = make_game();
    gs.ante = 3;
    let hands = gs.max_hands;
    let discards = gs.max_discards;

    gs.state = GameStateKind::Shop;
    gs.money = 50;
    gs.shop_voucher = Some(VoucherKind::Hieroglyph);
    gs.buy_voucher().unwrap();
    assert_eq!(gs.ante, 2);
    assert_eq!(gs.max_hands, hands - 1);

    gs.money = 50;
    gs.shop_voucher = Some(VoucherKind::Petroglyph);
    gs.buy_voucher().unwrap();
    assert_eq!(gs.ante, 1);
    assert_eq!(gs.max_discards, discards - 1);
}

// =========================================================
// Omen Globe and Telescope shape their packs
// =========================================================

#[test]
fn test_omen_globe_seeds_spectrals_into_arcana_packs() {
    let mut gs = make_game();
    gs.vouchers.push(VoucherKind::OmenGlobe);
    let mut spectrals = 0;
    for _ in 0..200 {
        let pack = gs.generate_pack_contents(PackKind::ArcanaPack);
        spectrals += pack
            .cards
            .iter()
            .filter(|c| matches!(c, PackCard::Consumable(crate::card::ConsumableCard::Spectral(_))))
            .count();
    }
    assert!(spectrals > 0, "Omen Globe should occasionally swap in a Spectral");
}

#[test]
fn test_telescope_leads_celestial_packs_with_the_most_played_planet() {
    let mut gs = make_game();
    gs.vouchers.push(VoucherKind::Telescope);
    gs.hand_levels.get_mut(&HandType::Flush).unwrap().played = 9;

    for _ in 0..20 {
        let pack = gs.generate_pack_contents(PackKind::CelestialPack);
        match &pack.cards[0] {
            PackCard::Consumable(crate::card::ConsumableCard::Planet(p)) => {
                assert_eq!(p.hand_type(), HandType::Flush);
            }
            other => panic!("expected a planet, got {:?}", other),
        }
    }
}

#[test]
fn test_celestial_packs_are_random_without_telescope() {
    let mut gs = make_game();
    gs.hand_levels.get_mut(&HandType::Flush).unwrap().played = 9;
    let mut firsts = std::collections::HashSet::new();
    for _ in 0..60 {
        let pack = gs.generate_pack_contents(PackKind::CelestialPack);
        if let PackCard::Consumable(crate::card::ConsumableCard::Planet(p)) = &pack.cards[0] {
            firsts.insert(*p);
        }
    }
    assert!(firsts.len() > 1);
}

// =========================================================
// Shop pricing: edition surcharges and rounding
// =========================================================

/// card.lua:369 adds a flat surcharge per edition before the discount is applied.
#[test]
fn test_editions_raise_the_shop_price() {
    let gs = make_game();
    let price_of = |edition: Edition| {
        let mut j = joker(0, JokerKind::Blueprint); // base cost 10
        j.edition = edition;
        gs.debug_joker_price(&j)
    };

    assert_eq!(price_of(Edition::None), 10);
    assert_eq!(price_of(Edition::Foil), 12);
    assert_eq!(price_of(Edition::Holographic), 13);
    assert_eq!(price_of(Edition::Polychrome), 15);
    assert_eq!(price_of(Edition::Negative), 15);
}

/// A rental is $1 to acquire whatever it is (card.lua:381).
#[test]
fn test_rental_overrides_the_shop_price() {
    let gs = make_game();
    let mut j = joker(0, JokerKind::Blueprint);
    j.edition = Edition::Polychrome;
    j.rental = true;
    assert_eq!(gs.debug_joker_price(&j), 1);
}

#[test]
fn test_discounts_apply_after_the_edition_surcharge() {
    let mut gs = make_game();
    gs.vouchers.push(VoucherKind::ClearanceSale);
    let mut poly = joker(0, JokerKind::Blueprint);
    poly.edition = Edition::Polychrome;
    // (10 + 5 + 0.5) * 0.75 = 11.625 -> 11
    assert_eq!(gs.debug_joker_price(&poly), 11);
}

#[test]
fn test_the_boss_can_only_be_rerolled_from_the_blind_select_screen() {
    let mut gs = make_game();
    gs.vouchers.push(VoucherKind::DirectorsCut);
    gs.money = 100;

    gs.state = GameStateKind::Shop;
    assert!(matches!(gs.reroll_boss_blind(), Err(crate::game::BalatroError::NotInBlindSelect)),
        "the reroll button lives on the blind-select screen");

    gs.state = GameStateKind::BlindSelect;
    assert!(gs.reroll_boss_blind().is_ok());
    assert_eq!(gs.money, 90, "and it costs $10");
}
