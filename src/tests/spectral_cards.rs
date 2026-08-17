/// Tests for spectral card application via GameState.
///
/// Spectral cards (18 total):
///   Familiar, Grim, Incantation — destroy hand card, add enhanced cards to deck
///   Talisman, DejaVu, Trance, Medium — add seal to target card
///   Aura — add edition to target card
///   Wraith — create rare joker, money → $0
///   TheSoul — create legendary joker
///   Ectoplasm — add Negative edition to joker, -1 hand size
///   Immolate — destroy up to 5 hand cards, +$20
///   Ankh — copy random joker, destroy the rest (spare eternals)
///   Hex — add Polychrome to random joker, destroy the rest (spare eternals)
///   Cryptid — create 2 copies of target card
///   Sigil — convert all hand cards to a random suit
///   Ouija — convert all hand cards to a random rank, -1 hand size
///   BlackHole — upgrade every hand level by 1

use super::*;

// Helper: apply a spectral card in a round context.
fn apply_spectral(
    deck_cards: Vec<CardInstance>,
    hand_size: usize,
    spectral: SpectralCard,
    targets: Vec<usize>,
) -> GameState {
    let mut gs = make_game();
    setup_round(&mut gs, deck_cards, hand_size);
    gs.consumables.push(crate::card::ConsumableCard::Spectral(spectral));
    gs.use_consumable(0, targets).unwrap();
    gs
}

fn five_card_hand() -> Vec<CardInstance> {
    (0..10).map(|i| card(i as u64, Rank::Ace, Suit::Spades)).collect()
}

// =========================================================
// Seal-adding spectrals
// =========================================================

#[test]
fn test_talisman_adds_gold_seal() {
    let gs = apply_spectral(five_card_hand(), 5, SpectralCard::Talisman, vec![0]);
    assert_eq!(gs.deck[gs.hand[0]].seal, Seal::Gold);
}

#[test]
fn test_deja_vu_adds_red_seal() {
    let gs = apply_spectral(five_card_hand(), 5, SpectralCard::DejaVu, vec![0]);
    assert_eq!(gs.deck[gs.hand[0]].seal, Seal::Red);
}

#[test]
fn test_trance_adds_blue_seal() {
    let gs = apply_spectral(five_card_hand(), 5, SpectralCard::Trance, vec![0]);
    assert_eq!(gs.deck[gs.hand[0]].seal, Seal::Blue);
}

#[test]
fn test_medium_adds_purple_seal() {
    let gs = apply_spectral(five_card_hand(), 5, SpectralCard::Medium, vec![0]);
    assert_eq!(gs.deck[gs.hand[0]].seal, Seal::Purple);
}

// =========================================================
// Edition-adding spectrals
// =========================================================

#[test]
fn test_aura_adds_edition_to_target_card() {
    let gs = apply_spectral(five_card_hand(), 5, SpectralCard::Aura, vec![0]);
    let edition = gs.deck[gs.hand[0]].edition;
    assert!(
        matches!(edition, Edition::Foil | Edition::Holographic | Edition::Polychrome),
        "Aura should add Foil, Holographic, or Polychrome — got {:?}", edition
    );
}

#[test]
fn test_ectoplasm_adds_negative_edition_to_a_joker() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    gs.jokers.push(joker(99, JokerKind::Joker));
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::Ectoplasm));
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.jokers[0].edition, Edition::Negative, "Ectoplasm should set Negative edition on joker");
}

#[test]
fn test_ectoplasm_reduces_hand_size() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    gs.jokers.push(joker(99, JokerKind::Joker));
    let before = gs.hand_size;
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::Ectoplasm));
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.hand_size, before - 1, "Ectoplasm should reduce hand size by 1");
}

// =========================================================
// Deck-modifying spectrals
// =========================================================

#[test]
fn test_familiar_destroys_one_hand_card_and_adds_three_face_cards() {
    let gs = apply_spectral(five_card_hand(), 5, SpectralCard::Familiar, vec![]);
    // Started with 10 deck cards; destroyed 1, added 3 → 12
    assert_eq!(gs.deck.len(), 12, "Familiar should net +2 cards (destroy 1, add 3)");
    // The 3 added cards must be face cards (J/Q/K)
    let added: Vec<_> = gs.deck.iter().skip(9).collect(); // original 10 − 1 destroyed = 9, then +3
    for c in &added {
        assert!(
            c.rank.is_face(),
            "Familiar should add face cards, got {:?}", c.rank
        );
    }
}

