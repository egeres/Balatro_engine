use crate::card::*;
use crate::types::*;
use super::{GameState, GameStateKind, BlindKind, BalatroError, upgraded_voucher};

/// What Director's Cut and Retcon charge for a Boss re-roll (game.lua:606).
const BOSS_REROLL_COST: u32 = 10;

/// A voucher's flat price before discounts.
const VOUCHER_COST: u32 = 10;

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

    /// Pick one entry from `(item, weight)` pairs, drawing from the named stream.
    fn weighted_pick<T: Copy>(&mut self, key: &str, pool: &[(T, f64)]) -> Option<T> {
        let total: f64 = pool.iter().map(|(_, w)| *w).sum();
        if total <= 0.0 {
            return None;
        }
        let mut roll = self.rng.next_f64(key) * total;
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
        match self.weighted_pick("shop_slot", &pool)? {
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
        let suit = Suit::ALL[self.rng.range_usize("front", 0, Suit::ALL.len() - 1)];
        let rank = Rank::ALL[self.rng.range_usize("front", 0, Rank::ALL.len() - 1)];
        let id = self.next_id();
        let mut card = CardInstance::new(id, rank, suit);

        if self.has_voucher(VoucherKind::Illusion) && self.rng.next_f64("illusion") > 0.6 {
            let n = Enhancement::ALL.len() - 1;
            card.enhancement = Enhancement::ALL[self.rng.range_usize("illusion", 0, n)];
            card.edition = self.poll_edition(false);
            card.seal = Seal::ALL[self.rng.range_usize("illusion", 0, Seal::ALL.len() - 1)];
        }
        card
    }

    /// Roll an edition using the run's `edition_rate` (common_events.lua:2069).
    /// Hone doubles the Foil/Holo/Poly chances, Glow Up quadruples them.
    pub(crate) fn poll_edition(&mut self, allow_negative: bool) -> Edition {
        let rate = self.edition_rate;
        self.poll_edition_at_rate(rate, allow_negative)
    }

    /// Poll an edition at an explicit rate (`poll_edition`'s `_mod`, common_events.lua:2055).
    /// Negative is never scaled — only Foil, Holographic and Polychrome are.
    pub(crate) fn poll_edition_at_rate(&mut self, rate: f64, allow_negative: bool) -> Edition {
        let roll = self.rng.next_f64("edition_generic");
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

    /// The Foil / Holographic / Polychrome split that Aura and The Wheel of Fortune share:
    /// 50 / 35 / 15. Unlike [`Self::poll_edition`] the card always comes out with one.
    pub(crate) fn roll_positive_edition(&mut self, key: &str) -> Edition {
        match self.rng.next_f64(key) {
            r if r < 0.50 => Edition::Foil,
            r if r < 0.85 => Edition::Holographic,
            _ => Edition::Polychrome,
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
        POOL[self.rng.range_usize("spe_card", 0, POOL.len() - 1)]
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
                        offer.free = true;
                    }
                }
                TagKind::D6 => {
                    self.shop_rerolls_free = true;
                    self.recalculate_reroll_cost(true);
                }
                TagKind::Voucher => {
                    // A second voucher on offer this shop.
                    let v = self.random_voucher();
                    self.shop_offers.push(ShopOffer::new(ShopItem::Voucher(v), VOUCHER_COST));
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
                self.shop_offers[idx].free = true;
                return;
            }
        }
        if pool.is_empty() {
            return;
        }
        let kind = pool[self.rng.range_usize("tag_joker", 0, pool.len() - 1)];
        let id = self.next_id();
        let mut joker = JokerInstance::new(id, kind, edition.unwrap_or(Edition::None));
        joker.edition = edition.unwrap_or(Edition::None);
        let mut offer = ShopOffer::new(ShopItem::Joker(joker), kind.base_cost());
        // The edition tags hand the joker over for free (tag.lua:382).
        offer.free = edition.is_some();

        match slot {
            Some(idx) => self.shop_offers[idx] = offer,
            None => self.shop_offers.push(offer),
        }
    }

    /// Fill the shop's card slots. This is all a reroll touches: the booster packs and the
    /// voucher stay put (button_callbacks.lua:2871 only rebuilds `G.shop_jokers`).
    fn restock_shop_cards(&mut self) {
        // Card slots: base 2, +1 per Overstock voucher (game.lua:1885).
        let card_slots = 2
            + if self.has_voucher(VoucherKind::Overstock) { 1 } else { 0 }
            + if self.has_voucher(VoucherKind::OverstockPlus) { 1 } else { 0 };

        // Card slots are dropped and refilled; packs and the voucher survive the reroll.
        self.shop_offers
            .retain(|o| matches!(o.kind, ShopItem::Pack(_) | ShopItem::Voucher(_)));

        // Filled one at a time so each joker rolled is visible to the pool filter and cannot be
        // rolled twice into the same shop. Inserted at the front to keep the cards ahead of the
        // packs, which is the order callers index by.
        for i in 0..card_slots {
            if let Some(item) = self.roll_shop_slot() {
                let price = match &item {
                    ShopItem::Joker(j) => j.kind.base_cost(),
                    ShopItem::Consumable(c) => c.base_cost(),
                    ShopItem::PlayingCard(_) => 1,
                    ShopItem::Pack(p) => p.base_cost(),
                    ShopItem::Voucher(_) => 10,
                };
                self.shop_offers.insert(i, ShopOffer::new(item, price));
            }
        }
    }

    pub(crate) fn generate_shop(&mut self) {
        self.shop_offers.clear();
        self.restock_shop_cards();

        // Two booster pack slots, drawn from the full weighted pool.
        for _ in 0..2 {
            if let Some(pack) = self.weighted_pick("booster_pool", &Self::PACK_POOL) {
                let price = pack.base_cost();
                self.shop_offers.push(ShopOffer::new(ShopItem::Pack(pack), price));
            }
        }

        self.recalculate_reroll_cost(true);
        self.apply_shop_tags();
    }

    /// `calculate_reroll_cost` (common_events.lua:2263). A free reroll costs nothing and does not
    /// escalate the price; a paid one adds a dollar to the running increase for the rest of the
    /// round.
    pub(crate) fn recalculate_reroll_cost(&mut self, skip_increment: bool) {
        if self.free_rerolls > 0 || self.shop_rerolls_free {
            self.reroll_cost = 0;
            return;
        }
        if !skip_increment {
            self.reroll_cost_increase += 1;
        }
        self.reroll_cost = self.base_reroll_cost + self.reroll_cost_increase;
    }

    /// Whether `kind` can currently appear in a shop / pack / random-joker roll.
    ///
    /// Mirrors the pool gates in game.lua: legendaries are Soul-only, `enhancement_gate` jokers
    /// need the matching enhancement somewhere in the deck, and Gros Michel / Cavendish swap
    /// places once Gros Michel has gone extinct (`no_pool_flag` / `yes_pool_flag`).
    pub(crate) fn joker_in_pool(&self, kind: JokerKind) -> bool {
        // Legendaries are reachable only through The Soul, which asks `legendary_in_pool`.
        if kind.rarity() == 4 {
            return false;
        }
        self.joker_pool_gates_ok(kind)
    }

    /// Whether a Legendary can be drawn. The Soul goes straight to the legendary pool, but the
    /// usual gates still apply — you cannot be handed a second Perkeo without Showman
    /// (common_events.lua:1987).
    pub(crate) fn legendary_in_pool(&self, kind: JokerKind) -> bool {
        kind.rarity() == 4 && self.joker_pool_gates_ok(kind)
    }

    /// The gates every pool draw honours, whatever its rarity.
    fn joker_pool_gates_ok(&self, kind: JokerKind) -> bool {
        // Balatro flags a joker key as used while a copy of it exists anywhere — owned, sitting in
        // the shop, or inside an open pack (card.lua:352, cleared at :4745) — and only Showman
        // lifts the restriction (common_events.lua:1987).
        if !self.has_joker(JokerKind::Showman) && self.joker_kind_in_play(kind) {
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
        let roll = self.rng.next_f64("rarity");
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

        let idx = self.rng.range_usize("joker_pool", 0, pool.len() - 1);
        let kind = pool[idx];
        let id = self.next_id();

        // Editions are polled through the run's `edition_rate` (common_events.lua:2149), which is
        // what makes Hone and Glow Up worth buying. Negative is not scaled by it.
        let edition = self.poll_edition(true);

        let mut joker = JokerInstance::new(id, kind, edition);

        // Stake-based stickers (each 30% chance at the relevant stake threshold).
        // Eternal (Black+) and Perishable (Orange+) are mutually exclusive; Eternal wins if both
        // would trigger. Rental (Gold+) is independent and can combine with either.
        // A joker only takes a sticker its definition allows (`eternal_compat` /
        // `perishable_compat`, card.lua:517).
        // One roll decides both (common_events.lua:2138): above 0.7 is Eternal, 0.4 to 0.7 is
        // Perishable. Two independent 30% rolls left Perishable at 21% once Eternal won ties.
        let eternals_enabled = self.stake.at_least(Stake::Black);
        let perishables_enabled = self.stake.at_least(Stake::Orange);
        if eternals_enabled || perishables_enabled {
            let poll = self.rng.next_f64("etperpoll");
            if eternals_enabled && poll > 0.7 && kind.eternal_compat() {
                joker.eternal = true;
            } else if perishables_enabled && poll > 0.4 && poll <= 0.7 && kind.perishable_compat() {
                joker.perishable = true;
            }
        }
        if self.stake.at_least(Stake::Gold) && self.rng.next_f64("ssjr") > 0.7 {
            joker.rental = true;
        }

        Some(joker)
    }

    /// The voucher on offer for an ante. Each pair contributes one candidate: its upgrade if the
    /// base has been redeemed, otherwise the base itself. A pair that is fully redeemed drops out.
    pub(crate) fn random_voucher(&mut self) -> VoucherKind {
        let available: Vec<VoucherKind> = VoucherKind::BASE
            .into_iter()
            .map(|base| {
                if self.has_voucher(base) {
                    upgraded_voucher(base)
                } else {
                    base
                }
            })
            .filter(|v| !self.has_voucher(*v))
            .collect();
        if available.is_empty() {
            return VoucherKind::Overstock;
        }
        available[self.rng.range_usize("voucher", 0, available.len() - 1)]
    }

    pub(crate) fn random_tarot(&mut self) -> TarotCard {
        TarotCard::ALL[self.rng.range_usize("tarot", 0, TarotCard::ALL.len() - 1)]
    }

    /// A random Planet. The three secret ones join the pool only once their hand has been played.
    pub(crate) fn random_planet(&mut self) -> PlanetCard {
        let mut planets = PlanetCard::BASE.to_vec();
        planets.extend(PlanetCard::SECRET.into_iter().filter(|p| {
            self.hand_levels
                .get(&p.hand_type())
                .is_some_and(|h| h.played > 0)
        }));
        planets[self.rng.range_usize("planet", 0, planets.len() - 1)]
    }

    pub fn has_voucher(&self, v: VoucherKind) -> bool {
        self.vouchers.contains(&v)
    }

    /// The checks every shop purchase starts with: the right screen, an offer that exists, has
    /// not been bought, and is the kind of thing the caller means to buy. Returns its live price.
    ///
    /// Affordability is deliberately left out — the buy paths differ on whether money or a free
    /// slot is checked first, and the error the player sees depends on that order.
    fn check_offer(
        &self,
        index: usize,
        expected: &str,
        is_expected_kind: impl Fn(&ShopItem) -> bool,
    ) -> Result<u32, BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }
        let offer = self
            .shop_offers
            .get(index)
            .ok_or(BalatroError::IndexOutOfRange(index, self.shop_offers.len()))?;
        if offer.sold {
            return Err(BalatroError::AlreadySold);
        }
        if !is_expected_kind(&offer.kind) {
            return Err(BalatroError::WrongItemType(format!("Expected {expected}")));
        }
        Ok(self.offer_price(index).unwrap_or(0))
    }

    /// Refuse a purchase the player cannot cover. Credit Card debt counts as spendable.
    fn require_affordable(&self, price: u32) -> Result<(), BalatroError> {
        if self.can_afford(price as i32) {
            Ok(())
        } else {
            Err(BalatroError::NotEnoughMoney(price, self.money.max(0) as u32))
        }
    }

    pub fn buy_joker(&mut self, shop_index: usize) -> Result<(), BalatroError> {
        let price = self.check_offer(shop_index, "joker", |k| matches!(k, ShopItem::Joker(_)))?;
        self.require_affordable(price)?;
        if self.jokers.len() >= self.effective_joker_slots() {
            return Err(BalatroError::JokerSlotsFull);
        }

        self.money -= price as i32;
        if let ShopItem::Joker(j) = self.shop_offers[shop_index].kind.clone() {
            // A Negative joker brings its own slot; effective_joker_slots() derives that from
            // the board, so nothing needs adding here.
            self.jokers.push(j);
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

        let sell_value = self.jokers[joker_index].sell_value(self.discount_percent());
        self.money += sell_value as i32;

        self.notify_card_sold(Some(self.jokers[joker_index].id));

        // Luchador: selling it disables the current Boss blind's ability for the rest of the round
        if self.jokers[joker_index].kind == JokerKind::Luchador
            && matches!(self.current_blind, BlindKind::Boss)
        {
            self.disable_boss_blind();
        }

        // InvisibleJoker: when sold after 2+ rounds, duplicate a random other joker
        let is_invisible = self.jokers[joker_index].kind == JokerKind::InvisibleJoker;
        let invisible_rounds = self.jokers[joker_index].get_counter_i64("rounds");

        // Diet Cola: selling it hands you a free Double Tag (card.lua:2361, `selling_self`).
        let is_diet_cola = self.jokers[joker_index].kind == JokerKind::DietCola;

        self.jokers.remove(joker_index);

        if is_diet_cola {
            self.gain_tag(TagKind::DoubleFun);
        }

        if is_invisible && invisible_rounds >= 2 && self.jokers.len() < self.effective_joker_slots() {
            let candidates: Vec<usize> = (0..self.jokers.len())
                .filter(|&j| self.jokers[j].active && self.jokers[j].kind != JokerKind::InvisibleJoker)
                .collect();
            if !candidates.is_empty() {
                let pick = self.rng.range_usize("invisible", 0, candidates.len() - 1);
                let dup = self.jokers[candidates[pick]].clone();
                self.jokers.push(dup);
            }
        }

        // VerdantLeaf: selling a joker switches the blind off outright — `G.GAME.blind:disable()`
        // (card.lua:1615) — not merely the card debuffs it had applied. Matador stops paying out
        // on it from here, as it would for a Chicot or a sold Luchador.
        if let Some(BossBlind::VerdantLeaf) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                self.verdant_leaf_joker_sold = true;
                self.disable_boss_blind();
            }
        }

        Ok(())
    }

    pub fn buy_consumable(&mut self, shop_index: usize) -> Result<(), BalatroError> {
        let price =
            self.check_offer(shop_index, "consumable", |k| matches!(k, ShopItem::Consumable(_)))?;
        if !self.has_room_for_consumable() {
            return Err(BalatroError::ConsumableSlotsFull);
        }
        self.require_affordable(price)?;

        self.money -= price as i32;
        if let ShopItem::Consumable(c) = self.shop_offers[shop_index].kind.clone() {
            self.add_consumable(c);
        }
        self.shop_offers[shop_index].sold = true;
        Ok(())
    }

    pub fn buy_pack(&mut self, shop_index: usize) -> Result<(), BalatroError> {
        let price = self.check_offer(shop_index, "pack", |k| matches!(k, ShopItem::Pack(_)))?;
        self.require_affordable(price)?;
        let ShopItem::Pack(pack_kind) = self.shop_offers[shop_index].kind else {
            unreachable!("check_offer just confirmed this is a pack")
        };

        self.money -= price as i32;
        self.shop_offers[shop_index].sold = true;

        self.current_pack = Some(self.generate_pack_contents(pack_kind));
        self.on_booster_opened();
        self.state = GameStateKind::BoosterPack;
        Ok(())
    }

    pub fn buy_voucher(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }
        let voucher = self.shop_voucher.ok_or(BalatroError::NoVoucherAvailable)?;

        let price = self.voucher_price();
        self.require_affordable(price)?;

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
                // Each Planet card *held* in a consumable slot gives X1.5 Mult for its own hand
                // — handled in score_hand, via `ScoreInputs::observatory_planets`.
            }
            VoucherKind::Grabber | VoucherKind::NachoTong => {
                self.max_hands += 1;
                self.hands_remaining = self.hands_remaining.saturating_add(1);
            }
            VoucherKind::Wasteful | VoucherKind::Recyclomancy => {
                self.max_discards += 1;
                self.discards_remaining = self.discards_remaining.saturating_add(1);
            }
            // These raise how often the card type shows up in the shop, they are not discounts
            // (card.lua:1890, `tarot_rate = 4 * extra`).
            VoucherKind::TarotMerchant => self.tarot_rate = 9.6,
            VoucherKind::TarotTycoon => self.tarot_rate = 32.0,
            VoucherKind::PlanetMerchant => self.planet_rate = 9.6,
            VoucherKind::PlanetTycoon => self.planet_rate = 32.0,
            // Both set the interest cap outright rather than adding to it (game.lua:602).
            VoucherKind::SeedMoney => self.max_interest = self.max_interest.max(50),
            VoucherKind::MoneyTree => self.max_interest = self.max_interest.max(100),
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
                // -1 Ante, at the cost of a hand each round (card.lua:1957). The ante you have to
                // *reach* is untouched, so this lengthens the run and shrinks its blinds.
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
            VoucherKind::DirectorsCut | VoucherKind::Retcon => {
                // Unlock a paid Boss blind reroll — one per ante, or unlimited. No immediate
                // effect; `can_reroll_boss_blind` reads them.
            }
            VoucherKind::PaintBrush | VoucherKind::Palette => {
                self.hand_size += 1;
            }
        }
    }

    /// `Card:set_cost` (card.lua:369), in the game's order.
    ///
    /// The discount is applied **once**, to the base cost — recomputing it from an
    /// already-discounted price would compound it. The three overrides then land *after* the
    /// `max(1)` floor, which is why an Astronomer's Planet or a couponed card costs nothing
    /// rather than a dollar, and why a coupon beats a rental.
    fn price_card(
        &self,
        base_cost: u32,
        edition: Edition,
        rental: bool,
        astronomer_free: bool,
        couponed: bool,
    ) -> u32 {
        let mut cost = card_shop_cost(base_cost, edition, false, self.discount_percent());

        if astronomer_free {
            cost = 0;
        }
        if rental {
            cost = 1;
        }
        if couponed {
            cost = 0;
        }
        cost
    }

    /// What buying shop slot `index` actually costs right now.
    ///
    /// Live rather than stored, because it moves under the player's feet: redeeming Clearance
    /// Sale or buying an Astronomer mid-shop reprices what is still on the shelf, which is what
    /// Balatro's `set_cost` recomputation does.
    pub fn offer_price(&self, index: usize) -> Option<u32> {
        let offer = self.shop_offers.get(index)?;
        let (edition, rental) = match &offer.kind {
            ShopItem::Joker(j) => (j.edition, j.rental),
            _ => (Edition::None, false),
        };
        let astronomer = self.has_joker(JokerKind::Astronomer)
            && match &offer.kind {
                ShopItem::Consumable(ConsumableCard::Planet(_)) => true,
                ShopItem::Pack(p) => self.is_celestial_pack(*p),
                _ => false,
            };
        Some(self.price_card(offer.price, edition, rental, astronomer, offer.free))
    }

    /// What the voucher on offer costs. Vouchers are a flat price before discounts.
    pub fn voucher_price(&self) -> u32 {
        self.price_card(VOUCHER_COST, Edition::None, false, false, false)
    }

    /// Test hook for the pricing rule.
    pub fn debug_joker_price(&self, joker: &JokerInstance) -> u32 {
        self.price_card(joker.kind.base_cost(), joker.edition, joker.rental, false, false)
    }

    /// What selling joker `index` pays right now.
    pub fn joker_sell_value(&self, index: usize) -> Option<u32> {
        Some(self.jokers.get(index)?.sell_value(self.discount_percent()))
    }

    pub fn reroll_shop(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }

        let was_free = self.shop_rerolls_free || self.free_rerolls > 0;
        let actual_cost = if was_free { 0 } else { self.reroll_cost };
        self.require_affordable(actual_cost)?;

        self.money -= actual_cost as i32;
        if !self.shop_rerolls_free && self.free_rerolls > 0 {
            self.free_rerolls -= 1;
        }
        // Spending a free reroll does not push the price up (`calculate_reroll_cost(final_free)`).
        self.recalculate_reroll_cost(was_free);

        // Only the card slots are replaced — the packs and the voucher are not rerollable.
        self.restock_shop_cards();

        // Flash Card joker: +2 mult per reroll
        for j in self.jokers.iter_mut() {
            if j.kind == JokerKind::FlashCard {
                j.add_counter_i64("mult", 2);
            }
        }

        Ok(())
    }

    /// Buy a loose playing card from the shop and add it to the deck.
    /// Only reachable once Magic Trick has been redeemed.
    pub fn buy_playing_card(&mut self, shop_index: usize) -> Result<(), BalatroError> {
        let price = self.check_offer(shop_index, "playing card", |k| {
            matches!(k, ShopItem::PlayingCard(_))
        })?;
        self.require_affordable(price)?;
        let ShopItem::PlayingCard(card) = self.shop_offers[shop_index].kind.clone() else {
            unreachable!("check_offer just confirmed this is a playing card")
        };

        self.money -= price as i32;
        self.shop_offers[shop_index].sold = true;

        self.add_card_to_draw_pile(card);
        self.notify_playing_cards_added(1);
        Ok(())
    }

    /// Re-roll the upcoming Boss blind for $10. Director's Cut allows one per ante;
    /// Retcon lifts the limit (game.lua:606, :620).
    /// Whether a Boss reroll is available: the right screen, an unlock, an unspent allowance
    /// this ante, and the $10 to hand.
    pub fn can_reroll_boss_blind(&self) -> bool {
        if !matches!(self.state, GameStateKind::BlindSelect) {
            return false;
        }
        let unlimited = self.has_voucher(VoucherKind::Retcon);
        if !unlimited && !self.has_voucher(VoucherKind::DirectorsCut) {
            return false;
        }
        if !unlimited && self.boss_rerolled_this_ante {
            return false;
        }
        self.can_afford(BOSS_REROLL_COST as i32)
    }

    pub fn reroll_boss_blind(&mut self) -> Result<(), BalatroError> {
        // The reroll button lives on the blind-select screen (button_callbacks.lua:2784).
        if !matches!(self.state, GameStateKind::BlindSelect) {
            return Err(BalatroError::NotInBlindSelect);
        }
        let unlimited = self.has_voucher(VoucherKind::Retcon);
        if !unlimited && !self.has_voucher(VoucherKind::DirectorsCut) {
            return Err(BalatroError::NoVoucherAvailable);
        }
        if !unlimited && self.boss_rerolled_this_ante {
            return Err(BalatroError::BossBlindEffect(
                "Director's Cut allows only one Boss reroll per ante".to_string(),
            ));
        }
        let price = BOSS_REROLL_COST;
        self.require_affordable(price)?;
        self.money -= price as i32;
        self.boss_rerolled_this_ante = true;
        self.boss_blind = self.pick_boss_blind();
        Ok(())
    }

    pub fn leave_shop(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Shop) {
            return Err(BalatroError::NotInShop);
        }

        // Perkeo: at end of shop, creates a Negative copy of 1 random consumable in possession.
        // The Negative copy always fits, because it brings its own slot — one that goes away
        // again once the consumables are spent (`release_negative_consumable_slots`).
        if self.has_joker(JokerKind::Perkeo) && !self.consumables.is_empty() {
            let idx = self.rng.range_usize("perkeo", 0, self.consumables.len() - 1);
            let copy = self.consumables[idx].card.clone();
            self.consumables.push(HeldConsumable::negative(copy));
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
