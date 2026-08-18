use crate::card::*;
use crate::types::*;
use crate::scoring::score_hand;
use crate::scoring::ScoreResult;
use crate::hand_eval::evaluate_hand;
use std::collections::HashMap;
use super::{GameState, GameStateKind, BlindKind, BalatroError, HistoryEvent, LastConsumable};

/// The result of a hand the Boss blind refused to score. The hand still counts as played and
/// still costs a hand — it simply produces nothing (state_events.lua:614).
fn zero_score(eval: &crate::hand_eval::HandEvalResult) -> ScoreResult {
    ScoreResult {
        hand_type: eval.hand_type,
        contained: eval.contained,
        hand_name: eval.hand_type.display_name().to_string(),
        scoring_card_indices: Vec::new(),
        base_chips: 0,
        base_mult: 0,
        final_chips: 0.0,
        final_mult: 0.0,
        final_score: 0.0,
        dollars_earned: 0,
        perma_chip_bonuses: Vec::new(),
        events: Vec::new(),
    }
}

impl GameState {
    /// Evaluate a set of cards with the current jokers' hand-shape modifiers applied
    /// (Four Fingers, Shortcut, Smeared Joker, Splash).
    pub(crate) fn preview_hand(&self, cards: &[CardInstance]) -> crate::hand_eval::HandEvalResult {
        let has = |k: JokerKind| self.jokers.iter().any(|j| j.kind == k && j.active);
        evaluate_hand(
            cards,
            has(JokerKind::FourFingers),
            has(JokerKind::Shortcut),
            has(JokerKind::SmearedJoker),
            has(JokerKind::Splash),
        )
    }

    /// Whether the current Boss blind's ability actually fires on this hand
    /// (`G.GAME.blind.triggered`, reset per play at state_events.lua:455). Matador pays out on it.
    ///
    /// Bosses that reject the hand outright — The Eye, The Mouth, The Psychic — also set it, but
    /// this engine surfaces those as an error from `play_hand` rather than playing a rejected
    /// hand, so they can never reach scoring.
    fn boss_ability_triggers(
        &self,
        eval: &crate::hand_eval::HandEvalResult,
        played: &[CardInstance],
    ) -> bool {
        if !matches!(self.current_blind, BlindKind::Boss) || self.boss_blind_disabled() {
            return false;
        }
        let Some(boss) = self.boss_blind else { return false };

        // A debuffed card in the scoring hand trips the blind (state_events.lua:656).
        if eval.scoring_indices.iter().any(|&i| played[i].debuffed) {
            return true;
        }

        match boss {
            // Fire on every hand played under them (blind.lua:484, :505, :513).
            BossBlind::TheHook | BossBlind::TheTooth | BossBlind::TheFlint => true,
            BossBlind::CrimsonHeart => !self.jokers.is_empty(),
            // The Arm only trips when there is a level to take away (blind.lua:551).
            BossBlind::TheArm => self
                .hand_levels
                .get(&eval.hand_type)
                .map(|h| h.level > 1)
                .unwrap_or(false),
            // The Ox trips on the most-played hand type (blind.lua:561).
            BossBlind::TheOx => {
                let this = self.hand_levels.get(&eval.hand_type).map(|h| h.played).unwrap_or(0);
                let most = self.hand_levels.values().map(|h| h.played).max().unwrap_or(0);
                most > 0 && this >= most
            }
            _ => false,
        }
    }

