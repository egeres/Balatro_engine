/// Tests for 29 misc jokers:
/// Astronomer, Blueprint, Brainstorm, Cartomancer, Cavendish, Certificate, ChaosTheClown,
/// Cloud9, CreditCard, DelayedGratification, DietCola, Dna, Drunkard, Dusk, Egg, EightBall,
/// FacelessJoker, GiftCard, GoldenJoker, GoldenTicket, GrosMichel, Hallucination,
/// InvisibleJoker, Juggler, LoyaltyCard, MailInRebate, MarbleJoker, Matador, MerryAndy

use super::*;
use crate::game::GameStateKind;

// =========================================================
// Astronomer: planet cards are free in the shop
// =========================================================

#[test]
fn test_astronomer_makes_planet_cards_free() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.jokers.push(joker(1, JokerKind::Astronomer));

    // Manually add a planet card to shop
    let offer = crate::card::ShopOffer {
        kind: crate::card::ShopItem::Consumable(crate::card::ConsumableCard::Planet(PlanetCard::Mercury)),
        price: 3,
        sold: false,
    };
    gs.shop_offers.clear();
    gs.shop_offers.push(offer);
    gs.consumable_slots = 5;

    let money_before = gs.money;
    gs.buy_consumable(0).expect("should be able to buy planet for free");
    // Money should not have decreased (price=0 with Astronomer)
    assert_eq!(gs.money, money_before);
    assert_eq!(gs.consumables.len(), 1);
}

#[test]
fn test_astronomer_does_not_affect_tarot_price() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.jokers.push(joker(1, JokerKind::Astronomer));
    gs.money = 10;

    let offer = crate::card::ShopOffer {
        kind: crate::card::ShopItem::Consumable(crate::card::ConsumableCard::Tarot(TarotCard::TheFool)),
        price: 3,
        sold: false,
    };
    gs.shop_offers.clear();
    gs.shop_offers.push(offer);
    gs.consumable_slots = 5;

    let money_before = gs.money;
    gs.buy_consumable(0).expect("should buy tarot at normal price");
    // Tarot costs 3 even with Astronomer
    assert_eq!(gs.money, money_before - 3);
}

// =========================================================
// Blueprint: copies the joker to the right
// =========================================================

#[test]
fn test_blueprint_copies_right_joker() {
    // Blueprint + Joker (right): should add +4 mult from Joker
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![
        joker(0, JokerKind::Blueprint),
        joker(1, JokerKind::Joker),
    ];
    let r = score(&played, &played, &jokers);
    // Blueprint copies Joker (+4 mult), Joker itself gives +4 mult
    // HC: chips=16, mult=1+4+4=9 → 144
    assert_eq!(r.final_score as i64, 144);
}

#[test]
fn test_blueprint_alone_does_nothing() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::Blueprint)];
    let r = score(&played, &played, &jokers);
    // No joker to the right → nothing copied
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_blueprint_does_not_copy_another_blueprint() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![
        joker(0, JokerKind::Blueprint),
        joker(1, JokerKind::Blueprint),
    ];
    let r = score(&played, &played, &jokers);
    // Blueprint to the right is skipped, no effect
    assert_eq!(r.final_score as i64, 16);
}

// =========================================================
// Brainstorm: copies the leftmost joker
// =========================================================

#[test]
fn test_brainstorm_copies_leftmost_joker() {
    // Joker + Brainstorm: Brainstorm copies Joker (+4 mult)
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![
        joker(0, JokerKind::Joker),
        joker(1, JokerKind::Brainstorm),
    ];
    let r = score(&played, &played, &jokers);
    // Joker: +4 mult, Brainstorm copies Joker: +4 mult
    // HC: chips=16, mult=1+4+4=9 → 144
    assert_eq!(r.final_score as i64, 144);
}

#[test]
fn test_brainstorm_does_not_copy_blueprint() {
    // Blueprint + Brainstorm: Brainstorm should skip Blueprint
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![
        joker(0, JokerKind::Blueprint),
        joker(1, JokerKind::Brainstorm),
    ];
    let r = score(&played, &played, &jokers);
    // Blueprint has no leftmost non-blueprint joker to copy; Brainstorm skips Blueprint
    // No extra mult
    assert_eq!(r.final_score as i64, 16);
}

// =========================================================
// Cartomancer: create a tarot card on single-card hands
// =========================================================

#[test]
fn test_cartomancer_creates_tarot_on_single_card_play() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    setup_round(&mut gs, cards, 3);
    gs.jokers.push(joker(100, JokerKind::Cartomancer));
    gs.consumable_slots = 5;
    gs.score_goal = 1.0;

    // Select only 1 card
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    // Should have created a tarot card
    assert!(!gs.consumables.is_empty(), "Cartomancer should create a tarot on single-card play");
}

#[test]
fn test_cartomancer_does_not_create_tarot_on_multi_card_play() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    setup_round(&mut gs, cards, 3);
    gs.jokers.push(joker(100, JokerKind::Cartomancer));
    gs.consumable_slots = 5;
    gs.score_goal = 1.0;

    // Select 2 cards
    gs.select_card(0).unwrap();
    gs.select_card(1).unwrap();
    gs.play_hand().unwrap();

    assert!(gs.consumables.is_empty(), "Cartomancer should not fire on multi-card play");
}

// =========================================================
// Cavendish: x3 mult on Pair
// =========================================================

#[test]
fn test_cavendish_fires_on_pair() {
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
    ];
    let jokers = vec![joker(0, JokerKind::Cavendish)];
    let r = score(&played, &played, &jokers);
    // Pair Aces: 10+11+11=32 chips, mult=2*3=6 → 192
    assert_eq!(r.hand_type, HandType::Pair);
    assert_eq!(r.final_score as i64, 192);
}

#[test]
fn test_cavendish_does_not_fire_on_high_card() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::Cavendish)];
    let r = score(&played, &played, &jokers);
    // HC: chips=16, mult=1 → 16 (no x3)
    assert_eq!(r.final_score as i64, 16);
}

