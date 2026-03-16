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
fn test_swashbuckler_applies_counter_mult() {
    let played = vec![card(0, Rank::Ace, Suit::Spades)];
    let mut j = joker(0, JokerKind::Swashbuckler);
    j.set_counter_i64("mult", 10);
    let r = score(&played, &played, &[j]);
    // HC: 16*(1+10)=176
    assert_eq!(r.final_score as i64, 176);
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
