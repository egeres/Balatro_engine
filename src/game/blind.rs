use crate::card::*;
use crate::types::*;
use std::collections::HashMap;
use super::{GameState, GameStateKind, BlindKind, BalatroError, HistoryEvent};

impl GameState {
    pub fn pick_boss_blind(&mut self) -> Option<BossBlind> {
        let all_bosses = vec![
            BossBlind::TheOx,
            BossBlind::TheHook,
            BossBlind::TheMouth,
            BossBlind::TheFish,
            BossBlind::TheClub,
            BossBlind::TheManacle,
            BossBlind::TheTooth,
            BossBlind::TheWall,
            BossBlind::TheHouse,
            BossBlind::TheMark,
            BossBlind::TheWheel,
            BossBlind::TheArm,
            BossBlind::ThePsychic,
            BossBlind::TheGoad,
            BossBlind::TheWater,
            BossBlind::TheEye,
            BossBlind::ThePlant,
            BossBlind::TheNeedle,
            BossBlind::TheHead,
            BossBlind::TheWindow,
            BossBlind::TheSerpent,
            BossBlind::ThePillar,
            BossBlind::TheFlint,
        ];

        // Special showdown bosses for ante 8+
        let ante = self.ante;
        if ante >= 8 {
            let showdowns = vec![
                BossBlind::CeruleanBell,
                BossBlind::VerdantLeaf,
                BossBlind::VioletVessel,
                BossBlind::AmberAcorn,
                BossBlind::CrimsonHeart,
            ];
            let idx = self.rng.range_usize(0, showdowns.len() - 1);
            return Some(showdowns[idx]);
        }

        let idx = self.rng.range_usize(0, all_bosses.len() - 1);
        Some(all_bosses[idx])
    }

    // =========================================================
    // Actions
    // =========================================================