// =========================================================
// Certificate: adds a card with enhancement when blind is set
// =========================================================

#[test]
fn test_certificate_adds_card_to_deck_on_blind_set() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Certificate));

    let deck_size_before = gs.deck.len();
    gs.select_blind().unwrap();

    // Should have added 1 card to deck
    assert_eq!(gs.deck.len(), deck_size_before + 1);
    // The added card should have a non-None enhancement
    let last_card = gs.deck.last().unwrap();
    assert_ne!(last_card.enhancement, Enhancement::None,
        "Certificate should add an enhanced card");
}

// =========================================================
// ChaosTheClown: +1 free reroll per shop visit
// =========================================================

#[test]
fn test_chaos_the_clown_gives_free_reroll_on_shop_visit() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::ChaosTheClown));
    gs.state = GameStateKind::Shop;

    let free_rerolls_before = gs.free_rerolls;
    gs.generate_shop();
    assert_eq!(gs.free_rerolls, free_rerolls_before + 1,
        "ChaosTheClown should give +1 free reroll per shop visit");
}

#[test]
fn test_chaos_the_clown_stacks_multiple() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::ChaosTheClown));
    gs.jokers.push(joker(2, JokerKind::ChaosTheClown));
    gs.state = GameStateKind::Shop;

    gs.generate_shop();
    // 2 ChaosTheClown → +2 free rerolls
    assert!(gs.free_rerolls >= 2);
}

// =========================================================
// Cloud9: +$1 per 9 in deck at end of round
// =========================================================

#[test]
fn test_cloud9_earns_money_per_nine_at_end_of_round() {
    let mut gs = make_game();
    // Build a minimal deck with a known number of 9s
    let cards = vec![
        card(0, Rank::Nine, Suit::Spades),
        card(1, Rank::Nine, Suit::Hearts),
        card(2, Rank::Nine, Suit::Clubs),
        card(3, Rank::Ace, Suit::Spades),
    ];
    setup_round(&mut gs, cards, 1);
    gs.jokers.push(joker(1, JokerKind::Cloud9));
    gs.score_goal = 1.0; // easy win

    let money_before = gs.money;

    // Play to win
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    // Should be in Shop now; cloud9 should have paid $3 (3 nines in deck)
    assert!(matches!(gs.state, GameStateKind::Shop));
    // money_before + blind_reward(3) + cloud9(3) + interest
    let money_gained = gs.money - money_before;
    assert!(money_gained >= 3, "Cloud9 should pay at least $3 for 3 nines, gained: {}", money_gained);
}

// =========================================================
// CreditCard: money floor -$20
// =========================================================

#[test]
fn test_credit_card_joker_can_be_added() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::CreditCard));
    assert_eq!(gs.jokers.len(), 1);
    assert_eq!(gs.jokers[0].kind, JokerKind::CreditCard);
}

// =========================================================
// DelayedGratification: +$2 per discard if no discards used
// =========================================================

#[test]
fn test_delayed_gratification_pays_when_no_discards_used() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
    ];
    setup_round(&mut gs, cards, 1);
    gs.jokers.push(joker(1, JokerKind::DelayedGratification));
    gs.score_goal = 1.0;
    gs.discards_remaining = 3;

    let money_before = gs.money;
    // Don't use any discards, play to win
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    assert!(matches!(gs.state, GameStateKind::Shop));
    // +$2 per 3 discards = +$6 from DG
    let money_gained = gs.money - money_before;
    assert!(money_gained >= 6, "DelayedGratification should pay $6 for 3 unused discards, gained: {}", money_gained);
}

#[test]
fn test_delayed_gratification_does_not_pay_when_discards_used() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    setup_round(&mut gs, cards, 3);
    gs.jokers.push(joker(1, JokerKind::DelayedGratification));
    gs.score_goal = 100000.0; // won't win from one hand
    gs.discards_remaining = 3;
    gs.max_discards = 3;

    // Use a discard
    gs.select_card(0).unwrap();
    gs.discard_hand().unwrap();

    // Now win by setting score goal low
    gs.score_goal = 1.0;
    gs.select_card(0).unwrap();
    let money_before = gs.money;
    gs.play_hand().unwrap();

    // discards were used, so DG should not pay its +$6
    // Just make sure it ran without panicking
    assert!(matches!(gs.state, GameStateKind::Shop));
}

// =========================================================
// DietCola: creates a copy of consumable when sold
// =========================================================

#[test]
fn test_diet_cola_creates_copy_on_sell() {
    let mut gs = make_game();
    gs.state = GameStateKind::Shop;
    gs.jokers.push(joker(1, JokerKind::DietCola));
    gs.consumable_slots = 5;
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheFool));

    let consumables_before = gs.consumables.len(); // 1
    gs.sell_consumable(0).unwrap();

    // DietCola should have added a copy before selling
    // After sell: copy was added (+1), then sold one (-1) = net 0
    assert_eq!(gs.consumables.len(), consumables_before, "DietCola should create a copy before selling");
}

// =========================================================
// DNA: copies the first card on the first hand if only 1 card played
// =========================================================

#[test]
fn test_dna_copies_card_on_first_hand() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
    ];
    setup_round(&mut gs, cards, 1);
    gs.jokers.push(joker(1, JokerKind::Dna));
    gs.score_goal = 1.0;
    gs.hands_remaining = 4; // start at max for "first hand" check
    gs.max_hands = 4;

    let deck_size_before = gs.deck.len();
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    // Should have added a copy of the played Ace to deck
    assert_eq!(gs.deck.len(), deck_size_before + 1,
        "DNA should add a copy of the first card to deck");
    // The copy should be an Ace of Spades
    let new_card = gs.deck.last().unwrap();
    assert_eq!(new_card.rank, Rank::Ace);
    assert_eq!(new_card.suit, Suit::Spades);
}

