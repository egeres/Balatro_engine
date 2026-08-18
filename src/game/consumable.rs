use crate::card::*;
use crate::types::*;
use super::{GameState, BalatroError, LastConsumable, rank_up};

/// The enhancements the card-creating spectrals roll from. Same set as [`Enhancement::ALL`], in
/// the order `create_card` walks it for these three (card.lua:1770).
const SPECTRAL_ENHANCEMENTS: [Enhancement; 8] = [
    Enhancement::Lucky, Enhancement::Gold, Enhancement::Wild, Enhancement::Mult,
    Enhancement::Bonus, Enhancement::Steel, Enhancement::Glass, Enhancement::Stone,
];

/// Card edits for [`GameState::change_targets`], named so the tarot table reads as a list of
/// what each card does rather than a wall of field assignments.
fn set_enhancement(enhancement: Enhancement) -> impl Fn(&mut CardInstance) {
    move |card| card.enhancement = enhancement
}

fn set_suit(suit: Suit) -> impl Fn(&mut CardInstance) {
    move |card| card.suit = suit
}

fn set_seal(seal: Seal) -> impl Fn(&mut CardInstance) {
    move |card| card.seal = seal
}

impl GameState {
    pub fn use_consumable(&mut self, consumable_index: usize, targets: Vec<usize>) -> Result<(), BalatroError> {
        if consumable_index >= self.consumables.len() {
            return Err(BalatroError::IndexOutOfRange(consumable_index, self.consumables.len()));
        }

        let consumable = self.consumables[consumable_index].card.clone();
        match &consumable {
            ConsumableCard::Planet(p) => {
                self.apply_planet(*p);
                self.planet_cards_used += 1;
                self.planet_types_used.insert(*p);
                // Constellation gains +0.1 Xmult per Planet card used (card.lua:2727)
                for i in self.joker_indices(JokerKind::Constellation) {
                    self.jokers[i].add_counter_f64("x_mult", 0.1);
                }
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

        let base_cost = self.consumables[consumable_index].base_cost();
        self.money += (base_cost / 2).max(1) as i32;
        self.consumables.remove(consumable_index);

        // Campfire counts every card sold, consumables included (card.lua:2394).
        self.notify_card_sold(None);
        Ok(())
    }

    /// Apply `change` to each of the first `max` cards the player selected, skipping any hand
    /// index that is out of range.
    ///
    /// Most tarots are exactly this: "enhance up to 2 selected cards", "convert up to 3 cards to
    /// Hearts", "add a Red Seal to 1 selected card".
    fn change_targets(&mut self, targets: &[usize], max: usize, change: impl Fn(&mut CardInstance)) {
        for &hand_idx in targets.iter().take(max) {
            if let Some(&card_idx) = self.hand.get(hand_idx) {
                change(&mut self.deck[card_idx]);
            }
        }
    }

    /// Apply `change` to every card currently in hand — what Sigil and Ouija do.
    fn change_whole_hand(&mut self, change: impl Fn(&mut CardInstance)) {
        for hand_idx in 0..self.hand.len() {
            let card_idx = self.hand[hand_idx];
            change(&mut self.deck[card_idx]);
        }
    }

    /// Destroy one random card in hand and replace it with `count` freshly generated ones — the
    /// shape Familiar, Grim and Incantation share, differing only in what they create.
    ///
    /// A single-entry `ranks` list is used as-is rather than rolled for: Grim always makes Aces,
    /// and rolling a one-way choice would burn a draw from the stream for nothing.
    fn destroy_one_and_create(&mut self, key: &str, count: usize, ranks: &[Rank]) {
        if self.hand.is_empty() {
            return;
        }
        let victim = self.rng.range_usize(key, 0, self.hand.len() - 1);
        let card_idx = self.hand.remove(victim);
        let dead_card = self.deck[card_idx].clone();
        self.notify_card_destroyed(&dead_card);
        self.destroy_deck_card(dead_card.id);

        for _ in 0..count {
            let rank = if ranks.len() == 1 {
                ranks[0]
            } else {
                ranks[self.rng.range_usize(key, 0, ranks.len() - 1)]
            };
            let suit = Suit::ALL[self.rng.range_usize(key, 0, Suit::ALL.len() - 1)];
            let n = SPECTRAL_ENHANCEMENTS.len() - 1;
            let enhancement = SPECTRAL_ENHANCEMENTS[self.rng.range_usize(key, 0, n)];
            let id = self.next_id();
            let mut card = CardInstance::new(id, rank, suit);
            card.enhancement = enhancement;
            self.add_card_to_draw_pile(card);
        }
        self.notify_playing_cards_added(count);
    }

    /// A random joker with no edition — the pool The Wheel of Fortune, Ectoplasm and Hex all draw
    /// from (`eligible_strength_jokers` / `eligible_editionless_jokers`, card.lua:4209, :4218).
    fn random_editionless_joker(&mut self) -> Option<usize> {
        let candidates: Vec<usize> = self
            .jokers
            .iter()
            .enumerate()
            .filter(|(_, j)| j.edition == Edition::None)
            .map(|(i, _)| i)
            .collect();
        if candidates.is_empty() {
            return None;
        }
        Some(candidates[self.rng.range_usize("editionless", 0, candidates.len() - 1)])
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
        self.log_event(
            "planet_used",
            serde_json::json!({
                "planet": format!("{:?}", planet),
                "hand_type": hand_type.display_name(),
            }),
        );
    }

    fn apply_tarot(&mut self, tarot: TarotCard, targets: &[usize]) -> Result<(), BalatroError> {
        // `targets` are hand indices, and every tarot below reads at most the first few.
        match tarot {
            // ── Enhance the selected cards ────────────────────────────────
            TarotCard::TheMagician => self.change_targets(targets, 2, set_enhancement(Enhancement::Lucky)),
            TarotCard::TheEmpress => self.change_targets(targets, 2, set_enhancement(Enhancement::Mult)),
            TarotCard::TheHierophant => self.change_targets(targets, 2, set_enhancement(Enhancement::Bonus)),
            TarotCard::TheLovers => self.change_targets(targets, 1, set_enhancement(Enhancement::Wild)),
            TarotCard::TheChariot => self.change_targets(targets, 1, set_enhancement(Enhancement::Steel)),
            TarotCard::Justice => self.change_targets(targets, 1, set_enhancement(Enhancement::Glass)),
            TarotCard::TheDevil => self.change_targets(targets, 1, set_enhancement(Enhancement::Gold)),
            TarotCard::TheTower => self.change_targets(targets, 1, set_enhancement(Enhancement::Stone)),

            // ── Convert up to 3 cards to one suit ─────────────────────────
            TarotCard::TheStar => self.change_targets(targets, 3, set_suit(Suit::Diamonds)),
            TarotCard::TheMoon => self.change_targets(targets, 3, set_suit(Suit::Clubs)),
            TarotCard::TheSun => self.change_targets(targets, 3, set_suit(Suit::Hearts)),
            TarotCard::TheWorld => self.change_targets(targets, 3, set_suit(Suit::Spades)),

            // ── Everything else ───────────────────────────────────────────
            TarotCard::Strength => {
                // Increase rank of up to 2 cards by 1
                self.change_targets(targets, 2, |c| c.rank = rank_up(c.rank));
            }
            TarotCard::TheHangedMan => {
                // Destroy up to 2 selected cards, highest hand index first so the removals do
                // not invalidate each other.
                let mut doomed: Vec<usize> = targets.iter().copied().take(2).collect();
                doomed.sort_unstable_by(|a, b| b.cmp(a));
                for hand_idx in doomed {
                    if hand_idx >= self.hand.len() {
                        continue;
                    }
                    let card_idx = self.hand.remove(hand_idx);
                    let dead_card = self.deck[card_idx].clone();
                    self.notify_card_destroyed(&dead_card);
                    self.destroy_deck_card(dead_card.id);
                }
            }
            TarotCard::TheHermit => {
                // Double money, gaining at most $20. Clamped at zero as well as at 20: a Credit
                // Card lets the balance go negative, and doubling a debt is not what "doubles
                // your money" means — the tarot can never cost you money.
                self.money += self.money.clamp(0, 20);
            }
            TarotCard::Temperance => {
                // Give money equal to sum of joker sell values (up to $50)
                let discount = self.discount_percent();
                let total: u32 = self.jokers.iter().map(|j| j.sell_value(discount)).sum();
                self.money += total.min(50) as i32;
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
                // 1/4 to add a random edition to a joker that has none (card.lua:4209).
                if self.roll_chance("wheel_of_fortune", 0.25) {
                    if let Some(idx) = self.random_editionless_joker() {
                        let edition = self.roll_positive_edition("wheel_of_fortune");
                        self.jokers[idx].edition = edition;
                    }
                }
            }
            TarotCard::Judgement => {
                // Create random joker
                if self.jokers.len() < self.effective_joker_slots() {
                    if let Some(j) = self.generate_random_joker() {
                        self.jokers.push(j);
                    }
                }
            }
            TarotCard::TheFool => {
                // Creates the most recently used Tarot or Planet card this run. The Fool itself
                // never counts as the "last used" consumable, hence the early return.
                if let Some(last) = self.last_consumable_used.clone() {
                    if self.has_room_for_consumable() {
                        self.add_consumable(match last {
                            LastConsumable::Tarot(t) => ConsumableCard::Tarot(t),
                            LastConsumable::Planet(p) => ConsumableCard::Planet(p),
                        });
                    }
                }
                return Ok(());
            }
            TarotCard::TheHighPriestess => {
                // Creates up to 2 random Planet cards (must have room)
                for _ in 0..2 {
                    if self.has_room_for_consumable() {
                        let planet = self.random_planet();
                        self.add_consumable(ConsumableCard::Planet(planet));
                    }
                }
            }
            TarotCard::TheEmperor => {
                // Creates up to 2 random Tarot cards (must have room)
                for _ in 0..2 {
                    self.create_tarot();
                }
            }
        }
        self.last_consumable_used = Some(LastConsumable::Tarot(tarot));
        Ok(())
    }

    fn apply_spectral(&mut self, spectral: SpectralCard, targets: &[usize]) -> Result<(), BalatroError> {
        match spectral {
            // Destroy 1 random card in hand, and replace it with several enhanced ones.
            SpectralCard::Familiar => {
                self.destroy_one_and_create("familiar_create", 3, &Rank::FACES)
            }
            SpectralCard::Ectoplasm => {
                // Negative on a random *editionless* joker (card.lua:4218), and a hand-size cost
                // that grows with every use: -1, then -2, then -3 (card.lua:1495).
                if let Some(idx) = self.random_editionless_joker() {
                    self.jokers[idx].edition = Edition::Negative;
                }
                self.hand_size = self.hand_size.saturating_sub(self.ectoplasm_uses + 1);
                self.ectoplasm_uses += 1;
            }
            SpectralCard::Aura => {
                // Add Foil/Holo/Poly to 1 selected card. Rolled only once there is a card to
                // put it on, so a mis-aimed Aura leaves the stream where it was.
                if targets.first().is_some_and(|&hi| hi < self.hand.len()) {
                    let edition = self.roll_positive_edition("aura");
                    self.change_targets(targets, 1, |c| c.edition = edition);
                }
            }
            SpectralCard::Hex => {
                // Polychrome on a random *editionless* joker, destroying the rest
                // (eternal jokers are spared).
                if let Some(idx) = self.random_editionless_joker() {
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
                    self.rng.shuffle("immolate", &mut hand_indices);
                    // Collect ids before any removal
                    let to_remove_ids: Vec<u64> = hand_indices[..count]
                        .iter()
                        .map(|&hi| self.deck[self.hand[hi]].id)
                        .collect();
                    // Notify Canio of any face cards being destroyed
                    for id in &to_remove_ids {
                        if let Some(card) = self.deck.iter().find(|c| c.id == *id).cloned() {
                            self.notify_card_destroyed(&card);
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
                // Negative edition is removed from the copy per wiki
                if !self.jokers.is_empty() {
                    let idx = self.rng.range_usize("ankh_choice", 0, self.jokers.len() - 1);
                    let chosen_id = self.jokers[idx].id;
                    let mut new_copy = self.jokers[idx].clone();
                    new_copy.id = self.next_id();
                    if new_copy.edition == Edition::Negative {
                        new_copy.edition = Edition::None;
                    }
                    // Retain: the eternal jokers + the copy (original is removed, copy is added below)
                    self.jokers.retain(|j| j.eternal && j.id != chosen_id);
                    self.jokers.push(new_copy);
                }
            }
            // Each of these stamps one seal onto a single selected card.
            SpectralCard::DejaVu => self.change_targets(targets, 1, set_seal(Seal::Red)),
            SpectralCard::Trance => self.change_targets(targets, 1, set_seal(Seal::Blue)),
            SpectralCard::Medium => self.change_targets(targets, 1, set_seal(Seal::Purple)),
            SpectralCard::Talisman => self.change_targets(targets, 1, set_seal(Seal::Gold)),

            SpectralCard::Grim => self.destroy_one_and_create("grim_create", 2, &[Rank::Ace]),
            SpectralCard::Incantation => {
                self.destroy_one_and_create("incantation_create", 4, &Rank::NUMBERS)
            }
            SpectralCard::Wraith => {
                // Creates a random Rare Joker; sets money to $0. Drawn from the live Rare pool,
                // so Showman, the enhancement gates and the duplicate rule all still apply.
                if self.jokers.len() < self.effective_joker_slots() {
                    let pool: Vec<JokerKind> = JokerKind::ALL
                        .iter()
                        .copied()
                        .filter(|k| k.rarity() == 3 && self.joker_in_pool(*k))
                        .collect();
                    if !pool.is_empty() {
                        let kind = pool[self.rng.range_usize("wraith", 0, pool.len() - 1)];
                        let id = self.next_id();
                        self.jokers.push(JokerInstance::new(id, kind, Edition::None));
                    }
                }
                self.money = 0;
            }
            SpectralCard::Sigil => {
                // Convert all cards in hand to a single random suit
                let suit = Suit::ALL[self.rng.range_usize("sigil", 0, Suit::ALL.len() - 1)];
                self.change_whole_hand(|c| c.suit = suit);
            }
            SpectralCard::Ouija => {
                // Convert all cards in hand to a single random rank; -1 hand size
                let rank = Rank::ALL[self.rng.range_usize("ouija", 0, Rank::ALL.len() - 1)];
                self.change_whole_hand(|c| c.rank = rank);
                self.hand_size = self.hand_size.saturating_sub(1);
            }
            SpectralCard::Cryptid => {
                // Create 2 copies of 1 selected card in hand
                let template = targets
                    .first()
                    .and_then(|&hi| self.hand.get(hi))
                    .map(|&card_idx| self.deck[card_idx].clone());
                if let Some(template) = template {
                    for _ in 0..2 {
                        let mut copy = template.clone();
                        copy.id = self.next_id();
                        self.add_card_to_draw_pile(copy);
                    }
                    self.notify_playing_cards_added(2);
                }
            }
            SpectralCard::TheSoul => {
                // Creates a Legendary Joker (requires open Joker slot). The legendary pool is
                // still subject to the duplicate rule — no second Perkeo without Showman.
                if self.jokers.len() < self.effective_joker_slots() {
                    let pool: Vec<JokerKind> = JokerKind::ALL
                        .iter()
                        .copied()
                        .filter(|k| self.legendary_in_pool(*k))
                        .collect();
                    if !pool.is_empty() {
                        let kind = pool[self.rng.range_usize("soul_", 0, pool.len() - 1)];
                        let id = self.next_id();
                        self.jokers.push(JokerInstance::new(id, kind, Edition::None));
                    }
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
