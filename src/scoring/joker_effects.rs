use crate::card::{CardInstance, JokerInstance};
use crate::types::*;
use super::{RoundTargets, ScoringContext};

pub struct JokerEffect {
    pub chips: i64,
    pub mult: i64,
    pub x_mult: f64,
    pub dollars: i32,
}

impl JokerEffect {
    pub(crate) fn new() -> Self {
        Self { chips: 0, mult: 0, x_mult: 1.0, dollars: 0 }
    }
}

// ---------------------------------------------------------------------------
// Blueprint / Brainstorm target resolution
// ---------------------------------------------------------------------------

/// Resolve which joker a copy joker imitates, following chains (card.lua:2304-2334).
///
/// Blueprint copies the joker immediately to its right; Brainstorm copies the leftmost joker,
/// whatever that is. Either can point at another copy joker, so the chain is followed until it
/// reaches a real joker — with a visit set, because Blueprint-next-to-Brainstorm forms a loop.
/// Balatro bounds the same loop with `context.blueprint > #G.jokers.cards + 1`, which likewise
/// ends in no effect.
///
/// Note this deliberately does *not* consult `blueprint_compat`: that flag only drives the
/// "Compatible/Incompatible" UI label (card.lua:4234). At runtime Blueprint calls straight into
/// the target's `calculate_joker`, and it is the target's own `not context.blueprint` guards that
/// stop state-mutating effects from being duplicated.
///
/// Returns `None` when the chain dead-ends, loops, or reaches an inactive joker.
pub(crate) fn copy_target(jokers: &[JokerInstance], idx: usize) -> Option<usize> {
    let mut seen = [false; 64];
    let mut cur = idx;
    loop {
        if cur < seen.len() {
            if seen[cur] {
                return None;
            }
            seen[cur] = true;
        }
        let next = match jokers.get(cur)?.kind {
            JokerKind::Blueprint => cur + 1,
            JokerKind::Brainstorm => 0,
            _ => return None,
        };
        let target = jokers.get(next)?;
        if next == cur {
            return None;
        }
        if matches!(target.kind, JokerKind::Blueprint | JokerKind::Brainstorm) {
            cur = next;
            continue;
        }
        if !target.active {
            return None;
        }
        return Some(next);
    }
}

/// True if this joker is a copy joker (Blueprint / Brainstorm).
fn is_copy_joker(kind: JokerKind) -> bool {
    matches!(kind, JokerKind::Blueprint | JokerKind::Brainstorm)
}

// ---------------------------------------------------------------------------
// Retrigger counting
// ---------------------------------------------------------------------------

pub(crate) fn count_retriggers(
    card_idx: usize,
    card: &CardInstance,
    jokers: &[JokerInstance],
    scoring_indices: &[usize],
    hands_remaining: u32,
) -> usize {
    let mut retriggers = 0usize;

    if card.seal == Seal::Red {
        retriggers += 1;
    }

    for (j_idx, joker) in jokers.iter().enumerate().filter(|(_, j)| j.active) {
        // A copy joker retriggers exactly like whatever it is imitating, reading that joker's
        // counters too (Seltzer's remaining hands, for instance).
        let joker = if is_copy_joker(joker.kind) {
            match copy_target(jokers, j_idx) {
                Some(t) => &jokers[t],
                None => continue,
            }
        } else {
            joker
        };
        match joker.kind {
            JokerKind::Dusk if hands_remaining == 0 => {
                retriggers += 1;
            }
            JokerKind::Hack => {
                if matches!(card.rank, Rank::Two | Rank::Three | Rank::Four | Rank::Five) {
                    retriggers += 1;
                }
            }
            JokerKind::SockAndBuskin => {
                if card.rank.is_face() {
                    retriggers += 1;
                }
            }
            JokerKind::HangingChad => {
                if scoring_indices.first() == Some(&card_idx) {
                    retriggers += 2;
                }
            }
            JokerKind::Seltzer => {
                if joker.get_counter_i64("hands") > 0 {
                    retriggers += 1;
                }
            }
            _ => {}
        }
    }

    retriggers
}

