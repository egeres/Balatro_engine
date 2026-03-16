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

        let played_cards: Vec<CardInstance> = played_hand_indices
            .iter()
            .map(|&i| self.deck[i].clone())
            .collect();

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
        );

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

        // Earn dollars from scoring
        self.money += result.dollars_earned;

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

        // Draw up to hand size
        self.draw_to_hand();

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

        self.draw_to_hand();
        Ok(())
    }

    fn win_round(&mut self) {
        let blind_dollars = self.blind_reward_dollars();
        self.money += blind_dollars;

        if self.deck_type == DeckType::Green {
            // Green deck: +$1 per remaining hand, +$1 per remaining discard; no interest
            self.money += self.hands_remaining as i32;
            self.money += self.discards_remaining as i32;
        } else {
            // Interest: $1 per $5 held, up to max_interest
            let interest = (self.money / 5).min(self.max_interest / 5).max(0);
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
            (BlindKind::Small, _) => 3,
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
