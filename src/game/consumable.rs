use crate::card::*;
use crate::types::*;
use std::collections::HashMap;
use super::{GameState, GameStateKind, BalatroError, HistoryEvent, LastConsumable, rank_up};

impl GameState {
    pub fn use_consumable(&mut self, consumable_index: usize, targets: Vec<usize>) -> Result<(), BalatroError> {
        if consumable_index >= self.consumables.len() {
            return Err(BalatroError::IndexOutOfRange(consumable_index, self.consumables.len()));
        }

        let consumable = self.consumables[consumable_index].clone();
        match &consumable {
            ConsumableCard::Planet(p) => {
                self.apply_planet(*p);
                self.planet_cards_used += 1;
                self.planet_types_used.insert(*p);
            }
            ConsumableCard::Tarot(t) => {
                self.apply_tarot(*t, &targets)?;
                self.tarot_cards_used += 1;
            }
            ConsumableCard::Spectral(s) => {
                self.apply_spectral(*s, &targets)?;
            }
        }

        self.consumables.remove(consumable_index);
        Ok(())
    }

    pub fn sell_consumable(&mut self, consumable_index: usize) -> Result<(), BalatroError> {
        if consumable_index >= self.consumables.len() {
            return Err(BalatroError::IndexOutOfRange(consumable_index, self.consumables.len()));
        }

        // DietCola: when sold, creates a foil copy of the consumable
        if self.jokers.iter().any(|j| j.kind == JokerKind::DietCola && j.active) {
            if self.consumables.len() < self.consumable_slots as usize {
                let copy = self.consumables[consumable_index].clone();
                self.consumables.push(copy);
            }
        }

        let base_cost = self.consumables[consumable_index].base_cost();
        self.money += (base_cost / 2).max(1) as i32;
        self.consumables.remove(consumable_index);
        Ok(())
    }

    fn apply_planet(&mut self, planet: PlanetCard) {
        let hand_type = planet.hand_type();
        if let Some(level) = self.hand_levels.get_mut(&hand_type) {
            level.level += 1;
            // Observatory: each planet use gives X1.5 Mult for this hand type
            if self.vouchers.contains(&VoucherKind::Observatory) {
                level.observatory_x_mult *= 1.5;
            }
        }
        self.last_consumable_used = Some(LastConsumable::Planet(planet));
        self.history.push(HistoryEvent {
            ante: self.ante,
            round: self.round,
            event_type: "planet_used".to_string(),
            data: serde_json::json!({
                "planet": format!("{:?}", planet),
                "hand_type": hand_type.display_name(),
            }),
        });
    }

