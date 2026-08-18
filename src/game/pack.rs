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
        let hallucinations = self
            .jokers
            .iter()
            .filter(|j| j.kind == JokerKind::Hallucination && j.active)
            .count();
        if hallucinations == 0 {
            return;
        }
        let oops = if self.jokers.iter().any(|j| j.kind == JokerKind::OopsAll6s && j.active) {
            2.0_f64
        } else {
            1.0_f64
        };
        for _ in 0..hallucinations {
            if self.consumables.len() >= self.consumable_slots as usize {
                break;
            }
            if self.rng.next_bool_prob("halu", (0.5 * oops).min(1.0)) {
                let tarot = self.random_tarot();
                self.consumables.push(ConsumableCard::Tarot(tarot));
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
                let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
                let ranks = [
                    Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
                    Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                    Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
                ];
                // card.lua:1759: 40% of the cards are Enhanced, editions are polled at a fixed
                // rate of 2, and 20% carry a seal (then uniform across the four).
                // Balatro keys these per ante (`pseudoseed('stdset'..ante)`).
                let stdset = crate::rng::keyed("stdset", self.ante);
                let stdseal = crate::rng::keyed("stdseal", self.ante);
                for _ in 0..cards_shown {
                    let suit_idx = self.rng.range_usize(&stdset, 0, 3);
                    let rank_idx = self.rng.range_usize(&stdset, 0, 12);
                    let id = self.next_id();
                    let mut card = CardInstance::new(id, ranks[rank_idx], suits[suit_idx]);

                    if self.rng.next_f64(&stdset) > 0.6 {
                        let enhancements = [
                            Enhancement::Bonus, Enhancement::Mult, Enhancement::Wild,
                            Enhancement::Glass, Enhancement::Steel, Enhancement::Stone,
                            Enhancement::Gold, Enhancement::Lucky,
                        ];
                        card.enhancement =
                            enhancements[self.rng.range_usize(&stdset, 0, enhancements.len() - 1)];
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
                // Add to deck
                let new_card = c.clone();
                let deck_idx = self.deck.len();
                self.deck.push(new_card);
                self.draw_pile.push(deck_idx);

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
                if self.consumables.len() < self.consumable_slots as usize {
                    self.consumables.push(c.clone());
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