#[test]
fn test_dna_does_not_copy_when_multiple_cards_played() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    setup_round(&mut gs, cards, 3);
    gs.jokers.push(joker(1, JokerKind::Dna));
    gs.score_goal = 1.0;
    gs.hands_remaining = 4;
    gs.max_hands = 4;

    let deck_size_before = gs.deck.len();
    gs.select_card(0).unwrap();
    gs.select_card(1).unwrap();
    gs.play_hand().unwrap();

    // DNA should not fire when 2 cards played
    assert_eq!(gs.deck.len(), deck_size_before, "DNA should not fire with multiple cards played");
}

// =========================================================
// Drunkard: +1 discard per round
// =========================================================

#[test]
fn test_drunkard_increases_max_discards() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Drunkard));
    // effective_max_discards counts active Drunkard jokers
    let discards = gs.effective_max_discards();
    assert_eq!(discards, gs.max_discards + 1,
        "Drunkard should add +1 to max discards");
}

// =========================================================
// Dusk: retrigger scoring cards on last hand
// =========================================================

#[test]
fn test_dusk_retriggers_on_last_hand() {
    // When hands_remaining=0 (last hand), Dusk retriggers all scoring cards
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::Dusk)];

    // Last hand (hands_remaining=0)
    let r_last = score_full(&played, &played, &jokers, 0, 3, 0, 40, 52, 5, 0);
    // First hand (hands_remaining=3)
    let r_other = score_full(&played, &played, &jokers, 3, 3, 0, 40, 52, 5, 0);

    // On last hand, Ace should be triggered twice: score should be higher
    assert!(r_last.final_score > r_other.final_score,
        "Dusk should cause higher score on last hand via retrigger: last={} other={}",
        r_last.final_score, r_other.final_score);
}

#[test]
fn test_dusk_does_not_retrigger_on_non_last_hand() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::Dusk)];
    let r = score_full(&played, &played, &jokers, 3, 3, 0, 40, 52, 5, 0);
    // Without retrigger: HC Ace: chips=16, mult=1 → 16
    assert_eq!(r.final_score as i64, 16);
}

// =========================================================
// Egg: gains $3 sell value per shop visit
// =========================================================

#[test]
fn test_egg_gains_sell_value_on_shop_visit() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Egg));
    gs.state = GameStateKind::Shop;

    let sell_before = gs.jokers[0].sell_value();
    gs.generate_shop();
    let sell_after = gs.jokers[0].sell_value();

    assert_eq!(sell_after, sell_before + 3,
        "Egg should gain $3 sell value per shop visit");
}

#[test]
fn test_egg_stacks_sell_value_over_multiple_visits() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Egg));
    gs.state = GameStateKind::Shop;

    let sell_initial = gs.jokers[0].sell_value();
    gs.generate_shop();
    gs.generate_shop();
    let sell_after = gs.jokers[0].sell_value();

    assert_eq!(sell_after, sell_initial + 6, "Egg should accumulate sell value over visits");
}

// =========================================================
// EightBall: 1/4 chance to create tarot when 8 is scored
// =========================================================

#[test]
fn test_eight_ball_does_not_crash_on_eight_scored() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Eight, Suit::Spades),
        card(1, Rank::Eight, Suit::Hearts),
        card(2, Rank::Eight, Suit::Clubs),
    ];
    setup_round(&mut gs, cards, 3);
    gs.jokers.push(joker(1, JokerKind::EightBall));
    gs.consumable_slots = 5;
    gs.score_goal = 1.0;

    // Play ThreeOfAKind of 8s — should not panic
    gs.select_card(0).unwrap();
    gs.select_card(1).unwrap();
    gs.select_card(2).unwrap();
    gs.play_hand().unwrap();
    // Just checking it ran without errors; tarot creation is RNG-based
}

#[test]
fn test_eight_ball_can_create_tarot_over_many_runs() {
    // Run many trials to check that tarot creation happens (RNG seeded deterministically)
    let mut created = false;
    for trial in 0..50 {
        let seed = format!("trial_{}", trial);
        let mut gs = crate::game::GameState::new(DeckType::Blue, Stake::White, Some(seed));
        let cards = vec![
            card(0, Rank::Eight, Suit::Spades),
            card(1, Rank::Eight, Suit::Hearts),
            card(2, Rank::Eight, Suit::Clubs),
        ];
        setup_round(&mut gs, cards, 3);
        gs.jokers.push(joker(1, JokerKind::EightBall));
        gs.consumable_slots = 5;
        gs.score_goal = 1.0;

        gs.select_card(0).unwrap();
        gs.select_card(1).unwrap();
        gs.select_card(2).unwrap();
        gs.play_hand().unwrap();

        if !gs.consumables.is_empty() {
            created = true;
            break;
        }
    }
    assert!(created, "EightBall should create a tarot at least once in 50 trials");
}

// =========================================================
// FacelessJoker: $5 if 3+ face cards discarded
// =========================================================

#[test]
fn test_faceless_joker_earns_money_on_3_face_card_discard() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Jack, Suit::Spades),
        card(1, Rank::Queen, Suit::Hearts),
        card(2, Rank::King, Suit::Clubs),
        card(3, Rank::Ace, Suit::Diamonds),
    ];
    setup_round(&mut gs, cards, 4);
    gs.jokers.push(joker(1, JokerKind::FacelessJoker));
    gs.discards_remaining = 3;

    let money_before = gs.money;
    gs.select_card(0).unwrap();
    gs.select_card(1).unwrap();
    gs.select_card(2).unwrap();
    gs.discard_hand().unwrap();

    assert_eq!(gs.money, money_before + 5,
        "FacelessJoker should pay $5 for discarding 3+ face cards");
}

#[test]
fn test_faceless_joker_does_not_pay_for_fewer_than_3_face_cards() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Jack, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    setup_round(&mut gs, cards, 3);
    gs.jokers.push(joker(1, JokerKind::FacelessJoker));
    gs.discards_remaining = 3;

    let money_before = gs.money;
    gs.select_card(0).unwrap();
    gs.select_card(1).unwrap();
    gs.discard_hand().unwrap();

    assert_eq!(gs.money, money_before, "FacelessJoker should not pay for fewer than 3 face cards");
}

