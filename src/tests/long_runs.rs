/// Long-run tests that simulate real multi-round game sessions.
///
/// Unlike the unit tests elsewhere in this suite, these tests exercise
/// the full GameState lifecycle end-to-end:
///
///   BlindSelect → select_blind() → Round → play_hand() → win_round()
///       → Shop → buy/sell jokers, open packs, use consumables → leave_shop()
///       → BlindSelect (next blind) → ...
///
/// Covered scenarios:
///   - `run_joker_economy_across_three_blinds`:
///       Buy and sell jokers across all three Ante 1 blinds, verifying exact
///       money at every step (blind rewards, interest, purchase prices, sell values).
///
///   - `run_pack_opening_and_consumable_chain`:
///       Buy a Celestial pack in the shop, override its contents for determinism,
///       take a Mercury planet card from the pack, apply it to raise Pair to level 2,
///       then inject a TheEmpress tarot in-round and confirm the Mult enhancement it
///       applies survives the rest of the round.
///
///   - `run_stickers_perishable_rental_eternal`:
///       Attach the three sticker types directly to jokers before any round, then
///       verify: perishable jokers count down and go inactive after the configured
///       number of win_rounds; rental jokers deduct $1 from money each time the
///       shop is left; and eternal jokers return `BalatroError::EternalCard` when
///       a sell is attempted.
///
/// Reproducibility: every test passes a named seed to GameState::new so that RNG
/// outputs (boss blind selection, deck shuffle, shop generation) are deterministic.
/// Where shop or pack contents matter for the test, those are overwritten in-place
/// so the test does not depend on RNG output.

use super::*;
use crate::card::{ConsumableCard, PackCard, ShopItem, ShopOffer};
use crate::game::{BalatroError, BlindKind};

// =========================================================
// Shared helpers
// =========================================================

/// Force-win the currently active round without needing a powerful deck.
///
/// Call this immediately after `select_blind()`. It:
///   1. Overrides `score_goal` to 1.0 so any card is enough to win.
///   2. Selects hand card at position 0.
///   3. Plays the hand, triggering `win_round()` and transitioning to `Shop`.
fn force_win(gs: &mut GameState) {
    gs.score_goal = 1.0;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert!(
        matches!(gs.state, GameStateKind::Shop),
        "force_win: expected Shop state after winning, got {:?}",
        gs.state
    );
}

// =========================================================
// Test 1 – Joker economy across three Ante 1 blinds
// =========================================================

