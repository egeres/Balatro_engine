/// Tests for rare/legendary jokers: counter-based and scaling jokers.

use super::*;

// =========================================================
// Counter-based flat-mult jokers
// =========================================================

#[test]
fn test_ceremonial_dagger_applies_counter_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::CeremonialDagger);
    j.set_counter_i64("mult", 10);
    let r = score(&played, &played, &[j]);
    // HC: 16*(1+10)=176
    assert_eq!(r.final_score as i64, 176);
}

#[test]
fn test_spare_trousers_applies_counter_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::SpareTrousers);
    j.set_counter_i64("mult", 8);
    let r = score(&played, &played, &[j]);
    // HC: 16*(1+8)=144
    assert_eq!(r.final_score as i64, 144);
}

#[test]
fn test_ride_the_bus_applies_counter_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::RideTheBus);
    j.set_counter_i64("mult", 6);
    let r = score(&played, &played, &[j]);
    // HC: 16*(1+6)=112
    assert_eq!(r.final_score as i64, 112);
}

#[test]
fn test_flash_card_applies_counter_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::FlashCard);
    j.set_counter_i64("mult", 4);
    let r = score(&played, &played, &[j]);
    // HC: 16*(1+4)=80
    assert_eq!(r.final_score as i64, 80);
}

#[test]
fn test_popcorn_applies_counter_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Popcorn);
    j.set_counter_i64("mult", 12);
    let r = score(&played, &played, &[j]);
    // HC: 16*(1+12)=208
    assert_eq!(r.final_score as i64, 208);
}

#[test]
fn test_swashbuckler_alone_adds_no_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let r = score(&played, &played, &[joker(0, JokerKind::Swashbuckler)]);
    // No other jokers → +0 mult. HC: 16 * 1 = 16
    assert_eq!(r.final_score as i64, 16);
}

#[test]
fn test_swashbuckler_adds_summed_sell_value_of_other_jokers() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    // Juggler costs 4 → sell 2; Drunkard costs 4 → sell 2. Neither scores anything itself.
    let r = score(
        &played,
        &played,
        &[
            joker(0, JokerKind::Swashbuckler),
            joker(1, JokerKind::Juggler),
            joker(2, JokerKind::Drunkard),
        ],
    );
    // HC mult 1 + 4 = 5, chips 16 → 80
    assert_eq!(r.final_mult as i64, 5);
    assert_eq!(r.final_score as i64, 80);
}

#[test]
fn test_swashbuckler_tracks_the_board_live() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    // Joker costs 2 → sell value max(2/2, 1) = 1
    let one = score(
        &played,
        &played,
        &[joker(0, JokerKind::Swashbuckler), joker(1, JokerKind::Joker)],
    );
    let two = score(
        &played,
        &played,
        &[
            joker(0, JokerKind::Swashbuckler),
            joker(1, JokerKind::Joker),
            joker(2, JokerKind::Juggler), // cost 4 → sell 2
        ],
    );
    // Swashbuckler's contribution grows with the board: +1 vs +1+2.
    // (The plain Joker also adds a flat +4 mult in both cases.)
    assert_eq!(one.final_mult as i64, 1 + 1 + 4);
    assert_eq!(two.final_mult as i64, 1 + 3 + 4);
}

#[test]
fn test_green_joker_applies_counter_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::GreenJoker);
    j.set_counter_i64("mult", 5);
    let r = score(&played, &played, &[j]);
    // HC: 16*(1+5)=96
    assert_eq!(r.final_score as i64, 96);
}

// =========================================================
// Counter-based chip jokers
// =========================================================

#[test]
fn test_runner_applies_counter_chips() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Runner);
    j.set_counter_i64("chips", 20);
    let r = score(&played, &played, &[j]);
    // HC: 16+20=36, mult=1 → 36
    assert_eq!(r.final_score as i64, 36);
}

#[test]
fn test_ice_cream_applies_counter_chips() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::IceCream);
    j.set_counter_i64("chips", 60);
    let r = score(&played, &played, &[j]);
    // HC: 16+60=76, mult=1 → 76
    assert_eq!(r.final_score as i64, 76);
}

