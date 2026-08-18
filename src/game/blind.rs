use crate::card::*;
use crate::rng::keyed;
use crate::types::*;
use std::collections::HashMap;
use super::{GameState, GameStateKind, BlindKind, BalatroError, HistoryEvent};

impl GameState {
    /// Pick the Boss blind for the current ante (`common_events.lua:2338`).
    ///
    /// Regular bosses are gated on their `min` ante and excluded on showdown antes; showdown
    /// bosses appear only when `ante % win_ante == 0` (and never at ante 1). Among the eligible
    /// ones Balatro draws from those used the fewest times so far, so a run cycles through the
    /// roster instead of repeating.
    pub fn pick_boss_blind(&mut self) -> Option<BossBlind> {
        let ante = self.ante.max(1);
        let win_ante = self.win_ante();
        let is_showdown_ante = ante % win_ante == 0 && ante >= 2;

        let eligible: Vec<BossBlind> = BossBlind::ALL
            .iter()
            .copied()
            .filter(|b| {
                if b.is_showdown() {
                    is_showdown_ante
                } else {
                    b.min_ante() <= ante && !is_showdown_ante
                }
            })
            .collect();
        if eligible.is_empty() {
            return None;
        }

        let min_use = eligible
            .iter()
            .map(|b| self.bosses_used.get(b).copied().unwrap_or(0))
            .min()
            .unwrap_or(0);
        let pool: Vec<BossBlind> = eligible
            .into_iter()
            .filter(|b| self.bosses_used.get(b).copied().unwrap_or(0) == min_use)
            .collect();

        let pick = pool[self.rng.range_usize("boss", 0, pool.len() - 1)];
        *self.bosses_used.entry(pick).or_insert(0) += 1;
        Some(pick)
    }

    // =========================================================
    // Actions
    // =========================================================

    pub fn select_blind(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::BlindSelect) {
            return Err(BalatroError::NotInBlindSelect);
        }
        self.apply_blind_select_tags();
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
        self.skips_this_run += 1;

        // Skipping a blind is the only source of tags.
        let tag = self.random_tag();
        self.gain_tag(tag);
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


    /// A random tag eligible at the current ante (`min_ante`, game.lua:225).
    pub(crate) fn random_tag(&mut self) -> TagKind {
        let pool: Vec<TagKind> = TagKind::ALL
            .iter()
            .copied()
            .filter(|t| t.min_ante() <= self.ante)
            .collect();
        pool[self.rng.range_usize("tag", 0, pool.len() - 1)]
    }

    /// Acquire a tag: immediate ones pay out at once, the rest queue up for their trigger.
    ///
    /// A pending Double Tag copies whatever comes next (tag.lua:319) — but never another
    /// Double Tag, and it is consumed doing so.
    pub(crate) fn gain_tag(&mut self, tag: TagKind) {
        let doubled = tag != TagKind::DoubleFun
            && self
                .tags
                .iter()
                .position(|t| *t == TagKind::DoubleFun)
                .map(|pos| {
                    self.tags.remove(pos);
                })
                .is_some();

        let copies = if doubled { 2 } else { 1 };
        for _ in 0..copies {
            if tag.trigger() == TagTrigger::Immediate {
                self.apply_immediate_tag(tag);
            } else {
                self.tags.push(tag);
            }
        }
    }

