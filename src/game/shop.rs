use crate::card::*;
use crate::types::*;
use std::collections::HashMap;
use super::{GameState, GameStateKind, BlindKind, BalatroError, HistoryEvent, upgraded_voucher};

impl GameState {
    /// Booster packs offered per shop, with Balatro's weights (game.lua:665-692).
    /// Every family and size is reachable; Mega variants are rare.
    const PACK_POOL: [(PackKind, f64); 20] = [
        (PackKind::ArcanaPackSmall, 1.0),
        (PackKind::ArcanaPack, 3.0),
        (PackKind::ArcanaPackJumbo, 2.0),
        (PackKind::ArcanaPackMega, 0.5),
        (PackKind::CelestialPackSmall, 1.0),
        (PackKind::CelestialPack, 3.0),
        (PackKind::CelestialPackJumbo, 2.0),
        (PackKind::CelestialPackMega, 0.5),
        (PackKind::StandardPackSmall, 1.0),
        (PackKind::StandardPack, 3.0),
        (PackKind::StandardPackJumbo, 2.0),
        (PackKind::StandardPackMega, 0.5),
        (PackKind::BuffoonPackSmall, 0.6),
        (PackKind::BuffoonPack, 0.6),
        (PackKind::BuffoonPackJumbo, 0.6),
        (PackKind::BuffoonPackMega, 0.15),
        (PackKind::SpectralPackSmall, 0.3),
        (PackKind::SpectralPack, 0.3),
        (PackKind::SpectralPackJumbo, 0.3),
        (PackKind::SpectralPackMega, 0.07),
    ];

    /// Pick one entry from `(item, weight)` pairs.
    fn weighted_pick<T: Copy>(&mut self, pool: &[(T, f64)]) -> Option<T> {
        let total: f64 = pool.iter().map(|(_, w)| *w).sum();
        if total <= 0.0 {
            return None;
        }
        let mut roll = self.rng.next_f64() * total;
        for (item, weight) in pool {
            roll -= *weight;
            if roll <= 0.0 {
                return Some(*item);
            }
        }
        pool.last().map(|(item, _)| *item)
    }

    /// What a single shop card slot turns into, by the current rates (game.lua:1901).
    fn roll_shop_slot(&mut self) -> Option<ShopItem> {
        #[derive(Clone, Copy, PartialEq)]
        enum Slot {
            Joker,
            Tarot,
            Planet,
            Spectral,
            PlayingCard,
        }
        let pool = [
            (Slot::Joker, self.joker_rate),
            (Slot::Tarot, self.tarot_rate),
            (Slot::Planet, self.planet_rate),
            (Slot::Spectral, self.spectral_rate),
            (Slot::PlayingCard, self.playing_card_rate),
        ];
        match self.weighted_pick(&pool)? {
            Slot::Joker => self.generate_random_joker().map(ShopItem::Joker),
            Slot::Tarot => {
                let t = self.random_tarot();
                Some(ShopItem::Consumable(ConsumableCard::Tarot(t)))
            }
            Slot::Planet => {
                let p = self.random_planet();
                Some(ShopItem::Consumable(ConsumableCard::Planet(p)))
            }
            Slot::Spectral => {
                let sp = self.random_spectral();
                Some(ShopItem::Consumable(ConsumableCard::Spectral(sp)))
            }
            Slot::PlayingCard => {
                let c = self.random_playing_card();
                Some(ShopItem::PlayingCard(c))
            }
        }
    }

    /// A random playing card for the shop. Illusion lets it carry an enhancement, an edition and
    /// a seal 40% of the time (UI_definitions.lua:772).
    pub(crate) fn random_playing_card(&mut self) -> CardInstance {
        let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
        let ranks = [
            Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven,
            Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
        ];
        let suit = suits[self.rng.range_usize(0, 3)];
        let rank = ranks[self.rng.range_usize(0, 12)];
        let id = self.next_id();
        let mut card = CardInstance::new(id, rank, suit);

        if self.has_voucher(VoucherKind::Illusion) && self.rng.next_f64() > 0.6 {
            let enhancements = [
                Enhancement::Bonus, Enhancement::Mult, Enhancement::Wild, Enhancement::Glass,
                Enhancement::Steel, Enhancement::Stone, Enhancement::Gold, Enhancement::Lucky,
            ];
            card.enhancement = enhancements[self.rng.range_usize(0, enhancements.len() - 1)];
            card.edition = self.poll_edition(false);
            let seals = [Seal::None, Seal::Gold, Seal::Red, Seal::Blue, Seal::Purple];
            card.seal = seals[self.rng.range_usize(0, seals.len() - 1)];
        }
        card
    }

