/// Tests for tarot card application via GameState.

use super::*;

// Helper: set up a round with cards in hand, then use a tarot on target hand indices.
fn apply_tarot_in_round(
    deck_cards: Vec<CardInstance>,
    hand_size: usize,
    tarot: TarotCard,
    targets: Vec<usize>,
) -> GameState {
    let mut gs = make_game();
    setup_round(&mut gs, deck_cards, hand_size);
    gs.consumables.push(crate::card::ConsumableCard::Tarot(tarot));
    gs.use_consumable(0, targets).unwrap();
    gs
}

// =========================================================
// Enhancement tarots
// =========================================================

#[test]
fn test_the_magician_adds_lucky_enhancement() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades), card(1, Rank::Two, Suit::Hearts)];
    let gs = apply_tarot_in_round(deck, 2, TarotCard::TheMagician, vec![0]);
    let card_idx = gs.hand[0];
    assert_eq!(gs.deck[card_idx].enhancement, Enhancement::Lucky);
}

#[test]
fn test_the_empress_adds_mult_enhancement() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades)];
    let gs = apply_tarot_in_round(deck, 1, TarotCard::TheEmpress, vec![0]);
    let card_idx = gs.hand[0];
    assert_eq!(gs.deck[card_idx].enhancement, Enhancement::Mult);
}

#[test]
fn test_the_hierophant_adds_bonus_enhancement() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades)];
    let gs = apply_tarot_in_round(deck, 1, TarotCard::TheHierophant, vec![0]);
    let card_idx = gs.hand[0];
    assert_eq!(gs.deck[card_idx].enhancement, Enhancement::Bonus);
}

#[test]
fn test_the_lovers_adds_wild_enhancement() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades)];
    let gs = apply_tarot_in_round(deck, 1, TarotCard::TheLovers, vec![0]);
    let card_idx = gs.hand[0];
    assert_eq!(gs.deck[card_idx].enhancement, Enhancement::Wild);
}

#[test]
fn test_the_chariot_adds_steel_enhancement() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades)];
    let gs = apply_tarot_in_round(deck, 1, TarotCard::TheChariot, vec![0]);
    let card_idx = gs.hand[0];
    assert_eq!(gs.deck[card_idx].enhancement, Enhancement::Steel);
}

#[test]
fn test_justice_adds_glass_enhancement() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades)];
    let gs = apply_tarot_in_round(deck, 1, TarotCard::Justice, vec![0]);
    let card_idx = gs.hand[0];
    assert_eq!(gs.deck[card_idx].enhancement, Enhancement::Glass);
}

#[test]
fn test_the_devil_adds_gold_enhancement() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades)];
    let gs = apply_tarot_in_round(deck, 1, TarotCard::TheDevil, vec![0]);
    let card_idx = gs.hand[0];
    assert_eq!(gs.deck[card_idx].enhancement, Enhancement::Gold);
}

#[test]
fn test_the_tower_adds_stone_enhancement() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades)];
    let gs = apply_tarot_in_round(deck, 1, TarotCard::TheTower, vec![0]);
    let card_idx = gs.hand[0];
    assert_eq!(gs.deck[card_idx].enhancement, Enhancement::Stone);
}

#[test]
fn test_the_magician_can_target_up_to_two_cards() {
    let deck = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    let gs = apply_tarot_in_round(deck, 3, TarotCard::TheMagician, vec![0, 1]);
    assert_eq!(gs.deck[gs.hand[0]].enhancement, Enhancement::Lucky);
    assert_eq!(gs.deck[gs.hand[1]].enhancement, Enhancement::Lucky);
    // Third card unchanged
    assert_eq!(gs.deck[gs.hand[2]].enhancement, Enhancement::None);
}

// =========================================================
// Suit tarots
// =========================================================

#[test]
fn test_the_star_converts_cards_to_diamonds() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades), card(1, Rank::Two, Suit::Hearts)];
    let gs = apply_tarot_in_round(deck, 2, TarotCard::TheStar, vec![0, 1]);
    assert_eq!(gs.deck[gs.hand[0]].suit, Suit::Diamonds);
    assert_eq!(gs.deck[gs.hand[1]].suit, Suit::Diamonds);
}

#[test]
fn test_the_moon_converts_cards_to_clubs() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades)];
    let gs = apply_tarot_in_round(deck, 1, TarotCard::TheMoon, vec![0]);
    assert_eq!(gs.deck[gs.hand[0]].suit, Suit::Clubs);
}

#[test]
fn test_the_sun_converts_cards_to_hearts() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades)];
    let gs = apply_tarot_in_round(deck, 1, TarotCard::TheSun, vec![0]);
    assert_eq!(gs.deck[gs.hand[0]].suit, Suit::Hearts);
}

#[test]
fn test_the_world_converts_cards_to_spades() {
    let deck = vec![card(0, Rank::Ace, Suit::Hearts)];
    let gs = apply_tarot_in_round(deck, 1, TarotCard::TheWorld, vec![0]);
    assert_eq!(gs.deck[gs.hand[0]].suit, Suit::Spades);
}

// =========================================================
// Manipulation tarots
// =========================================================