/// Retriggers for a card *held in hand* (state_events.lua:809-830).
///
/// Red Seal and Mime both add a repetition, and each repetition re-runs the card's own effect
/// (Steel) *and* every joker's held-in-hand effect (Baron, Shoot the Moon, Raised Fist).
/// Balatro only grants these when the card produced an effect in the first place, which the
/// caller decides.
pub(crate) fn count_hand_retriggers(card: &CardInstance, jokers: &[JokerInstance]) -> usize {
    let mut retriggers = 0usize;
    if card.seal == Seal::Red {
        retriggers += 1;
    }
    for (j_idx, joker) in jokers.iter().enumerate().filter(|(_, j)| j.active) {
        let kind = if is_copy_joker(joker.kind) {
            match copy_target(jokers, j_idx) {
                Some(t) => jokers[t].kind,
                None => continue,
            }
        } else {
            joker.kind
        };
        if kind == JokerKind::Mime {
            retriggers += 1;
        }
    }
    retriggers
}

// ---------------------------------------------------------------------------
// Phase 2 — per scoring-card effects
// ---------------------------------------------------------------------------

pub(crate) fn calc_joker_individual(
    joker: &JokerInstance,
    joker_idx: usize,
    all_jokers: &[JokerInstance],
    card_idx: usize,
    card: &CardInstance,
    scoring_indices: &[usize],
    played_cards: &[CardInstance],
    pareidolia: bool,
    targets: RoundTargets,
) -> JokerEffect {
    // Blueprint / Brainstorm copy every context, not just joker_main, so a copy joker sitting
    // next to Greedy Joker or Photograph fires here too.
    if is_copy_joker(joker.kind) {
        return match copy_target(all_jokers, joker_idx) {
            Some(t) => calc_joker_individual(
                &all_jokers[t], t, all_jokers, card_idx, card, scoring_indices, played_cards,
                pareidolia, targets,
            ),
            None => JokerEffect::new(),
        };
    }

    let mut effect = JokerEffect::new();
    let is_scoring = scoring_indices.contains(&card_idx);

    match joker.kind {
        JokerKind::GreedyJoker => {
            if is_scoring && card.effective_suits().contains(&Suit::Diamonds) {
                effect.mult += 3;
            }
        }
        JokerKind::LustyJoker => {
            if is_scoring && card.effective_suits().contains(&Suit::Hearts) {
                effect.mult += 3;
            }
        }
        JokerKind::WrathfulJoker => {
            if is_scoring && card.effective_suits().contains(&Suit::Spades) {
                effect.mult += 3;
            }
        }
        JokerKind::GluttonousJoker => {
            if is_scoring && card.effective_suits().contains(&Suit::Clubs) {
                effect.mult += 3;
            }
        }
        JokerKind::ScaryFace => {
            if is_scoring && card.is_face(pareidolia) {
                effect.chips += 30;
            }
        }
        JokerKind::Fibonacci => {
            if is_scoring && card.rank.is_fibonacci() {
                effect.mult += 8;
            }
        }
        JokerKind::EvenSteven => {
            if is_scoring && card.rank.is_even() {
                effect.mult += 4;
            }
        }
        JokerKind::OddTodd => {
            if is_scoring && card.rank.is_odd() {
                effect.chips += 31;
            }
        }
        JokerKind::Scholar => {
            if is_scoring && card.rank == Rank::Ace {
                effect.chips += 20;
                effect.mult += 4;
            }
        }
        JokerKind::Photograph => {
            if is_scoring && card.is_face(pareidolia) {
                let first_face = scoring_indices
                    .iter()
                    .find(|&&i| played_cards[i].is_face(pareidolia));
                if first_face == Some(&card_idx) {
                    effect.x_mult = 2.0;
                }
            }
        }
        JokerKind::WalkieTalkie => {
            if is_scoring && (card.rank == Rank::Ten || card.rank == Rank::Four) {
                effect.chips += 10;
                effect.mult += 4;
            }
        }
        JokerKind::SmileyFace => {
            if is_scoring && card.is_face(pareidolia) {
                effect.mult += 5;
            }
        }
        JokerKind::Hiker => {
            if is_scoring {
                effect.chips += 5;
            }
        }
        JokerKind::Arrowhead => {
            if is_scoring && card.effective_suits().contains(&Suit::Spades) {
                effect.chips += 50;
            }
        }
        JokerKind::OnyxAgate => {
            if is_scoring && card.effective_suits().contains(&Suit::Clubs) {
                effect.mult += 7;
            }
        }
        JokerKind::Bloodstone => {
            // Pre-rolled in round.rs: extra_x_mult set to 1.5 on 1/2 chance
            if is_scoring && card.effective_suits().contains(&Suit::Hearts) && card.extra_x_mult > 1.0 {
                effect.x_mult = card.extra_x_mult;
            }
        }
        JokerKind::RoughGem => {
            if is_scoring && card.effective_suits().contains(&Suit::Diamonds) {
                effect.dollars += 1;
            }
        }
        JokerKind::Triboulet => {
            if is_scoring && (card.rank == Rank::King || card.rank == Rank::Queen) {
                effect.x_mult = 2.0;
            }
        }
        JokerKind::TheIdol => {
            // Target is the round-wide idol card, re-rolled each round (card.lua:3127).
            if is_scoring
                && card.rank == targets.idol_rank
                && card.effective_suits().contains(&targets.idol_suit)
            {
                effect.x_mult = 2.0;
            }
        }
        _ => {}
    }
    effect
}