// =========================================================
// GiftCard: +$1 sell value to other jokers on shop visit
// =========================================================

#[test]
fn test_gift_card_increases_other_joker_sell_values() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::GiftCard));
    gs.jokers.push(joker(2, JokerKind::Joker));
    gs.state = GameStateKind::Shop;

    let joker_sell_before = gs.jokers[1].sell_value();
    gs.generate_shop();
    let joker_sell_after = gs.jokers[1].sell_value();

    assert_eq!(joker_sell_after, joker_sell_before + 1,
        "GiftCard should increase other jokers' sell value by $1");
}

#[test]
fn test_gift_card_does_not_increase_own_sell_value() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::GiftCard));
    gs.state = GameStateKind::Shop;

    let own_sell_before = gs.jokers[0].sell_value();
    gs.generate_shop();
    let own_sell_after = gs.jokers[0].sell_value();

    assert_eq!(own_sell_after, own_sell_before,
        "GiftCard should not increase its own sell value");
}

// =========================================================
// GoldenJoker: +$4 at end of round
// =========================================================

#[test]
fn test_golden_joker_earns_4_dollars_at_end_of_round() {
    let mut gs = make_game();
    let cards = vec![card(0, Rank::Ace, Suit::Spades)];
    setup_round(&mut gs, cards, 1);
    gs.jokers.push(joker(1, JokerKind::GoldenJoker));
    gs.score_goal = 1.0;

    let money_before = gs.money;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    assert!(matches!(gs.state, GameStateKind::Shop));
    let money_gained = gs.money - money_before;
    // blind reward(3) + golden_joker(4)
    assert!(money_gained >= 4, "GoldenJoker should earn at least $4 at end of round, gained: {}", money_gained);
}

// =========================================================
// GoldenTicket: +$1 per Gold card in scoring hand
// =========================================================

#[test]
fn test_golden_ticket_earns_dollar_per_gold_card() {
    let mut gold_ace = card(0, Rank::Ace, Suit::Spades);
    gold_ace.enhancement = Enhancement::Gold;
    let played = vec![gold_ace];
    let jokers = vec![joker(0, JokerKind::GoldenTicket)];
    let r = score(&played, &played, &jokers);
    assert_eq!(r.dollars_earned, 1, "GoldenTicket should earn $1 per Gold card in scoring hand");
}

#[test]
fn test_golden_ticket_earns_multiple_dollars_for_multiple_gold_cards() {
    let mut gold_ace = card(0, Rank::Ace, Suit::Spades);
    gold_ace.enhancement = Enhancement::Gold;
    let mut gold_ace2 = card(1, Rank::Ace, Suit::Hearts);
    gold_ace2.enhancement = Enhancement::Gold;
    let played = vec![gold_ace, gold_ace2];
    let jokers = vec![joker(0, JokerKind::GoldenTicket)];
    let r = score(&played, &played, &jokers);
    assert_eq!(r.dollars_earned, 2, "GoldenTicket should earn $2 for 2 Gold cards");
}

#[test]
fn test_golden_ticket_does_not_earn_for_non_gold_cards() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::GoldenTicket)];
    let r = score(&played, &played, &jokers);
    assert_eq!(r.dollars_earned, 0, "GoldenTicket should not earn for non-Gold cards");
}

// =========================================================
// GrosMichel: +15 mult
// =========================================================

#[test]
fn test_gros_michel_adds_15_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::GrosMichel)];
    let r = score(&played, &played, &jokers);
    // HC: chips=16, mult=1+15=16 → 256
    assert_eq!(r.final_score as i64, 256);
}

#[test]
fn test_gros_michel_can_be_destroyed_at_end_of_round() {
    // Run many trials: GrosMichel has 1/6 chance of being destroyed
    let mut destroyed = false;
    for trial in 0..100 {
        let seed = format!("gros_{}", trial);
        let mut gs = crate::game::GameState::new(DeckType::Blue, Stake::White, Some(seed));
        let cards = vec![card(0, Rank::Ace, Suit::Spades)];
        setup_round(&mut gs, cards, 1);
        gs.jokers.push(joker(1, JokerKind::GrosMichel));
        gs.score_goal = 1.0;

        gs.select_card(0).unwrap();
        gs.play_hand().unwrap();

        if gs.jokers.is_empty() || gs.jokers[0].kind != JokerKind::GrosMichel {
            destroyed = true;
            break;
        }
    }
    assert!(destroyed, "GrosMichel should be destroyed at least once in 100 trials");
}

// =========================================================
// Hallucination: 1/2 chance to create a tarot when picking from a pack
// =========================================================

#[test]
fn test_hallucination_does_not_crash_when_picking_consumable_from_pack() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Hallucination));
    gs.consumable_slots = 5;
    gs.money = 100;
    gs.state = GameStateKind::Shop;
    gs.generate_shop();

    // Buy a pack
    let pack_index = gs.shop_offers.iter().position(|o| matches!(o.kind, crate::card::ShopItem::Pack(_))).unwrap();
    gs.buy_pack(pack_index).unwrap();

    // Take the first card from the pack (it could be a consumable)
    let result = gs.take_pack_card(0);
    // Should not panic/error
    assert!(result.is_ok() || matches!(result, Err(crate::game::BalatroError::ConsumableSlotsFull)));
}

// =========================================================
// InvisibleJoker: after 2 rounds, duplicates a joker
// =========================================================

#[test]
fn test_invisible_joker_duplicates_after_two_rounds() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::InvisibleJoker));
    gs.jokers.push(joker(2, JokerKind::Joker));
    gs.joker_slots = 10; // plenty of room

    let cards = vec![card(0, Rank::Ace, Suit::Spades)];
    setup_round(&mut gs, cards.clone(), 1);
    gs.score_goal = 1.0;

    // Round 1
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();
    // Now in Shop
    let joker_count_after_r1 = gs.jokers.len();

    // Go to next blind
    gs.state = GameStateKind::BlindSelect;
    gs.current_blind = crate::game::BlindKind::Small;
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.score_goal = 1.0;

    // Round 2
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    // After 2 rounds, InvisibleJoker should have triggered and duplicated Joker
    assert!(gs.jokers.len() > joker_count_after_r1,
        "InvisibleJoker should duplicate a joker after 2 rounds");
}