#[test]
fn test_square_joker_applies_counter_chips() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::SquareJoker);
    j.set_counter_i64("chips", 16);
    let r = score(&played, &played, &[j]);
    // HC: 16+16=32, mult=1 → 32
    assert_eq!(r.final_score as i64, 32);
}

#[test]
fn test_wee_joker_applies_counter_chips() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::WeeJoker);
    j.set_counter_i64("chips", 8);
    let r = score(&played, &played, &[j]);
    // HC: 16+8=24, mult=1 → 24
    assert_eq!(r.final_score as i64, 24);
}

#[test]
fn test_castle_applies_counter_chips() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Castle);
    j.set_counter_i64("chips", 15);
    let r = score(&played, &played, &[j]);
    // HC: 16+15=31, mult=1 → 31
    assert_eq!(r.final_score as i64, 31);
}

// =========================================================
// Counter-based x-mult jokers
// =========================================================

#[test]
fn test_hologram_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Hologram);
    j.set_counter_f64("x_mult", 2.0);
    let r = score(&played, &played, &[j]);
    // HC: 16*2=32
    assert_eq!(r.final_score as i64, 32);
}

#[test]
fn test_vampire_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Vampire);
    j.set_counter_f64("x_mult", 1.5);
    let r = score(&played, &played, &[j]);
    // HC: 16*1.5=24
    assert_eq!(r.final_score as i64, 24);
}

#[test]
fn test_lucky_cat_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::LuckyCat);
    j.set_counter_f64("x_mult", 2.0);
    let r = score(&played, &played, &[j]);
    assert_eq!(r.final_score as i64, 32);
}

#[test]
fn test_constellation_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Constellation);
    j.set_counter_f64("x_mult", 2.0);
    let r = score(&played, &played, &[j]);
    assert_eq!(r.final_score as i64, 32);
}

#[test]
fn test_glass_joker_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::GlassJoker);
    j.set_counter_f64("x_mult", 1.5);
    let r = score(&played, &played, &[j]);
    assert_eq!(r.final_score as i64, 24);
}

#[test]
fn test_ramen_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Ramen);
    j.set_counter_f64("x_mult", 1.5);
    let r = score(&played, &played, &[j]);
    assert_eq!(r.final_score as i64, 24);
}

#[test]
fn test_hit_the_road_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::HitTheRoad);
    j.set_counter_f64("x_mult", 2.0);
    let r = score(&played, &played, &[j]);
    assert_eq!(r.final_score as i64, 32);
}

#[test]
fn test_madness_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Madness);
    j.set_counter_f64("x_mult", 2.0);
    let r = score(&played, &played, &[j]);
    assert_eq!(r.final_score as i64, 32);
}

#[test]
fn test_campfire_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Campfire);
    j.set_counter_f64("x_mult", 2.0);
    let r = score(&played, &played, &[j]);
    assert_eq!(r.final_score as i64, 32);
}

#[test]
fn test_yorick_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Yorick);
    j.set_counter_f64("x_mult", 2.0);
    let r = score(&played, &played, &[j]);
    assert_eq!(r.final_score as i64, 32);
}

#[test]
fn test_obelisk_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Obelisk);
    j.set_counter_f64("x_mult", 2.0);
    let r = score(&played, &played, &[j]);
    assert_eq!(r.final_score as i64, 32);
}

#[test]
fn test_canio_applies_counter_x_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Canio);
    j.set_counter_f64("x_mult", 2.0);
    let r = score(&played, &played, &[j]);
    assert_eq!(r.final_score as i64, 32);
}

// =========================================================
// Glass Joker: gains X0.75 per destroyed Glass card
// =========================================================

/// Destroy glass cards deterministically with The Hanged Man and check the counter grows.
fn destroy_with_hanged_man(deck_cards: Vec<CardInstance>, targets: Vec<usize>) -> GameState {
    let mut gs = make_game();
    setup_round(&mut gs, deck_cards, 3);
    gs.jokers.push(joker(0, JokerKind::GlassJoker));
    gs.consumables
        .push(crate::card::ConsumableCard::Tarot(TarotCard::TheHangedMan));
    gs.use_consumable(0, targets).unwrap();
    gs
}

fn glass(id: u64, rank: Rank, suit: Suit) -> CardInstance {
    let mut c = card(id, rank, suit);
    c.enhancement = Enhancement::Glass;
    c
}

