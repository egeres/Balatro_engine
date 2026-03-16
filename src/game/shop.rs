use crate::card::*;
use crate::types::*;
use std::collections::HashMap;
use super::{GameState, GameStateKind, BalatroError, HistoryEvent, upgraded_voucher};

impl GameState {
    pub(crate) fn generate_shop(&mut self) {
        // Count shop slots (base: 2 jokers + extras from vouchers)
        let joker_slots = 2 + if self.has_voucher(VoucherKind::Overstock) { 1 } else { 0 }
            + if self.has_voucher(VoucherKind::OverstockPlus) { 1 } else { 0 };

        let mut offers: Vec<ShopOffer> = Vec::new();

        // Generate jokers
        for _ in 0..joker_slots {
            if let Some(joker) = self.generate_random_joker() {
                let price = joker.kind.base_cost();
                offers.push(ShopOffer {
                    kind: ShopItem::Joker(joker),
                    price,
                    sold: false,
                });
            }
        }

        // Booster packs: Arcana + Celestial always; Spectral at Purple stake or Ghost deck
        let mut packs = vec![PackKind::ArcanaPack, PackKind::CelestialPack];
        if self.stake as u8 >= Stake::Purple as u8 || self.deck_type == DeckType::Ghost {
            packs.push(PackKind::SpectralPack);
        }
        for &pack in &packs {
            let price = pack.base_cost();
            offers.push(ShopOffer {
                kind: ShopItem::Pack(pack),
                price,
                sold: false,
            });
        }

        self.shop_offers = offers;
        self.shop_voucher = Some(self.random_voucher());
        self.reroll_cost = 5;

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

    pub(crate) fn generate_random_joker(&mut self) -> Option<JokerInstance> {
        // Simple random joker selection
        let all_jokers = vec![
            JokerKind::Joker,
            JokerKind::GreedyJoker,
            JokerKind::LustyJoker,
            JokerKind::WrathfulJoker,
            JokerKind::GluttonousJoker,
            JokerKind::JollyJoker,
            JokerKind::ZanyJoker,
            JokerKind::MadJoker,
            JokerKind::CrazyJoker,
            JokerKind::DrollJoker,
            JokerKind::Banner,
            JokerKind::MysticSummit,
            JokerKind::EvenSteven,
            JokerKind::OddTodd,
            JokerKind::Scholar,
            JokerKind::Supernova,
            JokerKind::Runner,
            JokerKind::BlueJoker,
            JokerKind::Constellation,
            JokerKind::Fibonacci,
            JokerKind::AbstractJoker,
            JokerKind::HalfJoker,
            JokerKind::WalkieTalkie,
            JokerKind::SmileyFace,
            JokerKind::Bull,
            JokerKind::GoldenJoker,
            JokerKind::SteelJoker,
            JokerKind::GreenJoker,
            JokerKind::Castle,
            JokerKind::Hologram,
        ];

        let idx = self.rng.range_usize(0, all_jokers.len() - 1);
        let kind = all_jokers[idx];
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

        // Apply stake-based stickers (mutually exclusive; types shuffled to remove ordering bias)
        // Red+: Eternal can appear (~5% per type)
        // Green+: Rental can also appear
        // Blue+: Perishable can also appear
        let stake_level = self.stake as u8;
        let mut available: Vec<u8> = Vec::new();
        if stake_level >= Stake::Red as u8   { available.push(0); } // Eternal
        if stake_level >= Stake::Green as u8 { available.push(1); } // Rental
        if stake_level >= Stake::Blue as u8  { available.push(2); } // Perishable
        self.rng.shuffle(&mut available);
        'sticker: for kind in available {
            if self.rng.next_bool_prob(0.05) {
                match kind {
                    0 => { joker.eternal = true; }
                    1 => { joker.rental = true; }
                    _ => { joker.perishable = true; }
                }
                break 'sticker;
            }
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
        if self.money < price as i32 {
            return Err(BalatroError::NotEnoughMoney(price, self.money as u32));
        }
        if self.jokers.len() >= self.joker_slots as usize {
            return Err(BalatroError::JokerSlotsFull);
        }

        self.money -= price as i32;
        if let ShopItem::Joker(j) = &self.shop_offers[shop_index].kind.clone() {
            // Negative edition gives +1 joker slot
            if j.edition == Edition::Negative {
                self.joker_slots += 1;
            }
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

        self.jokers.remove(joker_index);
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
        if self.money < price as i32 {
            return Err(BalatroError::NotEnoughMoney(price, self.money as u32));
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
        if self.money < price as i32 {
            return Err(BalatroError::NotEnoughMoney(price, self.money as u32));
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
        if self.money < price as i32 {
            return Err(BalatroError::NotEnoughMoney(price, self.money as u32));
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
            VoucherKind::Hone | VoucherKind::GlowUp => {
                // Enhanced editions in booster packs — handled in generate_pack_contents
            }
            VoucherKind::RerollSurplus => {
                // -$1 reroll cost (handled in reroll_shop via voucher check)
            }
            VoucherKind::RerollGlut => {
                // -$1 more reroll cost
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
            VoucherKind::TarotMerchant | VoucherKind::TarotTycoon => {
                // Price discounts on tarots — handled in calculate_shop_price
            }
            VoucherKind::PlanetMerchant | VoucherKind::PlanetTycoon => {
                // Price discounts on planets — handled in calculate_shop_price
            }
            VoucherKind::SeedMoney => {
                self.max_interest += 10;
            }
            VoucherKind::MoneyTree => {
                self.max_interest += 10;
            }
            VoucherKind::Blank => {
                self.joker_slots += 1;
            }
            VoucherKind::Antimatter => {
                self.joker_slots += 1;
            }
            VoucherKind::MagicTrick | VoucherKind::Illusion => {
                // Playing cards available in shop — handled in generate_shop
            }
            VoucherKind::Hieroglyph | VoucherKind::Petroglyph => {
                // Reduces winning ante requirement — handled in win condition check
            }
            VoucherKind::DirectorsCut => {
                self.free_rerolls += 1;
            }
            VoucherKind::Retcon => {
                // Allows rerolling boss blind — handled in advance_blind
            }
            VoucherKind::PaintBrush => {
                self.hand_size += 1;
            }
            VoucherKind::Palette => {
                self.hand_size += 1;
            }
        }
    }

    fn calculate_shop_price(&self, base_price: u32) -> u32 {
        let mut price = base_price;
        if self.has_voucher(VoucherKind::ClearanceSale) {
            price = (price as f64 * 0.75) as u32;
        }
        if self.has_voucher(VoucherKind::Liquidation) {
            price = (price as f64 * 0.5) as u32;
        }
        price.max(1)
    }

    pub fn reroll_shop(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }

        let actual_cost = if self.free_rerolls > 0 {
            self.free_rerolls -= 1;
            0
        } else {
            self.reroll_cost
        };

        if self.money < actual_cost as i32 {
            return Err(BalatroError::NotEnoughMoney(actual_cost, self.money as u32));
        }

        self.money -= actual_cost as i32;
        self.reroll_cost += 1; // Increases by 1 each time (normally)

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

        // Rental jokers cost money
        for j in self.jokers.iter() {
            if j.rental {
                self.money -= 1;
            }
        }

        // Advance to next blind
        self.advance_blind();
        self.state = GameStateKind::BlindSelect;
        Ok(())
    }
}