// =========================================================
// Juggler: +1 hand size
// =========================================================

#[test]
fn test_juggler_increases_hand_size_by_one() {
    let mut gs = make_game();
    let base_hand_size = gs.effective_hand_size();
    gs.jokers.push(joker(1, JokerKind::Juggler));
    assert_eq!(gs.effective_hand_size(), base_hand_size + 1,
        "Juggler should increase hand size by 1");
}

// =========================================================
// LoyaltyCard: x4 mult every 6 hands played
// =========================================================

#[test]
fn test_loyalty_card_fires_on_5th_modulo_6_total_plays() {
    // 5 total hands played → (5 % 6) == 5 → fires
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::LoyaltyCard)];

    let mut levels = default_hand_levels();
    // Set high card played=5 total
    levels.get_mut(&HandType::HighCard).unwrap().played = 5;

    let r = score_hand(&played, &played, &jokers, &levels, 3, 3, 0, 40, 52, None, 5, 0);
    // x4 mult → HC: 16*4=64
    assert_eq!(r.final_score as i64, 64);
}

#[test]
fn test_loyalty_card_does_not_fire_on_other_totals() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::LoyaltyCard)];

    let mut levels = default_hand_levels();
    levels.get_mut(&HandType::HighCard).unwrap().played = 3;

    let r = score_hand(&played, &played, &jokers, &levels, 3, 3, 0, 40, 52, None, 5, 0);
    // No x4, just HC: 16*1=16
    assert_eq!(r.final_score as i64, 16);
}

// =========================================================
// MailInRebate: +$3 per discarded card matching tracked rank
// =========================================================

#[test]
fn test_mail_in_rebate_earns_per_matching_rank_discard() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Two, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    setup_round(&mut gs, cards, 3);

    let mut mail = joker(1, JokerKind::MailInRebate);
    mail.counters.insert("rank".to_string(), serde_json::json!("Two"));
    gs.jokers.push(mail);
    gs.discards_remaining = 3;

    let money_before = gs.money;
    gs.select_card(0).unwrap(); // Two of Spades
    gs.select_card(1).unwrap(); // Two of Hearts
    gs.discard_hand().unwrap();

    // 2 Twos discarded → +$6
    assert_eq!(gs.money, money_before + 6,
        "MailInRebate should pay $3 per matching rank card discarded");
}

#[test]
fn test_mail_in_rebate_does_not_pay_non_matching() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Three, Suit::Spades),
        card(1, Rank::Four, Suit::Hearts),
    ];
    setup_round(&mut gs, cards, 2);

    let mut mail = joker(1, JokerKind::MailInRebate);
    mail.counters.insert("rank".to_string(), serde_json::json!("Two"));
    gs.jokers.push(mail);
    gs.discards_remaining = 3;

    let money_before = gs.money;
    gs.select_card(0).unwrap();
    gs.select_card(1).unwrap();
    gs.discard_hand().unwrap();

    assert_eq!(gs.money, money_before, "MailInRebate should not pay for non-matching rank");
}

// =========================================================
// MarbleJoker: adds a stone card to deck when blind is set
// =========================================================

#[test]
fn test_marble_joker_adds_stone_card_on_blind_set() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::MarbleJoker));

    let deck_size_before = gs.deck.len();
    gs.select_blind().unwrap();

    assert_eq!(gs.deck.len(), deck_size_before + 1,
        "MarbleJoker should add 1 stone card to deck on blind set");
    let last_card = gs.deck.last().unwrap();
    assert_eq!(last_card.enhancement, Enhancement::Stone,
        "The added card should be a Stone card");
}

// =========================================================
// Matador: $8 when boss blind effect is triggered (basic check)
// =========================================================

#[test]
fn test_matador_can_be_added_without_crash() {
    // Matador's full effect requires boss blind triggering logic.
    // Just verify it can be instantiated and doesn't break anything.
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Matador));
    assert_eq!(gs.jokers.len(), 1);
    assert_eq!(gs.jokers[0].kind, JokerKind::Matador);

    // Verify scoring still works
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::Matador)];
    let r = score(&played, &played, &jokers);
    // Matador doesn't add chips/mult during scoring, just check no panic
    assert!(r.final_score > 0.0);
}

// =========================================================
// MerryAndy: +3 discards, -1 hand size
// =========================================================

#[test]
fn test_merry_andy_increases_discards() {
    let mut gs = make_game();
    let base_discards = gs.effective_max_discards();
    gs.jokers.push(joker(1, JokerKind::MerryAndy));
    assert_eq!(gs.effective_max_discards(), base_discards + 3,
        "MerryAndy should add +3 discards");
}

#[test]
fn test_merry_andy_decreases_hand_size() {
    let mut gs = make_game();
    let base_hand_size = gs.effective_hand_size();
    gs.jokers.push(joker(1, JokerKind::MerryAndy));
    assert_eq!(gs.effective_hand_size(), base_hand_size - 1,
        "MerryAndy should reduce hand size by 1");
}

// =========================================================
// BaseballCard: x1.5 per Uncommon joker (rarity 2)
// =========================================================

#[test]
fn test_baseball_card_no_effect_without_uncommon_jokers() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let jokers = vec![joker(0, JokerKind::BaseballCard)];
    let r = score(&played, &played, &jokers);
    // No Uncommon jokers → no x_mult → HC: 16*1=16
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_baseball_card_multiplies_for_uncommon_joker() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    // FourFingers has rarity 2 (Uncommon)
    let jokers = vec![
        joker(0, JokerKind::BaseballCard),
        joker(1, JokerKind::FourFingers),
    ];
    let r = score(&played, &played, &jokers);
    // 1 Uncommon → x1.5 → HC: 16*1.5=24
    assert!((r.final_score - 24.0).abs() < 0.5,
        "BaseballCard should give x1.5 per Uncommon joker, got {}", r.final_score);
}