/// Win all three Ante 1 blinds while buying and selling jokers in each shop.
///
/// Money trace (Blue deck, White stake, starting money $4):
///
///   Small blind win  → +$3 reward, money=$7 → +$1 interest (floor(7/5)=1) → $8
///   Buy AbstractJoker ($4)                                                  → $4
///   leave_shop (no rentals)                                                 → $4
///
///   Big blind win    → +$4 reward, money=$8 → +$1 interest (floor(8/5)=1)  → $9
///   Sell AbstractJoker (+$2, sell_value=(4+1)/2=2)                          → $11
///   Buy Scholar ($4)                                                        → $7
///   leave_shop (no rentals)                                                 → $7
///
///   Boss blind win   → +$5 reward, money=$12 → +$2 interest (floor(12/5)=2) → $14
///   leave_shop → ante advances to 2                                          → $14
///
/// Interest formula: floor(money_after_reward / 5), capped at max_interest/5.
/// At White stake max_interest=25, so cap is 5 — not reached here.
#[test]
fn run_joker_economy_across_three_blinds() {
    let mut gs = GameState::new(DeckType::Blue, Stake::White, Some("ECONOMY1".to_string()));

    // Blue deck: max_hands=5, starting money=$4.
    assert_eq!(gs.money, 4);
    assert_eq!(gs.ante, 1);
    assert!(matches!(gs.current_blind, BlindKind::Small));

    // ── Small blind ──────────────────────────────────────────────
    gs.select_blind().unwrap();
    force_win(&mut gs);
    // reward=$3 → money=7; interest=floor(7/5)=1 → money=8
    assert_eq!(gs.money, 8, "after small blind win: expected $8");

    // Inject AbstractJoker (base_cost=4) at the front of shop offers.
    gs.shop_offers.insert(0, ShopOffer {
        kind: ShopItem::Joker(JokerInstance::new(1000, JokerKind::AbstractJoker, Edition::None)),
        price: 4,
        sold: false,
    });

    gs.buy_joker(0).unwrap();
    assert_eq!(gs.money, 4, "after buying AbstractJoker ($4): expected $4");
    assert_eq!(gs.jokers.len(), 1);
    assert_eq!(gs.jokers[0].kind, JokerKind::AbstractJoker);

    // Not a rental joker → no cost on leave_shop.
    gs.leave_shop().unwrap();
    assert_eq!(gs.money, 4);
    assert!(matches!(gs.current_blind, BlindKind::Big));

    // ── Big blind ────────────────────────────────────────────────
    gs.select_blind().unwrap();
    force_win(&mut gs);
    // reward=$4 → money=8; interest=floor(8/5)=1 → money=9
    assert_eq!(gs.money, 9, "after big blind win: expected $9");

    // Sell AbstractJoker: sell_value = (base_cost + 1) / 2 = (4+1)/2 = 2.
    gs.sell_joker(0).unwrap();
    assert_eq!(gs.money, 11, "after selling AbstractJoker (+$2): expected $11");
    assert!(gs.jokers.is_empty());

    // Inject Scholar (base_cost=4) at the front.
    gs.shop_offers.insert(0, ShopOffer {
        kind: ShopItem::Joker(JokerInstance::new(1001, JokerKind::Scholar, Edition::None)),
        price: 4,
        sold: false,
    });

    gs.buy_joker(0).unwrap();
    assert_eq!(gs.money, 7, "after buying Scholar ($4): expected $7");
    assert_eq!(gs.jokers[0].kind, JokerKind::Scholar);

    gs.leave_shop().unwrap();
    assert_eq!(gs.money, 7);
    assert!(matches!(gs.current_blind, BlindKind::Boss));

    // ── Boss blind ───────────────────────────────────────────────
    // Override boss_blind to None to avoid boss-specific per-card money effects
    // (e.g. TheTooth deducts $1 per card played). This preserves the $5 base reward
    // and neutral debuff behaviour so the money assertions below remain exact.
    gs.boss_blind = None;
    gs.select_blind().unwrap();
    force_win(&mut gs);
    // reward=$5 → money=12; interest=floor(12/5)=2 → money=14
    assert_eq!(gs.money, 14, "after boss blind win: expected $14");

    // Leave boss shop → ante advances to 2.
    gs.leave_shop().unwrap();
    assert_eq!(gs.ante, 2, "should have advanced to Ante 2 after boss blind");
    assert!(matches!(gs.current_blind, BlindKind::Small));

    // Scholar persists into the new ante.
    assert_eq!(gs.jokers.len(), 1);
    assert_eq!(gs.jokers[0].kind, JokerKind::Scholar, "Scholar should survive into Ante 2");
}

// =========================================================
// Test 2 – Pack opening and consumable chain
// =========================================================

