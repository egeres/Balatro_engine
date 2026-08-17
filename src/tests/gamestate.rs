/// Tests for GameState round-play integration.

use super::*;
use crate::card::ConsumableCard;
use crate::game::BalatroError;

#[test]
fn test_play_pair_through_game_state() {
    let mut gs = make_game();
    setup_round(
        &mut gs,
        vec![
            card(100, Rank::Ace, Suit::Spades),
            card(101, Rank::Ace, Suit::Hearts),
            card(102, Rank::Two, Suit::Clubs),
            card(103, Rank::Three, Suit::Diamonds),
            card(104, Rank::Four, Suit::Spades),
        ],
        5,
    );

    gs.select_card(0).unwrap();
    gs.select_card(1).unwrap();
    gs.play_hand().unwrap();

    // Pair of Aces = 64
    assert!(gs.score_accumulated >= 64.0);
    assert_eq!(gs.hands_remaining, gs.effective_max_hands() - 1);
}

#[test]
fn test_play_flush_through_game_state() {
    let mut gs = make_game();
    setup_round(
        &mut gs,
        vec![
            card(100, Rank::Ace, Suit::Spades),
            card(101, Rank::Three, Suit::Spades),
            card(102, Rank::Seven, Suit::Spades),
            card(103, Rank::Nine, Suit::Spades),
            card(104, Rank::Two, Suit::Spades),
        ],
        5,
    );

    for i in 0..5 {
        gs.select_card(i).unwrap();
    }
    gs.play_hand().unwrap();

    // Flush = 268
    assert!((gs.score_accumulated - 268.0).abs() < 1.0);
}

#[test]
fn test_hands_remaining_decrements_per_play() {
    let mut gs = make_game();
    setup_round(
        &mut gs,
        (0..52)
            .map(|i| card(i as u64, Rank::Ace, Suit::Spades))
            .collect(),
        8,
    );

    let before = gs.hands_remaining;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert_eq!(gs.hands_remaining, before - 1);
}

#[test]
fn test_discard_reduces_discards_remaining() {
    let mut gs = make_game();
    setup_round(
        &mut gs,
        (0..52)
            .map(|i| card(i as u64, Rank::Ace, Suit::Spades))
            .collect(),
        8,
    );

    let before = gs.discards_remaining;
    gs.select_card(0).unwrap();
    gs.discard_hand().unwrap();
    assert_eq!(gs.discards_remaining, before - 1);
}

#[test]
fn test_glass_card_does_not_always_break() {
    // Glass cards break 1/4 of the time; play 20 rounds, not all should break.
    let mut break_count = 0u32;
    for seed_n in 0..20u32 {
        let mut gs = GameState::new(
            DeckType::Blue,
            Stake::White,
            Some(format!("GLASSSEED{seed_n}")),
        );
        let mut glass_ace = card(200, Rank::Ace, Suit::Spades);
        glass_ace.enhancement = Enhancement::Glass;
        setup_round(&mut gs, vec![glass_ace, card(201, Rank::Two, Suit::Clubs)], 2);
        let initial_deck_len = gs.deck.len();
        gs.select_card(0).unwrap();
        gs.play_hand().unwrap();
        if gs.deck.len() < initial_deck_len {
            break_count += 1;
        }
    }
    assert!(break_count <= 15, "Glass broke {break_count}/20 times — too often");
    assert!(break_count >= 1, "Glass never broke across 20 attempts — likely broken logic");
}

#[test]
fn test_score_accumulates_across_multiple_plays() {
    let mut gs = make_game();
    setup_round(
        &mut gs,
        (0..20)
            .map(|i| card(i as u64, Rank::Ace, Suit::Spades))
            .collect(),
        8,
    );
    gs.score_goal = f64::MAX; // prevent auto-win after first hand

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    let after_first = gs.score_accumulated;
    assert!(after_first > 0.0);

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert!(gs.score_accumulated > after_first);
}