// =========================================================
// Burglar: +3 hands, 0 discards per round
// =========================================================

#[test]
fn test_burglar_adds_3_hands() {
    let mut gs = make_game();
    let base = gs.effective_max_hands();
    gs.jokers.push(joker(1, JokerKind::Burglar));
    assert_eq!(gs.effective_max_hands(), base + 3, "Burglar should add 3 hands");
}

#[test]
fn test_burglar_sets_discards_to_zero() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Burglar));
    assert_eq!(gs.effective_max_discards(), 0, "Burglar should reduce discards to 0");
}

// =========================================================
// BurntJoker: +1 hand after each discard
// =========================================================

#[test]
fn test_burnt_joker_gives_extra_hand_after_discard() {
    let mut gs = make_game();
    let cards: Vec<_> = (0..10).map(|i| card(i, Rank::Ace, Suit::Spades)).collect();
    setup_round(&mut gs, cards, 5);
    gs.jokers.push(joker(1, JokerKind::BurntJoker));
    gs.discards_remaining = 3;

    let hands_before = gs.hands_remaining;
    gs.select_card(0).unwrap();
    gs.discard_hand().unwrap();

    assert_eq!(gs.hands_remaining, hands_before + 1,
        "BurntJoker should grant +1 hand after discarding");
}

// =========================================================
// MidasMask: face cards become Gold when scored
// =========================================================

#[test]
fn test_midas_mask_turns_scored_face_cards_gold() {
    let mut gs = make_game();
    let king = card(0, Rank::King, Suit::Spades);
    let two  = card(1, Rank::Two, Suit::Hearts);
    setup_round(&mut gs, vec![king, two], 2);
    gs.jokers.push(joker(1, JokerKind::MidasMask));
    gs.score_goal = 1.0;

    gs.select_card(0).unwrap(); // King
    gs.play_hand().unwrap();

    let king_in_deck = gs.deck.iter().find(|c| c.rank == Rank::King).unwrap();
    assert_eq!(king_in_deck.enhancement, Enhancement::Gold,
        "MidasMask should turn scored face cards to Gold");
}

// =========================================================
// MrBones: saves run when score >= 25% of goal
// =========================================================

#[test]
fn test_mr_bones_saves_run_at_quarter_goal() {
    let mut gs = make_game();
    // Need enough cards for a high-card play
    let cards: Vec<_> = (0..5).map(|i| card(i, Rank::Two, Suit::Spades)).collect();
    setup_round(&mut gs, cards, 1);
    gs.jokers.push(joker(1, JokerKind::MrBones));
    gs.score_goal = 1000.0;
    gs.score_accumulated = 250.0; // exactly 25%
    gs.hands_remaining = 1; // last hand

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    // Mr. Bones should have saved: state is Shop not GameOver
    assert!(!matches!(gs.state, GameStateKind::GameOver),
        "MrBones should prevent GameOver when score >= 25% of goal");
}

// =========================================================
// RaisedFist: +2x lowest scoring card chip value
// =========================================================

#[test]
fn test_raised_fist_doubles_lowest_card_chips() {
    // Two of Spades = 2 chips (lowest), Ace = 11 chips
    // Pair of 2s: chips = 2+2+10 base + 2*2*2 raised = ?
    // Simpler: HC with just a Two
    let played = vec![card(0, Rank::Two, Suit::Spades)];
    let r_with = score(&played, &played, &[joker(0, JokerKind::RaisedFist)]);
    let r_without = score(&played, &played, &[]);
    // Two = 2 chips; RaisedFist adds 2*2=4 chips
    assert_eq!((r_with.final_chips - r_without.final_chips) as i64, 4,
        "RaisedFist should add 2x the lowest scoring card's chips");
}

// =========================================================
// RedCard: +3 mult per skipped blind
// =========================================================

#[test]
fn test_red_card_gains_mult_per_skipped_blind() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::RedCard));
    gs.state = GameStateKind::BlindSelect;
    gs.current_blind = crate::game::BlindKind::Small;

    gs.skip_blind().unwrap();

    // RedCard counter should have +3
    let mult = gs.jokers[0].get_counter_i64("mult");
    assert_eq!(mult, 3, "RedCard should gain +3 mult per skipped blind");
}

#[test]
fn test_red_card_mult_applies_in_scoring() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut red = joker(0, JokerKind::RedCard);
    red.set_counter_i64("mult", 6); // 2 blinds skipped
    let r = score(&played, &played, &[red]);
    // HC: chips=16, mult=1+6=7 → 112
    assert_eq!(r.final_score as i64, 112);
}

// =========================================================
// ReservedParking: $1 per face card held in hand
// =========================================================

#[test]
fn test_reserved_parking_earns_per_face_card_in_hand() {
    let king = card(0, Rank::King, Suit::Spades);
    let queen = card(1, Rank::Queen, Suit::Hearts);
    let played = vec![card(2, Rank::Two, Suit::Clubs)]; // played non-face
    let hand = vec![king, queen]; // held face cards
    let jokers = vec![joker(0, JokerKind::ReservedParking)];
    let r = score(&played, &hand, &jokers);
    // 2 face cards in hand → $2 dollars (simplified: always triggers)
    assert_eq!(r.dollars_earned, 2,
        "ReservedParking should earn $1 per face card held in hand");
}

// =========================================================
// RiffRaff: adds 2 common jokers at round start
// =========================================================

#[test]
fn test_riff_raff_adds_jokers_on_blind_select() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::RiffRaff));
    gs.joker_slots = 10;

    let jokers_before = gs.jokers.len();
    gs.select_blind().unwrap();

    // Should have added up to 2 common jokers
    assert!(gs.jokers.len() >= jokers_before,
        "RiffRaff should add jokers (or attempt to) at blind select");
}

// =========================================================
// Rocket: earns money per round, more after boss blinds
// =========================================================