/// Open a Celestial pack with deterministic contents, take a Mercury planet card,
/// apply it to raise Pair to level 2, then use TheEmpress tarot in-round to add a
/// Mult enhancement to a hand card, and verify both changes persist.
///
/// Sequence:
///   Ante 1, Small blind → win → shop
///   Inject CelestialPack ($4) → buy_pack → override contents with one Mercury
///   take_pack_card(0) → Mercury in consumables → use it → Pair level 1→2
///   leave_shop → Ante 1, Big blind
///   select_blind → hand drawn → inject TheEmpress → use on hand[0]
///   verify hand[0] deck card has Mult enhancement
///   win round → verify Pair still level 2 and Mult enhancement still on deck card
#[test]
fn run_pack_opening_and_consumable_chain() {
    let mut gs = GameState::new(DeckType::Blue, Stake::White, Some("PACKTEST1".to_string()));

    // ── Win Small blind ──────────────────────────────────────────
    gs.select_blind().unwrap();
    force_win(&mut gs);
    // money = 4+3+1 = 8

    // ── Shop: buy Celestial pack with known contents ─────────────
    // Replace shop offers entirely so buy_pack(0) is deterministic.
    gs.shop_offers = vec![ShopOffer {
        kind: ShopItem::Pack(PackKind::CelestialPack),
        price: 4,
        sold: false,
    }];

    gs.buy_pack(0).unwrap();
    assert_eq!(gs.money, 4, "after buying CelestialPack ($4) from $8: expected $4");
    assert!(matches!(gs.state, GameStateKind::BoosterPack));

    // Override pack contents: exactly one Mercury, one pick.
    {
        let pack = gs.current_pack.as_mut().unwrap();
        pack.cards = vec![PackCard::Consumable(ConsumableCard::Planet(PlanetCard::Mercury))];
        pack.picks_remaining = 1;
    }

    // Take Mercury from the pack.
    // picks_remaining hits 0 → skip_pack() fires automatically → state returns to Shop.
    gs.take_pack_card(0).unwrap();
    assert!(
        matches!(gs.state, GameStateKind::Shop),
        "after taking last pack card state should be Shop"
    );
    assert_eq!(gs.consumables.len(), 1);
    assert!(
        matches!(gs.consumables[0], ConsumableCard::Planet(PlanetCard::Mercury)),
        "consumables[0] should be Mercury"
    );

    // Use Mercury → Pair upgrades from level 1 to level 2.
    let pair_level_before = gs.hand_levels[&HandType::Pair].level;
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(
        gs.hand_levels[&HandType::Pair].level,
        pair_level_before + 1,
        "Mercury should raise Pair by one level"
    );
    assert_eq!(
        gs.hand_levels[&HandType::Pair].level, 2,
        "Pair should be at level 2 after using Mercury"
    );

    // ── Leave shop → Big blind ───────────────────────────────────
    gs.leave_shop().unwrap();
    assert!(
        matches!(gs.current_blind, BlindKind::Big),
        "should be at Big blind after leaving shop"
    );

    // ── Big blind: apply TheEmpress in-round ─────────────────────
    // begin_round draws a fresh hand from the shuffled deck.
    gs.select_blind().unwrap();
    assert!(!gs.hand.is_empty(), "hand should have cards after begin_round");

    // Record which deck card is at hand position 0 — we will track its enhancement.
    let target_deck_idx = gs.hand[0];

    // Inject TheEmpress tarot directly into consumables (simulates having purchased it).
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheEmpress));

    // Apply TheEmpress to hand[0] → sets Mult enhancement on the underlying deck card.
    // Target is a hand-relative index (0), not a deck index.
    gs.use_consumable(0, vec![0]).unwrap();
    assert_eq!(
        gs.deck[target_deck_idx].enhancement,
        Enhancement::Mult,
        "TheEmpress should give the target card Mult enhancement"
    );

    // Win the round; the deck card retains its enhancement (played cards are only moved to
    // discard_pile, not removed from deck — only destroy_deck_card removes from deck).
    force_win(&mut gs);

    assert_eq!(
        gs.deck[target_deck_idx].enhancement,
        Enhancement::Mult,
        "Mult enhancement must persist on the deck card after the round ends"
    );
    // Planet-level upgrades are permanent — Pair must still be at level 2.
    assert_eq!(
        gs.hand_levels[&HandType::Pair].level, 2,
        "Pair level should remain 2 after the round"
    );
}

// =========================================================
// Test 3 – Sticker mechanics: perishable, rental, eternal
// =========================================================

