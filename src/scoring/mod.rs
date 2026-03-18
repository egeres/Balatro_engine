use crate::card::{CardInstance, HandLevelData, JokerInstance};
use crate::hand_eval::evaluate_hand;
use crate::types::*;
use std::collections::HashMap;

/// The result of scoring a hand
#[derive(Debug, Clone)]
pub struct ScoreResult {
    pub hand_type: HandType,
    pub hand_name: String,
    pub scoring_card_indices: Vec<usize>,
    pub base_chips: i64,
    pub base_mult: i64,
    pub final_chips: f64,
    pub final_mult: f64,
    pub final_score: f64,
    pub dollars_earned: i32,
    /// Events that happened during scoring (for history / debugging)
    pub events: Vec<ScoreEvent>,
}

#[derive(Debug, Clone)]
pub struct ScoreEvent {
    pub source: String,
    pub kind: ScoreEventKind,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub enum ScoreEventKind {
    Chips,
    Mult,
    XMult,
    Dollars,
    Retrigger,
    CardDestroyed,
}

/// Context passed to the joker evaluators in Phase 4
pub struct ScoringContext<'a> {
    pub hand_type: HandType,
    pub scoring_cards: &'a [usize],
    pub played_cards: &'a [CardInstance],
    pub hand_cards: &'a [CardInstance],
    pub jokers: &'a [JokerInstance],
    pub hand_levels: &'a HashMap<HandType, HandLevelData>,
    pub hands_remaining: u32,
    pub discards_remaining: u32,
    pub money: i32,
    pub deck_cards_remaining: usize,
    pub total_deck_size: usize,
    pub boss_blind: Option<BossBlind>,
    pub joker_count: usize,
    pub joker_slot_count: usize,
    pub tarot_cards_used: u32,
    pub steel_count_in_deck: usize,
    pub stone_count_in_deck: usize,
    pub enhanced_count_in_deck: usize,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn push_effect_events(events: &mut Vec<ScoreEvent>, effect: &JokerEffect, source: &str) {
    if effect.chips != 0 {
        events.push(ScoreEvent { source: source.to_string(), kind: ScoreEventKind::Chips,  value: effect.chips as f64 });
    }
    if effect.mult != 0 {
        events.push(ScoreEvent { source: source.to_string(), kind: ScoreEventKind::Mult,   value: effect.mult as f64 });
    }
    if effect.x_mult != 1.0 {
        events.push(ScoreEvent { source: source.to_string(), kind: ScoreEventKind::XMult,  value: effect.x_mult });
    }
}

// ---------------------------------------------------------------------------
// Main scoring entry-point
// ---------------------------------------------------------------------------

pub fn score_hand(
    played_cards: &[CardInstance],
    hand_cards: &[CardInstance],
    jokers: &[JokerInstance],
    hand_levels: &HashMap<HandType, HandLevelData>,
    hands_remaining: u32,
    discards_remaining: u32,
    money: i32,
    deck_remaining: usize,
    total_deck: usize,
    boss_blind: Option<BossBlind>,
    joker_slot_count: usize,
    tarot_cards_used: u32,
    steel_count_in_deck: usize,
    stone_count_in_deck: usize,
    enhanced_count_in_deck: usize,
) -> ScoreResult {
    let has_four_fingers = jokers.iter().any(|j| j.kind == JokerKind::FourFingers && j.active);
    let has_shortcut    = jokers.iter().any(|j| j.kind == JokerKind::Shortcut     && j.active);
    let has_smeared     = jokers.iter().any(|j| j.kind == JokerKind::SmearedJoker && j.active);
    let has_splash      = jokers.iter().any(|j| j.kind == JokerKind::Splash       && j.active);
    let has_pareidolia  = jokers.iter().any(|j| j.kind == JokerKind::Pareidolia   && j.active);

    let eval = evaluate_hand(played_cards, has_four_fingers, has_shortcut, has_smeared, has_splash);
    let hand_type = eval.hand_type;
    let scoring_indices = eval.scoring_indices.clone();

    let level_data = hand_levels
        .get(&hand_type)
        .cloned()
        .unwrap_or_else(|| HandLevelData::new(true));

    let mut chips: f64 = level_data.chips(hand_type) as f64;
    let mut mult:  f64 = level_data.mult(hand_type)  as f64;
    if level_data.observatory_x_mult != 1.0 {
        mult *= level_data.observatory_x_mult;
    }

    let mut dollars_earned: i32 = 0;
    let mut events: Vec<ScoreEvent> = Vec::new();

    // Boss blind modifier — The Flint halves chips and mult
    if let Some(BossBlind::TheFlint) = boss_blind {
        chips = (chips / 2.0).ceil();
        mult  = (mult  / 2.0).ceil();
        events.push(ScoreEvent { source: "The Flint".to_string(), kind: ScoreEventKind::XMult, value: 0.5 });
    }

    // Pre-collect active jokers once; reused across Phase 1, 2, and 3.
    let active_jokers: Vec<&JokerInstance> = jokers.iter().filter(|j| j.active).collect();

    // ── PHASE 1: hand-level effects before card scoring ───────────────────
    for joker in &active_jokers {
        let effect = calc_joker_before(joker, hand_type);
        chips += effect.chips as f64;
        mult  += effect.mult  as f64;
        mult  *= effect.x_mult;
        dollars_earned += effect.dollars;
    }

    // ── PHASE 2: score each card in the scoring hand ──────────────────────
    for &card_idx in &scoring_indices {
        let card = &played_cards[card_idx];

        if card.debuffed {
            events.push(ScoreEvent {
                source: format!("{:?} of {:?}", card.rank, card.suit),
                kind: ScoreEventKind::Chips,
                value: 0.0,
            });
            continue;
        }

        let retriggers = count_retriggers(card_idx, card, jokers, &scoring_indices, hands_remaining);

        for _trigger in 0..=retriggers {
            let card_chips = card.base_chip_value() + card.chip_bonus();
            if card_chips != 0 {
                chips += card_chips as f64;
                events.push(ScoreEvent {
                    source: format!("{:?} of {:?}", card.rank, card.suit),
                    kind: ScoreEventKind::Chips,
                    value: card_chips as f64,
                });
            }

            let card_mult = card.flat_mult_bonus();
            if card_mult != 0 {
                mult += card_mult as f64;
                events.push(ScoreEvent {
                    source: format!("{:?} of {:?}", card.rank, card.suit),
                    kind: ScoreEventKind::Mult,
                    value: card_mult as f64,
                });
            }

            let card_xmult = card.x_mult_factor();
            if card_xmult != 1.0 {
                mult *= card_xmult;
                events.push(ScoreEvent {
                    source: format!("{:?} of {:?}", card.rank, card.suit),
                    kind: ScoreEventKind::XMult,
                    value: card_xmult,
                });
            }

            // Card edition bonuses
            let ed_chips = card.edition_chip_bonus();
            if ed_chips != 0 { chips += ed_chips as f64; }
            let ed_mult  = card.edition_mult_bonus();
            if ed_mult  != 0 { mult  += ed_mult  as f64; }
            let ed_xmult = card.edition_x_mult();
            if ed_xmult != 1.0 { mult *= ed_xmult; }

            // Gold seal
            if card.seal == Seal::Gold {
                dollars_earned += 3;
                events.push(ScoreEvent {
                    source: format!("{:?} of {:?} (Gold Seal)", card.rank, card.suit),
                    kind: ScoreEventKind::Dollars,
                    value: 3.0,
                });
            }

            // Per-card joker effects
            for joker in &active_jokers {
                let effect = calc_joker_individual(
                    joker, card_idx, card, &scoring_indices, played_cards, has_pareidolia,
                );
                chips += effect.chips as f64;
                mult  += effect.mult  as f64;
                mult  *= effect.x_mult;
                dollars_earned += effect.dollars;
                push_effect_events(&mut events, &effect, joker.kind.display_name());
            }
        }
    }

    // ── PHASE 3: held-hand cards — Steel x-mult and hand-card joker effects ──
    for card in hand_cards.iter().filter(|c| !c.debuffed) {
        let steel_xmult = card.steel_x_mult();
        if steel_xmult != 1.0 {
            let mime_count = jokers.iter().filter(|j| j.kind == JokerKind::Mime && j.active).count();
            for _ in 0..=mime_count {
                mult *= steel_xmult;
                events.push(ScoreEvent {
                    source: format!("{:?} of {:?} (Steel)", card.rank, card.suit),
                    kind: ScoreEventKind::XMult,
                    value: steel_xmult,
                });
            }
        }

        for joker in &active_jokers {
            let effect = calc_joker_hand_card(joker, card);
            mult  += effect.mult  as f64;
            mult  *= effect.x_mult;
            dollars_earned += effect.dollars;
        }
    }

    // ── PHASE 4: main joker effects (once per joker) ──────────────────────
    let ctx = ScoringContext {
        hand_type,
        scoring_cards: &scoring_indices,
        played_cards,
        hand_cards,
        jokers,
        hand_levels,
        hands_remaining,
        discards_remaining,
        money,
        deck_cards_remaining: deck_remaining,
        total_deck_size: total_deck,
        boss_blind,
        joker_count: jokers.len(),
        joker_slot_count,
        tarot_cards_used,
        steel_count_in_deck,
        stone_count_in_deck,
        enhanced_count_in_deck,
    };

    for (joker_idx, joker) in jokers.iter().enumerate() {
        if !joker.active { continue; }

        // Edition bonuses: Foil/Holographic apply BEFORE the joker's effect
        chips += joker.edition_chip_bonus() as f64;
        mult  += joker.edition_mult_bonus() as f64;

        let effect = calc_joker_main(joker, joker_idx, &ctx);
        chips += effect.chips as f64;
        mult  += effect.mult  as f64;
        mult  *= effect.x_mult;
        dollars_earned += effect.dollars;
        push_effect_events(&mut events, &effect, joker.kind.display_name());

        // Polychrome applies AFTER the joker's effect
        mult *= joker.edition_x_mult();
    }

    let final_score = chips * mult;

    ScoreResult {
        hand_type,
        hand_name: hand_type.display_name().to_string(),
        scoring_card_indices: scoring_indices,
        base_chips: level_data.chips(hand_type),
        base_mult:  level_data.mult(hand_type),
        final_chips: chips,
        final_mult:  mult,
        final_score,
        dollars_earned,
        events,
    }
}

pub(crate) mod joker_effects;
pub(crate) use joker_effects::{
    JokerEffect, calc_joker_before, calc_joker_individual,
    calc_joker_hand_card, calc_joker_main, count_retriggers,
};