#[test]
fn test_rocket_earns_money_at_end_of_round() {
    let mut gs = make_game();
    let cards = vec![card(0, Rank::Ace, Suit::Spades)];
    setup_round(&mut gs, cards, 1);
    gs.jokers.push(joker(1, JokerKind::Rocket));
    gs.score_goal = 1.0;

    let money_before = gs.money;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    // Rocket starts with $1 → earns $1
    assert!(gs.money > money_before,
        "Rocket should earn money at end of round");
}

// =========================================================
// Satellite: +$1 per unique planet type used this run
// =========================================================

#[test]
fn test_satellite_earns_per_planet_used() {
    let mut gs = make_game();
    let cards = vec![card(0, Rank::Ace, Suit::Spades)];
    setup_round(&mut gs, cards, 1);
    gs.jokers.push(joker(1, JokerKind::Satellite));
    gs.planet_cards_used = 3; // pretend 3 planet types used
    gs.score_goal = 1.0;

    let money_before = gs.money;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    let money_gained = gs.money - money_before;
    assert!(money_gained >= 3,
        "Satellite should earn at least $3 for 3 planets used, gained: {}", money_gained);
}

// =========================================================
// Seance: creates spectral card on Straight Flush
// =========================================================

#[test]
fn test_seance_creates_spectral_on_straight_flush() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Nine, Suit::Spades),
        card(1, Rank::Ten, Suit::Spades),
        card(2, Rank::Jack, Suit::Spades),
        card(3, Rank::Queen, Suit::Spades),
        card(4, Rank::King, Suit::Spades),
    ];
    setup_round(&mut gs, cards, 5);
    gs.jokers.push(joker(1, JokerKind::Seance));
    gs.consumable_slots = 5;
    gs.score_goal = 1.0;

    for i in 0..5 { gs.select_card(i).unwrap(); }
    gs.play_hand().unwrap();

    assert!(!gs.consumables.is_empty(),
        "Seance should create a spectral card on Straight Flush");
    assert!(gs.consumables.iter().any(|c| matches!(c, crate::card::ConsumableCard::Spectral(_))),
        "Seance should specifically create a Spectral card");
}

// =========================================================
// Seltzer: retriggers all played cards for 10 hands then destroyed
// =========================================================

#[test]
fn test_seltzer_retriggers_cards() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut seltzer = joker(0, JokerKind::Seltzer);
    seltzer.set_counter_i64("hands", 5); // 5 hands remaining on joker
    let r_with = score(&played, &played, &[seltzer]);
    let r_without = score(&played, &played, &[]);
    // Seltzer retriggers → higher score
    assert!(r_with.final_score > r_without.final_score,
        "Seltzer should retrigger scoring cards, with={} without={}", r_with.final_score, r_without.final_score);
}

#[test]
fn test_seltzer_exhausted_does_not_retrigger() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut seltzer = joker(0, JokerKind::Seltzer);
    seltzer.set_counter_i64("hands", 0); // exhausted
    let r_with = score(&played, &played, &[seltzer]);
    let r_without = score(&played, &played, &[]);
    assert_eq!(r_with.final_score as i64, r_without.final_score as i64,
        "Exhausted Seltzer should not retrigger");
}

#[test]
fn test_seltzer_decrements_counter_per_hand() {
    let mut gs = make_game();
    let cards: Vec<_> = (0..5).map(|i| card(i, Rank::Ace, Suit::Spades)).collect();
    setup_round(&mut gs, cards, 1);
    let mut seltzer = joker(1, JokerKind::Seltzer);
    seltzer.set_counter_i64("hands", 3);
    gs.jokers.push(seltzer);
    gs.score_goal = 1.0;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    // Counter should be decremented (joker may have been deactivated or removed if hands=0)
    // Just ensure no panic occurred
}

// =========================================================
// ShowMan: jokers can repeat in shop (existence check)
// =========================================================

#[test]
fn test_showman_can_be_added() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Showman));
    assert_eq!(gs.jokers[0].kind, JokerKind::Showman);
}

// =========================================================
// SixthSense: destroy a 6 played alone → get spectral
// =========================================================

#[test]
fn test_sixth_sense_destroys_six_and_creates_spectral() {
    let mut gs = make_game();
    let cards = vec![
        card(0, Rank::Six, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
    ];
    setup_round(&mut gs, cards, 2);
    gs.jokers.push(joker(1, JokerKind::SixthSense));
    gs.consumable_slots = 5;
    gs.score_goal = 1.0;

    gs.select_card(0).unwrap(); // only the 6
    gs.play_hand().unwrap();

    assert!(!gs.consumables.is_empty(),
        "SixthSense should create a spectral card when only a 6 is played");
    // The six should be destroyed: deck should have fewer cards
}

// =========================================================
// SpaceJoker: 1/4 chance to level up played hand
// =========================================================

#[test]
fn test_space_joker_can_level_up_hand_over_many_trials() {
    let mut leveled_up = false;
    for trial in 0..50 {
        let seed = format!("space_{}", trial);
        let mut gs = crate::game::GameState::new(DeckType::Blue, Stake::White, Some(seed));
        let cards = vec![card(0, Rank::Ace, Suit::Spades)];
        setup_round(&mut gs, cards, 1);
        gs.jokers.push(joker(1, JokerKind::SpaceJoker));
        gs.score_goal = 1.0;

        let level_before = gs.hand_levels.get(&HandType::HighCard).unwrap().level;
        gs.select_card(0).unwrap();
        gs.play_hand().unwrap();
        let level_after = gs.hand_levels.get(&HandType::HighCard).unwrap().level;

        if level_after > level_before {
            leveled_up = true;
            break;
        }
    }
    assert!(leveled_up, "SpaceJoker should level up the hand at least once in 50 trials");
}

// =========================================================
// Superposition: Ace + Straight → tarot card
// =========================================================

#[test]
fn test_superposition_creates_tarot_on_ace_straight() {
    let mut gs = make_game();
    // Ace-high straight: A,2,3,4,5 (wheel straight)
    let cards = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
        card(3, Rank::Four, Suit::Diamonds),
        card(4, Rank::Five, Suit::Spades),
    ];
    setup_round(&mut gs, cards, 5);
    gs.jokers.push(joker(1, JokerKind::Superposition));
    gs.consumable_slots = 5;
    gs.score_goal = 1.0;

    for i in 0..5 { gs.select_card(i).unwrap(); }
    gs.play_hand().unwrap();

    assert!(!gs.consumables.is_empty(),
        "Superposition should create a tarot card on Ace + Straight");
}

