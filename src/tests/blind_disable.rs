/// `Blind:disable()` (blind.lua:356) hands back everything the Boss blind had taken. Chicot does
/// it passively; Luchador does it part-way through a round the blind has already shaped.

use super::*;
use crate::game::{BlindKind, GameStateKind};

fn boss_round(boss: BossBlind, seed: &str) -> GameState {
    let mut gs = GameState::new(DeckType::Red, Stake::White, Some(seed.to_string()));
    gs.current_blind = BlindKind::Boss;
    gs.boss_blind = Some(boss);
    gs
}

// =========================================================
// The chip requirement
// =========================================================

#[test]
fn test_chicot_halves_the_walls_requirement() {
    let mut gs = boss_round(BossBlind::TheWall, "WALL1");
    gs.select_blind().unwrap();
    let armed = gs.score_goal;

    let mut gs = boss_round(BossBlind::TheWall, "WALL1");
    gs.jokers.push(joker(1, JokerKind::Chicot));
    gs.select_blind().unwrap();

    assert_eq!(gs.score_goal, armed / 2.0,
        "disabling The Wall halves its chips (blind.lua:377)");
}

#[test]
fn test_chicot_cuts_violet_vessel_to_a_third() {
    let mut gs = boss_round(BossBlind::VioletVessel, "VV1");
    gs.select_blind().unwrap();
    let armed = gs.score_goal;

    let mut gs = boss_round(BossBlind::VioletVessel, "VV1");
    gs.jokers.push(joker(1, JokerKind::Chicot));
    gs.select_blind().unwrap();

    assert_eq!(gs.score_goal, armed / 3.0,
        "disabling Violet Vessel cuts its chips to a third (blind.lua:393)");
}

#[test]
fn test_a_disabled_needle_keeps_its_easy_requirement() {
    // disable() only touches The Wall and Violet Vessel — the small goal is the blind itself,
    // not its ability, so Chicot must not raise it.
    let mut gs = boss_round(BossBlind::TheNeedle, "NDL1");
    gs.jokers.push(joker(1, JokerKind::Chicot));
    gs.select_blind().unwrap();

    let base = crate::game::get_base_blind_amount(gs.ante) as f64;
    assert_eq!(gs.score_goal, base, "The Needle is a 1x blind, disabled or not");
}

#[test]
fn test_selling_luchador_lowers_the_wall_mid_round() {
    let mut gs = boss_round(BossBlind::TheWall, "WALL2");
    gs.jokers.push(joker(1, JokerKind::Luchador));
    gs.select_blind().unwrap();
    let armed = gs.score_goal;

    gs.sell_joker(0).unwrap();

    assert_eq!(gs.score_goal, armed / 2.0,
        "the requirement is restated the moment the blind goes off");
}

// =========================================================
// Resources the blind had already taken
// =========================================================

#[test]
fn test_selling_luchador_hands_back_the_waters_discards() {
    let mut gs = boss_round(BossBlind::TheWater, "WATER1");
    gs.jokers.push(joker(1, JokerKind::Luchador));
    gs.select_blind().unwrap();
    assert_eq!(gs.discards_remaining, 0, "The Water opens on nothing");

    gs.sell_joker(0).unwrap();

    assert_eq!(gs.discards_remaining, gs.effective_max_discards(),
        "disable() calls ease_discard(discards_sub) to give them back");
    assert!(gs.discards_remaining > 0);
}

#[test]
fn test_selling_luchador_hands_back_the_needles_hands() {
    let mut gs = boss_round(BossBlind::TheNeedle, "NDL2");
    gs.jokers.push(joker(1, JokerKind::Luchador));
    gs.select_blind().unwrap();
    assert_eq!(gs.hands_remaining, 1, "The Needle opens on one hand");

    gs.sell_joker(0).unwrap();

    assert_eq!(gs.hands_remaining, gs.max_hands,
        "disable() calls ease_hands_played(hands_sub) to give them back");
}

#[test]
fn test_selling_luchador_turns_the_hand_face_up() {
    let mut gs = boss_round(BossBlind::TheMark, "MARK1");
    gs.jokers.push(joker(1, JokerKind::Luchador));
    gs.select_blind().unwrap();

    gs.sell_joker(0).unwrap();

    assert!(gs.hand.iter().all(|&di| !gs.deck[di].face_down),
        "disable() flips the hidden cards back over (blind.lua:364)");
}

#[test]
fn test_selling_luchador_releases_cerulean_bells_forced_card() {
    let mut gs = boss_round(BossBlind::CeruleanBell, "BELL1");
    gs.jokers.push(joker(1, JokerKind::Luchador));
    gs.select_blind().unwrap();
    assert!(gs.cerulean_forced_card_id.is_some());

    gs.sell_joker(0).unwrap();

    assert!(gs.cerulean_forced_card_id.is_none());
    // The card that was pinned can now be let go.
    if let Some(&pinned) = gs.selected_indices.first() {
        assert!(gs.deselect_card(pinned).is_ok());
    }
}

#[test]
fn test_selling_luchador_gives_the_manacles_card_back() {
    let mut gs = boss_round(BossBlind::TheManacle, "MAN1");
    gs.jokers.push(joker(1, JokerKind::Luchador));
    gs.select_blind().unwrap();
    let held = gs.hand.len();

    gs.sell_joker(0).unwrap();

    assert_eq!(gs.hand.len(), held + 1,
        "disable() restores the hand slot and draws into it (blind.lua:386)");
}

#[test]
fn test_selling_luchador_lifts_card_debuffs() {
    let mut gs = boss_round(BossBlind::TheClub, "CLUB1");
    gs.jokers.push(joker(1, JokerKind::Luchador));
    gs.select_blind().unwrap();
    assert!(gs.deck.iter().any(|c| c.debuffed));

    gs.sell_joker(0).unwrap();

    assert!(gs.deck.iter().all(|c| !c.debuffed));
}

#[test]
fn test_disabling_an_already_disabled_blind_changes_nothing() {
    let mut gs = boss_round(BossBlind::TheWater, "WATER2");
    gs.jokers.push(joker(1, JokerKind::Chicot));
    gs.jokers.push(joker(2, JokerKind::Luchador));
    gs.select_blind().unwrap();
    let discards = gs.discards_remaining;
    let hand = gs.hand.len();

    gs.sell_joker(1).unwrap();

    assert_eq!(gs.discards_remaining, discards, "Chicot had already switched it off");
    assert_eq!(gs.hand.len(), hand, "and no extra card is drawn");
}

#[test]
fn test_selling_luchador_in_the_shop_does_not_draw_cards() {
    let mut gs = boss_round(BossBlind::TheManacle, "SHOP1");
    gs.state = GameStateKind::Shop;
    gs.jokers.push(joker(1, JokerKind::Luchador));

    gs.sell_joker(0).unwrap();

    assert!(gs.boss_blind_manually_disabled, "the blind is still latched off");
    assert!(gs.hand.is_empty(), "but there is no round to restore anything to");
}