    /// Roll an edition using the run's `edition_rate` (common_events.lua:2069).
    /// Hone doubles the Foil/Holo/Poly chances, Glow Up quadruples them.
    pub(crate) fn poll_edition(&mut self, allow_negative: bool) -> Edition {
        let rate = self.edition_rate;
        self.poll_edition_at_rate(rate, allow_negative)
    }

    /// Poll an edition at an explicit rate. Standard packs use a fixed 2 rather than the run's
    /// `edition_rate` (card.lua:1761).
    pub(crate) fn poll_edition_at_rate(&mut self, rate: f64, allow_negative: bool) -> Edition {
        let roll = self.rng.next_f64();
        if allow_negative && roll > 1.0 - 0.003 {
            Edition::Negative
        } else if roll > 1.0 - 0.006 * rate {
            Edition::Polychrome
        } else if roll > 1.0 - 0.02 * rate {
            Edition::Holographic
        } else if roll > 1.0 - 0.04 * rate {
            Edition::Foil
        } else {
            Edition::None
        }
    }

    /// The Soul and Black Hole are hidden - they never come out of a normal Spectral roll.
    pub(crate) fn random_spectral(&mut self) -> SpectralCard {
        const POOL: [SpectralCard; 16] = [
            SpectralCard::Familiar, SpectralCard::Grim, SpectralCard::Incantation,
            SpectralCard::Talisman, SpectralCard::Aura, SpectralCard::Wraith,
            SpectralCard::Sigil, SpectralCard::Ouija, SpectralCard::Ectoplasm,
            SpectralCard::Immolate, SpectralCard::Ankh, SpectralCard::DejaVu,
            SpectralCard::Hex, SpectralCard::Trance, SpectralCard::Medium,
            SpectralCard::Cryptid,
        ];
        POOL[self.rng.range_usize(0, POOL.len() - 1)]
    }

    /// Consume the tags that act on a shop (tag.lua:344, :382, :393, :447).
    ///
    /// Uncommon/Rare force the rarity of one stocked joker, the edition tags stamp an edition on
    /// one and make it free, Coupon makes everything already on the shelf free, D6 zeroes the
    /// reroll price, and Voucher adds a second voucher slot.
    fn apply_shop_tags(&mut self) {
        let pending: Vec<TagKind> = self
            .tags
            .iter()
            .copied()
            .filter(|t| t.trigger() == TagTrigger::Shop)
            .collect();
        if pending.is_empty() {
            return;
        }
        self.tags.retain(|t| t.trigger() != TagTrigger::Shop);

        for tag in pending {
            match tag {
                TagKind::Coupon => {
                    self.shop_is_free = true;
                    for offer in self.shop_offers.iter_mut() {
                        offer.price = 0;
                    }
                }
                TagKind::D6 => {
                    self.shop_rerolls_free = true;
                    self.reroll_cost = 0;
                }
                TagKind::Voucher => {
                    // A second voucher on offer this shop.
                    let v = self.random_voucher();
                    self.shop_offers.push(ShopOffer {
                        kind: ShopItem::Voucher(v),
                        price: 10,
                        sold: false,
                    });
                }
                _ => {
                    if let Some(rarity) = tag.forced_rarity() {
                        self.force_shop_joker(Some(rarity), None);
                    } else if let Some(edition) = tag.forced_edition() {
                        self.force_shop_joker(None, Some(edition));
                    }
                }
            }
        }
    }

    /// Replace one stocked joker (or add one) so it matches the rarity / edition a tag promised.
    /// Edition tags also hand the joker over for free.
    fn force_shop_joker(&mut self, rarity: Option<u8>, edition: Option<Edition>) {
        let target = rarity.unwrap_or(1);
        let pool: Vec<JokerKind> = JokerKind::ALL
            .iter()
            .copied()
            .filter(|k| self.joker_in_pool(*k) && (rarity.is_none() || k.rarity() == target))
            .collect();

        let slot = self
            .shop_offers
            .iter()
            .position(|o| matches!(o.kind, ShopItem::Joker(_)) && !o.sold);

        if let Some(edition) = edition {
            // Stamp the edition on a joker already for sale if there is one.
            if let Some(idx) = slot {
                if let ShopItem::Joker(j) = &mut self.shop_offers[idx].kind {
                    j.edition = edition;
                }
                self.shop_offers[idx].price = 0;
                return;
            }
        }
        if pool.is_empty() {
            return;
        }
        let kind = pool[self.rng.range_usize(0, pool.len() - 1)];
        let id = self.next_id();
        let mut joker = JokerInstance::new(id, kind, edition.unwrap_or(Edition::None));
        joker.edition = edition.unwrap_or(Edition::None);
        let price = if edition.is_some() { 0 } else { kind.base_cost() };

        match slot {
            Some(idx) => {
                self.shop_offers[idx] = ShopOffer { kind: ShopItem::Joker(joker), price, sold: false };
            }
            None => {
                self.shop_offers.push(ShopOffer { kind: ShopItem::Joker(joker), price, sold: false });
            }
        }
    }