#[test]
fn test_the_hanged_man_destroys_selected_cards() {
    let deck = vec![card(0, Rank::Ace, Suit::Spades), card(1, Rank::Two, Suit::Hearts)];
    let gs = apply_tarot_in_round(deck, 2, TarotCard::TheHangedMan, vec![0]);
    // One card was destroyed → deck should have 1 card
    assert_eq!(gs.deck.len(), 1);
}

#[test]
fn test_the_hanged_man_can_destroy_two_cards() {
    let deck = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    let gs = apply_tarot_in_round(deck, 3, TarotCard::TheHangedMan, vec![0, 1]);
    assert_eq!(gs.deck.len(), 1);
}

#[test]
fn test_strength_increases_rank_by_one() {
    let deck = vec![card(0, Rank::Two, Suit::Spades)];
    let gs = apply_tarot_in_round(deck, 1, TarotCard::Strength, vec![0]);
    assert_eq!(gs.deck[gs.hand[0]].rank, Rank::Three);
}

#[test]
fn test_death_copies_right_card_to_left() {
    let deck = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
    ];
    let gs = apply_tarot_in_round(deck, 2, TarotCard::Death, vec![0, 1]);
    // Left (index 0) should now match right (index 1): Ace of Hearts
    let left_card = &gs.deck[gs.hand[0]];
    assert_eq!(left_card.rank, Rank::Ace);
    assert_eq!(left_card.suit, Suit::Hearts);
}

// =========================================================
// Money tarots
// =========================================================

#[test]
fn test_the_hermit_doubles_money() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.money = 10;
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheHermit));
    gs.use_consumable(0, vec![]).unwrap();
    // Should have doubled: 10 + 10 = 20
    assert_eq!(gs.money, 20);
}

#[test]
fn test_the_hermit_caps_gain_at_20() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.money = 100;
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheHermit));
    gs.use_consumable(0, vec![]).unwrap();
    // Cap: gain at most $20 → 100 + 20 = 120
    assert_eq!(gs.money, 120);
}

#[test]
fn test_temperance_gives_money_from_joker_sell_values() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.money = 0;
    // Add a Joker (base cost 2 → sell value 1) and an AbstractJoker (cost 4 → sell value 2)
    gs.jokers.push(joker(10, JokerKind::Joker)); // sell value 1
    gs.jokers.push(joker(11, JokerKind::AbstractJoker)); // sell value ~3
    let expected_total = gs.jokers.iter().map(|j| j.sell_value() as i32).sum::<i32>();
    let capped = expected_total.min(50);
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::Temperance));
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.money, capped);
}

// =========================================================
// Creation tarots
// =========================================================

#[test]
fn test_the_high_priestess_creates_planet_cards() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.consumables.clear();
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheHighPriestess));
    gs.use_consumable(0, vec![]).unwrap();
    // Should have created up to 2 Planet cards
    let planet_count = gs.consumables.iter().filter(|c| {
        matches!(c, crate::card::ConsumableCard::Planet(_))
    }).count();
    assert!(planet_count <= 2, "TheHighPriestess should create at most 2 Planets");
    assert!(planet_count >= 1, "TheHighPriestess should create at least 1 Planet");
}

#[test]
fn test_the_emperor_creates_tarot_cards() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.consumables.clear();
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheEmperor));
    gs.use_consumable(0, vec![]).unwrap();
    // Should have created up to 2 Tarot cards
    let tarot_count = gs.consumables.iter().filter(|c| {
        matches!(c, crate::card::ConsumableCard::Tarot(_))
    }).count();
    assert!(tarot_count <= 2, "TheEmperor should create at most 2 Tarots");
    assert!(tarot_count >= 1, "TheEmperor should create at least 1 Tarot");
}

#[test]
fn test_judgement_creates_a_joker() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::Judgement));
    let before = gs.jokers.len();
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.jokers.len(), before + 1, "Judgement should create 1 joker");
}

#[test]
fn test_the_wheel_of_fortune_may_add_edition_to_joker() {
    // TheWheelOfFortune: 25% chance to add a random edition to a random joker.
    // Run enough seeds until it fires at least once.
    let mut fired = false;
    for seed_n in 0..40u32 {
        let mut gs = GameState::new(
            DeckType::Blue, Stake::White,
            Some(format!("TWOF{seed_n}")),
        );
        setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
        gs.jokers.push(joker(99, JokerKind::Joker));
        gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheWheelOfFortune));
        gs.use_consumable(0, vec![]).unwrap();
        if gs.jokers[0].edition != Edition::None {
            let ed = gs.jokers[0].edition;
            assert!(
                matches!(ed, Edition::Foil | Edition::Holographic | Edition::Polychrome),
                "TheWheelOfFortune should add Foil/Holo/Poly, got {:?}", ed
            );
            fired = true;
            break;
        }
    }
    assert!(fired, "TheWheelOfFortune should have fired at least once across 40 seeds");
}

#[test]
fn test_the_fool_recreates_last_tarot() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);

    // First use TheHermit so it's the "last used"
    gs.money = 5;
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheHermit));
    gs.use_consumable(0, vec![]).unwrap();
    // Now use TheFool
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheFool));
    gs.use_consumable(0, vec![]).unwrap();
    // TheFool should have recreated TheHermit in consumables
    let has_hermit = gs.consumables.iter().any(|c| {
        matches!(c, crate::card::ConsumableCard::Tarot(TarotCard::TheHermit))
    });
    assert!(has_hermit, "TheFool should have recreated TheHermit");
}
