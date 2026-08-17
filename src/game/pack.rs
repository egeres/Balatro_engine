use crate::card::*;
use crate::types::*;
use super::{GameState, GameStateKind, BalatroError};

impl GameState {
    /// The Planet for the hand type played most this run, if any has been played
    /// (card.lua:1737). Used by Telescope.
    fn most_played_planet(&self) -> Option<PlanetCard> {
        let (&hand_type, _) = self
            .hand_levels
            .iter()
            .filter(|(_, h)| h.visible && h.played > 0)
            .max_by_key(|(ht, h)| (h.played, std::cmp::Reverse(ht.display_name())))?;
        [
            PlanetCard::Mercury, PlanetCard::Venus, PlanetCard::Earth, PlanetCard::Mars,
            PlanetCard::Jupiter, PlanetCard::Saturn, PlanetCard::Uranus, PlanetCard::Neptune,
            PlanetCard::Pluto, PlanetCard::PlanetX, PlanetCard::Ceres, PlanetCard::Eris,
        ]
        .into_iter()
        .find(|p| p.hand_type() == hand_type)
    }

    pub(crate) fn generate_pack_contents(&mut self, kind: PackKind) -> PackContents {
        let cards_shown = kind.cards_shown();
        let picks = kind.picks_allowed();
        let mut cards = Vec::new();

        match kind {
            PackKind::ArcanaPack
            | PackKind::ArcanaPackSmall
            | PackKind::ArcanaPackJumbo
            | PackKind::ArcanaPackMega => {
                // Omen Globe: each slot has a 1-in-5 chance of being a Spectral instead
                // (card.lua:1731).
                let omen_globe = self.has_voucher(VoucherKind::OmenGlobe);
                for _ in 0..cards_shown {
                    if omen_globe && self.rng.next_f64() > 0.8 {
                        let sp = self.random_spectral();
                        cards.push(PackCard::Consumable(ConsumableCard::Spectral(sp)));
                    } else {
                        let t = self.random_tarot();
                        cards.push(PackCard::Consumable(ConsumableCard::Tarot(t)));
                    }
                }
            }
            PackKind::CelestialPack
            | PackKind::CelestialPackSmall
            | PackKind::CelestialPackJumbo
            | PackKind::CelestialPackMega => {
                // Telescope: the first card is always the Planet for the most played hand
                // (card.lua:1737).
                let telescope_pick = if self.has_voucher(VoucherKind::Telescope) {
                    self.most_played_planet()
                } else {
                    None
                };
                for i in 0..cards_shown {
                    let p = match (i, telescope_pick) {
                        (0, Some(planet)) => planet,
                        _ => self.random_planet(),
                    };
                    cards.push(PackCard::Consumable(ConsumableCard::Planet(p)));
                }
            }
            PackKind::SpectralPack
            | PackKind::SpectralPackSmall
            | PackKind::SpectralPackJumbo
            | PackKind::SpectralPackMega => {
                for _ in 0..cards_shown {
                    let sp = self.random_spectral();
                    cards.push(PackCard::Consumable(ConsumableCard::Spectral(sp)));
                }
            }
            PackKind::BuffoonPack
            | PackKind::BuffoonPackSmall
            | PackKind::BuffoonPackJumbo
            | PackKind::BuffoonPackMega => {
                for _ in 0..cards_shown {
                    if let Some(j) = self.generate_random_joker() {
                        cards.push(PackCard::Joker(j));
                    }
                }
            }
            PackKind::StandardPack
            | PackKind::StandardPackSmall
            | PackKind::StandardPackJumbo
            | PackKind::StandardPackMega => {
                let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
                let ranks = [
                    Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
                    Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                    Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
                ];
                for _ in 0..cards_shown {
                    let suit_idx = self.rng.range_usize(0, 3);
                    let rank_idx = self.rng.range_usize(0, 12);
                    let id = self.next_id();
                    let mut card = CardInstance::new(id, ranks[rank_idx], suits[suit_idx]);
                    // Hone / Glow Up scale the edition chances via edition_rate.
                    card.edition = self.poll_edition(false);
                    cards.push(PackCard::PlayingCard(card));
                }
            }
        }

        PackContents {
            kind,
            cards,
            picks_remaining: picks,
        }
    }

    pub fn take_pack_card(&mut self, pack_index: usize) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::BoosterPack) {
            return Err(BalatroError::NotInPack);
        }
        let pack = match &self.current_pack {
            Some(p) => p,
            None => return Err(BalatroError::NotInPack),
        };
        if pack_index >= pack.cards.len() {
            return Err(BalatroError::IndexOutOfRange(pack_index, pack.cards.len()));
        }
        if pack.picks_remaining == 0 {
            return Err(BalatroError::NoPicksRemaining);
        }

        let card = self.current_pack.as_ref().unwrap().cards[pack_index].clone();

        match &card {
            PackCard::PlayingCard(c) => {
                // Add to deck
                let new_card = c.clone();
                let deck_idx = self.deck.len();
                self.deck.push(new_card);
                self.draw_pile.push(deck_idx);

                // Hologram joker: +0.25 Xmult
                for j in self.jokers.iter_mut() {
                    if j.kind == JokerKind::Hologram {
                        let cur = j.get_counter_f64("x_mult");
                        j.set_counter_f64("x_mult", cur + 0.25);
                    }
                }
            }
            PackCard::Joker(j) => {
                if self.jokers.len() < self.joker_slots as usize {
                    self.jokers.push(j.clone());
                } else {
                    return Err(BalatroError::JokerSlotsFull);
                }
            }
            PackCard::Consumable(c) => {
                if self.consumables.len() < self.consumable_slots as usize {
                    self.consumables.push(c.clone());
                    // Note: planet_cards_used / tarot_cards_used are incremented in use_consumable,
                    // not here — counting on pick would double-count when the card is later used.
                    // Apply planet/tarot immediately? No - user uses it separately via use_consumable

                    // Hallucination: 1/2 chance to create a tarot card when picking a consumable from a pack
                    // (guaranteed with OopsAll6s since 2 * 1/2 = 1)
                    if self.jokers.iter().any(|j| j.kind == JokerKind::Hallucination && j.active) {
                        let hall_oops = if self.jokers.iter().any(|j| j.kind == JokerKind::OopsAll6s && j.active) { 2.0_f64 } else { 1.0_f64 };
                        if self.rng.next_bool_prob((0.5 * hall_oops).min(1.0)) && self.consumables.len() < self.consumable_slots as usize {
                            let tarot = self.random_tarot();
                            self.consumables.push(ConsumableCard::Tarot(tarot));
                        }
                    }
                } else {
                    return Err(BalatroError::ConsumableSlotsFull);
                }
            }
        }

        let pack = self.current_pack.as_mut().unwrap();
        pack.cards.remove(pack_index);
        pack.picks_remaining -= 1;

        if pack.picks_remaining == 0 {
            self.skip_pack()?;
        }

        Ok(())
    }

    pub fn skip_pack(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::BoosterPack) {
            return Err(BalatroError::NotInPack);
        }
        self.current_pack = None;
        self.state = GameStateKind::Shop;
        Ok(())
    }

    // =========================================================
    // Consumable usage
    // =========================================================

}