    pub(crate) fn generate_shop(&mut self) {
        // Card slots: base 2, +1 per Overstock voucher (game.lua:1885).
        let card_slots = 2
            + if self.has_voucher(VoucherKind::Overstock) { 1 } else { 0 }
            + if self.has_voucher(VoucherKind::OverstockPlus) { 1 } else { 0 };

        // Cleared up front and filled in place, so each joker rolled is visible to the pool
        // filter and cannot be rolled twice in the same shop.
        self.shop_offers.clear();

        for _ in 0..card_slots {
            if let Some(item) = self.roll_shop_slot() {
                let price = match &item {
                    ShopItem::Joker(j) => self.joker_shop_price(j),
                    ShopItem::Consumable(c) => c.base_cost(),
                    ShopItem::PlayingCard(_) => 1,
                    ShopItem::Pack(p) => p.base_cost(),
                    ShopItem::Voucher(_) => 10,
                };
                self.shop_offers.push(ShopOffer { kind: item, price, sold: false });
            }
        }

        // Two booster pack slots, drawn from the full weighted pool.
        for _ in 0..2 {
            if let Some(pack) = self.weighted_pick(&Self::PACK_POOL) {
                let price = pack.base_cost();
                self.shop_offers.push(ShopOffer {
                    kind: ShopItem::Pack(pack),
                    price,
                    sold: false,
                });
            }
        }

        self.shop_voucher = Some(self.random_voucher());
        self.reroll_cost = self.base_reroll_cost;
        self.apply_shop_tags();

        // ChaosTheClown: +1 free reroll per shop visit
        let chaos_count = self.jokers.iter().filter(|j| j.kind == JokerKind::ChaosTheClown && j.active).count();
        self.free_rerolls += chaos_count as u32;

        // Egg: gains $3 sell value each time the shop is visited
        for j in self.jokers.iter_mut() {
            if j.kind == JokerKind::Egg && j.active {
                let cur = j.get_counter_i64("sell_bonus");
                j.set_counter_i64("sell_bonus", cur + 3);
            }
        }

        // GiftCard: +$1 sell value to all other jokers held
        let has_gift_card = self.jokers.iter().any(|j| j.kind == JokerKind::GiftCard && j.active);
        if has_gift_card {
            for j in self.jokers.iter_mut() {
                if j.kind != JokerKind::GiftCard {
                    let cur = j.get_counter_i64("sell_bonus");
                    j.set_counter_i64("sell_bonus", cur + 1);
                }
            }
        }
    }

    /// Whether `kind` can currently appear in a shop / pack / random-joker roll.
    ///
    /// Mirrors the pool gates in game.lua: legendaries are Soul-only, `enhancement_gate` jokers
    /// need the matching enhancement somewhere in the deck, and Gros Michel / Cavendish swap
    /// places once Gros Michel has gone extinct (`no_pool_flag` / `yes_pool_flag`).
    pub(crate) fn joker_in_pool(&self, kind: JokerKind) -> bool {
        if kind.rarity() == 4 {
            return false;
        }
        // Balatro flags a joker key as used while a copy of it exists anywhere — owned, sitting in
        // the shop, or inside an open pack (card.lua:352, cleared at :4745) — and only Showman
        // lifts the restriction (common_events.lua:1987).
        if !self.jokers.iter().any(|j| j.kind == JokerKind::Showman && j.active)
            && self.joker_kind_in_play(kind)
        {
            return false;
        }
        let deck_has = |e: Enhancement| self.deck.iter().any(|c| c.enhancement == e);
        match kind {
            JokerKind::SteelJoker => deck_has(Enhancement::Steel),
            JokerKind::StoneJoker => deck_has(Enhancement::Stone),
            JokerKind::GoldenTicket => deck_has(Enhancement::Gold),
            JokerKind::LuckyCat => deck_has(Enhancement::Lucky),
            JokerKind::GlassJoker => deck_has(Enhancement::Glass),
            JokerKind::GrosMichel => !self.gros_michel_extinct,
            JokerKind::Cavendish => self.gros_michel_extinct,
            _ => true,
        }
    }