#[test]
fn test_joker_applies_to_every_hand_played() {
    let mut gs_with = make_game();
    let mut gs_without = make_game();

    let deck: Vec<CardInstance> = (0..10)
        .map(|i| card(i as u64, Rank::Ace, Suit::Spades))
        .collect();

    setup_round(&mut gs_with, deck.clone(), 5);
    gs_with.jokers.push(joker(999, JokerKind::Joker));

    setup_round(&mut gs_without, deck, 5);

    for gs in [&mut gs_with, &mut gs_without] {
        gs.select_card(0).unwrap();
        gs.play_hand().unwrap();
    }

    assert!(gs_with.score_accumulated > gs_without.score_accumulated);
}

// =========================================================
// swap_jokers
// =========================================================

#[test]
fn test_swap_jokers_changes_order() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Joker));
    gs.jokers.push(joker(1, JokerKind::AbstractJoker));
    gs.jokers.push(joker(2, JokerKind::HalfJoker));

    gs.swap_jokers(0, 2).unwrap();

    assert_eq!(gs.jokers[0].kind, JokerKind::HalfJoker);
    assert_eq!(gs.jokers[1].kind, JokerKind::AbstractJoker);
    assert_eq!(gs.jokers[2].kind, JokerKind::Joker);
}

#[test]
fn test_swap_jokers_adjacent() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Joker));
    gs.jokers.push(joker(1, JokerKind::GreenJoker));

    gs.swap_jokers(0, 1).unwrap();

    assert_eq!(gs.jokers[0].kind, JokerKind::GreenJoker);
    assert_eq!(gs.jokers[1].kind, JokerKind::Joker);
}

#[test]
fn test_swap_jokers_same_index_is_noop() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Joker));
    gs.jokers.push(joker(1, JokerKind::GreenJoker));

    gs.swap_jokers(1, 1).unwrap();

    assert_eq!(gs.jokers[0].kind, JokerKind::Joker);
    assert_eq!(gs.jokers[1].kind, JokerKind::GreenJoker);
}

#[test]
fn test_swap_jokers_out_of_range_first_index() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Joker));

    let err = gs.swap_jokers(5, 0).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(5, 1)));
}

#[test]
fn test_swap_jokers_out_of_range_second_index() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Joker));

    let err = gs.swap_jokers(0, 5).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(5, 1)));
}

#[test]
fn test_swap_jokers_empty_list_returns_error() {
    let mut gs = make_game();
    let err = gs.swap_jokers(0, 1).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(_, _)));
}

#[test]
fn test_swap_jokers_order_affects_blueprint_scoring() {
    // Blueprint copies the joker immediately to its right.
    // [Blueprint, Joker] → Blueprint copies Joker's +4 mult → total mult = 1+4+4 = 9, chips=16 → 144
    // After swapping to [Joker, Blueprint] → Blueprint has nothing to its right → no copy → mult=1+4=5 → 80
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers_before = vec![joker(0, JokerKind::Blueprint), joker(1, JokerKind::Joker)];
    let jokers_after  = vec![joker(1, JokerKind::Joker), joker(0, JokerKind::Blueprint)];

    let r_before = score(&played, &played, &jokers_before);
    let r_after  = score(&played, &played, &jokers_after);

    assert_eq!(r_before.final_score as i64, 144, "Blueprint + Joker should score 144");
    assert_eq!(r_after.final_score  as i64,  80, "Joker + Blueprint (nothing right) should score 80");
}

// =========================================================
// swap_consumables
// =========================================================

#[test]
fn test_swap_consumables_changes_order() {
    let mut gs = make_game();
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheFool));
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheMagician));
    gs.consumables.push(ConsumableCard::Planet(PlanetCard::Mercury));

    gs.swap_consumables(0, 2).unwrap();

    assert_eq!(gs.consumables[0], ConsumableCard::Planet(PlanetCard::Mercury));
    assert_eq!(gs.consumables[1], ConsumableCard::Tarot(TarotCard::TheMagician));
    assert_eq!(gs.consumables[2], ConsumableCard::Tarot(TarotCard::TheFool));
}