    /// Joker effects that Balatro evaluates under `context.before` (card.lua:3411-3570), i.e.
    /// after the hand type is known but *before* any card scores. Their upgraded values therefore
    /// count towards the hand currently being played, not the next one.
    ///
    /// `played` is the working copy that will be scored, so mutations here (Vampire eating an
    /// enhancement, Midas Mask gilding a face card) apply to this hand.
    fn pre_score_joker_updates(
        &mut self,
        played: &mut [CardInstance],
        eval: &crate::hand_eval::HandEvalResult,
    ) {
        let hand_type = eval.hand_type;
        let scoring = &eval.scoring_indices;
        let pareidolia = self.has_pareidolia();
        let oops_mult = if self.jokers.iter().any(|j| j.kind == JokerKind::OopsAll6s && j.active) {
            2.0_f64
        } else {
            1.0_f64
        };

        for i in 0..self.jokers.len() {
            if !self.jokers[i].active {
                continue;
            }
            match self.jokers[i].kind {
                JokerKind::Misprint => {
                    // Rolls a fresh 0..=23 Mult every hand (card.lua:3701). Pre-rolled here so
                    // score_hand stays deterministic given the state it is handed.
                    let roll = self.rng.range_usize("misprint", 0, 23) as i64;
                    self.jokers[i].set_counter_i64("mult", roll);
                }
                JokerKind::LoyaltyCard => {
                    // X4 Mult every 6th hand played since the joker was acquired
                    // (card.lua:3633 counts from hands_played_at_create).
                    let n = self.jokers[i].get_counter_i64("hands") + 1;
                    self.jokers[i].set_counter_i64("hands", n);
                }
                JokerKind::SquareJoker => {
                    if played.len() == 4 {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 4);
                    }
                }
                JokerKind::WeeJoker => {
                    // +8 Chips per *scoring* 2 (card.lua:3083, `context.individual`), counted
                    // once per trigger so retriggers — Hack in particular — stack. Individual
                    // effects run before joker_main, so the gain counts on this hand.
                    let twos: usize = scoring
                        .iter()
                        .filter(|&&s| played[s].rank == Rank::Two && !played[s].debuffed)
                        .map(|&s| {
                            1 + crate::scoring::count_retriggers(
                                s,
                                &played[s],
                                &self.jokers,
                                scoring,
                                self.hands_remaining.saturating_sub(1),
                                pareidolia,
                            )
                        })
                        .sum();
                    if twos > 0 {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 8 * twos as i64);
                    }
                }
                JokerKind::Runner => {
                    // `next(context.poker_hands['Straight'])` — contained, not the hand's name.
                    if eval.contained.contains(HandType::Straight) {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 15);
                    }
                }
                JokerKind::GreenJoker => {
                    let cur = self.jokers[i].get_counter_i64("mult");
                    self.jokers[i].set_counter_i64("mult", cur + 1);
                }
                JokerKind::SpareTrousers => {
                    if eval.contained.contains(HandType::TwoPair) {
                        let cur = self.jokers[i].get_counter_i64("mult");
                        self.jokers[i].set_counter_i64("mult", cur + 2);
                    }
                }
                JokerKind::RideTheBus => {
                    let has_face = scoring.iter().any(|&s| played[s].is_face(pareidolia));
                    if has_face {
                        self.jokers[i].set_counter_i64("mult", 0);
                    } else {
                        let cur = self.jokers[i].get_counter_i64("mult");
                        self.jokers[i].set_counter_i64("mult", cur + 1);
                    }
                }
                JokerKind::Obelisk => {
                    // Reset unless some *other* visible hand has been played at least as often
                    // (card.lua:3543). `played` for this hand type is incremented after scoring,
                    // so add 1 here to match Balatro, which increments up front.
                    let this_plays = self
                        .hand_levels
                        .get(&hand_type)
                        .map(|h| h.played)
                        .unwrap_or(0)
                        + 1;
                    let another_is_at_least_as_played = self
                        .hand_levels
                        .iter()
                        .any(|(ht, h)| *ht != hand_type && h.visible && h.played >= this_plays);
                    if another_is_at_least_as_played {
                        let cur = self.jokers[i].get_counter_f64("x_mult");
                        self.jokers[i].set_counter_f64("x_mult", cur + 0.2);
                    } else {
                        self.jokers[i].set_counter_f64("x_mult", 1.0);
                    }
                }
                JokerKind::Vampire => {
                    // +0.1 Xmult per scoring enhanced card, and the enhancement is eaten before
                    // the card scores — an eaten Glass card does not give its X2 this hand.
                    let victims: Vec<usize> = scoring
                        .iter()
                        .copied()
                        .filter(|&s| played[s].enhancement != Enhancement::None)
                        .collect();
                    if !victims.is_empty() {
                        let cur = self.jokers[i].get_counter_f64("x_mult");
                        self.jokers[i].set_counter_f64("x_mult", cur + 0.1 * victims.len() as f64);
                        for s in victims {
                            let id = played[s].id;
                            played[s].enhancement = Enhancement::None;
                            if let Some(c) = self.deck.iter_mut().find(|c| c.id == id) {
                                c.enhancement = Enhancement::None;
                            }
                        }
                    }
                }
                JokerKind::MidasMask => {
                    for &s in scoring {
                        if played[s].is_face(pareidolia) {
                            let id = played[s].id;
                            played[s].enhancement = Enhancement::Gold;
                            if let Some(c) = self.deck.iter_mut().find(|c| c.id == id) {
                                c.enhancement = Enhancement::Gold;
                            }
                        }
                    }
                }
                JokerKind::SpaceJoker => {
                    // 1/4 to level up the played hand — before scoring, so the new level counts.
                    if self.rng.next_bool_prob("space", (0.25 * oops_mult).min(1.0)) {
                        if let Some(level) = self.hand_levels.get_mut(&hand_type) {
                            level.level += 1;
                        }
                    }
                }
                JokerKind::Dna => {
                    // First hand of the round, exactly one card played: copy it into the deck
                    // and draw the copy.
                    let max_h = self.effective_max_hands();
                    if self.hands_remaining == max_h && played.len() == 1 {
                        let mut copy = played[0].clone();
                        copy.id = self.next_id();
                        let deck_idx = self.deck.len();
                        self.deck.push(copy);
                        self.hand.push(deck_idx);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn select_card(&mut self, hand_index: usize) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        if hand_index >= self.hand.len() {
            return Err(BalatroError::IndexOutOfRange(hand_index, self.hand.len()));
        }
        if self.selected_indices.contains(&hand_index) {
            return Ok(()); // already selected
        }
        // Psychic boss: must play exactly 5
        // Max selected is 5 cards
        if self.selected_indices.len() >= 5 {
            return Err(BalatroError::TooManySelected);
        }
        self.selected_indices.push(hand_index);
        Ok(())
    }

    pub fn deselect_card(&mut self, hand_index: usize) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        if hand_index >= self.hand.len() {
            return Err(BalatroError::IndexOutOfRange(hand_index, self.hand.len()));
        }
        // CeruleanBell: the forced card cannot be deselected
        if let Some(forced_id) = self.cerulean_forced_card_id {
            let card_deck_idx = self.hand[hand_index];
            if self.deck[card_deck_idx].id == forced_id {
                return Err(BalatroError::BossBlindEffect(
                    "Cerulean Bell: this card cannot be deselected".to_string(),
                ));
            }
        }
        self.selected_indices.retain(|&x| x != hand_index);
        Ok(())
    }

    pub fn deselect_all(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        self.selected_indices.clear();
        Ok(())
    }