    /// Whether a copy of `kind` currently exists: held, offered in the shop, or in an open pack.
    fn joker_kind_in_play(&self, kind: JokerKind) -> bool {
        if self.jokers.iter().any(|j| j.kind == kind) {
            return true;
        }
        if self.shop_offers.iter().any(|o| match &o.kind {
            ShopItem::Joker(j) => j.kind == kind && !o.sold,
            _ => false,
        }) {
            return true;
        }
        if let Some(pack) = &self.current_pack {
            if pack.cards.iter().any(|c| match c {
                PackCard::Joker(j) => j.kind == kind,
                _ => false,
            }) {
                return true;
            }
        }
        false
    }

    pub(crate) fn generate_random_joker(&mut self) -> Option<JokerInstance> {
        // Rarity is rolled first, then a joker is drawn uniformly from that tier:
        // 70% Common / 25% Uncommon / 5% Rare. Legendaries only come from The Soul.
        let roll = self.rng.next_f64();
        let rarity: u8 = if roll < 0.70 {
            1
        } else if roll < 0.95 {
            2
        } else {
            3
        };

        let mut pool: Vec<JokerKind> = JokerKind::ALL
            .iter()
            .copied()
            .filter(|k| k.rarity() == rarity && self.joker_in_pool(*k))
            .collect();
        // Fall back to the whole pool if the rolled tier is empty (every Rare gated out, say).
        if pool.is_empty() {
            pool = JokerKind::ALL
                .iter()
                .copied()
                .filter(|k| self.joker_in_pool(*k))
                .collect();
        }
        if pool.is_empty() {
            return None;
        }

        let idx = self.rng.range_usize(0, pool.len() - 1);
        let kind = pool[idx];
        let id = self.next_id();

        // Random edition
        let edition_roll = self.rng.next_f64();
        let edition = if edition_roll < 0.003 {
            Edition::Negative
        } else if edition_roll < 0.006 {
            Edition::Polychrome
        } else if edition_roll < 0.02 {
            Edition::Holographic
        } else if edition_roll < 0.04 {
            Edition::Foil
        } else {
            Edition::None
        };

        let mut joker = JokerInstance::new(id, kind, edition);

        // Stake-based stickers (each 30% chance at the relevant stake threshold).
        // Eternal (Black+) and Perishable (Orange+) are mutually exclusive; Eternal wins if both
        // would trigger. Rental (Gold+) is independent and can combine with either.
        // A joker only takes a sticker its definition allows (`eternal_compat` /
        // `perishable_compat`, card.lua:517).
        let stake_level = self.stake as u8;
        if stake_level >= Stake::Black as u8
            && kind.eternal_compat()
            && self.rng.next_bool_prob(0.30)
        {
            joker.eternal = true;
        } else if stake_level >= Stake::Orange as u8
            && kind.perishable_compat()
            && self.rng.next_bool_prob(0.30)
        {
            joker.perishable = true;
        }
        if stake_level >= Stake::Gold as u8 && self.rng.next_bool_prob(0.30) {
            joker.rental = true;
        }

        Some(joker)
    }

    fn random_voucher(&mut self) -> VoucherKind {
        // Only offer base-tier vouchers (upgraded versions require buying the base first)
        let base_vouchers = vec![
            VoucherKind::Overstock,
            VoucherKind::ClearanceSale,
            VoucherKind::Hone,
            VoucherKind::RerollSurplus,
            VoucherKind::CrystalBall,
            VoucherKind::Telescope,
            VoucherKind::Grabber,
            VoucherKind::Wasteful,
            VoucherKind::TarotMerchant,
            VoucherKind::PlanetMerchant,
            VoucherKind::SeedMoney,
            VoucherKind::Blank,
            VoucherKind::MagicTrick,
            VoucherKind::Hieroglyph,
            VoucherKind::DirectorsCut,
            VoucherKind::PaintBrush,
        ];
        // If the player already has the base, offer the upgrade
        let available: Vec<VoucherKind> = base_vouchers
            .iter()
            .flat_map(|&base| {
                if self.vouchers.contains(&base) {
                    vec![upgraded_voucher(base)]
                } else if !self.vouchers.contains(&base) {
                    vec![base]
                } else {
                    vec![]
                }
            })
            .filter(|v| !self.vouchers.contains(v))
            .collect();
        if available.is_empty() {
            return VoucherKind::Overstock;
        }
        let idx = self.rng.range_usize(0, available.len() - 1);
        available[idx]
    }