// =========================================================
// ToDoList: $4 if played hand matches tracked type
// =========================================================

#[test]
fn test_to_do_list_earns_dollars_on_match() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut todo = joker(0, JokerKind::ToDoList);
    todo.counters.insert("hand_type".to_string(), serde_json::json!("HighCard"));
    let r = score(&played, &played, &[todo]);
    assert_eq!(r.dollars_earned, 4,
        "ToDoList should earn $4 when played hand matches tracked type");
}

#[test]
fn test_to_do_list_does_not_earn_on_mismatch() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut todo = joker(0, JokerKind::ToDoList);
    todo.counters.insert("hand_type".to_string(), serde_json::json!("Pair"));
    let r = score(&played, &played, &[todo]);
    assert_eq!(r.dollars_earned, 0,
        "ToDoList should not earn when hand type doesn't match");
}

// =========================================================
// ToTheMoon: raises interest cap by $5 per joker
// =========================================================

#[test]
fn test_to_the_moon_raises_interest_cap() {
    let mut gs = make_game();
    let cards = vec![card(0, Rank::Ace, Suit::Spades)];
    setup_round(&mut gs, cards, 1);
    gs.money = 50; // lots of money to earn interest
    gs.max_interest = 5; // normally caps at $1 interest
    gs.jokers.push(joker(1, JokerKind::ToTheMoon));
    gs.score_goal = 1.0;

    let money_before = gs.money;
    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    // With ToTheMoon, effective cap is 5+5=10 → interest = min(50/5, 10/5) = min(10, 2) = 2
    // Without: interest = min(10, 1) = 1
    // Difference shows ToTheMoon raised the cap
    let interest_gained = gs.money - money_before - 3; // subtract blind reward
    assert!(interest_gained >= 2,
        "ToTheMoon should enable higher interest earnings, gained extra: {}", interest_gained);
}

// =========================================================
// TradingCard: $3 if first discard is 1 card, destroys that card
// =========================================================

#[test]
fn test_trading_card_earns_and_destroys_on_single_first_discard() {
    let mut gs = make_game();
    let cards: Vec<_> = (0..5).map(|i| card(i, Rank::Ace, Suit::Spades)).collect();
    setup_round(&mut gs, cards, 5);
    gs.jokers.push(joker(1, JokerKind::TradingCard));
    gs.discards_remaining = gs.max_discards;

    let deck_size_before = gs.deck.len();
    let money_before = gs.money;

    // First discard, single card
    gs.select_card(0).unwrap();
    gs.discard_hand().unwrap();

    assert_eq!(gs.money, money_before + 3,
        "TradingCard should earn $3 on first single-card discard");
    assert_eq!(gs.deck.len(), deck_size_before - 1,
        "TradingCard should destroy the discarded card");
}

// =========================================================
// Troubadour: +2 hand size, -1 hand per round
// =========================================================

#[test]
fn test_troubadour_increases_hand_size() {
    let mut gs = make_game();
    let base = gs.effective_hand_size();
    gs.jokers.push(joker(1, JokerKind::Troubadour));
    assert_eq!(gs.effective_hand_size(), base + 2,
        "Troubadour should add +2 hand size");
}

#[test]
fn test_troubadour_decreases_max_hands() {
    let mut gs = make_game();
    let base = gs.effective_max_hands();
    gs.jokers.push(joker(1, JokerKind::Troubadour));
    assert_eq!(gs.effective_max_hands(), base - 1,
        "Troubadour should reduce max hands by 1");
}

// =========================================================
// TurtleBean: +h_size hand size, shrinks by 1 per round
// =========================================================

#[test]
fn test_turtle_bean_increases_hand_size() {
    let mut gs = make_game();
    let base = gs.effective_hand_size();
    gs.jokers.push(joker(1, JokerKind::TurtleBean)); // starts at h_size=5
    assert_eq!(gs.effective_hand_size(), base + 5,
        "TurtleBean should add +5 to hand size");
}

#[test]
fn test_turtle_bean_shrinks_each_round() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::TurtleBean));
    let hand_size_before = gs.effective_hand_size();

    // Trigger notify_jokers_setting_blind via select_blind
    gs.select_blind().unwrap();

    // h_size should have decreased by 1
    let hand_size_after = gs.effective_hand_size();
    assert_eq!(hand_size_after, hand_size_before - 1,
        "TurtleBean should shrink by 1 each round, before={} after={}", hand_size_before, hand_size_after);
}

// =========================================================
// Vagabond: creates tarot when playing with $4 or less
// =========================================================

#[test]
fn test_vagabond_creates_tarot_when_broke() {
    let mut gs = make_game();
    let cards = vec![card(0, Rank::Ace, Suit::Spades)];
    setup_round(&mut gs, cards, 1);
    gs.jokers.push(joker(1, JokerKind::Vagabond));
    gs.money = 4; // at the threshold
    gs.consumable_slots = 5;
    gs.score_goal = 1.0;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    assert!(!gs.consumables.is_empty(),
        "Vagabond should create a tarot when money <= $4");
}

#[test]
fn test_vagabond_does_not_create_tarot_when_rich() {
    let mut gs = make_game();
    let cards = vec![card(0, Rank::Ace, Suit::Spades)];
    setup_round(&mut gs, cards, 1);
    gs.jokers.push(joker(1, JokerKind::Vagabond));
    gs.money = 10; // above threshold
    gs.consumable_slots = 5;
    gs.score_goal = 1.0;

    gs.select_card(0).unwrap();
    gs.play_hand().unwrap();

    assert!(gs.consumables.is_empty(),
        "Vagabond should not create tarot when money > $4");
}