#[test]
fn test_grim_destroys_one_hand_card_and_adds_two_aces() {
    let gs = apply_spectral(five_card_hand(), 5, SpectralCard::Grim, vec![]);
    // Started 10; destroyed 1, added 2 → 11
    assert_eq!(gs.deck.len(), 11, "Grim should net +1 card (destroy 1, add 2)");
    let added: Vec<_> = gs.deck.iter().skip(9).collect();
    for c in &added {
        assert_eq!(c.rank, Rank::Ace, "Grim should add Aces, got {:?}", c.rank);
    }
}

#[test]
fn test_incantation_destroys_one_hand_card_and_adds_four_numbered_cards() {
    let gs = apply_spectral(five_card_hand(), 5, SpectralCard::Incantation, vec![]);
    // Started 10; destroyed 1, added 4 → 13
    assert_eq!(gs.deck.len(), 13, "Incantation should net +3 cards (destroy 1, add 4)");
    let added: Vec<_> = gs.deck.iter().skip(9).collect();
    for c in &added {
        assert!(
            !c.rank.is_face() && c.rank != Rank::Ace,
            "Incantation should add numbered cards (2–10), got {:?}", c.rank
        );
    }
}

#[test]
fn test_cryptid_creates_two_copies_of_target_card() {
    let deck = vec![
        card(0, Rank::King, Suit::Hearts),
        card(1, Rank::Two, Suit::Spades),
    ];
    let gs = apply_spectral(deck, 2, SpectralCard::Cryptid, vec![0]);
    // Original 2 + 2 copies = 4
    assert_eq!(gs.deck.len(), 4, "Cryptid should add 2 copies");
    // The 2 new cards should match the target (King of Hearts)
    let copies: Vec<_> = gs.deck.iter().skip(2).collect();
    for c in &copies {
        assert_eq!(c.rank, Rank::King);
        assert_eq!(c.suit, Suit::Hearts);
    }
}

// =========================================================
// Card-conversion spectrals
// =========================================================

#[test]
fn test_sigil_converts_all_hand_cards_to_same_suit() {
    let deck = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
        card(3, Rank::Four, Suit::Diamonds),
    ];
    let gs = apply_spectral(deck, 4, SpectralCard::Sigil, vec![]);
    let suits: Vec<_> = gs.hand.iter().map(|&i| gs.deck[i].suit).collect();
    let first = suits[0];
    assert!(
        suits.iter().all(|&s| s == first),
        "Sigil should convert all hand cards to the same suit"
    );
}

#[test]
fn test_ouija_converts_all_hand_cards_to_same_rank() {
    let deck = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
        card(3, Rank::Four, Suit::Diamonds),
    ];
    let gs = apply_spectral(deck, 4, SpectralCard::Ouija, vec![]);
    let ranks: Vec<_> = gs.hand.iter().map(|&i| gs.deck[i].rank).collect();
    let first = ranks[0];
    assert!(
        ranks.iter().all(|&r| r == first),
        "Ouija should convert all hand cards to the same rank"
    );
}

#[test]
fn test_ouija_reduces_hand_size() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    let before = gs.hand_size;
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::Ouija));
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.hand_size, before - 1, "Ouija should reduce hand size by 1");
}

// =========================================================
// Money-modifying spectrals
// =========================================================

#[test]
fn test_immolate_gains_20_dollars() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    gs.money = 0;
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::Immolate));
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.money, 20, "Immolate should give +$20");
}

#[test]
fn test_immolate_destroys_up_to_5_hand_cards() {
    let gs = apply_spectral(five_card_hand(), 5, SpectralCard::Immolate, vec![]);
    // 10 - 5 destroyed = 5
    assert_eq!(gs.deck.len(), 5, "Immolate should destroy up to 5 hand cards");
}

#[test]
fn test_wraith_sets_money_to_zero() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    gs.money = 30;
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::Wraith));
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.money, 0, "Wraith should set money to $0");
}