    pub(crate) fn random_tarot(&mut self) -> TarotCard {
        let tarots = vec![
            TarotCard::TheFool,
            TarotCard::TheMagician,
            TarotCard::TheHighPriestess,
            TarotCard::TheEmpress,
            TarotCard::TheEmperor,
            TarotCard::TheHierophant,
            TarotCard::TheLovers,
            TarotCard::TheChariot,
            TarotCard::Justice,
            TarotCard::TheHermit,
            TarotCard::TheWheelOfFortune,
            TarotCard::Strength,
            TarotCard::TheHangedMan,
            TarotCard::Death,
            TarotCard::Temperance,
            TarotCard::TheDevil,
            TarotCard::TheTower,
            TarotCard::TheStar,
            TarotCard::TheMoon,
            TarotCard::TheSun,
            TarotCard::Judgement,
            TarotCard::TheWorld,
        ];
        let idx = self.rng.range_usize(0, tarots.len() - 1);
        tarots[idx]
    }

    pub(crate) fn random_planet(&mut self) -> PlanetCard {
        let mut planets = vec![
            PlanetCard::Mercury,
            PlanetCard::Venus,
            PlanetCard::Earth,
            PlanetCard::Mars,
            PlanetCard::Jupiter,
            PlanetCard::Saturn,
            PlanetCard::Uranus,
            PlanetCard::Neptune,
            PlanetCard::Pluto,
        ];
        // Secret planets only available after playing the corresponding hand type
        if self.hand_levels.get(&HandType::FiveOfAKind).map(|h| h.played > 0).unwrap_or(false) {
            planets.push(PlanetCard::PlanetX);
        }
        if self.hand_levels.get(&HandType::FlushHouse).map(|h| h.played > 0).unwrap_or(false) {
            planets.push(PlanetCard::Ceres);
        }
        if self.hand_levels.get(&HandType::FlushFive).map(|h| h.played > 0).unwrap_or(false) {
            planets.push(PlanetCard::Eris);
        }
        let idx = self.rng.range_usize(0, planets.len() - 1);
        planets[idx]
    }

    pub fn has_voucher(&self, v: VoucherKind) -> bool {
        self.vouchers.contains(&v)
    }