    /// Tags that pay out the instant the blind is skipped (tag.lua:131).
    fn apply_immediate_tag(&mut self, tag: TagKind) {
        match tag {
            TagKind::Skip => {
                // $5 per blind skipped this run, counting this one.
                self.money += 5 * self.skips_this_run as i32;
            }
            TagKind::Handy => {
                self.money += self.hands_played_this_run as i32;
            }
            TagKind::Garbage => {
                self.money += self.unused_discards_this_run as i32;
            }
            TagKind::Economy => {
                // Doubles your money, capped at a $40 gain.
                self.money += self.money.max(0).min(40);
            }
            TagKind::Orbital => {
                // +3 levels to a random hand type.
                let hand_types: Vec<HandType> = self.hand_levels.keys().copied().collect();
                let pick = hand_types[self.rng.range_usize("orbital", 0, hand_types.len() - 1)];
                if let Some(level) = self.hand_levels.get_mut(&pick) {
                    level.level += 3;
                }
            }
            TagKind::TopUp => {
                // Up to 2 Common jokers, slots permitting.
                for _ in 0..2 {
                    if self.jokers.len() >= self.effective_joker_slots() {
                        break;
                    }
                    let pool: Vec<JokerKind> = JokerKind::ALL
                        .iter()
                        .copied()
                        .filter(|k| k.rarity() == 1 && self.joker_in_pool(*k))
                        .collect();
                    if pool.is_empty() {
                        break;
                    }
                    let kind = pool[self.rng.range_usize("top", 0, pool.len() - 1)];
                    let id = self.next_id();
                    self.jokers.push(JokerInstance::new(id, kind, Edition::None));
                }
            }
            _ => {}
        }
    }

    /// Consume the tags waiting on the blind-select screen: Boss Tag re-rolls the Boss, the
    /// pack tags queue a free booster.
    pub(crate) fn apply_blind_select_tags(&mut self) {
        let pending: Vec<TagKind> = self
            .tags
            .iter()
            .copied()
            .filter(|t| t.trigger() == TagTrigger::BlindSelect)
            .collect();
        if pending.is_empty() {
            return;
        }
        self.tags.retain(|t| t.trigger() != TagTrigger::BlindSelect);

        for tag in pending {
            if tag == TagKind::Boss {
                self.boss_blind = self.pick_boss_blind();
            } else if let Some(pack) = tag.free_pack() {
                // Only one pack can be open at a time; the rest are dropped, as in Balatro
                // where they queue behind the current choice.
                self.pending_free_pack = Some(pack);
            }
        }
    }

