/// Tests for planet card application — each planet upgrades a specific hand type by 1 level.
///
/// Planet → Hand type mapping:
///   Pluto   → HighCard        Mercury → Pair
///   Uranus  → TwoPair         Venus   → ThreeOfAKind
///   Saturn  → Straight        Jupiter → Flush
///   Earth   → FullHouse       Mars    → FourOfAKind
///   Neptune → StraightFlush   PlanetX → FiveOfAKind
///   Ceres   → FlushHouse      Eris    → FlushFive

use super::*;

// Helper: use a planet consumable from the consumables slot and return the game state.
fn apply_planet(planet: PlanetCard) -> GameState {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.consumables.push(crate::card::ConsumableCard::Planet(planet));
    gs.use_consumable(0, vec![]).unwrap();
    gs
}

// Helper: get the level of a hand type from the game state.
fn level_of(gs: &GameState, ht: HandType) -> u32 {
    gs.hand_levels[&ht].level
}

// Helper: check that only the target hand type was upgraded, others unchanged.
fn assert_only_upgraded(gs: &GameState, upgraded: HandType) {
    let all = [
        HandType::HighCard, HandType::Pair, HandType::TwoPair,
        HandType::ThreeOfAKind, HandType::Straight, HandType::Flush,
        HandType::FullHouse, HandType::FourOfAKind, HandType::StraightFlush,
        HandType::FiveOfAKind, HandType::FlushHouse, HandType::FlushFive,
    ];
    for ht in all {
        let expected = if ht == upgraded { 2 } else { 1 };
        assert_eq!(
            level_of(gs, ht), expected,
            "{:?} should be level {expected} after upgrading {:?}", ht, upgraded
        );
    }
}

// =========================================================
// One test per planet — verifies correct hand type is levelled
// =========================================================

#[test]
fn test_pluto_upgrades_high_card() {
    let gs = apply_planet(PlanetCard::Pluto);
    assert_eq!(level_of(&gs, HandType::HighCard), 2);
}

#[test]
fn test_mercury_upgrades_pair() {
    let gs = apply_planet(PlanetCard::Mercury);
    assert_eq!(level_of(&gs, HandType::Pair), 2);
}

#[test]
fn test_uranus_upgrades_two_pair() {
    let gs = apply_planet(PlanetCard::Uranus);
    assert_eq!(level_of(&gs, HandType::TwoPair), 2);
}

#[test]
fn test_venus_upgrades_three_of_a_kind() {
    let gs = apply_planet(PlanetCard::Venus);
    assert_eq!(level_of(&gs, HandType::ThreeOfAKind), 2);
}

#[test]
fn test_saturn_upgrades_straight() {
    let gs = apply_planet(PlanetCard::Saturn);
    assert_eq!(level_of(&gs, HandType::Straight), 2);
}

#[test]
fn test_jupiter_upgrades_flush() {
    let gs = apply_planet(PlanetCard::Jupiter);
    assert_eq!(level_of(&gs, HandType::Flush), 2);
}

#[test]
fn test_earth_upgrades_full_house() {
    let gs = apply_planet(PlanetCard::Earth);
    assert_eq!(level_of(&gs, HandType::FullHouse), 2);
}

#[test]
fn test_mars_upgrades_four_of_a_kind() {
    let gs = apply_planet(PlanetCard::Mars);
    assert_eq!(level_of(&gs, HandType::FourOfAKind), 2);
}

#[test]
fn test_neptune_upgrades_straight_flush() {
    let gs = apply_planet(PlanetCard::Neptune);
    assert_eq!(level_of(&gs, HandType::StraightFlush), 2);
}

#[test]
fn test_planet_x_upgrades_five_of_a_kind() {
    let gs = apply_planet(PlanetCard::PlanetX);
    assert_eq!(level_of(&gs, HandType::FiveOfAKind), 2);
}

#[test]
fn test_ceres_upgrades_flush_house() {
    let gs = apply_planet(PlanetCard::Ceres);
    assert_eq!(level_of(&gs, HandType::FlushHouse), 2);
}

#[test]
fn test_eris_upgrades_flush_five() {
    let gs = apply_planet(PlanetCard::Eris);
    assert_eq!(level_of(&gs, HandType::FlushFive), 2);
}

// =========================================================
// Correctness: only the targeted hand type changes
// =========================================================

#[test]
fn test_mercury_only_upgrades_pair_not_others() {
    let gs = apply_planet(PlanetCard::Mercury);
    assert_only_upgraded(&gs, HandType::Pair);
}

#[test]
fn test_jupiter_only_upgrades_flush_not_others() {
    let gs = apply_planet(PlanetCard::Jupiter);
    assert_only_upgraded(&gs, HandType::Flush);
}

// =========================================================
// Stacking: multiple planets on the same hand type
// =========================================================

#[test]
fn test_two_mercury_planets_bring_pair_to_level_3() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Mercury));
    gs.use_consumable(0, vec![]).unwrap();
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Mercury));
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(level_of(&gs, HandType::Pair), 3);
}

// =========================================================
// Level-up actually affects scoring chips/mult
// =========================================================

#[test]
fn test_mercury_level_2_pair_scores_higher_than_level_1() {
    // Level 1 Pair of Aces: chips=10+11+11=32, mult=2 → 64
    let played = vec![
        card(0, Rank::Ace, Suit::Spades),
        card(1, Rank::Ace, Suit::Hearts),
    ];

    let r_lvl1 = score(&played, &played, &[]);
    assert_eq!(r_lvl1.final_score as i64, 64);

    // Level 2 Pair: chips += level_chip_bonus (15), mult += level_mult_bonus (1)
    // chips=32+15=47, mult=2+1=3 → 141
    let mut levels = default_hand_levels();
    levels.get_mut(&HandType::Pair).unwrap().level = 2;
    let r_lvl2 = score_hand(&played, &played, &[], &levels, 3, 3, 0, 40, 52, None, 5, 0, 0);

    assert!(r_lvl2.final_score > r_lvl1.final_score,
        "Level 2 Pair should score more than Level 1");
    assert_eq!(r_lvl2.final_score as i64, 141);
}

#[test]
fn test_planet_consumed_is_removed_from_consumables() {
    let mut gs = make_game();
    setup_round(&mut gs, vec![card(0, Rank::Ace, Suit::Spades)], 1);
    gs.consumables.push(crate::card::ConsumableCard::Planet(PlanetCard::Mercury));
    assert_eq!(gs.consumables.len(), 1);
    gs.use_consumable(0, vec![]).unwrap();
    assert_eq!(gs.consumables.len(), 0, "Planet card should be consumed after use");
}