/// Verify all three joker sticker types behave correctly across multiple rounds.
///
/// Three jokers are added directly before any blind is played:
///   - Scholar    (eternal=true, id=2000):   sell attempt must fail
///   - Joker base (rental=true,  id=2001):   costs $1 per leave_shop call
///   - GreenJoker (perishable, rounds_left=2, id=2002): disabled after 2 win_rounds
///
/// Sequence:
///   select_blind (Small) → force_win
///     → GreenJoker rounds_left: 2→1, active: still true
///   leave_shop → money deducted by $1 (rental Joker)
///
///   select_blind (Big) → force_win
///     → GreenJoker rounds_left: 1→0, active: false
///   try sell_joker on Scholar → Err(EternalCard)
///   leave_shop → money deducted by $1 again
///
///   All three jokers remain in the roster (none were destroyed by selling).
#[test]
fn run_stickers_perishable_rental_eternal() {
    let mut gs = GameState::new(DeckType::Blue, Stake::White, Some("STICKERTEST1".to_string()));

    // Add an eternal Scholar.
    let mut eternal_scholar = JokerInstance::new(2000, JokerKind::Scholar, Edition::None);
    eternal_scholar.eternal = true;
    gs.jokers.push(eternal_scholar);

    // Add a rental Joker base.
    let mut rental_joker = JokerInstance::new(2001, JokerKind::Joker, Edition::None);
    rental_joker.rental = true;
    gs.jokers.push(rental_joker);

    // Add a perishable GreenJoker with 2 rounds remaining.
    let mut perishable_green = JokerInstance::new(2002, JokerKind::GreenJoker, Edition::None);
    perishable_green.perishable = true;
    perishable_green.perishable_rounds_left = 2;
    gs.jokers.push(perishable_green);

    assert_eq!(gs.jokers.len(), 3);

    // ── Round 1 (Small blind) ────────────────────────────────────
    gs.select_blind().unwrap();
    force_win(&mut gs);

    // Perishable counter: 2 → 1; joker still active.
    let gi = gs.jokers.iter().position(|j| j.id == 2002).unwrap();
    assert_eq!(
        gs.jokers[gi].perishable_rounds_left, 1,
        "GreenJoker should have 1 round left after first win"
    );
    assert!(
        gs.jokers[gi].active,
        "GreenJoker should still be active after first win"
    );

    // leave_shop deducts $1 for the rental Joker.
    let money_before_leave1 = gs.money;
    gs.leave_shop().unwrap();
    assert_eq!(
        gs.money,
        money_before_leave1 - 1,
        "rental Joker should deduct $1 on leave_shop"
    );

    // ── Round 2 (Big blind) ──────────────────────────────────────
    gs.select_blind().unwrap();
    force_win(&mut gs);

    // Perishable counter: 1 → 0; joker is now disabled.
    let gi = gs.jokers.iter().position(|j| j.id == 2002).unwrap();
    assert_eq!(
        gs.jokers[gi].perishable_rounds_left, 0,
        "GreenJoker should have 0 rounds left after second win"
    );
    assert!(
        !gs.jokers[gi].active,
        "GreenJoker should be inactive (active=false) once perishable expires"
    );

    // Attempting to sell the eternal Scholar must fail.
    let si = gs.jokers.iter().position(|j| j.id == 2000).unwrap();
    let sell_result = gs.sell_joker(si);
    assert!(
        matches!(sell_result, Err(BalatroError::EternalCard)),
        "selling an Eternal joker must return EternalCard, got {:?}",
        sell_result
    );

    // Scholar is still in the roster after the failed sell.
    assert!(
        gs.jokers.iter().any(|j| j.id == 2000),
        "Scholar should remain in jokers after failed eternal sell"
    );

    // leave_shop deducts another $1 for the rental Joker.
    let money_before_leave2 = gs.money;
    gs.leave_shop().unwrap();
    assert_eq!(
        gs.money,
        money_before_leave2 - 1,
        "rental Joker should deduct $1 on second leave_shop"
    );

    // All three jokers remain — none were destroyed.
    assert_eq!(
        gs.jokers.len(), 3,
        "all three jokers should still be present (eternal, rental, disabled perishable)"
    );
}
