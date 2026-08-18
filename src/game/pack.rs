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
        planet_for_hand(hand_type)
    }

    /// Whether this pack is a Celestial one, in any size. Astronomer hands these out for free.
    pub(crate) fn is_celestial_pack(&self, kind: PackKind) -> bool {
        matches!(
            kind,
            PackKind::CelestialPackSmall
                | PackKind::CelestialPack
                | PackKind::CelestialPackJumbo
                | PackKind::CelestialPackMega
        )
    }

    /// Joker effects that fire the moment a booster pack is opened (`context.open_booster`,
    /// card.lua:2334) — once per pack, whatever kind it is, not once per card taken out of it.
    pub(crate) fn on_booster_opened(&mut self) {
        for _ in 0..self.count_joker(JokerKind::Hallucination) {
            if !self.has_room_for_consumable() {
                break;
            }
            if self.roll_chance("halu", 0.5) {
                self.create_tarot();
            }
        }
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
                    if omen_globe && self.rng.next_f64("omen_globe") > 0.8 {
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
                // card.lua:1759: 40% of the cards are Enhanced, editions are polled at a fixed
                // rate of 2, and 20% carry a seal (then uniform across the four).
                // Balatro keys these per ante (`pseudoseed('stdset'..ante)`).
                let stdset = crate::rng::keyed("stdset", self.ante);
                let stdseal = crate::rng::keyed("stdseal", self.ante);
                for _ in 0..cards_shown {
                    let suit = Suit::ALL[self.rng.range_usize(&stdset, 0, Suit::ALL.len() - 1)];
                    let rank = Rank::ALL[self.rng.range_usize(&stdset, 0, Rank::ALL.len() - 1)];
                    let id = self.next_id();
                    let mut card = CardInstance::new(id, rank, suit);

                    if self.rng.next_f64(&stdset) > 0.6 {
                        let n = Enhancement::ALL.len() - 1;
                        card.enhancement = Enhancement::ALL[self.rng.range_usize(&stdset, 0, n)];
                    }

                    // Standard packs pass a local rate of 2 as `_mod`, which multiplies with the
                    // run's own `edition_rate` (card.lua:1760), so Hone and Glow Up stack on top.
                    let rate = 2.0 * self.edition_rate;
                    card.edition = self.poll_edition_at_rate(rate, false);

                    if self.rng.next_f64(&stdseal) > 0.8 {
                        let seal_roll = self.rng.next_f64("stdsealtype");
                        card.seal = if seal_roll > 0.75 {
                            Seal::Red
                        } else if seal_roll > 0.5 {
                            Seal::Blue
                        } else if seal_roll > 0.25 {
                            Seal::Gold
                        } else {
                            Seal::Purple
                        };
                    }

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
                self.add_card_to_draw_pile(c.clone());
                self.notify_playing_cards_added(1);
            }
            PackCard::Joker(j) => {
                if self.jokers.len() < self.effective_joker_slots() {
                    self.jokers.push(j.clone());
                } else {
                    return Err(BalatroError::JokerSlotsFull);
                }
            }
            PackCard::Consumable(c) => {
                if self.has_room_for_consumable() {
                    self.add_consumable(c.clone());
                    // Note: planet_cards_used / tarot_cards_used are incremented in use_consumable,
                    // not here — counting on pick would double-count when the card is later used.
                } else {
                    return Err(BalatroError::ConsumableSlotsFull);
                }
            }
        }

        let pack = self.current_pack.as_mut().unwrap();
        pack.cards.remove(pack_index);
        pack.picks_remaining -= 1;

        // Taking the last pick closes the pack, but it is not a *skip* — Balatro only broadcasts
        // `skipping_booster` from the Skip button (button_callbacks.lua:2558), so Red Card gets
        // nothing here.
        if pack.picks_remaining == 0 {
            self.close_pack();
        }

        Ok(())
    }

    /// Walk away from the open pack, leaving whatever is still in it.
    pub fn skip_pack(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::BoosterPack) {
            return Err(BalatroError::NotInPack);
        }
        // Red Card feeds on skipped **booster packs** — `context.skipping_booster`
        // (card.lua:2441) — not on skipped blinds.
        for j in self.jokers.iter_mut() {
            if j.kind == JokerKind::RedCard && j.active {
                j.add_counter_i64("mult", 3);
            }
        }
        self.close_pack();
        Ok(())
    }

    /// Put the pack away and return to the shop, with none of a skip's side effects.
    fn close_pack(&mut self) {
        self.current_pack = None;
        self.state = GameStateKind::Shop;
    }

    // =========================================================
    // Consumable usage
    // =========================================================

}