    /// Open the free booster pack a tag granted, if one is waiting.
    pub fn open_pending_free_pack(&mut self) -> Result<(), BalatroError> {
        let Some(pack) = self.pending_free_pack.take() else {
            return Err(BalatroError::NotInPack);
        };
        self.current_pack = Some(self.generate_pack_contents(pack));
        self.on_booster_opened();
        self.state = GameStateKind::BoosterPack;
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
        self.boss_blind_manually_disabled = false;

        // Reset per-round hand played counters
        for data in self.hand_levels.values_mut() {
            data.played_this_round = 0;
        }

        // Juggle Tag: +3 hand size for this round only.
        let juggles = self.tags.iter().filter(|t| **t == TagKind::Juggle).count();
        self.tags.retain(|t| *t != TagKind::Juggle);
        self.juggle_hand_size = 3 * juggles as u32;

        self.reroll_round_targets();

        // Reset draw pile
        self.draw_pile = (0..self.deck.len()).collect();
        self.rng.shuffle("shuffle", &mut self.draw_pile);

        // Reset face-down state for all cards
        for card in self.deck.iter_mut() {
            card.face_down = false;
        }

        // Apply boss blind debuffs to cards
        self.apply_boss_blind_debuffs();

        // AmberAcorn: shuffle joker order at the start of the blind
        if let Some(BossBlind::AmberAcorn) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                if !self.boss_blind_disabled() {
                    self.rng.shuffle("amber_acorn", &mut self.jokers);
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

    /// Re-roll the round-wide joker targets (`state_events.lua:273-276`).
    ///
    /// The Idol, Castle and Mail-In Rebate each sample a random non-Stone card from the full deck
    /// and read a property off it; Ancient Joker picks a suit *different* from the current one, so
    /// it always changes. Defaults (Ace of Spades / Spades / Ace) stand in if the deck has no
    /// eligible card, matching `common_events.lua:2272`.
    fn reroll_round_targets(&mut self) {
        // Keyed per ante, as Balatro does (`pseudoseed('idol'..ante)`).
        let ante = self.ante;
        let eligible: Vec<usize> = self
            .deck
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.is_stone())
            .map(|(i, _)| i)
            .collect();

        let mut targets = crate::scoring::RoundTargets::default();

        if !eligible.is_empty() {
            let idol = self.deck[eligible[self.rng.range_usize(&keyed("idol", ante), 0, eligible.len() - 1)]].clone();
            targets.idol_rank = idol.rank;
            targets.idol_suit = idol.suit;

            let mail = self.deck[eligible[self.rng.range_usize(&keyed("mail", ante), 0, eligible.len() - 1)]].clone();
            targets.mail_rank = mail.rank;

            let castle = self.deck[eligible[self.rng.range_usize(&keyed("cas", ante), 0, eligible.len() - 1)]].clone();
            targets.castle_suit = castle.suit;
        }

        // Ancient Joker never repeats the previous round's suit.
        let others: Vec<Suit> = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds]
            .into_iter()
            .filter(|s| *s != self.round_targets.ancient_suit)
            .collect();
        targets.ancient_suit = others[self.rng.range_usize(&keyed("anc", ante), 0, others.len() - 1)];

        self.round_targets = targets;
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
                if !self.boss_blind_disabled() {
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
        let mut size = self.hand_size + self.juggle_hand_size;
        // TheManacle: -1 hand size during Boss blind
        if let Some(BossBlind::TheManacle) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                if !self.boss_blind_disabled() {
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

        // Blinds that draw cards face down (`Blind:stay_flipped`, blind.lua:604).
        if matches!(self.current_blind, BlindKind::Boss) {
            if !self.boss_blind_disabled() {
                let newly_drawn: Vec<usize> = (start_hand_len..self.hand.len()).collect();
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
                            if self.rng.range_usize("wheel", 0, 6) == 0 {
                                let card_idx = self.hand[hand_idx];
                                self.deck[card_idx].face_down = true;
                            }
                        }
                    }
                    Some(BossBlind::TheHouse) => {
                        // Only the opening hand of the round is hidden — nothing has been played
                        // or discarded yet (blind.lua:611).
                        if self.hands_remaining == self.effective_max_hands()
                            && self.discards_remaining == self.effective_max_discards()
                        {
                            for hand_idx in newly_drawn {
                                let card_idx = self.hand[hand_idx];
                                self.deck[card_idx].face_down = true;
                            }
                        }
                    }
                    Some(BossBlind::TheMark) => {
                        // Face cards are hidden, not disabled (blind.lua:614). It calls
                        // `is_face(true)`, so Pareidolia hides the whole hand.
                        let pareidolia = self.has_pareidolia();
                        for hand_idx in newly_drawn {
                            let card_idx = self.hand[hand_idx];
                            if self.deck[card_idx].is_face(pareidolia) {
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
                if !self.boss_blind_disabled() {
                    let newly_drawn_count = self.hand.len() - start_hand_len;
                    if newly_drawn_count > 0 {
                        let offset = self.rng.range_usize("cerulean_bell", 0, newly_drawn_count - 1);
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
        if self.boss_blind_disabled() {
            return;
        }

        // `Blind:debuff_card` goes through `is_suit` / `is_face` (blind.lua:626), so a Wild Card
        // is caught by every suit blind, a Stone card by none, Smeared Joker widens the net to
        // the whole colour, and Pareidolia hands The Plant the entire deck.
        let smeared = self.has_smeared();
        let pareidolia = self.has_pareidolia();
        let debuff_suit = |gs: &mut Self, suit: Suit| {
            for card in gs.deck.iter_mut() {
                if card.is_suit(suit, smeared) {
                    card.debuffed = true;
                }
            }
        };

        match boss {
            BossBlind::TheClub => debuff_suit(self, Suit::Clubs),
            BossBlind::TheGoad => debuff_suit(self, Suit::Spades),
            BossBlind::TheHead => debuff_suit(self, Suit::Hearts),
            BossBlind::TheWindow => debuff_suit(self, Suit::Diamonds),
            BossBlind::ThePlant => {
                for card in self.deck.iter_mut() {
                    if card.is_face(pareidolia) {
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
                let played_ids: std::collections::HashSet<u64> =
                    self.played_card_ids_this_ante.iter().copied().collect();
                for card in self.deck.iter_mut() {
                    if played_ids.contains(&card.id) {
                        card.debuffed = true;
                    }
                }
            }
            _ => {}
        }
    }

    /// Ceremonial Dagger: on blind select, destroy the joker immediately to its right and gain
    /// Mult equal to twice that joker's sell value (`card.lua:2561`). Eternal jokers are spared,
    /// and a joker already marked for slicing cannot be sliced twice.
    ///
    /// Runs before the other setting-blind effects because a sliced joker must not get to fire
    /// its own effect — Balatro guards those with `not self.getting_sliced`.
    fn apply_ceremonial_daggers(&mut self) {
        let mut sliced: Vec<u64> = Vec::new();
        for i in 0..self.jokers.len() {
            if self.jokers[i].kind != JokerKind::CeremonialDagger || !self.jokers[i].active {
                continue;
            }
            let Some(target) = self.jokers.get(i + 1) else { continue };
            if target.eternal || sliced.contains(&target.id) {
                continue;
            }
            let gained = target.sell_value() as i64 * 2;
            sliced.push(target.id);
            let cur = self.jokers[i].get_counter_i64("mult");
            self.jokers[i].set_counter_i64("mult", cur + gained);
        }
        if !sliced.is_empty() {
            self.jokers.retain(|j| !sliced.contains(&j.id));
        }
    }

    fn notify_jokers_setting_blind(&mut self) {
        self.apply_ceremonial_daggers();

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
                        let pick = self.rng.range_usize("madness", 0, destroyable.len() - 1);
                        let idx = destroyable[pick];
                        self.jokers.remove(idx);
                    }
                }
                JokerKind::Cartomancer => {
                    // Creates a Tarot when the Blind is selected — nothing to do with what you
                    // then play (card.lua `first_hand_drawn`, en-us.lua j_cartomancer).
                    if self.consumables.len() < self.consumable_slots as usize {
                        let tarot = self.random_tarot();
                        self.consumables.push(ConsumableCard::Tarot(tarot));
                    }
                }
                JokerKind::Certificate => {
                    // Adds a playing card with a random seal straight to *hand* (card.lua:2465
                    // emplaces into G.hand), so it is playable this round rather than being
                    // shuffled somewhere into the draw pile.
                    let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
                    let ranks = [
                        Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
                        Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                        Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
                    ];
                    let seals = [Seal::Gold, Seal::Red, Seal::Blue, Seal::Purple];
                    let suit_idx = self.rng.range_usize("cert_fr", 0, 3);
                    let rank_idx = self.rng.range_usize("cert_fr", 0, 12);
                    let seal_idx = self.rng.range_usize("certsl", 0, seals.len() - 1);
                    let new_id = self.next_id();
                    let mut new_card = CardInstance::new(new_id, ranks[rank_idx], suits[suit_idx]);
                    new_card.seal = seals[seal_idx];
                    let deck_idx = self.deck.len();
                    self.deck.push(new_card);
                    self.hand.push(deck_idx);
                }
                JokerKind::RiffRaff => {
                    // Two *Common* jokers, drawn from the Common pool directly — rolling the
                    // whole pool and discarding non-Commons would yield roughly 1.4.
                    for _ in 0..2 {
                        if self.jokers.len() >= self.effective_joker_slots() {
                            break;
                        }
                        let pool: Vec<JokerKind> = JokerKind::ALL
                            .iter()
                            .copied()
                            .filter(|k| k.rarity() == 1 && self.joker_in_pool(*k))
                            .collect();
                        if pool.is_empty() {
                            break;
                        }
                        let kind = pool[self.rng.range_usize("rif", 0, pool.len() - 1)];
                        let id = self.next_id();
                        self.jokers.push(JokerInstance::new(id, kind, Edition::None));
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
                    let idx = self.rng.range_usize("to_do", 0, hand_types.len() - 1);
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

        // Cards these jokers just conjured (Certificate, Marble Joker) still have to answer to
        // the Boss blind (`G.GAME.blind:debuff_card(_card)`, card.lua:2472).
        self.apply_boss_blind_debuffs();
    }
}
