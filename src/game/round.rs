use crate::card::*;
use crate::types::*;
use crate::scoring::score_hand;
use crate::scoring::ScoreResult;
use super::{GameState, GameStateKind, BlindKind, BalatroError};

/// Seance's pool: every spectral except the ones that need a target card, plus Hex.
const SEANCE_SPECTRALS: [SpectralCard; 11] = [
    SpectralCard::Familiar, SpectralCard::Grim, SpectralCard::Incantation, SpectralCard::Aura,
    SpectralCard::Wraith, SpectralCard::Ectoplasm, SpectralCard::Ankh, SpectralCard::DejaVu,
    SpectralCard::Hex, SpectralCard::Medium, SpectralCard::Cryptid,
];

/// Sixth Sense's pool, which trades Ectoplasm, Hex and Cryptid for Talisman.
const SIXTH_SENSE_SPECTRALS: [SpectralCard; 9] = [
    SpectralCard::Familiar, SpectralCard::Grim, SpectralCard::Incantation, SpectralCard::Talisman,
    SpectralCard::Aura, SpectralCard::Wraith, SpectralCard::Ankh, SpectralCard::DejaVu,
    SpectralCard::Medium,
];

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
        crate::hand_eval::evaluate_hand(
            cards,
            self.has_joker(JokerKind::FourFingers),
            self.has_joker(JokerKind::Shortcut),
            self.has_joker(JokerKind::SmearedJoker),
            self.has_joker(JokerKind::Splash),
        )
    }

    /// Whether the current Boss blind's ability actually fires on this hand
    /// (`G.GAME.blind.triggered`, reset per play at state_events.lua:455). Matador pays out on it.
    ///
    /// The bosses that refuse a hand outright — The Eye, The Mouth, The Psychic — set it too;
    /// see [`GameState::blind_debuffs_hand`].
    fn boss_ability_triggers(
        &self,
        eval: &crate::hand_eval::HandEvalResult,
        played: &[CardInstance],
    ) -> bool {
        let Some(boss) = self.active_boss() else { return false };

        // A debuffed card in the scoring hand trips the blind (state_events.lua:656).
        if eval.scoring_indices.iter().any(|&i| played[i].debuffed) {
            return true;
        }

        // A refused hand trips the blind (blind.lua:527, :541, :546).
        if self.blind_debuffs_hand(eval, played.len()) {
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
            // The Ox trips on the hand it named when the round began (blind.lua:562).
            BossBlind::TheOx => self.ox_target_hand == Some(eval.hand_type),
            _ => false,
        }
    }

    /// Whether the Boss blind refuses to score this hand (`Blind:debuff_hand`, blind.lua:519).
    ///
    /// A refused hand is not blocked: it is played, it counts as played, it costs a hand, and it
    /// scores nothing, because `evaluate_play` skips its entire scoring block — jokers included —
    /// when this returns true (state_events.lua:614).
    fn blind_debuffs_hand(
        &self,
        eval: &crate::hand_eval::HandEvalResult,
        played_count: usize,
    ) -> bool {
        match self.active_boss() {
            // `debuff = {h_size_ge = 5}` (game.lua:280): fewer than five cards is refused.
            Some(BossBlind::ThePsychic) => played_count < 5,
            // No hand type twice in a round. The counter already counts this hand, so two means
            // it has come up before.
            Some(BossBlind::TheEye) => self
                .hand_levels
                .get(&eval.hand_type)
                .map(|h| h.played_this_round > 1)
                .unwrap_or(false),
            // Only one hand type all round.
            Some(BossBlind::TheMouth) => self
                .hand_levels
                .iter()
                .any(|(ht, h)| *ht != eval.hand_type && h.played_this_round > 0),
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
                    // Hands played since the joker was acquired, counting this one
                    // (card.lua:3633 counts from `hands_played_at_create`). X4 Mult on every 6th.
                    self.jokers[i].add_counter_i64("hands", 1);
                }
                JokerKind::SquareJoker => {
                    if played.len() == 4 {
                        self.jokers[i].add_counter_i64("chips", 4);
                    }
                }
                JokerKind::WeeJoker => {
                    // +8 Chips per *scoring* 2 (card.lua:3083, `context.individual`), counted
                    // once per trigger so retriggers — Hack in particular — stack. Individual
                    // effects run before joker_main, so the gain counts on this hand.
                    let twos: usize = scoring
                        .iter()
                        .filter(|&&s| played[s].has_rank(Rank::Two) && !played[s].debuffed)
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
                        self.jokers[i].add_counter_i64("chips", 8 * twos as i64);
                    }
                }
                JokerKind::Runner => {
                    // `next(context.poker_hands['Straight'])` — contained, not the hand's name.
                    if eval.contained.contains(HandType::Straight) {
                        self.jokers[i].add_counter_i64("chips", 15);
                    }
                }
                JokerKind::GreenJoker => {
                    self.jokers[i].add_counter_i64("mult", 1);
                }
                JokerKind::SpareTrousers => {
                    if eval.contained.contains(HandType::TwoPair) {
                        self.jokers[i].add_counter_i64("mult", 2);
                    }
                }
                JokerKind::RideTheBus => {
                    // A scoring face card resets the run of consecutive faceless hands.
                    if scoring.iter().any(|&s| played[s].is_face(pareidolia)) {
                        self.jokers[i].set_counter_i64("mult", 0);
                    } else {
                        self.jokers[i].add_counter_i64("mult", 1);
                    }
                }
                JokerKind::Obelisk => {
                    // Reset unless some *other* visible hand has been played at least as often
                    // (card.lua:3543). `played` already counts this hand — it was bumped before
                    // the `before` pass, as Balatro does.
                    let this_plays = self
                        .hand_levels
                        .get(&hand_type)
                        .map(|h| h.played)
                        .unwrap_or(0);
                    let another_is_at_least_as_played = self
                        .hand_levels
                        .iter()
                        .any(|(ht, h)| *ht != hand_type && h.visible && h.played >= this_plays);
                    if another_is_at_least_as_played {
                        self.jokers[i].add_counter_f64("x_mult", 0.2);
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
                        self.jokers[i].add_counter_f64("x_mult", 0.1 * victims.len() as f64);
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
                    if self.roll_chance("space", 0.25) {
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
                        self.notify_playing_cards_added(1);
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
        // `has_rank` rather than a bare comparison: a Stone card shows no rank to select by.
        for i in 0..self.hand.len() {
            let card_idx = self.hand[i];
            if self.deck[card_idx].has_rank(rank)
                && !self.selected_indices.contains(&i)
                && self.selected_indices.len() < 5
            {
                self.selected_indices.push(i);
            }
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

        let played_hand_indices: Vec<usize> = self
            .selected_indices
            .iter()
            .map(|&hi| self.hand[hi])
            .collect();

        let mut played_cards: Vec<CardInstance> = played_hand_indices
            .iter()
            .map(|&i| self.deck[i].clone())
            .collect();

        // The hand type is locked in before any joker gets to touch the cards
        // (get_poker_hand_info runs at the top of evaluate_play), so compute it once here and
        // hand it to the `before` pass.
        let pre_eval = self.preview_hand(&played_cards);

        // Balatro books the hand as played at the very top of `evaluate_play`
        // (state_events.lua:574-578) — before the blind may refuse it, before the `before` joker
        // pass, before anything scores. Everything downstream therefore reads a counter that
        // already includes this hand: Supernova pays out on it, Card Sharp asks for *two*, and
        // the three secret hands become visible the moment you first land one.
        if let Some(level) = self.hand_levels.get_mut(&pre_eval.hand_type) {
            level.played += 1;
            level.played_this_round += 1;
            level.visible = true;
        }

        let boss_ability_triggered = self.boss_ability_triggers(&pre_eval, &played_cards);

        let hand_debuffed = self.blind_debuffs_hand(&pre_eval, played_cards.len());

        if !hand_debuffed {
            self.pre_score_joker_updates(&mut played_cards, &pre_eval);
        }

        // Bloodstone: pre-roll 1/2 chance x1.5 per scoring Hearts card. Re-evaluated rather than
        // reusing `pre_eval` because the `before` pass may have changed which cards score.
        if !hand_debuffed && self.has_joker(JokerKind::Bloodstone) {
            let smeared = self.has_smeared();
            for idx in self.preview_hand(&played_cards).scoring_indices {
                let card = &played_cards[idx];
                if !card.debuffed
                    && card.is_suit(Suit::Hearts, smeared)
                    && self.roll_chance("bloodstone", 0.5)
                {
                    played_cards[idx].extra_x_mult = 1.5;
                }
            }
        }

        // Lucky card: pre-roll probabilistic effects so score_hand sees them as flat bonuses.
        // +20 Mult on 1/5 (written into extra_mult so flat_mult_bonus picks it up).
        // $20 on 1/15 (counted here, paid out after scoring).
        let mut lucky_dollar_count: i32 = 0;
        let lucky_cards: Vec<usize> = match hand_debuffed {
            true => Vec::new(),
            false => played_cards
                .iter()
                .enumerate()
                .filter(|(_, c)| c.enhancement == Enhancement::Lucky && !c.debuffed)
                .map(|(idx, _)| idx)
                .collect(),
        };
        for idx in lucky_cards {
            if self.roll_chance("lucky_mult", 1.0 / 5.0) {
                played_cards[idx].extra_mult += 20;
            }
            if self.roll_chance("lucky_money", 1.0 / 15.0) {
                lucky_dollar_count += 1;
                // LuckyCat joker: gains +0.25 x_mult per successful Lucky trigger
                for j in self.jokers.iter_mut() {
                    if j.kind == JokerKind::LuckyCat && j.active {
                        j.add_counter_f64("x_mult", 0.25);
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
        if self.boss_effect_active(BossBlind::TheArm) {
            let arm_hand = self.preview_hand(&played_cards).hand_type;
            if let Some(level) = self.hand_levels.get_mut(&arm_hand) {
                level.level = level.level.saturating_sub(1).max(1);
            }
        }

        // CrimsonHeart: disable one random active joker for the duration of this hand
        let mut crimson_disabled_joker_id: Option<u64> = None;
        if self.boss_effect_active(BossBlind::CrimsonHeart) {
            let active_jokers: Vec<usize> = self
                .jokers
                .iter()
                .enumerate()
                .filter(|(_, j)| j.active)
                .map(|(i, _)| i)
                .collect();
            if !active_jokers.is_empty() {
                let pick = self.rng.range_usize("crimson_heart", 0, active_jokers.len() - 1);
                let idx = active_jokers[pick];
                self.jokers[idx].active = false;
                crimson_disabled_joker_id = Some(self.jokers[idx].id);
            }
        }

        // Observatory pays out on the Planet cards still sitting in the consumable slots.
        let observatory_planets: Vec<HandType> = match self.has_voucher(VoucherKind::Observatory) {
            true => self
                .consumables
                .iter()
                .filter_map(|c| match c.card {
                    ConsumableCard::Planet(p) => Some(p.hand_type()),
                    _ => None,
                })
                .collect(),
            false => Vec::new(),
        };

        let mut inputs = crate::scoring::ScoreInputs::new(
            &played_cards,
            &hand_cards,
            &self.jokers,
            &self.hand_levels,
        );
        inputs.observatory_planets = &observatory_planets;
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
        inputs.discount_percent = self.discount_percent();
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

        // TheOx: playing the hand it named at the start of the round sets money to $0.
        if self.boss_effect_active(BossBlind::TheOx)
            && self.ox_target_hand == Some(result.hand_type)
        {
            self.money = 0;
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
            self.post_play_joker_updates(&result, &played_cards);
        }

        // Vagabond: create a tarot if money <= $4 when playing a hand
        if !hand_debuffed
            && self.money <= 4
            && self.has_joker(JokerKind::Vagabond)
            && self.has_room_for_consumable()
        {
            let tarot = self.random_tarot();
            self.add_consumable(ConsumableCard::Tarot(tarot));
        }

        // Earn dollars from scoring
        self.money += result.dollars_earned;
        // Lucky card $20 bonus (1/15 chance per scored Lucky card, pre-rolled above)
        self.money += lucky_dollar_count * 20;

        let pareidolia = self.has_pareidolia();

        // BusinessCard: 1/2 chance to earn $2 per scoring face card
        if !hand_debuffed && self.has_joker(JokerKind::BusinessCard) {
            for &idx in &result.scoring_card_indices {
                let card = &played_cards[idx];
                if !card.debuffed && card.is_face(pareidolia) && self.roll_chance("business", 0.5) {
                    self.money += 2;
                }
            }
        }

        // ReservedParking: 1/2 chance to earn $1 per face card held in hand
        if !hand_debuffed && self.has_joker(JokerKind::ReservedParking) {
            for card in &hand_cards {
                if !card.debuffed && card.is_face(pareidolia) && self.roll_chance("parking", 0.5) {
                    self.money += 1;
                }
            }
        }

        // Tooth boss: -$1 per card played
        if self.boss_effect_active(BossBlind::TheTooth) {
            self.money -= played_cards.len() as i32;
        }

        self.discard_selected_cards();

        // Process glass cards: chance to destroy.
        // Only *scoring*, non-debuffed glass cards can break (state_events.lua:961).
        for &sci in &result.scoring_card_indices {
            let card = played_cards[sci].clone();
            if hand_debuffed || card.enhancement != Enhancement::Glass || card.debuffed {
                continue;
            }
            if self.roll_chance("glass", 0.25) {
                // Notify jokers first — Glass Joker and Canio read the card before it goes.
                self.notify_card_destroyed(&card);
                // Remove card from deck (destroy_deck_card remaps all index collections)
                self.destroy_deck_card(card.id);
                self.log_event(
                    "card_destroyed",
                    serde_json::json!({
                        "reason": "glass_break",
                        "card": format!("{:?} of {:?}", card.rank, card.suit)
                    }),
                );
            }
        }

        self.log_event(
            "hand_played",
            serde_json::json!({
                "hand_type": result.hand_name,
                "chips": result.final_chips,
                "mult": result.final_mult,
                "score": result.final_score,
                "accumulated": self.score_accumulated,
                "goal": self.score_goal,
            }),
        );

        // TheHook: discard 2 additional random cards from remaining hand after each play
        if self.boss_effect_active(BossBlind::TheHook) {
            for _ in 0..2.min(self.hand.len()) {
                let pick = self.rng.range_usize("hook", 0, self.hand.len() - 1);
                let card_idx = self.hand.remove(pick);
                self.deck[card_idx].face_down = false;
                self.discard_pile.push(card_idx);
            }
        }

        // The Fish hides the cards drawn immediately after a play, and only those
        // (`self.prepped`, blind.lua:487, cleared once the draw is done).
        self.fish_prepped = true;
        self.refill_hand();

        // Check for round win
        if self.score_accumulated >= self.score_goal {
            self.win_round();
        } else if self.hands_remaining == 0 {
            // Out of hands and short of the goal. Mr. Bones saves a run that got at least a
            // quarter of the way there, and destroys *itself* doing so — a second copy is still
            // on the board for the next brush with death.
            let mr_bones = self
                .jokers
                .iter()
                .position(|j| j.kind == JokerKind::MrBones && !j.eternal);
            match mr_bones.filter(|_| self.score_accumulated >= self.score_goal / 4.0) {
                Some(pos) => {
                    self.jokers.remove(pos);
                    self.win_round();
                }
                None => {
                    self.state = GameStateKind::GameOver;
                    self.log_event(
                        "game_over",
                        serde_json::json!({
                            "score": self.score_accumulated,
                            "goal": self.score_goal,
                        }),
                    );
                }
            }
        }

        Ok(result)
    }

    /// Move the selected cards out of hand and onto the discard pile, and clear the selection.
    ///
    /// Walked highest-index-first so the removals do not invalidate the ones still to come.
    /// Returns the deck indices in that same order — the caller may still owe those cards
    /// something, as a discard does for a Purple Seal.
    fn discard_selected_cards(&mut self) -> Vec<usize> {
        let mut selected = self.selected_indices.clone();
        selected.sort_unstable_by(|a, b| b.cmp(a));
        self.selected_indices.clear();

        selected
            .into_iter()
            .map(|hand_idx| {
                let card_idx = self.hand.remove(hand_idx);
                self.deck[card_idx].face_down = false;
                self.discard_pile.push(card_idx);
                card_idx
            })
            .collect()
    }

    /// Draw back up after a hand is played or discarded.
    ///
    /// The Serpent is the exception: it always deals exactly 3, however much room there is
    /// (blind.lua:596), which is what makes it hurt.
    fn refill_hand(&mut self) {
        if !self.boss_effect_active(BossBlind::TheSerpent) {
            self.draw_to_hand();
            return;
        }
        for _ in 0..3.min(self.draw_pile.len()) {
            let card_idx = self.draw_pile.remove(0);
            self.hand.push(card_idx);
        }
    }

    fn post_play_joker_updates(&mut self, result: &ScoreResult, played: &[CardInstance]) {
        // Jokers that consume themselves. They are removed outright rather than deactivated, so
        // they stop occupying a slot and stop counting towards Abstract Joker / Joker Stencil.
        let mut expired_jokers: Vec<u64> = Vec::new();

        for i in 0..self.jokers.len() {
            match self.jokers[i].kind {
                JokerKind::IceCream => {
                    // -5 chips per hand played; melts away entirely at 0
                    let left = (self.jokers[i].get_counter_i64("chips") - 5).max(0);
                    self.jokers[i].set_counter_i64("chips", left);
                    if left == 0 {
                        expired_jokers.push(self.jokers[i].id);
                    }
                }
                JokerKind::Seltzer => {
                    // Retriggers all cards for 10 hands, then destroys itself
                    self.jokers[i].add_counter_i64("hands", -1);
                    if self.jokers[i].get_counter_i64("hands") <= 0 {
                        expired_jokers.push(self.jokers[i].id);
                    }
                }
                JokerKind::EightBall => {
                    // 1/4 chance of a Tarot per scoring 8.
                    let eights = result
                        .scoring_card_indices
                        .iter()
                        .filter(|&&idx| played[idx].has_rank(Rank::Eight))
                        .count();
                    for _ in 0..eights {
                        if self.roll_chance("8ball", 0.25) {
                            self.create_tarot();
                        }
                    }
                }
                JokerKind::Seance => {
                    // Straight Flush only. A Flush Five is five of the same rank, which is not a
                    // straight, so `poker_hands['Straight Flush']` stays empty for it.
                    if result.contained.contains(HandType::StraightFlush) {
                        self.create_spectral_from("seance", &SEANCE_SPECTRALS);
                    }
                }
                JokerKind::Superposition => {
                    // Ace + Straight → create a tarot card
                    let has_ace = result
                        .scoring_card_indices
                        .iter()
                        .any(|&idx| played[idx].has_rank(Rank::Ace));
                    if has_ace && result.contained.contains(HandType::Straight) {
                        self.create_tarot();
                    }
                }
                JokerKind::SixthSense => {
                    // Only fires on the *first* hand of the round (card.lua:2604). The 6 is
                    // destroyed either way; a full consumable slot only skips the spectral.
                    let is_first_hand = self.hands_remaining + 1 == self.effective_max_hands();
                    if is_first_hand && played.len() == 1 && played[0].has_rank(Rank::Six) {
                        self.create_spectral_from("sixth", &SIXTH_SENSE_SPECTRALS);
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
                // Hologram, Madness, Castle, Flash Card and Campfire also scale, but on events
                // that happen elsewhere — a card added to the deck, a blind selected, a discard,
                // a reroll, a joker sold. Each is handled where its event lives.
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

        let discarded_cards: Vec<CardInstance> = self
            .selected_indices
            .iter()
            .map(|&hi| self.deck[self.hand[hi]].clone())
            .collect();

        for card_idx in self.discard_selected_cards() {
            // Purple Seal: create a tarot when discarded
            if self.deck[card_idx].seal == Seal::Purple {
                self.create_tarot();
            }
        }
        self.discards_remaining -= 1;

        // FacelessJoker: $5 if 3+ face cards were discarded
        let pareidolia = self.has_pareidolia();
        if discarded_cards.iter().filter(|c| c.is_face(pareidolia)).count() >= 3 {
            self.money += 5 * self.count_joker(JokerKind::FacelessJoker) as i32;
        }

        // MailInRebate: +$5 per discarded card matching the round's rank, which is re-rolled
        // every round (common_events.lua:2288).
        let mail_count = self.count_joker(JokerKind::MailInRebate);
        if mail_count > 0 {
            let matching = discarded_cards
                .iter()
                .filter(|c| c.has_rank(self.round_targets.mail_rank))
                .count();
            self.money += 5 * matching as i32 * mail_count as i32;
        }

        // `discards_remaining` was already decremented, so the first discard leaves it at max-1.
        let is_first_discard =
            self.discards_remaining == self.effective_max_discards().saturating_sub(1);

        // TradingCard: the first discard of the round, if it is a single card, pays $3 and
        // destroys that card.
        if is_first_discard
            && discarded_cards.len() == 1
            && self.has_joker(JokerKind::TradingCard)
        {
            self.money += 3;
            self.destroy_deck_card(discarded_cards[0].id);
        }

        // BurntJoker: on first discard of the round, upgrade the level of the discarded hand type
        let burnt_count = self.count_joker(JokerKind::BurntJoker);
        if is_first_discard && burnt_count > 0 {
            let discarded_hand = self.preview_hand(&discarded_cards).hand_type;
            if let Some(level) = self.hand_levels.get_mut(&discarded_hand) {
                level.level += burnt_count as u32;
            }
        }

        // Post-discard joker updates
        let smeared = self.has_smeared();
        let mut eaten_jokers: Vec<u64> = Vec::new();
        for i in 0..self.jokers.len() {
            let kind = self.jokers[i].kind;
            match kind {
                JokerKind::GreenJoker => {
                    let cur = self.jokers[i].get_counter_i64("mult");
                    self.jokers[i].set_counter_i64("mult", (cur - 1).max(0));
                }
                JokerKind::Yorick => {
                    // Counts individual cards discarded, not discard actions, and pays +1 Xmult
                    // at every 23rd one.
                    let prev = self.jokers[i].get_counter_i64("discards");
                    let total = prev + discarded_cards.len() as i64;
                    self.jokers[i].set_counter_i64("discards", total);
                    let milestones = total / 23 - prev / 23;
                    if milestones > 0 {
                        self.jokers[i].add_counter_f64("x_mult", milestones as f64);
                    }
                }
                JokerKind::Castle => {
                    // Target suit is re-rolled every round (common_events.lua:2312).
                    let target_suit = self.round_targets.castle_suit;
                    let count = discarded_cards.iter().filter(|c| c.is_suit(target_suit, smeared)).count();
                    if count > 0 {
                        self.jokers[i].add_counter_i64("chips", 3 * count as i64);
                    }
                }
                JokerKind::HitTheRoad => {
                    // Gains X0.5 Mult for every Jack discarded this round
                    // `not context.other_card.debuff` (card.lua:2834): a debuffed Jack feeds it nothing.
                    let jacks = discarded_cards
                        .iter()
                        .filter(|c| c.has_rank(Rank::Jack) && !c.debuffed)
                        .count();
                    if jacks > 0 {
                        self.jokers[i].add_counter_f64("x_mult", 0.5 * jacks as f64);
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

        self.log_event(
            "discarded",
            serde_json::json!({
                "cards": discarded_cards.iter().map(|c| format!("{:?} of {:?}", c.rank, c.suit)).collect::<Vec<_>>(),
            }),
        );

        self.refill_hand();
        Ok(())
    }

    fn win_round(&mut self) {
        // `evaluate_round` lays out every payout row in one synchronous pass and only lets the
        // dollars land afterwards, so the interest row is worked out from the balance the round
        // *ended* on — before the blind reward, the unused hands, the jokers and the tags have
        // paid anything (state_events.lua:1191). Snapshot it, or interest earns on its own payout.
        let money_at_round_end = self.money;

        // Beating the blind turns Amber Acorn's jokers back over (`Blind:defeat`, blind.lua:338).
        for j in self.jokers.iter_mut() {
            j.face_down = false;
        }

        // Garbage Tag pays out per discard left unused across the run.
        self.unused_discards_this_run += self.discards_remaining;
        let is_boss = matches!(self.current_blind, BlindKind::Boss);

        // Investment Tag: $25 once the Boss blind is beaten (tag.lua:117).
        if is_boss {
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
        self.money += 4 * self.count_joker(JokerKind::GoldenJoker) as i32;

        // Egg and Gift Card gain sell value at the *end of the round* (card.lua:2985, :2993).
        // Doing it when the shop is stocked let a player farm them by rerolling.
        for i in self.joker_indices(JokerKind::Egg) {
            self.jokers[i].add_counter_i64("sell_bonus", 3);
        }
        if self.has_joker(JokerKind::GiftCard) {
            for j in self.jokers.iter_mut().filter(|j| j.kind != JokerKind::GiftCard) {
                j.add_counter_i64("sell_bonus", 1);
            }
        }

        // Rocket: earns dollars equal to its counter; +$2 per boss blind beaten
        for i in self.joker_indices(JokerKind::Rocket) {
            self.money += self.jokers[i].get_counter_i64("dollars") as i32;
            if is_boss {
                self.jokers[i].add_counter_i64("dollars", 2);
            }
        }

        // Satellite: +$1 per unique planet type used this run
        self.money +=
            (self.planet_types_used.len() * self.count_joker(JokerKind::Satellite)) as i32;

        // Cloud9: +$1 per 9 in full deck at end of round
        let nines_in_deck = self.deck.iter().filter(|c| c.has_rank(Rank::Nine) && !c.debuffed).count();
        self.money += (nines_in_deck * self.count_joker(JokerKind::Cloud9)) as i32;

        // DelayedGratification: +$2 per available discard if no discards were used this round
        let max_disc = self.effective_max_discards();
        if self.discards_remaining == max_disc {
            let payers = self.count_joker(JokerKind::DelayedGratification);
            self.money += max_disc as i32 * 2 * payers as i32;
        }

        // The two bananas spoil at the end of a round: Gros Michel on 1 in 6, Cavendish on
        // 1 in 1000. Walked back-to-front so a removal cannot shift a position still to come.
        for pos in self.joker_indices(JokerKind::GrosMichel).into_iter().rev() {
            if !self.jokers[pos].eternal && self.roll_chance("gros_michel", 1.0 / 6.0) {
                self.jokers.remove(pos);
                // Extinction is permanent: Gros Michel leaves the pool, Cavendish joins it.
                self.gros_michel_extinct = true;
            }
        }
        for pos in self.joker_indices(JokerKind::Cavendish).into_iter().rev() {
            if !self.jokers[pos].eternal && self.roll_chance("cavendish", 1.0 / 1000.0) {
                self.jokers.remove(pos);
            }
        }

        // Popcorn: -4 mult per round (not per hand); destroyed when mult reaches 0
        let mut eaten: Vec<u64> = Vec::new();
        for i in self.joker_indices(JokerKind::Popcorn) {
            let left = (self.jokers[i].get_counter_i64("mult") - 4).max(0);
            self.jokers[i].set_counter_i64("mult", left);
            if left == 0 {
                eaten.push(self.jokers[i].id);
            }
        }
        if !eaten.is_empty() {
            self.jokers.retain(|j| !eaten.contains(&j.id));
        }

        // InvisibleJoker: counts rounds survived; the duplication happens on sell, not here
        for i in self.joker_indices(JokerKind::InvisibleJoker) {
            self.jokers[i].add_counter_i64("rounds", 1);
        }

        // ToTheMoon raises the interest *amount* paid per $5, not the cap (card.lua:614 bumps
        // G.GAME.interest_amount). Payout is amount × min(money/5, cap/5) — state_events.lua:1202.
        let to_the_moon_count = self.count_joker(JokerKind::ToTheMoon);
        let green = self.deck_type == DeckType::Green;

        // Every hand left unplayed pays out, on every deck — `money_per_hand or 1`
        // (state_events.lua:1165). Only a Challenge ever switches it off, so a vanilla run banks
        // a dollar per unused hand every round. The Green Deck raises it to $2 and adds $1 per
        // unused discard, and gives up interest to do it
        // (`extra_hand_bonus = 2, extra_discard_bonus = 1, no_interest = true`, game.lua:631).
        let money_per_hand = if green { 2 } else { 1 };
        self.money += money_per_hand * self.hands_remaining as i32;
        if green {
            self.money += self.discards_remaining as i32;
        }

        if !green {
            let interest_amount = 1 + to_the_moon_count as i32;
            let interest_steps = (money_at_round_end / 5).min(self.max_interest / 5).max(0);
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
                    if !self.has_room_for_consumable() {
                        break;
                    }
                    self.add_consumable(ConsumableCard::Planet(planet));
                }
            }
        }

        self.log_event(
            "round_won",
            serde_json::json!({
                "score": self.score_accumulated,
                "goal": self.score_goal,
                "dollars_earned": blind_dollars,
            }),
        );

        // The shop's voucher is drawn once per ante, when the Boss falls
        // (state_events.lua:263) — not once per shop. One bought early is gone for the rest of
        // the ante, and one you skip is still there next shop.
        if is_boss {
            self.shop_voucher = Some(self.random_voucher());
            // The next ante's tags are drawn here too, and eligibility is judged against the
            // ante about to begin (button_callbacks.lua:2951).
            let next_ante = self.ante + 1;
            self.draw_blind_tags_for_ante(next_ante);
        }

        // Mark blind as defeated
        let blind_slot = match self.current_blind {
            BlindKind::Small => 0,
            BlindKind::Big => 1,
            BlindKind::Boss => 2,
        };
        self.blind_defeated_this_ante[blind_slot] = true;

        if is_boss {
            // Campfire's stack is what you paid for the Boss run; it resets once the Boss falls.
            for i in self.joker_indices(JokerKind::Campfire) {
                self.jokers[i].set_counter_f64("x_mult", 1.0);
            }
            // Anaglyph deck: a Double Tag for every Boss beaten.
            if self.deck_type == DeckType::Anaglyph {
                self.gain_tag(TagKind::DoubleFun);
            }
            // Beating the final ante's Boss ends the run.
            if self.ante >= self.win_ante() {
                self.log_event("game_won", serde_json::json!({}));
                self.state = GameStateKind::GameOver;
                return;
            }
        }

        // Every won blind leads to the shop; `leave_shop` is what advances the blind afterwards.
        self.state = GameStateKind::Shop;
        self.generate_shop();
    }

    fn blind_reward_dollars(&self) -> i32 {
        match self.current_blind {
            // Red stake and above: Small Blind gives no cash reward
            BlindKind::Small if self.stake.at_least(Stake::Red) => 0,
            BlindKind::Small => 3,
            BlindKind::Big => 4,
            // Boss blinds pay $5, and the showdown bosses $8.
            BlindKind::Boss => match self.boss_blind {
                Some(b) if b.is_showdown() => 8,
                _ => 5,
            },
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