#[test]
fn test_glass_joker_gains_x_mult_when_a_glass_card_is_destroyed() {
    let deck = vec![
        glass(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    let gs = destroy_with_hanged_man(deck, vec![0]);
    assert_eq!(gs.jokers[0].get_counter_f64("x_mult"), 1.75);
}

#[test]
fn test_glass_joker_stacks_per_destroyed_glass_card() {
    let deck = vec![
        glass(0, Rank::Ace, Suit::Spades),
        glass(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    let gs = destroy_with_hanged_man(deck, vec![0, 1]);
    assert_eq!(gs.jokers[0].get_counter_f64("x_mult"), 2.5);
}

#[test]
fn test_glass_joker_ignores_destroyed_non_glass_cards() {
    let deck = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Two, Suit::Hearts),
        card(2, Rank::Three, Suit::Clubs),
    ];
    let gs = destroy_with_hanged_man(deck, vec![0, 1]);
    assert_eq!(gs.jokers[0].get_counter_f64("x_mult"), 1.0);
}

// =========================================================
// Ramen: X2 Mult, loses X0.01 per discarded card
// =========================================================

/// Discard `per_discard` cards `times` times and return the resulting state.
fn ramen_after_discards(start_x: f64, per_discard: usize, times: usize) -> GameState {
    let mut gs = make_game();
    let deck: Vec<CardInstance> = (0..40)
        .map(|i| card(i, Rank::Two, Suit::Spades))
        .collect();
    setup_round(&mut gs, deck, 8);
    let mut j = joker(0, JokerKind::Ramen);
    j.set_counter_f64("x_mult", start_x);
    gs.jokers.push(j);
    gs.discards_remaining = times as u32;
    for _ in 0..times {
        for k in 0..per_discard {
            gs.select_card(k).unwrap();
        }
        gs.discard_hand().unwrap();
    }
    gs
}

#[test]
fn test_ramen_starts_at_x2() {
    assert_eq!(
        joker(0, JokerKind::Ramen).get_counter_f64("x_mult"),
        2.0
    );
}

#[test]
fn test_ramen_loses_x001_per_discarded_card() {
    let gs = ramen_after_discards(2.0, 5, 1);
    assert!(
        (gs.jokers[0].get_counter_f64("x_mult") - 1.95).abs() < 1e-9,
        "got {}",
        gs.jokers[0].get_counter_f64("x_mult")
    );
}

#[test]
fn test_ramen_decays_per_card_not_per_discard_action() {
    let one_big = ramen_after_discards(2.0, 4, 1);
    let two_small = ramen_after_discards(2.0, 2, 2);
    assert!(
        (one_big.jokers[0].get_counter_f64("x_mult")
            - two_small.jokers[0].get_counter_f64("x_mult"))
        .abs()
            < 1e-9
    );
}

#[test]
fn test_ramen_is_eaten_when_it_would_drop_to_x1() {
    let gs = ramen_after_discards(1.01, 1, 1);
    assert!(
        gs.jokers.is_empty(),
        "Ramen should be destroyed instead of dropping to X1"
    );
}

#[test]
fn test_ramen_survives_at_exactly_above_the_threshold() {
    let gs = ramen_after_discards(1.02, 1, 1);
    assert_eq!(gs.jokers.len(), 1);
    assert!((gs.jokers[0].get_counter_f64("x_mult") - 1.01).abs() < 1e-9);
}

// =========================================================
// Ceremonial Dagger: slices the joker to its right on blind select
// =========================================================

fn dagger_setup(jokers: Vec<JokerInstance>) -> GameState {
    let mut gs = make_game();
    gs.jokers = jokers;
    gs.select_blind().unwrap();
    gs
}

#[test]
fn test_ceremonial_dagger_destroys_joker_to_its_right() {
    let gs = dagger_setup(vec![
        joker(0, JokerKind::CeremonialDagger),
        joker(1, JokerKind::Blueprint), // cost 10 → sell value 5
    ]);
    assert_eq!(gs.jokers.len(), 1);
    assert_eq!(gs.jokers[0].kind, JokerKind::CeremonialDagger);
    // 2 x sell value (10/2 = 5) = +10 Mult
    assert_eq!(gs.jokers[0].get_counter_i64("mult"), 10);
}

#[test]
fn test_ceremonial_dagger_does_nothing_when_rightmost() {
    let gs = dagger_setup(vec![
        joker(0, JokerKind::Joker),
        joker(1, JokerKind::CeremonialDagger),
    ]);
    assert_eq!(gs.jokers.len(), 2);
    assert_eq!(gs.jokers[1].get_counter_i64("mult"), 0);
}

#[test]
fn test_ceremonial_dagger_spares_eternal_jokers() {
    let mut eternal = joker(1, JokerKind::Blueprint);
    eternal.eternal = true;
    let gs = dagger_setup(vec![joker(0, JokerKind::CeremonialDagger), eternal]);
    assert_eq!(gs.jokers.len(), 2);
    assert_eq!(gs.jokers[0].get_counter_i64("mult"), 0);
}

#[test]
fn test_ceremonial_dagger_accumulates_across_blinds() {
    let mut gs = make_game();
    gs.jokers = vec![
        joker(0, JokerKind::CeremonialDagger),
        joker(1, JokerKind::Blueprint),   // sell 5 → +10
        joker(2, JokerKind::Brainstorm),  // sell 5 → +10
    ];
    gs.select_blind().unwrap();
    assert_eq!(gs.jokers[0].get_counter_i64("mult"), 10);

    gs.state = crate::game::GameStateKind::BlindSelect;
    gs.select_blind().unwrap();
    assert_eq!(gs.jokers.len(), 1);
    assert_eq!(gs.jokers[0].get_counter_i64("mult"), 20);
}

#[test]
fn test_sliced_joker_does_not_fire_its_own_setting_blind_effect() {
    // Marble Joker adds a stone card on blind select; sliced first, it must not.
    let mut gs = make_game();
    let deck_before = gs.deck.len();
    gs.jokers = vec![
        joker(0, JokerKind::CeremonialDagger),
        joker(1, JokerKind::MarbleJoker),
    ];
    gs.select_blind().unwrap();
    assert_eq!(gs.jokers.len(), 1);
    assert_eq!(gs.deck.len(), deck_before, "sliced Marble Joker must not add a stone card");
}

// =========================================================
// Perkeo: a Negative copy of a held consumable when the shop closes
// =========================================================

#[test]
fn test_perkeo_copies_a_consumable_when_leaving_the_shop() {
    let mut gs = make_game();
    gs.state = crate::game::GameStateKind::Shop;
    gs.jokers.push(joker(1, JokerKind::Perkeo));
    gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheFool));

    gs.leave_shop().unwrap();

    assert_eq!(gs.consumables.len(), 2, "Perkeo should copy the held consumable");
    assert_eq!(gs.consumables[1], crate::card::ConsumableCard::Tarot(TarotCard::TheFool));
    assert_eq!(gs.consumable_slots, 3, "the Negative copy brings its own slot");
}

#[test]
fn test_perkeo_slot_goes_away_with_the_card() {
    // In Balatro the extra slot belongs to the Negative card, so it must not accumulate across
    // shops for the rest of the run.
    let mut gs = make_game();
    gs.state = crate::game::GameStateKind::Shop;
    gs.jokers.push(joker(1, JokerKind::Perkeo));
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Mercury));

    gs.leave_shop().unwrap();
    assert_eq!(gs.consumable_slots, 3);

    // Spend both consumables; the borrowed slot is handed back.
    gs.use_consumable(0, vec![]).unwrap();
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.consumable_slots, 2, "the Negative slot should be released once spent");
}

#[test]
fn test_perkeo_slots_do_not_creep_over_many_shops() {
    let mut gs = make_game();
    gs.jokers.push(joker(1, JokerKind::Perkeo));

    for _ in 0..5 {
        gs.state = crate::game::GameStateKind::Shop;
        gs.consumables.clear();
        gs.release_negative_consumable_slots();
        gs.consumables.push(crate::card::ConsumableCard::Tarot(TarotCard::TheFool));
        gs.leave_shop().unwrap();
    }

    assert_eq!(gs.consumable_slots, 3,
        "each shop lends exactly one slot, it should not stack to 7");
}