    fn apply_tarot(&mut self, tarot: TarotCard, targets: &[usize]) -> Result<(), BalatroError> {
        // Get the target cards from hand (targets are hand indices)
        match tarot {
            TarotCard::TheHangedMan => {
                // Destroy up to 2 selected cards
                let mut sorted = targets.to_vec();
                sorted.sort_by(|a, b| b.cmp(a));
                for &hi in &sorted {
                    if hi < self.hand.len() {
                        let card_idx = self.hand.remove(hi);
                        let dead_card = self.deck[card_idx].clone();
                        let dead_id = dead_card.id;
                        self.notify_face_card_destroyed(&dead_card);
                        self.destroy_deck_card(dead_id);
                    }
                }
            }
            TarotCard::TheHermit => {
                // Double money (up to $20 gain)
                let gain = self.money.min(20);
                self.money += gain;
            }
            TarotCard::Temperance => {
                // Give money equal to sum of joker sell values (up to $50)
                let total: u32 = self.jokers.iter().map(|j| j.sell_value()).sum();
                let gain = total.min(50);
                self.money += gain as i32;
            }
            TarotCard::TheMagician => {
                // Enhance up to 2 cards as Lucky
                for &hi in targets.iter().take(2) {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].enhancement = Enhancement::Lucky;
                    }
                }
            }
            TarotCard::TheEmpress => {
                // Enhance up to 2 cards as Mult
                for &hi in targets.iter().take(2) {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].enhancement = Enhancement::Mult;
                    }
                }
            }
            TarotCard::TheHierophant => {
                // Enhance up to 2 cards as Bonus
                for &hi in targets.iter().take(2) {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].enhancement = Enhancement::Bonus;
                    }
                }
            }
            TarotCard::TheLovers => {
                // Enhance 1 card as Wild
                if let Some(&hi) = targets.first() {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].enhancement = Enhancement::Wild;
                    }
                }
            }
            TarotCard::TheChariot => {
                // Enhance 1 card as Steel
                if let Some(&hi) = targets.first() {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].enhancement = Enhancement::Steel;
                    }
                }
            }
            TarotCard::Justice => {
                // Enhance 1 card as Glass
                if let Some(&hi) = targets.first() {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].enhancement = Enhancement::Glass;
                    }
                }
            }
            TarotCard::TheDevil => {
                // Enhance 1 card as Gold
                if let Some(&hi) = targets.first() {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].enhancement = Enhancement::Gold;
                    }
                }
            }
            TarotCard::TheTower => {
                // Enhance 1 card as Stone
                if let Some(&hi) = targets.first() {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].enhancement = Enhancement::Stone;
                    }
                }
            }
            TarotCard::TheStar => {
                // Convert up to 3 cards to Diamonds
                for &hi in targets.iter().take(3) {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].suit = Suit::Diamonds;
                    }
                }
            }
            TarotCard::TheMoon => {
                // Convert up to 3 cards to Clubs
                for &hi in targets.iter().take(3) {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].suit = Suit::Clubs;
                    }
                }
            }
            TarotCard::TheSun => {
                // Convert up to 3 cards to Hearts
                for &hi in targets.iter().take(3) {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].suit = Suit::Hearts;
                    }
                }
            }
            TarotCard::TheWorld => {
                // Convert up to 3 cards to Spades
                for &hi in targets.iter().take(3) {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].suit = Suit::Spades;
                    }
                }
            }
            TarotCard::Strength => {
                // Increase rank of up to 2 cards by 1
                for &hi in targets.iter().take(2) {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        let new_rank = rank_up(self.deck[card_idx].rank);
                        self.deck[card_idx].rank = new_rank;
                    }
                }
            }
            TarotCard::Death => {
                // Select 2 cards: the LEFT card becomes a full copy of the RIGHT card
                // (copies rank, suit, enhancement, edition, seal — only id stays)
                if targets.len() == 2 {
                    let left_hi = targets[0];
                    let right_hi = targets[1];
                    if left_hi < self.hand.len() && right_hi < self.hand.len() {
                        let left_deck_idx = self.hand[left_hi];
                        let right_deck_idx = self.hand[right_hi];
                        let right_card = self.deck[right_deck_idx].clone();
                        self.deck[left_deck_idx].rank = right_card.rank;
                        self.deck[left_deck_idx].suit = right_card.suit;
                        self.deck[left_deck_idx].enhancement = right_card.enhancement;
                        self.deck[left_deck_idx].edition = right_card.edition;
                        self.deck[left_deck_idx].seal = right_card.seal;
                    }
                }
            }
            TarotCard::TheWheelOfFortune => {
                // 1/4 chance to add random edition to random joker (1/2 with OopsAll6s)
                let wheel_oops = if self.jokers.iter().any(|j| j.kind == JokerKind::OopsAll6s && j.active) { 2.0_f64 } else { 1.0_f64 };
                if !self.jokers.is_empty() && self.rng.next_bool_prob((0.25 * wheel_oops).min(1.0)) {
                    let idx = self.rng.range_usize(0, self.jokers.len() - 1);
                    let editions = [Edition::Foil, Edition::Holographic, Edition::Polychrome];
                    let ed_idx = self.rng.range_usize(0, 2);
                    self.jokers[idx].edition = editions[ed_idx];
                }
            }
            TarotCard::Judgement => {
                // Create random joker
                if self.jokers.len() < self.joker_slots as usize {
                    if let Some(j) = self.generate_random_joker() {
                        self.jokers.push(j);
                    }
                }
            }
            TarotCard::TheFool => {
                // Creates the most recently used Tarot or Planet card this run
                // The Fool itself does not count as the "last used" consumable
                match &self.last_consumable_used.clone() {
                    Some(LastConsumable::Tarot(t)) => {
                        if self.consumables.len() < self.consumable_slots as usize {
                            self.consumables.push(ConsumableCard::Tarot(*t));
                        }
                    }
                    Some(LastConsumable::Planet(p)) => {
                        if self.consumables.len() < self.consumable_slots as usize {
                            self.consumables.push(ConsumableCard::Planet(*p));
                        }
                    }
                    None => {}
                }
                return Ok(());
            }
            TarotCard::TheHighPriestess => {
                // Creates up to 2 random Planet cards (must have room)
                for _ in 0..2 {
                    if self.consumables.len() < self.consumable_slots as usize {
                        let planet = self.random_planet();
                        self.consumables.push(ConsumableCard::Planet(planet));
                    }
                }
            }
            TarotCard::TheEmperor => {
                // Creates up to 2 random Tarot cards (must have room)
                for _ in 0..2 {
                    if self.consumables.len() < self.consumable_slots as usize {
                        let tarot = self.random_tarot();
                        self.consumables.push(ConsumableCard::Tarot(tarot));
                    }
                }
            }
        }
        self.last_consumable_used = Some(LastConsumable::Tarot(tarot));
        Ok(())
    }

    fn apply_spectral(&mut self, spectral: SpectralCard, targets: &[usize]) -> Result<(), BalatroError> {
        match spectral {
            SpectralCard::Familiar => {
                // Destroy 1 random card in hand, add 3 random enhanced face cards to deck
                if !self.hand.is_empty() {
                    let idx = self.rng.range_usize(0, self.hand.len() - 1);
                    let card_idx = self.hand.remove(idx);
                    let dead_card = self.deck[card_idx].clone();
                    let card_id = dead_card.id;
                    self.notify_face_card_destroyed(&dead_card);
                    self.destroy_deck_card(card_id);
                    // Add 3 enhanced face cards
                    let faces = [Rank::Jack, Rank::Queen, Rank::King];
                    let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
                    let enhancements = [
                        Enhancement::Lucky, Enhancement::Gold,
                        Enhancement::Wild, Enhancement::Mult,
                        Enhancement::Bonus, Enhancement::Steel,
                        Enhancement::Glass, Enhancement::Stone,
                    ];
                    for _ in 0..3 {
                        let rank = faces[self.rng.range_usize(0, 2)];
                        let suit = suits[self.rng.range_usize(0, 3)];
                        let enh = enhancements[self.rng.range_usize(0, 7)];
                        let id = self.next_id();
                        let mut card = CardInstance::new(id, rank, suit);
                        card.enhancement = enh;
                        let di = self.deck.len();
                        self.deck.push(card);
                        self.draw_pile.push(di);
                    }
                }
            }
            SpectralCard::Ectoplasm => {
                // Add Negative edition to random joker, -1 hand size
                if !self.jokers.is_empty() {
                    let idx = self.rng.range_usize(0, self.jokers.len() - 1);
                    self.jokers[idx].edition = Edition::Negative;
                }
                self.hand_size = self.hand_size.saturating_sub(1);
            }
            SpectralCard::Aura => {
                // Add Foil/Holo/Poly to 1 selected card
                if let Some(&hi) = targets.first() {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        let editions = [Edition::Foil, Edition::Holographic, Edition::Polychrome];
                        let idx = self.rng.range_usize(0, 2);
                        self.deck[card_idx].edition = editions[idx];
                    }
                }
            }
            SpectralCard::Hex => {
                // Add Polychrome to random joker, destroy the rest (eternal jokers are spared)
                if !self.jokers.is_empty() {
                    let idx = self.rng.range_usize(0, self.jokers.len() - 1);
                    let chosen_id = self.jokers[idx].id;
                    self.jokers[idx].edition = Edition::Polychrome;
                    self.jokers.retain(|j| j.id == chosen_id || j.eternal);
                }
            }
            SpectralCard::Immolate => {
                // Destroy up to 5 random cards in hand, gain $20
                let count = self.hand.len().min(5);
                if count > 0 {
                    let mut hand_indices: Vec<usize> = (0..self.hand.len()).collect();
                    self.rng.shuffle(&mut hand_indices);
                    // Collect ids before any removal
                    let to_remove_ids: Vec<u64> = hand_indices[..count]
                        .iter()
                        .map(|&hi| self.deck[self.hand[hi]].id)
                        .collect();
                    // Notify Canio of any face cards being destroyed
                    for id in &to_remove_ids {
                        if let Some(card) = self.deck.iter().find(|c| c.id == *id).cloned() {
                            self.notify_face_card_destroyed(&card);
                        }
                    }
                    // Remove from hand (descending order to keep indices valid)
                    let mut sorted_hi: Vec<usize> = hand_indices[..count].to_vec();
                    sorted_hi.sort_unstable_by(|a, b| b.cmp(a));
                    for hi in sorted_hi {
                        self.hand.remove(hi);
                    }
                    // Remove from deck (remaps all index collections)
                    self.destroy_deck_cards(&to_remove_ids);
                }
                self.money += 20;
            }
            SpectralCard::Ankh => {
                // Copy a random joker, destroy the others (eternal jokers are spared)
                if !self.jokers.is_empty() {
                    let idx = self.rng.range_usize(0, self.jokers.len() - 1);
                    let chosen_id = self.jokers[idx].id;
                    let mut new_copy = self.jokers[idx].clone();
                    new_copy.id = self.next_id();
                    // Retain: the eternal jokers + the copy (original is removed, copy is added below)
                    self.jokers.retain(|j| j.eternal && j.id != chosen_id);
                    self.jokers.push(new_copy);
                }
            }
            SpectralCard::DejaVu => {
                // Add Red Seal to 1 selected card
                if let Some(&hi) = targets.first() {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].seal = Seal::Red;
                    }
                }
            }
            SpectralCard::Trance => {
                // Add Blue Seal to 1 selected card
                if let Some(&hi) = targets.first() {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].seal = Seal::Blue;
                    }
                }
            }
            SpectralCard::Medium => {
                // Add Purple Seal to 1 selected card
                if let Some(&hi) = targets.first() {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].seal = Seal::Purple;
                    }
                }
            }
            SpectralCard::Grim => {
                // Destroy 1 random card in hand, add 2 random enhanced Aces to deck
                if !self.hand.is_empty() {
                    let idx = self.rng.range_usize(0, self.hand.len() - 1);
                    let card_idx = self.hand.remove(idx);
                    let dead_card = self.deck[card_idx].clone();
                    let card_id = dead_card.id;
                    self.notify_face_card_destroyed(&dead_card);
                    self.destroy_deck_card(card_id);
                    let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
                    let enhancements = [
                        Enhancement::Lucky, Enhancement::Gold,
                        Enhancement::Wild, Enhancement::Mult,
                        Enhancement::Bonus, Enhancement::Steel,
                        Enhancement::Glass, Enhancement::Stone,
                    ];
                    for _ in 0..2 {
                        let suit = suits[self.rng.range_usize(0, 3)];
                        let enh = enhancements[self.rng.range_usize(0, 7)];
                        let id = self.next_id();
                        let mut card = CardInstance::new(id, Rank::Ace, suit);
                        card.enhancement = enh;
                        let di = self.deck.len();
                        self.deck.push(card);
                        self.draw_pile.push(di);
                    }
                }
            }
            SpectralCard::Incantation => {
                // Destroy 1 random card in hand, add 4 random enhanced numbered cards to deck
                if !self.hand.is_empty() {
                    let idx = self.rng.range_usize(0, self.hand.len() - 1);
                    let card_idx = self.hand.remove(idx);
                    let dead_card = self.deck[card_idx].clone();
                    let card_id = dead_card.id;
                    self.notify_face_card_destroyed(&dead_card);
                    self.destroy_deck_card(card_id);
                    let number_ranks = [
                        Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
                        Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                    ];
                    let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
                    let enhancements = [
                        Enhancement::Lucky, Enhancement::Gold,
                        Enhancement::Wild, Enhancement::Mult,
                        Enhancement::Bonus, Enhancement::Steel,
                        Enhancement::Glass, Enhancement::Stone,
                    ];
                    for _ in 0..4 {
                        let rank = number_ranks[self.rng.range_usize(0, 8)];
                        let suit = suits[self.rng.range_usize(0, 3)];
                        let enh = enhancements[self.rng.range_usize(0, 7)];
                        let id = self.next_id();
                        let mut card = CardInstance::new(id, rank, suit);
                        card.enhancement = enh;
                        let di = self.deck.len();
                        self.deck.push(card);
                        self.draw_pile.push(di);
                    }
                }
            }
            SpectralCard::Talisman => {
                // Add a Gold Seal to 1 selected card
                if let Some(&hi) = targets.first() {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        self.deck[card_idx].seal = Seal::Gold;
                    }
                }
            }
            SpectralCard::Wraith => {
                // Creates a random Rare Joker; sets money to $0
                if self.jokers.len() < self.joker_slots as usize {
                    // Pick a random Rare joker
                    let rare_jokers = vec![
                        JokerKind::Dna, JokerKind::Vagabond, JokerKind::Baron,
                        JokerKind::Obelisk, JokerKind::BaseballCard, JokerKind::AncientJoker,
                        JokerKind::Campfire, JokerKind::Blueprint, JokerKind::WeeJoker,
                        JokerKind::HitTheRoad, JokerKind::TheDuo, JokerKind::TheTrio,
                        JokerKind::TheFamily, JokerKind::TheOrder, JokerKind::TheTribe,
                        JokerKind::Stuntman, JokerKind::InvisibleJoker, JokerKind::Brainstorm,
                        JokerKind::DriversLicense, JokerKind::BurntJoker,
                    ];
                    let idx = self.rng.range_usize(0, rare_jokers.len() - 1);
                    let kind = rare_jokers[idx];
                    let id = self.next_id();
                    self.jokers.push(JokerInstance::new(id, kind, Edition::None));
                }
                self.money = 0;
            }
            SpectralCard::Sigil => {
                // Convert all cards in hand to a single random suit
                let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
                let suit = suits[self.rng.range_usize(0, 3)];
                for &card_idx in &self.hand {
                    self.deck[card_idx].suit = suit;
                }
            }
            SpectralCard::Ouija => {
                // Convert all cards in hand to a single random rank; -1 hand size
                let ranks = [
                    Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
                    Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                    Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
                ];
                let rank = ranks[self.rng.range_usize(0, 12)];
                for &card_idx in &self.hand {
                    self.deck[card_idx].rank = rank;
                }
                self.hand_size = self.hand_size.saturating_sub(1);
            }
            SpectralCard::Cryptid => {
                // Create 2 copies of 1 selected card in hand
                if let Some(&hi) = targets.first() {
                    if hi < self.hand.len() {
                        let card_idx = self.hand[hi];
                        let template = self.deck[card_idx].clone();
                        for _ in 0..2 {
                            let id = self.next_id();
                            let mut copy = template.clone();
                            copy.id = id;
                            let di = self.deck.len();
                            self.deck.push(copy);
                            self.draw_pile.push(di);
                        }
                    }
                }
            }
            SpectralCard::TheSoul => {
                // Creates a Legendary Joker (requires open Joker slot)
                if self.jokers.len() < self.joker_slots as usize {
                    let legendaries = vec![
                        JokerKind::Canio, JokerKind::Triboulet, JokerKind::Yorick,
                        JokerKind::Chicot, JokerKind::Perkeo,
                    ];
                    let idx = self.rng.range_usize(0, legendaries.len() - 1);
                    let kind = legendaries[idx];
                    let id = self.next_id();
                    self.jokers.push(JokerInstance::new(id, kind, Edition::None));
                }
            }
            SpectralCard::BlackHole => {
                // Upgrade every poker hand by 1 level
                for level in self.hand_levels.values_mut() {
                    level.level += 1;
                }
            }
        }
        Ok(())
    }

    // =========================================================
    // Query methods
    // =========================================================
}
