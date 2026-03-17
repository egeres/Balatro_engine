use crate::card::*;
use crate::types::*;
use crate::scoring::score_hand;
use crate::scoring::ScoreResult;
use std::collections::HashMap;
use super::{GameState, GameStateKind, BlindKind, BalatroError, HistoryEvent, LastConsumable};

impl GameState {
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

        // Lucky card: pre-roll probabilistic effects so score_hand sees them as flat bonuses.
        // +20 Mult on 1/5 (written into extra_mult so flat_mult_bonus picks it up).
        // $20 on 1/15 (counted here, paid out after scoring).
        let mut lucky_dollar_count: i32 = 0;
        for card in played_cards.iter_mut() {
            if card.enhancement == Enhancement::Lucky && !card.debuffed {
                if self.rng.next_bool_prob(1.0 / 5.0) {
                    card.extra_mult += 20;
                }
                if self.rng.next_bool_prob(1.0 / 15.0) {
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

        // CrimsonHeart: disable one random active joker for the duration of this hand
        let crimson_disabled_joker_id: Option<u64> = if let Some(BossBlind::CrimsonHeart) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                let luchador_active = self.jokers.iter().any(|j| {
                    (j.kind == JokerKind::Luchador || j.kind == JokerKind::Chicot) && j.active
                });
                if !luchador_active {
                    let active_jokers: Vec<usize> = self.jokers.iter().enumerate()
                        .filter(|(_, j)| j.active)
                        .map(|(i, _)| i)
                        .collect();
                    if !active_jokers.is_empty() {
                        let pick = self.rng.range_usize(0, active_jokers.len() - 1);
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

        let result = score_hand(
            &played_cards,
            &hand_cards,
            &self.jokers,
            &self.hand_levels,
            self.hands_remaining - 1,
            self.discards_remaining,
            self.money,
            self.draw_pile.len(),
            self.deck.len(),
            self.boss_blind,
            self.joker_slots as usize,
            self.tarot_cards_used,
            steel_count_in_deck,
        );

        // CrimsonHeart: re-enable the temporarily disabled joker
        if let Some(disabled_id) = crimson_disabled_joker_id {
            if let Some(j) = self.jokers.iter_mut().find(|j| j.id == disabled_id) {
                j.active = true;
            }
        }

        // Update hand level stats
        if let Some(level) = self.hand_levels.get_mut(&result.hand_type) {
            level.played += 1;
            level.played_this_round += 1;
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

        // Post-scoring joker updates
        self.post_play_joker_updates(&result, &played_cards, &hand_cards);

        // Cartomancer: create a tarot when playing a single-card hand
        if played_cards.len() == 1 {
            if self.jokers.iter().any(|j| j.kind == JokerKind::Cartomancer && j.active) {
                if self.consumables.len() < self.consumable_slots as usize {
                    let tarot = self.random_tarot();
                    self.consumables.push(ConsumableCard::Tarot(tarot));
                }
            }
        }

        // Vagabond: create a tarot if money <= $4 when playing a hand
        if self.money <= 4 {
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
            self.discard_pile.push(card_idx);
        }
        self.selected_indices.clear();

        // Process glass cards: chance to destroy
        for card in &played_cards {
            if card.enhancement == Enhancement::Glass {
                // 1/4 chance to break
                if self.rng.next_bool_prob(0.25) {
                    // Remove card from deck (destroy_deck_card remaps all index collections)
                    self.destroy_deck_card(card.id);
                    self.notify_face_card_destroyed(card);
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
                let disabled = self.jokers.iter().any(|j| {
                    (j.kind == JokerKind::Luchador || j.kind == JokerKind::Chicot) && j.active
                });
                if !disabled {
                    let discard_count = 2.min(self.hand.len());
                    for _ in 0..discard_count {
                        if self.hand.is_empty() { break; }
                        let pick = self.rng.range_usize(0, self.hand.len() - 1);
                        let card_idx = self.hand.remove(pick);
                        self.discard_pile.push(card_idx);
                    }
                }
            }
        }

        // Draw: TheSerpent draws exactly 3; otherwise fill to hand size
        let is_serpent = matches!(self.boss_blind, Some(BossBlind::TheSerpent))
            && matches!(self.current_blind, BlindKind::Boss)
            && !self.jokers.iter().any(|j| {
                (j.kind == JokerKind::Luchador || j.kind == JokerKind::Chicot) && j.active
            });
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
        let hand_type = result.hand_type;
        for i in 0..self.jokers.len() {
            let kind = self.jokers[i].kind;
            match kind {
                JokerKind::Runner => {
                    // +15 chips if straight
                    if hand_type == HandType::Straight || hand_type == HandType::StraightFlush {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 15);
                    }
                }
                JokerKind::IceCream => {
                    // -5 chips per hand played
                    let cur = self.jokers[i].get_counter_i64("chips");
                    let new = (cur - 5).max(0);
                    self.jokers[i].set_counter_i64("chips", new);
                    if new == 0 {
                        // Destroy joker
                        self.jokers[i].active = false;
                    }
                }
                JokerKind::Popcorn => {
                    let cur = self.jokers[i].get_counter_i64("mult");
                    let new = (cur - 4).max(0);
                    self.jokers[i].set_counter_i64("mult", new);
                    if new == 0 {
                        self.jokers[i].active = false;
                    }
                }
                JokerKind::SquareJoker => {
                    // +4 chips if 4 cards played exactly
                    if played.len() == 4 {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 4);
                    }
                }
                JokerKind::WeeJoker => {
                    // +8 chips for each 2 played
                    let twos = played.iter().filter(|c| c.rank == Rank::Two).count();
                    if twos > 0 {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 8 * twos as i64);
                    }
                }
                JokerKind::GreenJoker => {
                    // +1 mult per hand played, -1 per discard
                    let cur = self.jokers[i].get_counter_i64("mult");
                    self.jokers[i].set_counter_i64("mult", cur + 1);
                }
                JokerKind::SpareTrousers => {
                    if hand_type == HandType::TwoPair {
                        let cur = self.jokers[i].get_counter_i64("mult");
                        self.jokers[i].set_counter_i64("mult", cur + 2);
                    }
                }
                JokerKind::RideTheBus => {
                    let scoring_has_face = result
                        .scoring_card_indices
                        .iter()
                        .any(|&i| played[i].rank.is_face());
                    if !scoring_has_face {
                        let cur = self.jokers[i].get_counter_i64("mult");
                        self.jokers[i].set_counter_i64("mult", cur + 1);
                    } else {
                        self.jokers[i].set_counter_i64("mult", 0);
                    }
                }
                JokerKind::Hologram => {
                    // +0.25 Xmult for each playing card added to deck
                    // (tracked when cards are added to deck)
                }
                JokerKind::Vampire => {
                    // +0.1 Xmult for each enhanced card scored, remove enhancement
                    let enhanced: Vec<usize> = result
                        .scoring_card_indices
                        .iter()
                        .filter(|&&si| played[si].enhancement != Enhancement::None)
                        .copied()
                        .collect();
                    if !enhanced.is_empty() {
                        let cur = self.jokers[i].get_counter_f64("x_mult");
                        let new = cur + 0.1 * enhanced.len() as f64;
                        self.jokers[i].set_counter_f64("x_mult", new);
                        // Remove enhancements from those cards in deck
                        for si in enhanced {
                            let deck_idx = self.hand
                                .iter()
                                .find(|&&di| {
                                    self.deck[di].id == played[si].id
                                });
                            if let Some(&di) = deck_idx {
                                self.deck[di].enhancement = Enhancement::None;
                            }
                        }
                    }
                }
                JokerKind::Obelisk => {
                    // Xmult increases if you play different hand types
                    // Track which hand type was played most
                    // simplified: +0.2 if this hand type is NOT the most played
                    let this_plays = self
                        .hand_levels
                        .get(&hand_type)
                        .map(|h| h.played)
                        .unwrap_or(0);
                    let most_plays = self
                        .hand_levels
                        .values()
                        .map(|h| h.played)
                        .max()
                        .unwrap_or(0);
                    if this_plays < most_plays {
                        let cur = self.jokers[i].get_counter_f64("x_mult");
                        self.jokers[i].set_counter_f64("x_mult", cur + 0.2);
                    } else {
                        // Reset obelisk if this is the most played
                        self.jokers[i].set_counter_f64("x_mult", 1.0);
                    }
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
                    // 1/4 chance to create a tarot card when an 8 is scored
                    let eights_scored = result.scoring_card_indices.iter()
                        .filter(|&&idx| played[idx].rank == Rank::Eight)
                        .count();
                    for _ in 0..eights_scored {
                        if self.rng.next_bool_prob(0.25) {
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
                    if new_val <= 0 && !self.jokers[i].eternal {
                        self.jokers[i].active = false;
                    }
                }
                JokerKind::SpaceJoker => {
                    // 1/4 chance to level up the played hand
                    if self.rng.next_bool_prob(0.25) {
                        if let Some(level) = self.hand_levels.get_mut(&result.hand_type) {
                            level.level += 1;
                        }
                    }
                }
                JokerKind::Seance => {
                    // Create a spectral card if a Straight Flush is played
                    if matches!(result.hand_type, HandType::StraightFlush | HandType::FlushFive) {
                        if self.consumables.len() < self.consumable_slots as usize {
                            let spectrals = [
                                SpectralCard::Familiar, SpectralCard::Grim, SpectralCard::Incantation,
                                SpectralCard::Aura, SpectralCard::Wraith, SpectralCard::Ectoplasm,
                                SpectralCard::Ankh, SpectralCard::DejaVu, SpectralCard::Hex,
                                SpectralCard::Medium, SpectralCard::Cryptid,
                            ];
                            let idx = self.rng.range_usize(0, spectrals.len() - 1);
                            self.consumables.push(ConsumableCard::Spectral(spectrals[idx]));
                        }
                    }
                }
                JokerKind::Superposition => {
                    // Ace + Straight → create a tarot card
                    let has_ace = result.scoring_card_indices.iter()
                        .any(|&idx| played[idx].rank == Rank::Ace);
                    if has_ace && matches!(result.hand_type, HandType::Straight | HandType::StraightFlush) {
                        if self.consumables.len() < self.consumable_slots as usize {
                            let tarot = self.random_tarot();
                            self.consumables.push(ConsumableCard::Tarot(tarot));
                        }
                    }
                }
                JokerKind::SixthSense => {
                    // If only a 6 is played, destroy it and create a spectral card
                    if played.len() == 1 && played[0].rank == Rank::Six {
                        if self.consumables.len() < self.consumable_slots as usize {
                            let spectrals = [
                                SpectralCard::Familiar, SpectralCard::Grim, SpectralCard::Incantation,
                                SpectralCard::Talisman, SpectralCard::Aura, SpectralCard::Wraith,
                                SpectralCard::Ankh, SpectralCard::DejaVu, SpectralCard::Medium,
                            ];
                            let idx = self.rng.range_usize(0, spectrals.len() - 1);
                            self.consumables.push(ConsumableCard::Spectral(spectrals[idx]));
                            // Destroy the 6
                            let card_id = played[0].id;
                            self.destroy_deck_card(card_id);
                        }
                    }
                }
                JokerKind::MidasMask => {
                    // Face cards scored become Gold enhancement
                    for &sci in &result.scoring_card_indices {
                        if played[sci].rank.is_face() {
                            // Find the card in deck by id and set enhancement
                            let card_id = played[sci].id;
                            if let Some(deck_card) = self.deck.iter_mut().find(|c| c.id == card_id) {
                                deck_card.enhancement = Enhancement::Gold;
                            }
                        }
                    }
                }
                JokerKind::Dna => {
                    // On the first hand of a round, if only 1 card was played, add a permanent copy to deck
                    let max_h = self.effective_max_hands();
                    let was_first_hand = self.hands_remaining + 1 == max_h;
                    if was_first_hand && played.len() == 1 {
                        let card_to_copy = played[0].clone();
                        let new_id = self.next_id();
                        let mut new_card = card_to_copy;
                        new_card.id = new_id;
                        let deck_idx = self.deck.len();
                        self.deck.push(new_card);
                        self.draw_pile.push(deck_idx);
                    }
                }
                _ => {}
            }
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
        if discarded_cards.iter().filter(|c| c.rank.is_face()).count() >= 3 {
            let count = self.jokers.iter().filter(|j| j.kind == JokerKind::FacelessJoker && j.active).count();
            self.money += 5 * count as i32;
        }

        // MailInRebate: +$5 for each discarded card matching the tracked rank
        for j_idx in 0..self.jokers.len() {
            if self.jokers[j_idx].kind == JokerKind::MailInRebate && self.jokers[j_idx].active {
                let rank_str = self.jokers[j_idx].counters.get("rank").and_then(|v| v.as_str()).unwrap_or("Two").to_string();
                let target_rank = match rank_str.as_str() {
                    "Two" => Rank::Two,
                    "Three" => Rank::Three,
                    "Four" => Rank::Four,
                    "Five" => Rank::Five,
                    "Six" => Rank::Six,
                    "Seven" => Rank::Seven,
                    "Eight" => Rank::Eight,
                    "Nine" => Rank::Nine,
                    "Ten" => Rank::Ten,
                    "Jack" => Rank::Jack,
                    "Queen" => Rank::Queen,
                    "King" => Rank::King,
                    "Ace" => Rank::Ace,
                    _ => Rank::Two,
                };
                let matching = discarded_cards.iter().filter(|c| c.rank == target_rank).count();
                self.money += 5 * matching as i32;
            }
        }

        // BurntJoker: +1 hand after discarding
        let burnt_count = self.jokers.iter().filter(|j| j.kind == JokerKind::BurntJoker && j.active).count();
        self.hands_remaining += burnt_count as u32;

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

        // Post-discard joker updates
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
                    let suit_str = self.jokers[i]
                        .counters
                        .get("suit")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Spades")
                        .to_string();
                    let target_suit = match suit_str.as_str() {
                        "Spades" => Suit::Spades,
                        "Hearts" => Suit::Hearts,
                        "Clubs" => Suit::Clubs,
                        "Diamonds" => Suit::Diamonds,
                        _ => Suit::Spades,
                    };
                    let count = discarded_cards
                        .iter()
                        .filter(|c| c.suit == target_suit)
                        .count();
                    if count > 0 {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 3 * count as i64);
                    }
                }
                _ => {}
            }
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
            && !self.jokers.iter().any(|j| {
                (j.kind == JokerKind::Luchador || j.kind == JokerKind::Chicot) && j.active
            });
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

        // Satellite: +$1 per unique planet type used this run
        let planet_types_used = self.planet_cards_used.min(9); // 9 unique planet types max
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

        // GrosMichel: 1/6 chance to be destroyed at end of round
        let gm_positions: Vec<usize> = self.jokers.iter().enumerate()
            .filter(|(_, j)| j.kind == JokerKind::GrosMichel && j.active && !j.eternal)
            .map(|(i, _)| i)
            .collect();
        for pos in gm_positions.iter().rev() {
            if self.rng.next_bool_prob(1.0 / 6.0) {
                self.jokers.remove(*pos);
            }
        }

        // InvisibleJoker: after 2 rounds, duplicate a random joker for free
        for i in 0..self.jokers.len() {
            if self.jokers[i].kind == JokerKind::InvisibleJoker && self.jokers[i].active {
                let rounds = self.jokers[i].get_counter_i64("rounds") + 1;
                self.jokers[i].set_counter_i64("rounds", rounds % 2);
                if rounds >= 2 && self.jokers.len() < self.joker_slots as usize {
                    // Find a random joker that is not InvisibleJoker itself
                    let candidates: Vec<usize> = (0..self.jokers.len())
                        .filter(|&j| j != i && self.jokers[j].active && self.jokers[j].kind != JokerKind::InvisibleJoker)
                        .collect();
                    if !candidates.is_empty() {
                        let pick = self.rng.range_usize(0, candidates.len() - 1);
                        let dup = self.jokers[candidates[pick]].clone();
                        self.jokers.push(dup);
                    }
                }
            }
        }

        // ToTheMoon: +$1 interest per $5 held (raises effective interest cap by 1 per joker)
        let to_the_moon_count = self.jokers.iter().filter(|j| j.kind == JokerKind::ToTheMoon && j.active).count();

        if self.deck_type == DeckType::Green {
            // Green deck: +$1 per remaining hand, +$1 per remaining discard; no interest
            self.money += self.hands_remaining as i32;
            self.money += self.discards_remaining as i32;
        } else {
            // Interest: $1 per $5 held, up to max_interest (ToTheMoon raises cap by 5 each)
            let effective_max = self.max_interest + 5 * to_the_moon_count as i32;
            let interest = (self.money / 5).min(effective_max / 5).max(0);
            self.money += interest;
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

        // Blue seal: create planet for cards in hand at round end
        let hand_copy: Vec<usize> = self.hand.clone();
        for &card_idx in &hand_copy {
            if self.deck[card_idx].seal == Seal::Blue {
                if self.consumables.len() < self.consumable_slots as usize {
                    let planet = self.random_planet();
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
            self.tags.push(TagKind::DoubleFun);
        }

        // Check if ante 8 boss beaten = game won
        if self.ante >= 8 && matches!(self.current_blind, BlindKind::Boss) {
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
                self.ante += 1;
                self.round = 1;
                self.current_blind = BlindKind::Small;
                self.blind_defeated_this_ante = [false; 3];
                self.boss_blind = self.pick_boss_blind();
            }
        }
    }
}