    pub fn buy_joker(&mut self, shop_index: usize) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }
        if shop_index >= self.shop_offers.len() {
            return Err(BalatroError::IndexOutOfRange(shop_index, self.shop_offers.len()));
        }
        let offer = &self.shop_offers[shop_index];
        if offer.sold {
            return Err(BalatroError::AlreadySold);
        }
        if !matches!(offer.kind, ShopItem::Joker(_)) {
            return Err(BalatroError::WrongItemType("Expected joker".to_string()));
        }

        // Calculate price with voucher discounts
        let price = self.calculate_shop_price(offer.price);
        if !self.can_afford(price as i32) {
            return Err(BalatroError::NotEnoughMoney(price, self.money.max(0) as u32));
        }
        if self.jokers.len() >= self.effective_joker_slots() {
            return Err(BalatroError::JokerSlotsFull);
        }

        self.money -= price as i32;
        if let ShopItem::Joker(j) = &self.shop_offers[shop_index].kind.clone() {
            // A Negative joker brings its own slot; effective_joker_slots() derives that from
            // the board, so nothing needs adding here.
            self.jokers.push(j.clone());
        }
        self.shop_offers[shop_index].sold = true;
        Ok(())
    }

    pub fn sell_joker(&mut self, joker_index: usize) -> Result<(), BalatroError> {
        if joker_index >= self.jokers.len() {
            return Err(BalatroError::IndexOutOfRange(joker_index, self.jokers.len()));
        }
        if self.jokers[joker_index].eternal {
            return Err(BalatroError::EternalCard);
        }

        let sell_value = self.jokers[joker_index].sell_value();
        self.money += sell_value as i32;

        // Campfire joker: +0.25 Xmult when joker is sold
        for i in 0..self.jokers.len() {
            if self.jokers[i].kind == JokerKind::Campfire {
                let cur = self.jokers[i].get_counter_f64("x_mult");
                self.jokers[i].set_counter_f64("x_mult", cur + 0.25);
            }
        }

        // Luchador: selling it disables the current Boss blind's ability for the rest of the round
        if self.jokers[joker_index].kind == JokerKind::Luchador
            && matches!(self.current_blind, BlindKind::Boss)
            && !self.boss_blind_disabled()
        {
            self.boss_blind_manually_disabled = true;
            // Debuffs were applied when the round began; lift them now that the blind is off.
            for card in self.deck.iter_mut() {
                card.debuffed = false;
            }
        }

        // InvisibleJoker: when sold after 2+ rounds, duplicate a random other joker
        let is_invisible = self.jokers[joker_index].kind == JokerKind::InvisibleJoker;
        let invisible_rounds = self.jokers[joker_index].get_counter_i64("rounds");

        self.jokers.remove(joker_index);

        if is_invisible && invisible_rounds >= 2 && self.jokers.len() < self.effective_joker_slots() {
            let candidates: Vec<usize> = (0..self.jokers.len())
                .filter(|&j| self.jokers[j].active && self.jokers[j].kind != JokerKind::InvisibleJoker)
                .collect();
            if !candidates.is_empty() {
                let pick = self.rng.range_usize(0, candidates.len() - 1);
                let dup = self.jokers[candidates[pick]].clone();
                self.jokers.push(dup);
            }
        }

        // VerdantLeaf: first joker sold lifts the all-cards-debuffed effect
        if let Some(BossBlind::VerdantLeaf) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) && !self.verdant_leaf_joker_sold {
                self.verdant_leaf_joker_sold = true;
                for card in self.deck.iter_mut() {
                    card.debuffed = false;
                }
            }
        }

        Ok(())
    }

    pub fn buy_consumable(&mut self, shop_index: usize) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }
        if shop_index >= self.shop_offers.len() {
            return Err(BalatroError::IndexOutOfRange(shop_index, self.shop_offers.len()));
        }
        let offer = &self.shop_offers[shop_index];
        if offer.sold {
            return Err(BalatroError::AlreadySold);
        }
        if !matches!(offer.kind, ShopItem::Consumable(_)) {
            return Err(BalatroError::WrongItemType("Expected consumable".to_string()));
        }
        if self.consumables.len() >= self.consumable_slots as usize {
            return Err(BalatroError::ConsumableSlotsFull);
        }

        let base_price = self.calculate_shop_price(offer.price);
        // Astronomer: planet cards are free
        let price = if self.jokers.iter().any(|j| j.kind == JokerKind::Astronomer && j.active) {
            if let ShopItem::Consumable(ConsumableCard::Planet(_)) = &self.shop_offers[shop_index].kind {
                0
            } else {
                base_price
            }
        } else {
            base_price
        };
        if !self.can_afford(price as i32) {
            return Err(BalatroError::NotEnoughMoney(price, self.money.max(0) as u32));
        }

        self.money -= price as i32;
        if let ShopItem::Consumable(c) = self.shop_offers[shop_index].kind.clone() {
            self.consumables.push(c);
        }
        self.shop_offers[shop_index].sold = true;
        Ok(())
    }

    pub fn buy_pack(&mut self, shop_index: usize) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }
        if shop_index >= self.shop_offers.len() {
            return Err(BalatroError::IndexOutOfRange(shop_index, self.shop_offers.len()));
        }
        let offer = &self.shop_offers[shop_index];
        if offer.sold {
            return Err(BalatroError::AlreadySold);
        }
        let pack_kind = match &offer.kind {
            ShopItem::Pack(p) => *p,
            _ => return Err(BalatroError::WrongItemType("Expected pack".to_string())),
        };

        let price = self.calculate_shop_price(offer.price);
        if !self.can_afford(price as i32) {
            return Err(BalatroError::NotEnoughMoney(price, self.money.max(0) as u32));
        }

        self.money -= price as i32;
        self.shop_offers[shop_index].sold = true;

        // Generate pack contents
        let contents = self.generate_pack_contents(pack_kind);
        self.current_pack = Some(contents);
        self.state = GameStateKind::BoosterPack;
        Ok(())
    }

    pub fn buy_voucher(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }
        let voucher = match self.shop_voucher {
            Some(v) => v,
            None => return Err(BalatroError::NoVoucherAvailable),
        };

        let price = self.calculate_shop_price(10);
        if !self.can_afford(price as i32) {
            return Err(BalatroError::NotEnoughMoney(price, self.money.max(0) as u32));
        }

        self.money -= price as i32;
        self.apply_voucher(voucher);
        self.vouchers.push(voucher);
        self.shop_voucher = None;
        Ok(())
    }

    fn apply_voucher(&mut self, voucher: VoucherKind) {
        match voucher {
            VoucherKind::Overstock | VoucherKind::OverstockPlus => {
                // Extra shop card slots — handled in generate_shop
            }
            VoucherKind::ClearanceSale | VoucherKind::Liquidation => {
                // Price discounts — handled in calculate_shop_price
            }
            VoucherKind::Hone => {
                // Foil/Holo/Poly appear twice as often (card.lua:1900).
                self.edition_rate = 2.0;
            }
            VoucherKind::GlowUp => {
                self.edition_rate = 4.0;
            }
            VoucherKind::RerollSurplus | VoucherKind::RerollGlut => {
                // Rerolls cost $2 less each (card.lua:1925).
                self.base_reroll_cost = self.base_reroll_cost.saturating_sub(2);
                self.reroll_cost = self.reroll_cost.saturating_sub(2);
            }
            VoucherKind::CrystalBall => {
                self.consumable_slots += 1;
            }
            VoucherKind::OmenGlobe => {
                // Spectral cards can appear in Arcana packs — handled in generate_pack_contents
            }
            VoucherKind::Telescope => {
                // Celestial packs show 1 extra card — handled in generate_pack_contents
            }
            VoucherKind::Observatory => {
                // Planet cards used give +0.5 Xmult — handled in apply_planet
            }
            VoucherKind::Grabber => {
                self.max_hands += 1;
                self.hands_remaining = self.hands_remaining.saturating_add(1);
            }
            VoucherKind::NachoTong => {
                self.max_hands += 1;
                self.hands_remaining = self.hands_remaining.saturating_add(1);
            }
            VoucherKind::Wasteful => {
                self.max_discards += 1;
                self.discards_remaining = self.discards_remaining.saturating_add(1);
            }
            VoucherKind::Recyclomancy => {
                self.max_discards += 1;
                self.discards_remaining = self.discards_remaining.saturating_add(1);
            }
            // These raise how often the card type shows up in the shop, they are not discounts
            // (card.lua:1890, `tarot_rate = 4 * extra`).
            VoucherKind::TarotMerchant => self.tarot_rate = 9.6,
            VoucherKind::TarotTycoon => self.tarot_rate = 32.0,
            VoucherKind::PlanetMerchant => self.planet_rate = 9.6,
            VoucherKind::PlanetTycoon => self.planet_rate = 32.0,
            VoucherKind::SeedMoney => {
                // Sets the cap outright rather than adding to it (game.lua:602).
                self.max_interest = self.max_interest.max(50);
            }
            VoucherKind::MoneyTree => {
                self.max_interest = self.max_interest.max(100);
            }
            VoucherKind::Blank => {
                // "Does nothing?" (en-us.lua:2988). It exists only to unlock Antimatter.
            }
            VoucherKind::Antimatter => {
                self.joker_slots += 1;
            }
            VoucherKind::MagicTrick | VoucherKind::Illusion => {
                // Loose playing cards start appearing in the shop (card.lua:1905).
                // Illusion additionally lets them carry enhancements/editions/seals.
                self.playing_card_rate = 4.0;
            }
            VoucherKind::Hieroglyph => {
                // -1 Ante, at the cost of a hand each round (card.lua:1957).
                self.ante = self.ante.saturating_sub(1).max(1);
                self.max_hands = self.max_hands.saturating_sub(1);
                self.hands_remaining = self.hands_remaining.saturating_sub(1);
            }
            VoucherKind::Petroglyph => {
                // -1 Ante, at the cost of a discard each round.
                self.ante = self.ante.saturating_sub(1).max(1);
                self.max_discards = self.max_discards.saturating_sub(1);
                self.discards_remaining = self.discards_remaining.saturating_sub(1);
            }
            VoucherKind::DirectorsCut => {
                // Unlocks a paid Boss blind reroll; no immediate effect.
            }
            VoucherKind::Retcon => {
                // Unlimited paid Boss blind rerolls; no immediate effect.
            }
            VoucherKind::PaintBrush => {
                self.hand_size += 1;
            }
            VoucherKind::Palette => {
                self.hand_size += 1;
            }
        }
    }

    /// `Card:set_cost` (card.lua:368): editions add a flat surcharge, then the discount applies,
    /// with a +0.5 nudge before the floor.
    fn calculate_shop_price(&self, base_price: u32) -> u32 {
        self.calculate_shop_price_with_edition(base_price, Edition::None, false)
    }

    fn calculate_shop_price_with_edition(
        &self,
        base_price: u32,
        edition: Edition,
        rental: bool,
    ) -> u32 {
        // A rental costs a flat $1 whatever it is (card.lua:381).
        if rental {
            return 1;
        }
        let extra = match edition {
            Edition::Foil => 2.0,
            Edition::Holographic => 3.0,
            Edition::Polychrome => 5.0,
            Edition::Negative => 5.0,
            Edition::None => 0.0,
        };
        let discount_percent = if self.has_voucher(VoucherKind::Liquidation) {
            50.0
        } else if self.has_voucher(VoucherKind::ClearanceSale) {
            25.0
        } else {
            0.0
        };
        let cost = ((base_price as f64 + extra + 0.5) * (100.0 - discount_percent) / 100.0).floor();
        (cost.max(1.0)) as u32
    }

    /// Test hook for the pricing rule.
    pub fn debug_joker_price(&self, joker: &JokerInstance) -> u32 {
        self.joker_shop_price(joker)
    }

    /// The shop price of a joker, including its edition surcharge and rental discount.
    fn joker_shop_price(&self, joker: &JokerInstance) -> u32 {
        self.calculate_shop_price_with_edition(
            joker.kind.base_cost(),
            joker.edition,
            joker.rental,
        )
    }

    pub fn reroll_shop(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }

        let actual_cost = if self.shop_rerolls_free {
            0
        } else if self.free_rerolls > 0 {
            self.free_rerolls -= 1;
            0
        } else {
            self.reroll_cost
        };

        if !self.can_afford(actual_cost as i32) {
            return Err(BalatroError::NotEnoughMoney(actual_cost, self.money.max(0) as u32));
        }

        self.money -= actual_cost as i32;
        self.reroll_cost += 1; // Escalates by $1 per reroll within a shop

        // Regenerate joker offers
        self.generate_shop();

        // Flash Card joker: +2 mult per reroll
        for j in self.jokers.iter_mut() {
            if j.kind == JokerKind::FlashCard {
                let cur = j.get_counter_i64("mult");
                j.set_counter_i64("mult", cur + 2);
            }
        }

        Ok(())
    }

    /// Buy a loose playing card from the shop and add it to the deck.
    /// Only reachable once Magic Trick has been redeemed.
    pub fn buy_playing_card(&mut self, shop_index: usize) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }
        if shop_index >= self.shop_offers.len() {
            return Err(BalatroError::IndexOutOfRange(shop_index, self.shop_offers.len()));
        }
        let offer = &self.shop_offers[shop_index];
        if offer.sold {
            return Err(BalatroError::AlreadySold);
        }
        let ShopItem::PlayingCard(card) = offer.kind.clone() else {
            return Err(BalatroError::WrongItemType("Expected playing card".to_string()));
        };
        let price = self.calculate_shop_price(offer.price);
        if !self.can_afford(price as i32) {
            return Err(BalatroError::NotEnoughMoney(price, self.money.max(0) as u32));
        }

        self.money -= price as i32;
        self.shop_offers[shop_index].sold = true;

        let deck_idx = self.deck.len();
        self.deck.push(card);
        self.draw_pile.push(deck_idx);

        // Hologram gains +0.25 Xmult for each playing card added to the deck.
        for j in self.jokers.iter_mut() {
            if j.kind == JokerKind::Hologram && j.active {
                let cur = j.get_counter_f64("x_mult");
                j.set_counter_f64("x_mult", cur + 0.25);
            }
        }
        Ok(())
    }

    /// Re-roll the upcoming Boss blind for $10. Director's Cut allows one per ante;
    /// Retcon lifts the limit (game.lua:606, :620).
    pub fn reroll_boss_blind(&mut self) -> Result<(), BalatroError> {
        let unlimited = self.has_voucher(VoucherKind::Retcon);
        if !unlimited && !self.has_voucher(VoucherKind::DirectorsCut) {
            return Err(BalatroError::NoVoucherAvailable);
        }
        if !unlimited && self.boss_rerolled_this_ante {
            return Err(BalatroError::BossBlindEffect(
                "Director's Cut allows only one Boss reroll per ante".to_string(),
            ));
        }
        let price = 10u32;
        if !self.can_afford(price as i32) {
            return Err(BalatroError::NotEnoughMoney(price, self.money.max(0) as u32));
        }
        self.money -= price as i32;
        self.boss_rerolled_this_ante = true;
        self.boss_blind = self.pick_boss_blind();
        Ok(())
    }

    pub fn leave_shop(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }

        // Perkeo: at end of shop, creates a Negative copy of 1 random consumable in possession
        // The Negative copy always fits (it grants +1 consumable slot)
        if self.jokers.iter().any(|j| j.kind == JokerKind::Perkeo && j.active)
            && !self.consumables.is_empty()
        {
            let idx = self.rng.range_usize(0, self.consumables.len() - 1);
            let mut copy = self.consumables[idx].clone();
            // The copy is "Negative" — represented by expanding consumable slots and adding it
            self.consumable_slots += 1;
            self.consumables.push(copy);
        }

        // Rental jokers charge their rate every round (`rental_rate = 3`, game.lua:1915).
        let rentals = self.jokers.iter().filter(|j| j.rental).count();
        self.money -= 3 * rentals as i32;

        // Coupon and D6 only cover the shop they were spent on.
        self.shop_is_free = false;
        self.shop_rerolls_free = false;

        // Advance to next blind
        self.advance_blind();
        self.state = GameStateKind::BlindSelect;
        Ok(())
    }
}