    pub fn select_cards_by_rank(&mut self, rank: Rank) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        for i in 0..self.hand.len() {
            let card_idx = self.hand[i];
            if self.deck[card_idx].rank == rank && !self.selected_indices.contains(&i) {
                if self.selected_indices.len() < 5 {
                    self.selected_indices.push(i);
                }
            }
        }
        Ok(())
    }

    pub fn deselect_cards_by_suit(&mut self, suit: Suit) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        let to_deselect: Vec<usize> = self
            .selected_indices
            .iter()
            .filter(|&&i| {
                let card_idx = self.hand[i];
                self.deck[card_idx].suit == suit
            })
            .copied()
            .collect();
        for i in to_deselect {
            self.selected_indices.retain(|&x| x != i);
        }
        Ok(())
    }

    pub fn play_hand(&mut self) -> Result<ScoreResult, BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        if self.selected_indices.is_empty() {
            return Err(BalatroError::NoCardsSelected);
        }
        if self.hands_remaining == 0 {
            return Err(BalatroError::NoHandsRemaining);
        }

        // Needle boss: only 1 hand
        if let Some(BossBlind::TheNeedle) = self.boss_blind {
            if !matches!(self.current_blind, BlindKind::Boss) {
                // Only apply to boss blind
            } else if self.hands_remaining < self.effective_max_hands() {
                return Err(BalatroError::BossBlindEffect("The Needle: only 1 hand allowed".to_string()));
            }
        }

        let played_hand_indices: Vec<usize> = self
            .selected_indices
            .iter()
            .map(|&hi| self.hand[hi])
            .collect();

        let mut played_cards: Vec<CardInstance> = played_hand_indices
            .iter()
            .map(|&i| self.deck[i].clone())
            .collect();

        // TheEye / TheMouth: evaluate hand type early to enforce restrictions
        if matches!(self.current_blind, BlindKind::Boss) {
            if !self.boss_blind_disabled() {
                let has_four_fingers = self.jokers.iter().any(|j| j.kind == JokerKind::FourFingers && j.active);
                let has_shortcut = self.jokers.iter().any(|j| j.kind == JokerKind::Shortcut && j.active);
                let has_smeared = self.jokers.iter().any(|j| j.kind == JokerKind::SmearedJoker && j.active);
                let has_splash = self.jokers.iter().any(|j| j.kind == JokerKind::Splash && j.active);
                let preview = evaluate_hand(&played_cards, has_four_fingers, has_shortcut, has_smeared, has_splash);

                // TheEye: no repeat hand types this round
                if let Some(BossBlind::TheEye) = self.boss_blind {
                    let already_played = self.hand_levels
                        .get(&preview.hand_type)
                        .map(|h| h.played_this_round > 0)
                        .unwrap_or(false);
                    if already_played {
                        return Err(BalatroError::BossBlindEffect(
                            format!("The Eye: {:?} has already been played this round", preview.hand_type)
                        ));
                    }
                }

                // TheMouth: only one hand type per round
                if let Some(BossBlind::TheMouth) = self.boss_blind {
                    let other_type_played = self.hand_levels.iter()
                        .any(|(ht, h)| *ht != preview.hand_type && h.played_this_round > 0);
                    if other_type_played {
                        return Err(BalatroError::BossBlindEffect(
                            "The Mouth: only one hand type may be played per round".to_string()
                        ));
                    }
                }
            }
        }

        // The hand type is locked in before any joker gets to touch the cards
        // (get_poker_hand_info runs at the top of evaluate_play), so compute it once here and
        // hand it to the `before` pass.
        let pre_eval = self.preview_hand(&played_cards);
        let boss_ability_triggered = self.boss_ability_triggers(&pre_eval, &played_cards);

        // The Psychic rejects any hand of fewer than 5 cards (`debuff = {h_size_ge = 5}`,
        // game.lua:280). A rejected hand is not blocked — it is played and simply scores nothing,
        // burning the hand, because `evaluate_play` skips its whole scoring block when
        // `Blind:debuff_hand` returns true (state_events.lua:614).
        let hand_debuffed = matches!(self.current_blind, BlindKind::Boss)
            && matches!(self.boss_blind, Some(BossBlind::ThePsychic))
            && !self.boss_blind_disabled()
            && played_cards.len() < 5;

        if !hand_debuffed {
            self.pre_score_joker_updates(&mut played_cards, &pre_eval);
        }

        // OopsAll6s: doubles all listed probabilities when active
        let oops_active = self.jokers.iter().any(|j| j.kind == JokerKind::OopsAll6s && j.active);
        let oops_mult = if oops_active { 2.0_f64 } else { 1.0_f64 };

        // Bloodstone: pre-roll 1/2 chance x1.5 per scoring Hearts card
        let has_bloodstone = !hand_debuffed
            && self.jokers.iter().any(|j| j.kind == JokerKind::Bloodstone && j.active);
        if has_bloodstone {
            let has_four_fingers = self.jokers.iter().any(|j| j.kind == JokerKind::FourFingers && j.active);
            let has_shortcut = self.jokers.iter().any(|j| j.kind == JokerKind::Shortcut && j.active);
            let has_smeared = self.jokers.iter().any(|j| j.kind == JokerKind::SmearedJoker && j.active);
            let has_splash = self.jokers.iter().any(|j| j.kind == JokerKind::Splash && j.active);
            let pre_eval = crate::hand_eval::evaluate_hand(&played_cards, has_four_fingers, has_shortcut, has_smeared, has_splash);
            for &idx in &pre_eval.scoring_indices {
                let card = &mut played_cards[idx];
                if !card.debuffed && card.is_suit(Suit::Hearts, has_smeared) {
                    if self.rng.next_bool_prob("bloodstone", (0.5 * oops_mult).min(1.0)) {
                        card.extra_x_mult = 1.5;
                    }
                }
            }
        }

        // Lucky card: pre-roll probabilistic effects so score_hand sees them as flat bonuses.
        // +20 Mult on 1/5 (written into extra_mult so flat_mult_bonus picks it up).
        // $20 on 1/15 (counted here, paid out after scoring).
        let mut lucky_dollar_count: i32 = 0;
        for card in played_cards.iter_mut() {
            if !hand_debuffed && card.enhancement == Enhancement::Lucky && !card.debuffed {
                if self.rng.next_bool_prob("lucky_mult", (1.0 / 5.0) * oops_mult) {
                    card.extra_mult += 20;
                }
                if self.rng.next_bool_prob("lucky_money", (1.0 / 15.0) * oops_mult) {
                    lucky_dollar_count += 1;
                    // LuckyCat joker: gains +0.25 x_mult per successful Lucky trigger
                    for j in self.jokers.iter_mut() {
                        if j.kind == JokerKind::LuckyCat && j.active {
                            let cur = j.get_counter_f64("x_mult");
                            j.set_counter_f64("x_mult", cur + 0.25);
                        }
                    }
                }
            }
        }

        let hand_card_indices: Vec<usize> = self
            .hand
            .iter()
            .filter(|&&i| !played_hand_indices.contains(&i))
            .copied()
            .collect();
        let hand_cards: Vec<CardInstance> = hand_card_indices
            .iter()
            .map(|&i| self.deck[i].clone())
            .collect();

        let steel_count_in_deck = self.deck.iter()
            .filter(|c| c.enhancement == Enhancement::Steel)
            .count();
        let stone_count_in_deck = self.deck.iter()
            .filter(|c| c.is_stone())
            .count();
        let enhanced_count_in_deck = self.deck.iter()
            .filter(|c| c.enhancement != Enhancement::None)
            .count();

        // TheArm: decrease the level of the played poker hand by 1 (minimum 1) before scoring
        if let Some(BossBlind::TheArm) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                if !self.boss_blind_disabled() {
                    // Determine the hand type that will be played
                    let has_four_fingers = self.jokers.iter().any(|j| j.kind == JokerKind::FourFingers && j.active);
                    let has_shortcut = self.jokers.iter().any(|j| j.kind == JokerKind::Shortcut && j.active);
                    let has_smeared = self.jokers.iter().any(|j| j.kind == JokerKind::SmearedJoker && j.active);
                    let has_splash = self.jokers.iter().any(|j| j.kind == JokerKind::Splash && j.active);
                    let arm_preview = evaluate_hand(&played_cards, has_four_fingers, has_shortcut, has_smeared, has_splash);
                    if let Some(level) = self.hand_levels.get_mut(&arm_preview.hand_type) {
                        if level.level > 1 {
                            level.level -= 1;
                        }
                    }
                }
            }
        }

        // CrimsonHeart: disable one random active joker for the duration of this hand
        let crimson_disabled_joker_id: Option<u64> = if let Some(BossBlind::CrimsonHeart) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                if !self.boss_blind_disabled() {
                    let active_jokers: Vec<usize> = self.jokers.iter().enumerate()
                        .filter(|(_, j)| j.active)
                        .map(|(i, _)| i)
                        .collect();
                    if !active_jokers.is_empty() {
                        let pick = self.rng.range_usize("crimson_heart", 0, active_jokers.len() - 1);
                        let idx = active_jokers[pick];
                        let id = self.jokers[idx].id;
                        self.jokers[idx].active = false;
                        Some(id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let mut inputs = crate::scoring::ScoreInputs::new(
            &played_cards,
            &hand_cards,
            &self.jokers,
            &self.hand_levels,
        );
        inputs.hands_remaining = self.hands_remaining - 1;
        inputs.discards_remaining = self.discards_remaining;
        inputs.money = self.money;
        inputs.deck_cards_remaining = self.draw_pile.len();
        inputs.total_deck_size = self.deck.len();
        inputs.starting_deck_size = self.starting_deck_size;
        inputs.boss_blind = self.boss_blind;
        inputs.boss_ability_triggered = boss_ability_triggered;
        inputs.joker_slot_count = self.effective_joker_slots();
        inputs.tarot_cards_used = self.tarot_cards_used;
        inputs.steel_count_in_deck = steel_count_in_deck;
        inputs.stone_count_in_deck = stone_count_in_deck;
        inputs.enhanced_count_in_deck = enhanced_count_in_deck;
        inputs.round_targets = self.round_targets;
        // The hand was locked in before the `before` pass ran, so a joker that mutated the cards
        // (Vampire eating a Wild Card's enhancement) cannot change what is being scored.
        inputs.eval = Some(&pre_eval);

        let result = if hand_debuffed {
            zero_score(&pre_eval)
        } else {
            score_hand(inputs)
        };

        // Hiker's permanent chip bonuses, applied to the real deck cards.
        for (card_id, gain) in &result.perma_chip_bonuses {
            if let Some(deck_card) = self.deck.iter_mut().find(|c| c.id == *card_id) {
                deck_card.extra_chips += *gain;
            }
        }

        // CrimsonHeart: re-enable the temporarily disabled joker
        if let Some(disabled_id) = crimson_disabled_joker_id {
            if let Some(j) = self.jokers.iter_mut().find(|j| j.id == disabled_id) {
                j.active = true;
            }
        }

        self.last_hand_played = Some(result.hand_type);

        // Update hand level stats
        if let Some(level) = self.hand_levels.get_mut(&result.hand_type) {
            level.played += 1;
            level.played_this_round += 1;
        }

        // TheOx: playing the most-played hand type this run sets money to $0
        if let Some(BossBlind::TheOx) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                if !self.boss_blind_disabled() {
                    let max_played = self.hand_levels.values().map(|h| h.played).max().unwrap_or(0);
                    if max_played > 0 {
                        let played_hand_count = self.hand_levels
                            .get(&result.hand_type)
                            .map(|h| h.played)
                            .unwrap_or(0);
                        if played_hand_count >= max_played {
                            self.money = 0;
                        }
                    }
                }
            }
        }

        // ThePillar: record played card IDs for this Ante
        for card in &played_cards {
            if !self.played_card_ids_this_ante.contains(&card.id) {
                self.played_card_ids_this_ante.push(card.id);
            }
        }

        // Plasma deck: balance chips and mult (replace both with their average)
        let final_score = if self.deck_type == DeckType::Plasma {
            let avg = (result.final_chips + result.final_mult) / 2.0;
            avg * avg
        } else {
            result.final_score
        };

        self.score_accumulated += final_score;
        self.hands_remaining -= 1;
        self.hands_played_this_run += 1;

        // Post-scoring joker updates. A debuffed hand never reaches the joker phase at all.
        if !hand_debuffed {
            self.post_play_joker_updates(&result, &played_cards, &hand_cards);
        }

        // Vagabond: create a tarot if money <= $4 when playing a hand
        if !hand_debuffed && self.money <= 4 {
            if self.jokers.iter().any(|j| j.kind == JokerKind::Vagabond && j.active) {
                if self.consumables.len() < self.consumable_slots as usize {
                    let tarot = self.random_tarot();
                    self.consumables.push(ConsumableCard::Tarot(tarot));
                }
            }
        }

        // Earn dollars from scoring
        self.money += result.dollars_earned;
        // Lucky card $20 bonus (1/15 chance per scored Lucky card, pre-rolled above)
        self.money += lucky_dollar_count * 20;

        // BusinessCard: 1/2 chance to earn $2 per scoring face card (doubled to 1.0 with OopsAll6s)
        if !hand_debuffed && self.jokers.iter().any(|j| j.kind == JokerKind::BusinessCard && j.active) {
            let pareidolia = self.jokers.iter().any(|j| j.kind == JokerKind::Pareidolia && j.active);
            for &idx in &result.scoring_card_indices {
                let card = &played_cards[idx];
                if !card.debuffed && card.is_face(pareidolia) {
                    if self.rng.next_bool_prob("business", (0.5 * oops_mult).min(1.0)) {
                        self.money += 2;
                    }
                }
            }
        }

        // ReservedParking: 1/2 chance to earn $1 per face card held in hand (doubled to 1.0 with OopsAll6s)
        if !hand_debuffed && self.jokers.iter().any(|j| j.kind == JokerKind::ReservedParking && j.active) {
            let pareidolia = self.has_pareidolia();
            for card in &hand_cards {
                if card.is_face(pareidolia) && !card.debuffed {
                    if self.rng.next_bool_prob("parking", (0.5 * oops_mult).min(1.0)) {
                        self.money += 1;
                    }
                }
            }
        }

        // Tooth boss: -$1 per card played
        if let Some(BossBlind::TheTooth) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                self.money -= played_cards.len() as i32;
            }
        }

        // Discard played cards, draw new ones
        let played_sel = self.selected_indices.clone();
        // Remove played cards from hand (in reverse order to maintain indices)
        let mut sorted_sel = played_sel.clone();
        sorted_sel.sort_by(|a, b| b.cmp(a));
        for si in &sorted_sel {
            let card_idx = self.hand.remove(*si);
            self.deck[card_idx].face_down = false;
            self.discard_pile.push(card_idx);
        }
        self.selected_indices.clear();

        // Process glass cards: chance to destroy.
        // Only *scoring*, non-debuffed glass cards can break (state_events.lua:961).
        for &sci in &result.scoring_card_indices {
            let card = &played_cards[sci];
            if !hand_debuffed && card.enhancement == Enhancement::Glass && !card.debuffed {
                // 1/4 chance to break (1/2 with OopsAll6s)
                if self.rng.next_bool_prob("glass", (0.25 * oops_mult).min(1.0)) {
                    // Notify jokers first — Glass Joker and Canio read the card before it goes.
                    self.notify_card_destroyed(card);
                    // Remove card from deck (destroy_deck_card remaps all index collections)
                    self.destroy_deck_card(card.id);
                    self.history.push(HistoryEvent {
                        ante: self.ante,
                        round: self.round,
                        event_type: "card_destroyed".to_string(),
                        data: serde_json::json!({
                            "reason": "glass_break",
                            "card": format!("{:?} of {:?}", card.rank, card.suit)
                        }),
                    });
                }
            }
        }

        // Dusk joker: retrigger scoring cards on last hand
        let is_last_hand = self.hands_remaining == 0;
        if is_last_hand && self.jokers.iter().any(|j| j.kind == JokerKind::Dusk && j.active) {
            // Would trigger retriggers (already handled in scoring via retrigger count)
        }

        // Plasma deck: if score_accumulated >= goal, balance chips/mult
        // (already handled in scoring)

        // Log the hand play
        self.history.push(HistoryEvent {
            ante: self.ante,
            round: self.round,
            event_type: "hand_played".to_string(),
            data: serde_json::json!({
                "hand_type": result.hand_name,
                "chips": result.final_chips,
                "mult": result.final_mult,
                "score": result.final_score,
                "accumulated": self.score_accumulated,
                "goal": self.score_goal,
            }),
        });

        // TheHook: discard 2 additional random cards from remaining hand after each play
        if let Some(BossBlind::TheHook) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                if !self.boss_blind_disabled() {
                    let discard_count = 2.min(self.hand.len());
                    for _ in 0..discard_count {
                        if self.hand.is_empty() { break; }
                        let pick = self.rng.range_usize("hook", 0, self.hand.len() - 1);
                        let card_idx = self.hand.remove(pick);
                        self.deck[card_idx].face_down = false;
                        self.discard_pile.push(card_idx);
                    }
                }
            }
        }

        // Draw: TheSerpent draws exactly 3; otherwise fill to hand size
        let is_serpent = matches!(self.boss_blind, Some(BossBlind::TheSerpent))
            && matches!(self.current_blind, BlindKind::Boss)
            && !self.boss_blind_disabled();
        if is_serpent {
            let draw_count = 3.min(self.draw_pile.len());
            for _ in 0..draw_count {
                if self.draw_pile.is_empty() { break; }
                let card_idx = self.draw_pile.remove(0);
                self.hand.push(card_idx);
            }
        } else {
            self.draw_to_hand();
        }

        // Check for round win
        if self.score_accumulated >= self.score_goal {
            self.win_round();
        } else if self.hands_remaining == 0 {
            // Out of hands, didn't meet goal
            // Check Mr. Bones joker
            let mr_bones = self.jokers.iter().any(|j| j.kind == JokerKind::MrBones && !j.eternal);
            if mr_bones && self.score_accumulated >= self.score_goal / 4.0 {
                // Mr. Bones saves you (then gets destroyed)
                self.jokers.retain(|j| j.kind != JokerKind::MrBones);
                self.win_round();
            } else {
                self.state = GameStateKind::GameOver;
                self.history.push(HistoryEvent {
                    ante: self.ante,
                    round: self.round,
                    event_type: "game_over".to_string(),
                    data: serde_json::json!({
                        "score": self.score_accumulated,
                        "goal": self.score_goal,
                    }),
                });
            }
        }

        Ok(result)
    }

    fn post_play_joker_updates(&mut self, result: &ScoreResult, played: &[CardInstance], hand: &[CardInstance]) {
        // Jokers that consume themselves. They are removed outright rather than deactivated, so
        // they stop occupying a slot and stop counting towards Abstract Joker / Joker Stencil.
        let mut expired_jokers: Vec<u64> = Vec::new();
        let hand_type = result.hand_type;
        let oops_mult = if self.jokers.iter().any(|j| j.kind == JokerKind::OopsAll6s && j.active) { 2.0_f64 } else { 1.0_f64 };
        for i in 0..self.jokers.len() {
            let kind = self.jokers[i].kind;
            match kind {
                                JokerKind::IceCream => {
                    // -5 chips per hand played; melts away entirely at 0
                    let cur = self.jokers[i].get_counter_i64("chips");
                    let new = (cur - 5).max(0);
                    self.jokers[i].set_counter_i64("chips", new);
                    if new == 0 {
                        expired_jokers.push(self.jokers[i].id);
                    }
                }
                                                                                                JokerKind::Hologram => {
                    // +0.25 Xmult for each playing card added to deck
                    // (tracked when cards are added to deck)
                }
                                                JokerKind::Madness => {
                    // +0.5 Xmult when blind is entered (done at begin_round)
                }
                JokerKind::Castle => {
                    // +3 chips for each card of the current suit discarded
                    // (handled in discard)
                }
                JokerKind::FlashCard => {
                    // +2 mult per reroll (handled in shop)
                }
                JokerKind::Campfire => {
                    // +0.25 Xmult for each joker sold
                    // (handled in sell_joker)
                }
                JokerKind::EightBall => {
                    // 1/4 chance to create a tarot card when an 8 is scored (1/2 with OopsAll6s)
                    let eights_scored = result.scoring_card_indices.iter()
                        .filter(|&&idx| played[idx].rank == Rank::Eight)
                        .count();
                    for _ in 0..eights_scored {
                        if self.rng.next_bool_prob("8ball", (0.25 * oops_mult).min(1.0)) {
                            if self.consumables.len() < self.consumable_slots as usize {
                                let tarot = self.random_tarot();
                                self.consumables.push(ConsumableCard::Tarot(tarot));
                            }
                        }
                    }
                }
                JokerKind::Seltzer => {
                    // Retriggers all cards for 10 hands, then destroys itself
                    let cur = self.jokers[i].get_counter_i64("hands");
                    let new_val = cur - 1;
                    self.jokers[i].set_counter_i64("hands", new_val);
                    if new_val <= 0 {
                        expired_jokers.push(self.jokers[i].id);
                    }
                }
                                JokerKind::Seance => {
                    // Straight Flush only. A Flush Five is five of the same rank, which is not a
                    // straight, so `poker_hands['Straight Flush']` stays empty for it.
                    if result.contained.contains(HandType::StraightFlush) {
                        if self.consumables.len() < self.consumable_slots as usize {
                            let spectrals = [
                                SpectralCard::Familiar, SpectralCard::Grim, SpectralCard::Incantation,
                                SpectralCard::Aura, SpectralCard::Wraith, SpectralCard::Ectoplasm,
                                SpectralCard::Ankh, SpectralCard::DejaVu, SpectralCard::Hex,
                                SpectralCard::Medium, SpectralCard::Cryptid,
                            ];
                            let idx = self.rng.range_usize("seance", 0, spectrals.len() - 1);
                            self.consumables.push(ConsumableCard::Spectral(spectrals[idx]));
                        }
                    }
                }
                JokerKind::Superposition => {
                    // Ace + Straight → create a tarot card
                    let has_ace = result.scoring_card_indices.iter()
                        .any(|&idx| played[idx].rank == Rank::Ace);
                    if has_ace && result.contained.contains(HandType::Straight) {
                        if self.consumables.len() < self.consumable_slots as usize {
                            let tarot = self.random_tarot();
                            self.consumables.push(ConsumableCard::Tarot(tarot));
                        }
                    }
                }
                JokerKind::SixthSense => {
                    // Only fires on the *first* hand of the round (card.lua:2604). The 6 is
                    // destroyed either way; a full consumable slot only skips the spectral.
                    let is_first_hand = self.hands_remaining + 1 == self.effective_max_hands();
                    if is_first_hand && played.len() == 1 && played[0].rank == Rank::Six {
                        if self.consumables.len() < self.consumable_slots as usize {
                            let spectrals = [
                                SpectralCard::Familiar, SpectralCard::Grim, SpectralCard::Incantation,
                                SpectralCard::Talisman, SpectralCard::Aura, SpectralCard::Wraith,
                                SpectralCard::Ankh, SpectralCard::DejaVu, SpectralCard::Medium,
                            ];
                            let idx = self.rng.range_usize("sixth", 0, spectrals.len() - 1);
                            self.consumables.push(ConsumableCard::Spectral(spectrals[idx]));
                        }
                        // Take the card out of hand before destroying it — destroy_deck_card
                        // remaps stored indices and assumes the card is no longer referenced.
                        let card_id = played[0].id;
                        if let Some(hi) = self.hand.iter().position(|&di| self.deck[di].id == card_id)
                        {
                            self.hand.remove(hi);
                            self.selected_indices.retain(|&s| s != hi);
                            for s in self.selected_indices.iter_mut() {
                                if *s > hi {
                                    *s -= 1;
                                }
                            }
                        }
                        self.destroy_deck_card(card_id);
                    }
                }
                                                _ => {}
            }
        }

        if !expired_jokers.is_empty() {
            self.jokers.retain(|j| !expired_jokers.contains(&j.id));
        }
    }

    pub fn discard_hand(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        if self.selected_indices.is_empty() {
            return Err(BalatroError::NoCardsSelected);
        }
        if self.discards_remaining == 0 {
            return Err(BalatroError::NoDiscardsRemaining);
        }

        // Water boss: reduce discards (tracked separately)
        let discarded_cards: Vec<CardInstance> = self
            .selected_indices
            .iter()
            .map(|&hi| self.deck[self.hand[hi]].clone())
            .collect();

        // Sort indices in reverse to maintain validity during removal
        let mut sorted_sel = self.selected_indices.clone();
        sorted_sel.sort_by(|a, b| b.cmp(a));
        for si in &sorted_sel {
            let card_idx = self.hand.remove(*si);
            self.deck[card_idx].face_down = false;
            self.discard_pile.push(card_idx);

            // Purple Seal: create tarot when discarded
            if self.deck[card_idx].seal == Seal::Purple {
                // Add a random tarot to consumables if space
                if self.consumables.len() < self.consumable_slots as usize {
                    let tarot = self.random_tarot();
                    self.consumables.push(ConsumableCard::Tarot(tarot));
                }
            }
        }
        self.selected_indices.clear();
        self.discards_remaining -= 1;

        // FacelessJoker: $5 if 3+ face cards were discarded
        let pareidolia = self.has_pareidolia();
        if discarded_cards.iter().filter(|c| c.is_face(pareidolia)).count() >= 3 {
            let count = self.jokers.iter().filter(|j| j.kind == JokerKind::FacelessJoker && j.active).count();
            self.money += 5 * count as i32;
        }

        // MailInRebate: +$5 per discarded card matching the round's rank, which is re-rolled
        // every round (common_events.lua:2288).
        let mail_count = self
            .jokers
            .iter()
            .filter(|j| j.kind == JokerKind::MailInRebate && j.active)
            .count();
        if mail_count > 0 {
            let matching = discarded_cards
                .iter()
                .filter(|c| c.rank == self.round_targets.mail_rank)
                .count();
            self.money += 5 * matching as i32 * mail_count as i32;
        }

        // TradingCard: if first discard of the round and only 1 card, earn $3 and destroy the card
        // discards_remaining was already decremented, so first discard leaves it at max-1
        let is_first_discard = self.discards_remaining == self.effective_max_discards().saturating_sub(1);
        if is_first_discard && discarded_cards.len() == 1 {
            if self.jokers.iter().any(|j| j.kind == JokerKind::TradingCard && j.active) {
                self.money += 3;
                let card_id = discarded_cards[0].id;
                self.destroy_deck_card(card_id);
            }
        }

        // BurntJoker: on first discard of the round, upgrade the level of the discarded hand type
        if is_first_discard {
            let burnt_count = self.jokers.iter().filter(|j| j.kind == JokerKind::BurntJoker && j.active).count();
            if burnt_count > 0 {
                let has_four_fingers = self.jokers.iter().any(|j| j.kind == JokerKind::FourFingers && j.active);
                let has_shortcut = self.jokers.iter().any(|j| j.kind == JokerKind::Shortcut && j.active);
                let has_smeared = self.jokers.iter().any(|j| j.kind == JokerKind::SmearedJoker && j.active);
                let has_splash = self.jokers.iter().any(|j| j.kind == JokerKind::Splash && j.active);
                let discard_eval = evaluate_hand(&discarded_cards, has_four_fingers, has_shortcut, has_smeared, has_splash);
                if let Some(level) = self.hand_levels.get_mut(&discard_eval.hand_type) {
                    level.level += burnt_count as u32;
                }
            }
        }

        // Post-discard joker updates
        let mut eaten_jokers: Vec<u64> = Vec::new();
        for i in 0..self.jokers.len() {
            let kind = self.jokers[i].kind;
            match kind {
                JokerKind::GreenJoker => {
                    let cur = self.jokers[i].get_counter_i64("mult");
                    self.jokers[i].set_counter_i64("mult", (cur - 1).max(0));
                }
                JokerKind::Yorick => {
                    // Count individual cards discarded, not discard actions
                    let cards_this_discard = discarded_cards.len() as i64;
                    let prev = self.jokers[i].get_counter_i64("discards");
                    let new_total = prev + cards_this_discard;
                    self.jokers[i].set_counter_i64("discards", new_total);
                    // Gain +1 Xmult for every 23rd card discarded
                    let prev_milestones = prev / 23;
                    let new_milestones = new_total / 23;
                    if new_milestones > prev_milestones {
                        let gained = new_milestones - prev_milestones;
                        let cur = self.jokers[i].get_counter_f64("x_mult");
                        self.jokers[i].set_counter_f64("x_mult", cur + gained as f64);
                    }
                }
                JokerKind::Castle => {
                    // Target suit is re-rolled every round (common_events.lua:2312).
                    let target_suit = self.round_targets.castle_suit;
                    let count = discarded_cards
                        .iter()
                        .filter(|c| c.suit == target_suit)
                        .count();
                    if count > 0 {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 3 * count as i64);
                    }
                }
                JokerKind::HitTheRoad => {
                    // Gains X0.5 Mult for every Jack discarded this round
                    let jacks = discarded_cards.iter().filter(|c| c.rank == Rank::Jack).count();
                    if jacks > 0 {
                        let cur = self.jokers[i].get_counter_f64("x_mult");
                        self.jokers[i].set_counter_f64("x_mult", cur + 0.5 * jacks as f64);
                    }
                }
                JokerKind::Ramen => {
                    // Loses X0.01 per discarded card; eaten once it would drop to X1 (card.lua:2757).
                    // `context.discard` fires once per card (state_events.lua:404), so this is
                    // per-card, not per-discard-action.
                    let mut x = self.jokers[i].get_counter_f64("x_mult");
                    for _ in 0..discarded_cards.len() {
                        if x - 0.01 <= 1.0 {
                            eaten_jokers.push(self.jokers[i].id);
                            break;
                        }
                        x -= 0.01;
                    }
                    self.jokers[i].set_counter_f64("x_mult", x);
                }
                _ => {}
            }
        }

        if !eaten_jokers.is_empty() {
            self.jokers.retain(|j| !eaten_jokers.contains(&j.id));
        }

        // Blue Seal: create planet when card held in hand at round end
        // (handled at round end, not here)

        // Hook boss blind: discard 2 random cards
        // (applied before player discards normally)

        // (Green deck end-of-round money is handled in win_round)

        self.history.push(HistoryEvent {
            ante: self.ante,
            round: self.round,
            event_type: "discarded".to_string(),
            data: serde_json::json!({
                "cards": discarded_cards.iter().map(|c| format!("{:?} of {:?}", c.rank, c.suit)).collect::<Vec<_>>(),
            }),
        });

        // TheSerpent: draw exactly 3 after discard instead of filling to hand size
        let is_serpent_discard = matches!(self.boss_blind, Some(BossBlind::TheSerpent))
            && matches!(self.current_blind, BlindKind::Boss)
            && !self.boss_blind_disabled();
        if is_serpent_discard {
            let draw_count = 3.min(self.draw_pile.len());
            for _ in 0..draw_count {
                if self.draw_pile.is_empty() { break; }
                let card_idx = self.draw_pile.remove(0);
                self.hand.push(card_idx);
            }
        } else {
            self.draw_to_hand();
        }
        Ok(())
    }

    fn win_round(&mut self) {
        // Garbage Tag pays out per discard left unused across the run.
        self.unused_discards_this_run += self.discards_remaining;

        // Investment Tag: $25 once the Boss blind is beaten (tag.lua:117).
        if matches!(self.current_blind, BlindKind::Boss) {
            let investments = self.tags.iter().filter(|t| **t == TagKind::Investment).count();
            if investments > 0 {
                self.tags.retain(|t| *t != TagKind::Investment);
                self.money += 25 * investments as i32;
            }
        }

        let blind_dollars = self.blind_reward_dollars();
        self.money += blind_dollars;

        // Gold Card enhancement: $3 per Gold card held in hand at end of round
        let gold_cards_in_hand = self.hand.iter()
            .filter(|&&di| self.deck[di].enhancement == Enhancement::Gold && !self.deck[di].debuffed)
            .count();
        self.money += 3 * gold_cards_in_hand as i32;

        // GoldenJoker: +$4 at end of round
        let golden_joker_count = self.jokers.iter().filter(|j| j.kind == JokerKind::GoldenJoker && j.active).count();
        self.money += 4 * golden_joker_count as i32;

        // Rocket: earns dollars equal to its counter; +$2 per boss blind beaten
        let is_boss = matches!(self.current_blind, BlindKind::Boss);
        for i in 0..self.jokers.len() {
            if self.jokers[i].kind == JokerKind::Rocket && self.jokers[i].active {
                let earn = self.jokers[i].get_counter_i64("dollars");
                self.money += earn as i32;
                if is_boss {
                    let new_earn = earn + 2;
                    self.jokers[i].set_counter_i64("dollars", new_earn);
                }
            }
        }

        // Satellite: +$1 per unique planet type used this run (12 possible types)
        let planet_types_used = self.planet_types_used.len();
        let satellite_count = self.jokers.iter().filter(|j| j.kind == JokerKind::Satellite && j.active).count();
        self.money += planet_types_used as i32 * satellite_count as i32;

        // Cloud9: +$1 per 9 in full deck at end of round
        let nines_in_deck = self.deck.iter().filter(|c| c.rank == Rank::Nine && !c.debuffed).count();
        let cloud9_count = self.jokers.iter().filter(|j| j.kind == JokerKind::Cloud9 && j.active).count();
        self.money += nines_in_deck as i32 * cloud9_count as i32;

        // DelayedGratification: +$2 per available discard if no discards were used this round
        let max_disc = self.effective_max_discards();
        if self.discards_remaining == max_disc {
            let dg_count = self.jokers.iter().filter(|j| j.kind == JokerKind::DelayedGratification && j.active).count();
            self.money += max_disc as i32 * 2 * dg_count as i32;
        }

        // OopsAll6s doubles all listed probabilities at end of round too
        let win_oops_active = self.jokers.iter().any(|j| j.kind == JokerKind::OopsAll6s && j.active);
        let win_oops_mult = if win_oops_active { 2.0_f64 } else { 1.0_f64 };

        // GrosMichel: 1/6 chance to be destroyed at end of round (1/3 with OopsAll6s)
        let gm_positions: Vec<usize> = self.jokers.iter().enumerate()
            .filter(|(_, j)| j.kind == JokerKind::GrosMichel && j.active && !j.eternal)
            .map(|(i, _)| i)
            .collect();
        for pos in gm_positions.iter().rev() {
            if self.rng.next_bool_prob("gros_michel", (1.0 / 6.0) * win_oops_mult) {
                self.jokers.remove(*pos);
                // Extinction is permanent: Gros Michel leaves the pool, Cavendish joins it.
                self.gros_michel_extinct = true;
            }
        }

        // Cavendish: 1/1000 chance to be destroyed at end of round (1/500 with OopsAll6s)
        let cav_positions: Vec<usize> = self.jokers.iter().enumerate()
            .filter(|(_, j)| j.kind == JokerKind::Cavendish && j.active && !j.eternal)
            .map(|(i, _)| i)
            .collect();
        for pos in cav_positions.iter().rev() {
            if self.rng.next_bool_prob("cavendish", (1.0 / 1000.0) * win_oops_mult) {
                self.jokers.remove(*pos);
            }
        }

        // Popcorn: -4 mult per round (not per hand); destroyed when mult reaches 0
        let mut eaten: Vec<u64> = Vec::new();
        for i in 0..self.jokers.len() {
            if self.jokers[i].kind == JokerKind::Popcorn && self.jokers[i].active {
                let cur = self.jokers[i].get_counter_i64("mult");
                let new = (cur - 4).max(0);
                self.jokers[i].set_counter_i64("mult", new);
                if new == 0 {
                    eaten.push(self.jokers[i].id);
                }
            }
        }
        if !eaten.is_empty() {
            self.jokers.retain(|j| !eaten.contains(&j.id));
        }

        // InvisibleJoker: increment round counter each round (duplication happens on sell, not here)
        for i in 0..self.jokers.len() {
            if self.jokers[i].kind == JokerKind::InvisibleJoker && self.jokers[i].active {
                let rounds = self.jokers[i].get_counter_i64("rounds") + 1;
                self.jokers[i].set_counter_i64("rounds", rounds);
            }
        }

        // ToTheMoon raises the interest *amount* paid per $5, not the cap (card.lua:614 bumps
        // G.GAME.interest_amount). Payout is amount × min(money/5, cap/5) — state_events.lua:1202.
        let to_the_moon_count = self.jokers.iter().filter(|j| j.kind == JokerKind::ToTheMoon && j.active).count();

        if self.deck_type == DeckType::Green {
            // Green deck: $2 per remaining hand, $1 per remaining discard, and no interest
            // (`extra_hand_bonus = 2, extra_discard_bonus = 1, no_interest = true`, game.lua:631)
            self.money += 2 * self.hands_remaining as i32;
            self.money += self.discards_remaining as i32;
        } else {
            let interest_amount = 1 + to_the_moon_count as i32;
            let interest_steps = (self.money / 5).min(self.max_interest / 5).max(0);
            self.money += interest_amount * interest_steps;
        }

        // Perishable jokers: decrement rounds remaining; disable when expired
        for j in self.jokers.iter_mut() {
            if j.perishable && j.active {
                if j.perishable_rounds_left > 0 {
                    j.perishable_rounds_left -= 1;
                }
                if j.perishable_rounds_left == 0 {
                    j.active = false;
                }
            }
        }

        // Blue Seal: each sealed card held at round end creates the Planet for the hand you
        // played most recently (card.lua:1046), not a random one.
        if let Some(last_hand) = self.last_hand_played {
            if let Some(planet) = planet_for_hand(last_hand) {
                let sealed = self
                    .hand
                    .iter()
                    .filter(|&&di| self.deck[di].seal == Seal::Blue && !self.deck[di].debuffed)
                    .count();
                for _ in 0..sealed {
                    if self.consumables.len() >= self.consumable_slots as usize {
                        break;
                    }
                    self.consumables.push(ConsumableCard::Planet(planet));
                }
            }
        }

        // Log victory
        self.history.push(HistoryEvent {
            ante: self.ante,
            round: self.round,
            event_type: "round_won".to_string(),
            data: serde_json::json!({
                "score": self.score_accumulated,
                "goal": self.score_goal,
                "dollars_earned": blind_dollars,
            }),
        });

        // Mark blind as defeated
        match self.current_blind {
            BlindKind::Small => self.blind_defeated_this_ante[0] = true,
            BlindKind::Big => self.blind_defeated_this_ante[1] = true,
            BlindKind::Boss => self.blind_defeated_this_ante[2] = true,
        }

        // Campfire: reset x_mult to X1 when Boss Blind is defeated
        if matches!(self.current_blind, BlindKind::Boss) {
            for j in self.jokers.iter_mut() {
                if j.kind == JokerKind::Campfire && j.active {
                    j.set_counter_f64("x_mult", 1.0);
                }
            }
        }

        // Anaglyph deck: gain a Double Tag (DoubleFun) after defeating each Boss Blind
        if matches!(self.current_blind, BlindKind::Boss) && self.deck_type == DeckType::Anaglyph {
            self.gain_tag(TagKind::DoubleFun);
        }

        // Check if ante 8 boss beaten = game won
        if self.ante >= self.win_ante() && matches!(self.current_blind, BlindKind::Boss) {
            self.history.push(HistoryEvent {
                ante: self.ante,
                round: self.round,
                event_type: "game_won".to_string(),
                data: serde_json::json!({}),
            });
            self.state = GameStateKind::GameOver;
            return;
        }

        // If boss blind won, advance to next ante
        if matches!(self.current_blind, BlindKind::Boss) {
            // Advance to shop before going to next ante
            self.state = GameStateKind::Shop;
            self.generate_shop();
        } else {
            // Small/Big blind won → go to shop
            self.state = GameStateKind::Shop;
            self.generate_shop();
        }
    }

    fn blind_reward_dollars(&self) -> i32 {
        match (self.current_blind.clone(), self.boss_blind) {
            // Red stake and above: Small Blind gives no cash reward
            (BlindKind::Small, _) => {
                if self.stake as u8 >= Stake::Red as u8 { 0 } else { 3 }
            }
            (BlindKind::Big, _) => 4,
            (BlindKind::Boss, Some(b)) => {
                // boss blinds give 5$ (showdowns give 8$)
                match b {
                    BossBlind::CeruleanBell
                    | BossBlind::VerdantLeaf
                    | BossBlind::VioletVessel
                    | BossBlind::AmberAcorn
                    | BossBlind::CrimsonHeart => 8,
                    _ => 5,
                }
            }
            _ => 5,
        }
    }

    pub(crate) fn advance_blind(&mut self) {
        match self.current_blind {
            BlindKind::Small => {
                self.current_blind = BlindKind::Big;
                self.round = 2;
            }
            BlindKind::Big => {
                self.current_blind = BlindKind::Boss;
                self.round = 3;
            }
            BlindKind::Boss => {
                self.juggle_hand_size = 0;
                self.ante += 1;
                self.round = 1;
                self.current_blind = BlindKind::Small;
                self.blind_defeated_this_ante = [false; 3];
                self.boss_blind = self.pick_boss_blind();
                self.boss_rerolled_this_ante = false;
                self.played_card_ids_this_ante.clear();
            }
        }
    }
}
