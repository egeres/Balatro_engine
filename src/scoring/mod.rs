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

/// Randomised targets that Balatro stores on `G.GAME.current_round` and re-rolls at the start of
/// every round (`state_events.lua:273-276`). They are global, not per-joker: two copies of The
/// Idol chase the same card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundTargets {
    /// The Idol: X2 Mult for each scoring card of this exact rank *and* suit.
    pub idol_rank: Rank,
    pub idol_suit: Suit,
    /// Ancient Joker: X1.5 Mult per scoring card of this suit.
    pub ancient_suit: Suit,
    /// Castle: +3 Chips per discarded card of this suit.
    pub castle_suit: Suit,
    /// Mail-In Rebate: $5 per discarded card of this rank.
    pub mail_rank: Rank,
}

impl Default for RoundTargets {
    /// Balatro's pre-roll defaults (`common_events.lua:2272`, `:2289`, `game.lua:1949-1952`).
    fn default() -> Self {
        Self {
            idol_rank: Rank::Ace,
            idol_suit: Suit::Spades,
            ancient_suit: Suit::Spades,
            castle_suit: Suit::Spades,
            mail_rank: Rank::Ace,
        }
    }
}

/// Context passed to the joker evaluators in the main joker phase
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
    /// Size of the deck at the start of the run (G.GAME.starting_deck_size). Deck-dependent:
    /// 52 for most decks, 40 for Abandoned. Used by Erosion.
    pub starting_deck_size: usize,
    pub boss_blind: Option<BossBlind>,
    pub joker_count: usize,
    pub joker_slot_count: usize,
    pub tarot_cards_used: u32,
    pub steel_count_in_deck: usize,
    pub stone_count_in_deck: usize,
    pub enhanced_count_in_deck: usize,
    pub round_targets: RoundTargets,
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
    starting_deck_size: usize,
    boss_blind: Option<BossBlind>,
    joker_slot_count: usize,
    tarot_cards_used: u32,
    steel_count_in_deck: usize,
    stone_count_in_deck: usize,
    enhanced_count_in_deck: usize,
    round_targets: RoundTargets,
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

    // Pre-collect active jokers once; reused across the card phases below.
    let active_jokers: Vec<&JokerInstance> = jokers.iter().filter(|j| j.active).collect();

    // ── PHASE 1: score each card in the scoring hand ──────────────────────
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
            for (j_idx, joker) in jokers.iter().enumerate().filter(|(_, j)| j.active) {
                let effect = calc_joker_individual(
                    joker, j_idx, jokers, card_idx, card, &scoring_indices, played_cards,
                    has_pareidolia, round_targets,
                );
                chips += effect.chips as f64;
                mult  += effect.mult  as f64;
                mult  *= effect.x_mult;
                dollars_earned += effect.dollars;
                push_effect_events(&mut events, &effect, joker.kind.display_name());
            }
        }
    }

    // ── PHASE 2: held-hand cards — Steel x-mult and hand-card joker effects ──
    // Red Seal and Mime add repetitions that re-run *everything* a held card does, the card's own
    // Steel x-mult and every joker's held-in-hand effect alike (state_events.lua:789-830).
    for card in hand_cards.iter().filter(|c| !c.debuffed) {
        let steel_xmult = card.steel_x_mult();
        let joker_effects: Vec<(usize, JokerEffect)> = jokers
            .iter()
            .enumerate()
            .filter(|(_, j)| j.active)
            .map(|(j_idx, joker)| (j_idx, calc_joker_hand_card(joker, j_idx, jokers, card)))
            .filter(|(_, e)| e.mult != 0 || e.x_mult != 1.0 || e.dollars != 0)
            .collect();

        // Balatro only grants the repetition when the card actually did something.
        let did_something = steel_xmult != 1.0 || !joker_effects.is_empty();
        let repetitions = if did_something {
            1 + count_hand_retriggers(card, jokers)
        } else {
            1
        };

        for _ in 0..repetitions {
            if steel_xmult != 1.0 {
                mult *= steel_xmult;
                events.push(ScoreEvent {
                    source: format!("{:?} of {:?} (Steel)", card.rank, card.suit),
                    kind: ScoreEventKind::XMult,
                    value: steel_xmult,
                });
            }
            for (j_idx, _) in &joker_effects {
                let effect = calc_joker_hand_card(&jokers[*j_idx], *j_idx, jokers, card);
                mult += effect.mult as f64;
                mult *= effect.x_mult;
                dollars_earned += effect.dollars;
                push_effect_events(&mut events, &effect, jokers[*j_idx].kind.display_name());
            }
        }
    }

    // ── PHASE 3: main joker effects, in joker order (once per joker) ──────
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
        starting_deck_size,
        boss_blind,
        joker_count: jokers.len(),
        joker_slot_count,
        tarot_cards_used,
        steel_count_in_deck,
        stone_count_in_deck,
        enhanced_count_in_deck,
        round_targets,
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
    JokerEffect, calc_joker_individual, calc_joker_hand_card, calc_joker_main,
    count_hand_retriggers, count_retriggers,
};
