use crate::card::{CardInstance, HandLevelData, JokerInstance};
use crate::hand_eval::{evaluate_hand, HandEvalResult};
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
    /// Permanent chip bonuses Hiker wrote onto scoring cards, as `(card id, chips)`.
    /// The caller persists these onto the deck.
    pub perma_chip_bonuses: Vec<(u64, i64)>,
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
    /// Whether this Boss blind's ability actually did something to this hand
    /// (`G.GAME.blind.triggered`). Matador pays out on it.
    pub boss_ability_triggered: bool,
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

/// Everything `score_hand` needs. Built with [`ScoreInputs::new`] and then adjusted field by
/// field, so adding a new input does not churn every call site.
pub struct ScoreInputs<'a> {
    pub played_cards: &'a [CardInstance],
    pub hand_cards: &'a [CardInstance],
    pub jokers: &'a [JokerInstance],
    pub hand_levels: &'a HashMap<HandType, HandLevelData>,

    /// Hands left *after* this one — Acrobat and Dusk key off it being 0.
    pub hands_remaining: u32,
    pub discards_remaining: u32,
    pub money: i32,
    pub deck_cards_remaining: usize,
    pub total_deck_size: usize,
    pub starting_deck_size: usize,
    pub boss_blind: Option<BossBlind>,
    pub boss_ability_triggered: bool,
    pub joker_slot_count: usize,
    pub tarot_cards_used: u32,
    pub steel_count_in_deck: usize,
    pub stone_count_in_deck: usize,
    pub enhanced_count_in_deck: usize,
    pub round_targets: RoundTargets,

    /// The hand type and scoring cards, decided *before* any joker touched the cards.
    ///
    /// Balatro calls `get_poker_hand_info` at the top of `evaluate_play` (state_events.lua:572),
    /// so the hand is locked in before the `before` phase runs. That matters when a joker mutates
    /// the played cards: Vampire eating a Wild Card's enhancement must not retroactively break a
    /// flush. Leave it `None` to evaluate from `played_cards`.
    pub eval: Option<&'a HandEvalResult>,
}

impl<'a> ScoreInputs<'a> {
    pub fn new(
        played_cards: &'a [CardInstance],
        hand_cards: &'a [CardInstance],
        jokers: &'a [JokerInstance],
        hand_levels: &'a HashMap<HandType, HandLevelData>,
    ) -> Self {
        Self {
            played_cards,
            hand_cards,
            jokers,
            hand_levels,
            hands_remaining: 0,
            discards_remaining: 0,
            money: 0,
            deck_cards_remaining: 0,
            total_deck_size: 52,
            starting_deck_size: 52,
            boss_blind: None,
            boss_ability_triggered: false,
            joker_slot_count: 5,
            tarot_cards_used: 0,
            steel_count_in_deck: 0,
            stone_count_in_deck: 0,
            enhanced_count_in_deck: 0,
            round_targets: RoundTargets::default(),
            eval: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Main scoring entry-point
// ---------------------------------------------------------------------------

pub fn score_hand(inputs: ScoreInputs) -> ScoreResult {
    let ScoreInputs {
        played_cards,
        hand_cards,
        jokers,
        hand_levels,
        hands_remaining,
        discards_remaining,
        money,
        deck_cards_remaining,
        total_deck_size,
        starting_deck_size,
        boss_blind,
        boss_ability_triggered,
        joker_slot_count,
        tarot_cards_used,
        steel_count_in_deck,
        stone_count_in_deck,
        enhanced_count_in_deck,
        round_targets,
        eval,
    } = inputs;

    let has_pareidolia = jokers.iter().any(|j| j.kind == JokerKind::Pareidolia && j.active);

    // Use the caller's locked-in hand where it gave one; otherwise work it out here.
    let owned_eval;
    let eval: &HandEvalResult = match eval {
        Some(e) => e,
        None => {
            let has = |k: JokerKind| jokers.iter().any(|j| j.kind == k && j.active);
            owned_eval = evaluate_hand(
                played_cards,
                has(JokerKind::FourFingers),
                has(JokerKind::Shortcut),
                has(JokerKind::SmearedJoker),
                has(JokerKind::Splash),
            );
            &owned_eval
        }
    };
    let hand_type = eval.hand_type;
    let scoring_indices = eval.scoring_indices.clone();

    // Hiker writes a permanent bonus onto the cards it scores, and a retriggered card sees that
    // bonus on its later triggers (card.lua:3067). Work on a local copy so the growth is visible
    // mid-hand, and report the totals back so the caller can persist them onto the deck.
    let mut played: Vec<CardInstance> = played_cards.to_vec();
    let hiker_count = count_effective_jokers(jokers, JokerKind::Hiker) as i64;
    let mut perma_chip_bonuses: Vec<(u64, i64)> = Vec::new();

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

    // ── PHASE 1: score each card in the scoring hand ──────────────────────
    for &card_idx in &scoring_indices {
        if played[card_idx].debuffed {
            let card = &played[card_idx];
            events.push(ScoreEvent {
                source: format!("{:?} of {:?}", card.rank, card.suit),
                kind: ScoreEventKind::Chips,
                value: 0.0,
            });
            continue;
        }

        let retriggers = count_retriggers(
            card_idx, &played[card_idx], jokers, &scoring_indices, hands_remaining,
        );

        for _trigger in 0..=retriggers {
            let card = played[card_idx].clone();

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
                    joker, j_idx, jokers, card_idx, &card, &scoring_indices, &played,
                    has_pareidolia, round_targets,
                );
                chips += effect.chips as f64;
                mult  += effect.mult  as f64;
                mult  *= effect.x_mult;
                dollars_earned += effect.dollars;
                push_effect_events(&mut events, &effect, joker.kind.display_name());
            }

            // Hiker bumps the card's permanent chips *after* it scored this trigger, so the
            // next trigger of the same card scores the boosted value.
            if hiker_count > 0 {
                let gain = 5 * hiker_count;
                played[card_idx].extra_chips += gain;
                let id = played[card_idx].id;
                match perma_chip_bonuses.iter_mut().find(|(cid, _)| *cid == id) {
                    Some((_, total)) => *total += gain,
                    None => perma_chip_bonuses.push((id, gain)),
                }
                events.push(ScoreEvent {
                    source: JokerKind::Hiker.display_name().to_string(),
                    kind: ScoreEventKind::Chips,
                    value: 0.0,
                });
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
            .map(|(j_idx, joker)| (j_idx, calc_joker_hand_card(joker, j_idx, jokers, card, hand_cards)))
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
                let effect = calc_joker_hand_card(&jokers[*j_idx], *j_idx, jokers, card, hand_cards);
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
        played_cards: &played,
        hand_cards,
        jokers,
        hand_levels,
        hands_remaining,
        discards_remaining,
        money,
        deck_cards_remaining,
        total_deck_size,
        starting_deck_size,
        boss_blind,
        boss_ability_triggered,
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
        perma_chip_bonuses,
        events,
    }
}

pub(crate) mod joker_effects;
pub(crate) use joker_effects::{
    JokerEffect, calc_joker_individual, calc_joker_hand_card, calc_joker_main,
    count_effective_jokers, count_hand_retriggers, count_retriggers,
};