// =========================================================
// Joker-creating spectrals
// =========================================================

#[test]
fn test_wraith_creates_a_rare_joker() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::Wraith));
    let before = gs.jokers.len();
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.jokers.len(), before + 1, "Wraith should add 1 joker");
}

#[test]
fn test_the_soul_creates_a_legendary_joker() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::TheSoul));
    let before = gs.jokers.len();
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.jokers.len(), before + 1, "TheSoul should add 1 joker");
    let legendary_kinds = [
        JokerKind::Canio, JokerKind::Triboulet, JokerKind::Yorick,
        JokerKind::Chicot, JokerKind::Perkeo,
    ];
    assert!(
        legendary_kinds.contains(&gs.jokers.last().unwrap().kind),
        "TheSoul should create a legendary joker"
    );
}

// =========================================================
// Joker-modifying spectrals
// =========================================================

#[test]
fn test_hex_adds_polychrome_to_a_joker() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    gs.jokers.push(joker(10, JokerKind::Joker));
    gs.jokers.push(joker(11, JokerKind::AbstractJoker));
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::Hex));
    gs.use_consumable(0, vec![]).unwrap();
    // Exactly 1 joker survives (the chosen one) with Polychrome
    assert_eq!(gs.jokers.len(), 1, "Hex should destroy all but one joker");
    assert_eq!(gs.jokers[0].edition, Edition::Polychrome, "Hex chosen joker should be Polychrome");
}

#[test]
fn test_hex_spares_eternal_jokers() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    gs.jokers.push(joker(10, JokerKind::Joker));
    let mut eternal_j = joker(11, JokerKind::AbstractJoker);
    eternal_j.eternal = true;
    gs.jokers.push(eternal_j);
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::Hex));
    gs.use_consumable(0, vec![]).unwrap();
    // Eternal joker must survive
    assert!(
        gs.jokers.iter().any(|j| j.eternal),
        "Hex should not destroy eternal jokers"
    );
}

#[test]
fn test_ankh_copies_one_joker_and_destroys_others() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    gs.jokers.push(joker(10, JokerKind::Joker));
    gs.jokers.push(joker(11, JokerKind::Scholar));
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::Ankh));
    gs.use_consumable(0, vec![]).unwrap();
    // Ankh: copy chosen, destroy rest → 1 survivor
    assert_eq!(gs.jokers.len(), 1, "Ankh should leave exactly 1 joker (the copy)");
}

#[test]
fn test_ankh_spares_eternal_jokers() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    gs.jokers.push(joker(10, JokerKind::Joker));
    let mut eternal_j = joker(11, JokerKind::AbstractJoker);
    eternal_j.eternal = true;
    gs.jokers.push(eternal_j);
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::Ankh));
    gs.use_consumable(0, vec![]).unwrap();
    assert!(
        gs.jokers.iter().any(|j| j.eternal),
        "Ankh should not destroy eternal jokers"
    );
}

// =========================================================
// Level-scaling spectral
// =========================================================

#[test]
fn test_black_hole_upgrades_every_hand_level_by_1() {
    let gs = apply_spectral(five_card_hand(), 5, SpectralCard::BlackHole, vec![]);
    let all_hand_types = [
        HandType::HighCard, HandType::Pair, HandType::TwoPair,
        HandType::ThreeOfAKind, HandType::Straight, HandType::Flush,
        HandType::FullHouse, HandType::FourOfAKind, HandType::StraightFlush,
        HandType::FiveOfAKind, HandType::FlushHouse, HandType::FlushFive,
    ];
    for ht in all_hand_types {
        assert_eq!(
            gs.hand_levels[&ht].level, 2,
            "BlackHole should upgrade {:?} to level 2", ht
        );
    }
}

// =========================================================
// Spectral consumed after use
// =========================================================

#[test]
fn test_spectral_card_is_consumed_after_use() {
    let mut gs = make_game();
    setup_round(&mut gs, five_card_hand(), 5);
    gs.consumables.push(crate::card::ConsumableCard::Spectral(SpectralCard::BlackHole));
    assert_eq!(gs.consumables.len(), 1);
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.consumables.len(), 0, "Spectral should be removed after use");
}

// =========================================================
// Editionless targeting and Ectoplasm's escalating cost
// =========================================================