// ---------------------------------------------------------------------------
// Phase 3 — effects for cards held in hand (not played)
// ---------------------------------------------------------------------------

pub(crate) fn calc_joker_hand_card(
    joker: &JokerInstance,
    joker_idx: usize,
    all_jokers: &[JokerInstance],
    card: &CardInstance,
    hand_cards: &[CardInstance],
) -> JokerEffect {
    if is_copy_joker(joker.kind) {
        return match copy_target(all_jokers, joker_idx) {
            Some(t) => calc_joker_hand_card(&all_jokers[t], t, all_jokers, card, hand_cards),
            None => JokerEffect::new(),
        };
    }

    let mut effect = JokerEffect::new();
    match joker.kind {
        JokerKind::Baron => {
            if card.rank == Rank::King && !card.debuffed {
                effect.x_mult = 1.5;
            }
        }
        JokerKind::ShootTheMoon => {
            if card.rank == Rank::Queen && !card.debuffed {
                effect.mult += 13;
            }
        }
        JokerKind::RaisedFist => {
            // Doubles the *nominal* value of the lowest-ranked card held in hand
            // (card.lua:3320). Balatro scans with `temp_ID >= base.id`, so ties go to the
            // rightmost card, and Stone cards are skipped outright.
            if let Some(lowest) = lowest_held_card(hand_cards) {
                if lowest.id == card.id && !card.debuffed {
                    effect.mult += 2 * card.rank.base_chips();
                }
            }
        }
        _ => {}
    }
    effect
}

/// The card Raised Fist keys off: lowest rank held in hand, ties to the rightmost, Stone cards
/// excluded. Debuffed cards still take part in the selection — Balatro then yields no Mult,
/// which falls out of the held-card phase skipping debuffed cards.
fn lowest_held_card(hand_cards: &[CardInstance]) -> Option<&CardInstance> {
    let mut chosen: Option<&CardInstance> = None;
    for c in hand_cards.iter().filter(|c| !c.is_stone()) {
        match chosen {
            Some(cur) if cur.rank.numeric_value() < c.rank.numeric_value() => {}
            _ => chosen = Some(c),
        }
    }
    chosen
}

// ---------------------------------------------------------------------------
// Phase 4 — main joker effects (once per hand)
// ---------------------------------------------------------------------------