    pub fn select_blind(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::BlindSelect) {
            return Err(BalatroError::NotInBlindSelect);
        }
        self.begin_round();
        Ok(())
    }

    pub fn skip_blind(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::BlindSelect) {
            return Err(BalatroError::NotInBlindSelect);
        }
        // Can't skip boss blind
        if matches!(self.current_blind, BlindKind::Boss) {
            return Err(BalatroError::CannotSkipBoss);
        }

        // Record skip for tags / Throwback / RedCard jokers
        self.skipped_blinds.push((self.ante, self.round));
        for j in self.jokers.iter_mut() {
            match j.kind {
                JokerKind::Throwback => {
                    let skips = j.get_counter_i64("skips");
                    j.set_counter_i64("skips", skips + 1);
                }
                JokerKind::RedCard => {
                    let mult = j.get_counter_i64("mult");
                    j.set_counter_i64("mult", mult + 3);
                }
                _ => {}
            }
        }

        // Advance to next blind
        self.advance_blind();
        Ok(())
    }

    fn begin_round(&mut self) {
        self.state = GameStateKind::Round;
        self.score_accumulated = 0.0;
        self.hands_remaining = self.effective_max_hands();
        self.discards_remaining = self.effective_max_discards();
        self.selected_indices.clear();
        self.hand.clear();
        self.discard_pile.clear();

        // Reset showdown blind state
        self.verdant_leaf_joker_sold = false;
        self.cerulean_forced_card_id = None;

        // Reset per-round hand played counters
        for data in self.hand_levels.values_mut() {
            data.played_this_round = 0;
        }

        // Reset draw pile
        self.draw_pile = (0..self.deck.len()).collect();
        self.rng.shuffle(&mut self.draw_pile);

        // Reset face-down state for all cards
        for card in self.deck.iter_mut() {
            card.face_down = false;
        }

        // Apply boss blind debuffs to cards
        self.apply_boss_blind_debuffs();

        // AmberAcorn: shuffle joker order at the start of the blind
        if let Some(BossBlind::AmberAcorn) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                let disabled = self.jokers.iter().any(|j| {
                    (j.kind == JokerKind::Luchador || j.kind == JokerKind::Chicot) && j.active
                });
                if !disabled {
                    self.rng.shuffle(&mut self.jokers);
                }
            }
        }

        // Set score goal
        self.score_goal = self.get_blind_chip_goal();

        // Draw initial hand
        self.draw_to_hand();

        // Notify jokers of blind selection
        self.notify_jokers_setting_blind();
    }

    pub(crate) fn effective_max_hands(&self) -> u32 {
        let mut hands = self.max_hands;
        for j in &self.jokers {
            if !j.active {
                continue;
            }
            match j.kind {
                JokerKind::Troubadour => {
                    hands = hands.saturating_sub(1);
                }
                JokerKind::Burglar => {
                    hands += 3;
                }
                _ => {}
            }
        }
        hands
    }

    pub(crate) fn effective_max_discards(&self) -> u32 {
        let mut discards = self.max_discards;
        // Blue stake and above: -1 discard per round
        if self.stake as u8 >= Stake::Blue as u8 {
            discards = discards.saturating_sub(1);
        }
        // TheWater: start with 0 discards
        if let Some(BossBlind::TheWater) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                let disabled = self.jokers.iter().any(|j| {
                    (j.kind == JokerKind::Luchador || j.kind == JokerKind::Chicot) && j.active
                });
                if !disabled {
                    return 0;
                }
            }
        }
        for j in &self.jokers {
            if !j.active {
                continue;
            }
            match j.kind {
                JokerKind::MerryAndy => discards += 3,
                JokerKind::Drunkard => discards += 1,
                JokerKind::Burglar => discards = 0,
                _ => {}
            }
        }
        discards
    }

    pub fn effective_hand_size(&self) -> u32 {
        let mut size = self.hand_size;
        // TheManacle: -1 hand size during Boss blind
        if let Some(BossBlind::TheManacle) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                let disabled = self.jokers.iter().any(|j| {
                    (j.kind == JokerKind::Luchador || j.kind == JokerKind::Chicot) && j.active
                });
                if !disabled {
                    size = size.saturating_sub(1);
                }
            }
        }
        for j in &self.jokers {
            if !j.active {
                continue;
            }
            match j.kind {
                JokerKind::Juggler => size += 1,
                JokerKind::Troubadour => size += 2,
                JokerKind::Stuntman => size = size.saturating_sub(2),
                JokerKind::MerryAndy => size = size.saturating_sub(1),
                JokerKind::TurtleBean => {
                    let h = j.get_counter_i64("h_size");
                    size = size.saturating_add(h as u32);
                }
                _ => {}
            }
        }
        // Psychic boss blind forces 5-card hands if hand_size >= 5
        if let Some(BossBlind::ThePsychic) = self.boss_blind {
            if matches!(self.state, GameStateKind::Round) {
                size = size.max(5);
            }
        }
        size
    }

    pub(crate) fn draw_to_hand(&mut self) {
        let hand_size = self.effective_hand_size() as usize;
        let start_hand_len = self.hand.len();
        while self.hand.len() < hand_size && !self.draw_pile.is_empty() {
            let card_idx = self.draw_pile.remove(0);
            self.hand.push(card_idx);
        }

        // TheFish: all newly drawn cards after the first hand are face-down
        // TheWheel: each newly drawn card has a 1-in-7 chance of being face-down
        if matches!(self.current_blind, BlindKind::Boss) {
            let luchador_active = self.jokers.iter().any(|j| {
                (j.kind == JokerKind::Luchador || j.kind == JokerKind::Chicot) && j.active
            });
            if !luchador_active {
                let newly_drawn = start_hand_len..self.hand.len();
                match self.boss_blind {
                    Some(BossBlind::TheFish) => {
                        // All cards face-down after the initial draw.
                        // After the first play, hands_remaining < effective_max_hands().
                        if self.hands_remaining < self.effective_max_hands() {
                            for hand_idx in newly_drawn {
                                let card_idx = self.hand[hand_idx];
                                self.deck[card_idx].face_down = true;
                            }
                        }
                    }
                    Some(BossBlind::TheWheel) => {
                        // 1-in-7 chance per newly drawn card
                        for hand_idx in newly_drawn {
                            if self.rng.range_usize(0, 6) == 0 {
                                let card_idx = self.hand[hand_idx];
                                self.deck[card_idx].face_down = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // CeruleanBell: one random newly-drawn card is always selected (forced)
        if let Some(BossBlind::CeruleanBell) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                let luchador_active = self.jokers.iter().any(|j| {
                    (j.kind == JokerKind::Luchador || j.kind == JokerKind::Chicot) && j.active
                });
                if !luchador_active {
                    let newly_drawn_count = self.hand.len() - start_hand_len;
                    if newly_drawn_count > 0 {
                        let offset = self.rng.range_usize(0, newly_drawn_count - 1);
                        let forced_hand_idx = start_hand_len + offset;
                        let card_deck_idx = self.hand[forced_hand_idx];
                        self.cerulean_forced_card_id = Some(self.deck[card_deck_idx].id);
                        if !self.selected_indices.contains(&forced_hand_idx) {
                            self.selected_indices.push(forced_hand_idx);
                        }
                    }
                }
            }
        }
    }

    fn apply_boss_blind_debuffs(&mut self) {
        // Reset all debuffs first
        for card in self.deck.iter_mut() {
            card.debuffed = false;
        }
        let boss = match self.current_blind {
            BlindKind::Boss => self.boss_blind,
            _ => return,
        };
        let Some(boss) = boss else { return };

        // Luchador and Chicot both disable the boss blind's special effect
        if self.jokers.iter().any(|j| (j.kind == JokerKind::Luchador || j.kind == JokerKind::Chicot) && j.active) {
            return;
        }

        match boss {
            BossBlind::TheClub => {
                for card in self.deck.iter_mut() {
                    if card.suit == Suit::Clubs {
                        card.debuffed = true;
                    }
                }
            }
            BossBlind::TheGoad => {
                for card in self.deck.iter_mut() {
                    if card.suit == Suit::Spades {
                        card.debuffed = true;
                    }
                }
            }
            BossBlind::TheHead => {
                for card in self.deck.iter_mut() {
                    if card.suit == Suit::Hearts {
                        card.debuffed = true;
                    }
                }
            }
            BossBlind::TheWindow => {
                for card in self.deck.iter_mut() {
                    if card.suit == Suit::Diamonds {
                        card.debuffed = true;
                    }
                }
            }
            BossBlind::ThePlant => {
                for card in self.deck.iter_mut() {
                    if card.rank.is_face() {
                        card.debuffed = true;
                    }
                }
            }
            BossBlind::TheMark => {
                for card in self.deck.iter_mut() {
                    if card.rank.is_face() {
                        card.debuffed = true;
                    }
                }
            }
            BossBlind::VerdantLeaf => {
                // All cards debuffed until at least 1 joker is sold
                if !self.verdant_leaf_joker_sold {
                    for card in self.deck.iter_mut() {
                        card.debuffed = true;
                    }
                }
            }
            BossBlind::ThePillar => {
                // Debuff any card whose ID was played in an earlier round this Ante
                for card in self.deck.iter_mut() {
                    if self.played_card_ids_this_ante.contains(&card.id) {
                        card.debuffed = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn notify_jokers_setting_blind(&mut self) {
        // Process jokers that trigger when blind is set
        let joker_kinds: Vec<JokerKind> = self.jokers.iter().map(|j| j.kind).collect();
        for kind in joker_kinds {
            match kind {
                JokerKind::MarbleJoker => {
                    // Add 1 stone card to deck
                    let id = self.next_id();
                    let mut stone = CardInstance::new(id, Rank::Ace, Suit::Spades);
                    stone.enhancement = Enhancement::Stone;
                    let deck_idx = self.deck.len();
                    self.deck.push(stone);
                    self.draw_pile.push(deck_idx);
                }
                JokerKind::Madness => {
                    // Gain +0.5 Xmult, then destroy 1 random non-eternal joker (excluding Madness itself)
                    if let Some(pos) = self.jokers.iter().position(|j| j.kind == JokerKind::Madness && j.active) {
                        let cur = self.jokers[pos].get_counter_f64("x_mult");
                        self.jokers[pos].set_counter_f64("x_mult", cur + 0.5);
                    }
                    let destroyable: Vec<usize> = self.jokers.iter().enumerate()
                        .filter(|(_, j)| j.kind != JokerKind::Madness && !j.eternal)
                        .map(|(i, _)| i)
                        .collect();
                    if !destroyable.is_empty() {
                        let pick = self.rng.range_usize(0, destroyable.len() - 1);
                        let idx = destroyable[pick];
                        self.jokers.remove(idx);
                    }
                }
                JokerKind::Certificate => {
                    // Add a playing card with a random seal to the hand
                    let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
                    let ranks = [
                        Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
                        Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                        Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
                    ];
                    let seals = [Seal::Gold, Seal::Red, Seal::Blue, Seal::Purple];
                    let suit_idx = self.rng.range_usize(0, 3);
                    let rank_idx = self.rng.range_usize(0, 12);
                    let seal_idx = self.rng.range_usize(0, seals.len() - 1);
                    let new_id = self.next_id();
                    let mut new_card = CardInstance::new(new_id, ranks[rank_idx], suits[suit_idx]);
                    new_card.seal = seals[seal_idx];
                    let deck_idx = self.deck.len();
                    self.deck.push(new_card);
                    self.draw_pile.push(deck_idx);
                }
                JokerKind::RiffRaff => {
                    // Add 2 common jokers (rarity 1) at the start of each round
                    for _ in 0..2 {
                        if self.jokers.len() < self.joker_slots as usize {
                            if let Some(new_joker) = self.generate_random_joker() {
                                // Only add if it's a common joker
                                if new_joker.kind.rarity() == 1 {
                                    self.jokers.push(new_joker);
                                }
                            }
                        }
                    }
                }
                JokerKind::TurtleBean => {
                    // TurtleBean shrinks by 1 each round; destroyed when h_size reaches 0
                    if let Some(pos) = self.jokers.iter().position(|j| j.kind == JokerKind::TurtleBean && j.active) {
                        let cur = self.jokers[pos].get_counter_i64("h_size");
                        let new_val = cur - 1;
                        self.jokers[pos].set_counter_i64("h_size", new_val);
                        if new_val <= 0 && !self.jokers[pos].eternal {
                            self.jokers.remove(pos);
                        }
                    }
                }
                JokerKind::ToDoList => {
                    // Randomize the target hand type each round
                    let hand_types = [
                        "HighCard", "Pair", "TwoPair", "ThreeOfAKind", "Straight",
                        "Flush", "FullHouse", "FourOfAKind", "StraightFlush",
                    ];
                    let idx = self.rng.range_usize(0, hand_types.len() - 1);
                    if let Some(pos) = self.jokers.iter().position(|j| j.kind == JokerKind::ToDoList) {
                        self.jokers[pos].counters.insert(
                            "hand_type".to_string(),
                            serde_json::json!(hand_types[idx]),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