fn game_with_jokers(jokers: Vec<JokerInstance>) -> GameState {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.jokers = jokers;
    gs
}

fn use_spectral(gs: &mut GameState, s: SpectralCard, targets: Vec<usize>) {
    gs.consumables.push(crate::card::ConsumableCard::Spectral(s));
    let idx = gs.consumables.len() - 1;
    gs.use_consumable(idx, targets).unwrap();
}

/// Hex/Ectoplasm/Wheel of Fortune all draw from jokers that have no edition (card.lua:4218).
#[test]
fn test_ectoplasm_only_targets_editionless_jokers() {
    let mut poly = joker(0, JokerKind::Joker);
    poly.edition = Edition::Polychrome;
    let mut gs = game_with_jokers(vec![poly, joker(1, JokerKind::Fibonacci)]);

    use_spectral(&mut gs, SpectralCard::Ectoplasm, vec![]);
    assert_eq!(gs.jokers[0].edition, Edition::Polychrome, "must not overwrite");
    assert_eq!(gs.jokers[1].edition, Edition::Negative);
}

#[test]
fn test_ectoplasm_hand_size_cost_escalates() {
    let mut gs = game_with_jokers(vec![
        joker(0, JokerKind::Joker),
        joker(1, JokerKind::Fibonacci),
        joker(2, JokerKind::Scholar),
    ]);
    let base = gs.hand_size;

    use_spectral(&mut gs, SpectralCard::Ectoplasm, vec![]);
    assert_eq!(gs.hand_size, base - 1);
    use_spectral(&mut gs, SpectralCard::Ectoplasm, vec![]);
    assert_eq!(gs.hand_size, base - 3, "second use costs 2");
    use_spectral(&mut gs, SpectralCard::Ectoplasm, vec![]);
    assert_eq!(gs.hand_size, base - 6, "third use costs 3");
}

#[test]
fn test_hex_only_targets_editionless_jokers() {
    let mut foil = joker(0, JokerKind::Joker);
    foil.edition = Edition::Foil;
    let mut gs = game_with_jokers(vec![foil, joker(1, JokerKind::Fibonacci)]);

    use_spectral(&mut gs, SpectralCard::Hex, vec![]);
    assert_eq!(gs.jokers.len(), 1);
    assert_eq!(gs.jokers[0].kind, JokerKind::Fibonacci);
    assert_eq!(gs.jokers[0].edition, Edition::Polychrome);
}

#[test]
fn test_wheel_of_fortune_never_overwrites_an_existing_edition() {
    for seed in 0..40 {
        let mut poly = joker(0, JokerKind::Joker);
        poly.edition = Edition::Polychrome;
        let mut gs = make_game();
        gs.rng = crate::rng::Rng::new(&format!("WHEEL{}", seed));
        setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
        gs.jokers = vec![poly];

        gs.consumables
            .push(crate::card::ConsumableCard::Tarot(TarotCard::TheWheelOfFortune));
        gs.use_consumable(0, vec![]).unwrap();
        assert_eq!(gs.jokers[0].edition, Edition::Polychrome);
    }
}

// =========================================================
// Self-consuming jokers are destroyed, not just switched off
// =========================================================

#[test]
fn test_ice_cream_is_removed_when_it_melts() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    let mut ic = joker(0, JokerKind::IceCream);
    ic.set_counter_i64("chips", 5);
    gs.jokers = vec![ic];
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert!(gs.jokers.is_empty(), "Ice Cream should free its joker slot");
}

#[test]
fn test_popcorn_is_removed_when_it_runs_out() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    let mut pc = joker(0, JokerKind::Popcorn);
    pc.set_counter_i64("mult", 4);
    gs.jokers = vec![pc];
    gs.score_goal = 1.0; // win the round so end-of-round processing runs

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert!(gs.jokers.is_empty(), "Popcorn should free its joker slot");
}

#[test]
fn test_seltzer_is_removed_when_it_runs_out() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    let mut sz = joker(0, JokerKind::Seltzer);
    sz.set_counter_i64("hands", 1);
    gs.jokers = vec![sz];
    gs.score_goal = f64::MAX;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    assert!(gs.jokers.is_empty(), "Seltzer should free its joker slot");
}