pub(crate) fn calc_joker_main(
    joker: &JokerInstance,
    joker_idx: usize,
    ctx: &ScoringContext,
) -> JokerEffect {
    // Blueprint copies the joker to its right, Brainstorm the leftmost one, following chains
    // (card.lua:4225). Resolving the target here means every phase copies identically.
    if is_copy_joker(joker.kind) {
        return match copy_target(ctx.jokers, joker_idx) {
            Some(t) => calc_joker_main(&ctx.jokers[t], t, ctx),
            None => JokerEffect::new(),
        };
    }

    let mut effect = JokerEffect::new();
    let hand_type = ctx.hand_type;
    let scoring_cards = ctx.scoring_cards;
    let played = ctx.played_cards;
    let hand = ctx.hand_cards;

    match joker.kind {
        // ── Hand-type effects (t_mult / t_chips, card.lua:3660) ───────────
        // These live in `joker_main` like every other joker effect, so they land *after* card
        // scoring. Applying them earlier would let a card's X-mult multiply them.
        JokerKind::JollyJoker   if hand_type.contains_pair()            => { effect.mult  +=   8; }
        JokerKind::ZanyJoker    if hand_type.contains_three_of_a_kind() => { effect.mult  +=  12; }
        JokerKind::MadJoker     if hand_type.contains_two_pair()        => { effect.mult  +=  10; }
        JokerKind::CrazyJoker   if hand_type.contains_straight()        => { effect.mult  +=  12; }
        JokerKind::DrollJoker   if hand_type.contains_flush()           => { effect.mult  +=  10; }
        JokerKind::SlyJoker     if hand_type.contains_pair()            => { effect.chips +=  50; }
        JokerKind::WilyJoker    if hand_type.contains_three_of_a_kind() => { effect.chips += 100; }
        JokerKind::CleverJoker  if hand_type.contains_two_pair()        => { effect.chips +=  80; }
        JokerKind::DeviousJoker if hand_type.contains_straight()        => { effect.chips += 100; }
        JokerKind::CraftyJoker  if hand_type.contains_flush()           => { effect.chips +=  80; }

        // ── Fixed numeric effects ─────────────────────────────────────────
        JokerKind::Joker      => { effect.mult  +=  4; }
        JokerKind::GrosMichel => { effect.mult  += 15; }
        JokerKind::Stuntman   => { effect.chips += 250; }

        // ── Counter-based: mult ───────────────────────────────────────────
        JokerKind::RideTheBus
        | JokerKind::GreenJoker
        | JokerKind::SpareTrousers
        | JokerKind::FlashCard
        | JokerKind::CeremonialDagger
        | JokerKind::Popcorn
        | JokerKind::RedCard => {
            effect.mult += joker.get_counter_i64("mult");
        }

        // Derived from the current board rather than a stored counter (card.lua:4240 recomputes
        // ability.mult on every update): Mult equal to the summed sell value of every other joker.
        JokerKind::Swashbuckler => {
            effect.mult += ctx
                .jokers
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx != joker_idx)
                .map(|(_, j)| j.sell_value() as i64)
                .sum::<i64>();
        }

        // ── Counter-based: chips ──────────────────────────────────────────
        JokerKind::Runner
        | JokerKind::IceCream
        | JokerKind::SquareJoker
        | JokerKind::WeeJoker
        | JokerKind::Castle => {
            effect.chips += joker.get_counter_i64("chips");
        }

        // ── Counter-based: x_mult ─────────────────────────────────────────
        JokerKind::Hologram
        | JokerKind::Vampire
        | JokerKind::Obelisk
        | JokerKind::LuckyCat
        | JokerKind::Constellation
        | JokerKind::GlassJoker
        | JokerKind::Ramen
        | JokerKind::HitTheRoad
        | JokerKind::Madness
        | JokerKind::Campfire
        | JokerKind::Yorick
        | JokerKind::Canio => {
            let x = joker.get_counter_f64("x_mult");
            if x > 1.0 { effect.x_mult = x; }
        }

        // ── Deck / context scaling ────────────────────────────────────────
        JokerKind::JokerStencil => {
            let empty = ctx.joker_slot_count.saturating_sub(ctx.joker_count);
            effect.x_mult = 1.0 + empty as f64;
        }
        JokerKind::AbstractJoker => {
            effect.mult += ctx.joker_count as i64 * 3;
        }
        JokerKind::BlueJoker => {
            effect.chips += ctx.deck_cards_remaining as i64 * 2;
        }
        JokerKind::Erosion => {
            let below = (ctx.starting_deck_size as i64 - ctx.total_deck_size as i64).max(0);
            effect.mult += below * 4;
        }
        JokerKind::StoneJoker => {
            effect.chips += ctx.stone_count_in_deck as i64 * 25;
        }
        JokerKind::SteelJoker => {
            let n = ctx.steel_count_in_deck;
            if n > 0 { effect.x_mult = 1.0 + 0.2 * n as f64; }
        }
        JokerKind::BaseballCard => {
            let n = ctx.jokers.iter().filter(|j| j.active && j.kind.rarity() == 2).count();
            if n > 0 { effect.x_mult = 1.5_f64.powi(n as i32); }
        }

        // ── Hand-condition effects ────────────────────────────────────────
        JokerKind::HalfJoker => {
            if played.len() <= 3 { effect.mult += 20; }
        }
        JokerKind::Cavendish => { effect.x_mult = 3.0; }
        JokerKind::TheDuo    if hand_type.contains_pair()           => { effect.x_mult = 2.0; }
        JokerKind::TheTrio   if hand_type.contains_three_of_a_kind() => { effect.x_mult = 3.0; }
        JokerKind::TheFamily if hand_type.contains_four_of_a_kind() => { effect.x_mult = 4.0; }
        JokerKind::TheOrder  if hand_type.contains_straight()       => { effect.x_mult = 3.0; }
        JokerKind::TheTribe  if hand_type.contains_flush()          => { effect.x_mult = 2.0; }
        JokerKind::Acrobat => {
            if ctx.hands_remaining == 0 { effect.x_mult = 3.0; }
        }
        JokerKind::DriversLicense => {
            if ctx.enhanced_count_in_deck >= 16 { effect.x_mult = 3.0; }
        }
        JokerKind::CardSharp => {
            let played_this_round = ctx.hand_levels
                .get(&hand_type)
                .map(|h| h.played_this_round)
                .unwrap_or(0);
            if played_this_round > 0 { effect.x_mult = 3.0; }
        }
        JokerKind::LoyaltyCard => {
            let total: u32 = ctx.hand_levels.values().map(|h| h.played).sum();
            if total > 0 && (total % 6) == 5 { effect.x_mult = 4.0; }
        }

        // ── Suit / card-set conditions ────────────────────────────────────
        JokerKind::Blackboard => {
            let all_dark = hand.iter().all(|c| {
                c.effective_suits().iter().any(|s| matches!(s, Suit::Spades | Suit::Clubs))
            });
            if all_dark { effect.x_mult = 3.0; }
        }
        JokerKind::SeeingDouble => {
            let has_club = scoring_cards.iter()
                .any(|&i| played[i].effective_suits().contains(&Suit::Clubs));
            let has_non_club = scoring_cards.iter()
                .any(|&i| played[i].effective_suits().iter().any(|s| *s != Suit::Clubs));
            if has_club && has_non_club { effect.x_mult = 2.0; }
        }
        JokerKind::FlowerPot => {
            let suits_present: std::collections::HashSet<Suit> = scoring_cards
                .iter()
                .flat_map(|&i| played[i].effective_suits())
                .collect();
            if suits_present.len() == 4 { effect.x_mult = 3.0; }
        }
        JokerKind::AncientJoker => {
            // Target suit is re-rolled every round (card.lua:3255).
            let target = ctx.round_targets.ancient_suit;
            for &idx in scoring_cards {
                if played[idx].effective_suits().contains(&target) {
                    effect.x_mult *= 1.5;
                }
            }
        }

        // ── Game-state scaling ────────────────────────────────────────────
        JokerKind::Banner => {
            effect.chips += ctx.discards_remaining as i64 * 30;
        }
        JokerKind::MysticSummit => {
            if ctx.discards_remaining == 0 { effect.mult += 15; }
        }
        JokerKind::Supernova => {
            let plays = ctx.hand_levels.get(&hand_type).map(|h| h.played).unwrap_or(0);
            effect.mult += plays as i64;
        }
        JokerKind::Misprint => {
            effect.mult += 11; // simplified: average of the 0–23 range
        }
        JokerKind::Bootstraps => {
            effect.mult += (ctx.money / 5).max(0) as i64 * 2;
        }
        JokerKind::Bull => {
            effect.chips += ctx.money.max(0) as i64 * 2;
        }
        JokerKind::FortuneTeller => {
            effect.mult += ctx.tarot_cards_used as i64;
        }
        JokerKind::Throwback => {
            let skips = joker.get_counter_i64("skips");
            if skips > 0 { effect.x_mult = 1.0 + 0.25 * skips as f64; }
        }
        JokerKind::GoldenTicket => {
            let gold = scoring_cards.iter()
                .filter(|&&i| played[i].enhancement == Enhancement::Gold)
                .count();
            effect.dollars += gold as i32 * 4;
        }
        JokerKind::ToDoList => {
            let target_str = joker.counters.get("hand_type").and_then(|v| v.as_str()).unwrap_or("HighCard");
            let target = match target_str {
                "Pair"          => HandType::Pair,
                "TwoPair"       => HandType::TwoPair,
                "ThreeOfAKind"  => HandType::ThreeOfAKind,
                "Straight"      => HandType::Straight,
                "Flush"         => HandType::Flush,
                "FullHouse"     => HandType::FullHouse,
                "FourOfAKind"   => HandType::FourOfAKind,
                "StraightFlush" => HandType::StraightFlush,
                "FiveOfAKind"   => HandType::FiveOfAKind,
                "FlushHouse"    => HandType::FlushHouse,
                "FlushFive"     => HandType::FlushFive,
                _               => HandType::HighCard,
            };
            if hand_type == target { effect.dollars += 4; }
        }

        // ── Copy effects ──────────────────────────────────────────────────
        // Handled up front via copy_target(); nothing to do here.
        JokerKind::Blueprint | JokerKind::Brainstorm => {}

        _ => {}
    }

    effect
}