#[test]
fn test_swap_consumables_adjacent() {
    let mut gs = make_game();
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheSun));
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheMoon));

    gs.swap_consumables(0, 1).unwrap();

    assert_eq!(gs.consumables[0], ConsumableCard::Tarot(TarotCard::TheMoon));
    assert_eq!(gs.consumables[1], ConsumableCard::Tarot(TarotCard::TheSun));
}

#[test]
fn test_swap_consumables_same_index_is_noop() {
    let mut gs = make_game();
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheSun));
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheMoon));

    gs.swap_consumables(0, 0).unwrap();

    assert_eq!(gs.consumables[0], ConsumableCard::Tarot(TarotCard::TheSun));
    assert_eq!(gs.consumables[1], ConsumableCard::Tarot(TarotCard::TheMoon));
}

#[test]
fn test_swap_consumables_out_of_range_first_index() {
    let mut gs = make_game();
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheFool));

    let err = gs.swap_consumables(3, 0).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(3, 1)));
}

#[test]
fn test_swap_consumables_out_of_range_second_index() {
    let mut gs = make_game();
    gs.consumables.push(ConsumableCard::Tarot(TarotCard::TheFool));

    let err = gs.swap_consumables(0, 3).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(3, 1)));
}

#[test]
fn test_swap_consumables_empty_list_returns_error() {
    let mut gs = make_game();
    let err = gs.swap_consumables(0, 1).unwrap_err();
    assert!(matches!(err, BalatroError::IndexOutOfRange(_, _)));
}

// =========================================================
// Round-wide joker targets (The Idol / Ancient / Castle / Mail-In Rebate)
// =========================================================

/// The Idol, Castle and Mail all sample a real card from the deck, so their targets must be
/// ranks/suits that actually exist in it.
#[test]
fn test_round_targets_are_drawn_from_the_deck() {
    let mut gs = make_game();
    gs.select_blind().unwrap();
    let t = gs.round_targets;
    assert!(gs.deck.iter().any(|c| c.rank == t.idol_rank && c.suit == t.idol_suit));
    assert!(gs.deck.iter().any(|c| c.suit == t.castle_suit));
    assert!(gs.deck.iter().any(|c| c.rank == t.mail_rank));
}

/// Ancient Joker picks a suit different from the previous round's (common_events.lua:2303).
#[test]
fn test_ancient_suit_always_changes_between_rounds() {
    let mut gs = make_game();
    gs.select_blind().unwrap();
    let mut prev = gs.round_targets.ancient_suit;
    for _ in 0..8 {
        gs.state = GameStateKind::BlindSelect;
        gs.select_blind().unwrap();
        let next = gs.round_targets.ancient_suit;
        assert_ne!(next, prev, "Ancient Joker suit must change every round");
        prev = next;
    }
}

/// Targets are re-rolled per round rather than frozen at joker creation.
#[test]
fn test_round_targets_are_rerolled_each_round() {
    let mut gs = make_game();
    gs.select_blind().unwrap();
    let first = gs.round_targets;
    let mut changed = false;
    for _ in 0..20 {
        gs.state = GameStateKind::BlindSelect;
        gs.select_blind().unwrap();
        if gs.round_targets.idol_rank != first.idol_rank
            || gs.round_targets.idol_suit != first.idol_suit
            || gs.round_targets.castle_suit != first.castle_suit
            || gs.round_targets.mail_rank != first.mail_rank
        {
            changed = true;
            break;
        }
    }
    assert!(changed, "round targets should be re-rolled between rounds");
}

// =========================================================
// Shop joker pool
// =========================================================

/// Draw a lot of jokers and report which kinds and rarities came out.
fn sample_pool(gs: &mut GameState, n: usize) -> Vec<JokerKind> {
    (0..n)
        .filter_map(|_| gs.generate_random_joker().map(|j| j.kind))
        .collect()
}

#[test]
fn test_pool_is_not_limited_to_a_handful_of_jokers() {
    let mut gs = make_game();
    let sample = sample_pool(&mut gs, 4000);
    let distinct: std::collections::HashSet<_> = sample.iter().copied().collect();
    assert!(
        distinct.len() > 90,
        "expected most of the roster to be reachable, saw {}",
        distinct.len()
    );
}

#[test]
fn test_pool_never_yields_legendaries() {
    let mut gs = make_game();
    let sample = sample_pool(&mut gs, 2000);
    assert!(
        sample.iter().all(|k| k.rarity() != 4),
        "legendary jokers come from The Soul only"
    );
}

#[test]
fn test_pool_is_rarity_weighted() {
    let mut gs = make_game();
    let sample = sample_pool(&mut gs, 4000);
    let frac = |r: u8| sample.iter().filter(|k| k.rarity() == r).count() as f64 / sample.len() as f64;
    // 70 / 25 / 5, with slack for sampling noise.
    assert!((frac(1) - 0.70).abs() < 0.05, "common {}", frac(1));
    assert!((frac(2) - 0.25).abs() < 0.05, "uncommon {}", frac(2));
    assert!((frac(3) - 0.05).abs() < 0.03, "rare {}", frac(3));
}

#[test]
fn test_enhancement_gated_jokers_stay_out_until_the_deck_qualifies() {
    let mut gs = make_game();
    // A fresh deck has no enhancements at all.
    assert!(!gs.joker_in_pool(JokerKind::SteelJoker));
    assert!(!gs.joker_in_pool(JokerKind::GlassJoker));
    assert!(!gs.joker_in_pool(JokerKind::GoldenTicket));

    gs.deck[0].enhancement = Enhancement::Steel;
    assert!(gs.joker_in_pool(JokerKind::SteelJoker));
    assert!(!gs.joker_in_pool(JokerKind::GlassJoker));
}

#[test]
fn test_gros_michel_and_cavendish_swap_places_on_extinction() {
    let mut gs = make_game();
    assert!(gs.joker_in_pool(JokerKind::GrosMichel));
    assert!(!gs.joker_in_pool(JokerKind::Cavendish));

    gs.gros_michel_extinct = true;
    assert!(!gs.joker_in_pool(JokerKind::GrosMichel));
    assert!(gs.joker_in_pool(JokerKind::Cavendish));
}

// =========================================================
// Showman: lifts the no-duplicates rule
// =========================================================

#[test]
fn test_a_held_joker_is_excluded_from_the_pool() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Fibonacci));
    assert!(!gs.joker_in_pool(JokerKind::Fibonacci));
    assert!(gs.joker_in_pool(JokerKind::Scholar));
}

#[test]
fn test_showman_puts_held_jokers_back_in_the_pool() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Fibonacci));
    gs.jokers.push(joker(1, JokerKind::Showman));
    assert!(gs.joker_in_pool(JokerKind::Fibonacci));
    assert!(gs.joker_in_pool(JokerKind::Showman));
}

#[test]
fn test_a_shop_offer_blocks_a_second_copy() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.generate_shop();
    let offered: Vec<JokerKind> = gs
        .shop_offers
        .iter()
        .filter_map(|o| match &o.kind {
            crate::card::ShopItem::Joker(j) => Some(j.kind),
            _ => None,
        })
        .collect();
    assert!(offered.len() >= 2);
    for k in &offered {
        assert!(!gs.joker_in_pool(*k), "{:?} is on the shelf and should be blocked", k);
    }
}

#[test]
fn test_a_shop_never_stocks_the_same_joker_twice() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    for _ in 0..50 {
        gs.generate_shop();
        let offered: Vec<JokerKind> = gs
            .shop_offers
            .iter()
            .filter_map(|o| match &o.kind {
                crate::card::ShopItem::Joker(j) => Some(j.kind),
                _ => None,
            })
            .collect();
        let distinct: std::collections::HashSet<_> = offered.iter().copied().collect();
        assert_eq!(distinct.len(), offered.len(), "duplicate joker in one shop: {:?}", offered);
    }
}

#[test]
fn test_pool_still_yields_jokers_with_showman_and_a_full_board() {
    let mut gs = make_game();
    gs.jokers.push(joker(0, JokerKind::Showman));
    for (i, k) in JokerKind::ALL.iter().take(40).enumerate() {
        gs.jokers.push(joker(100 + i as u64, *k));
    }
    assert!(gs.generate_random_joker().is_some());
}
