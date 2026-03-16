use crate::card::*;
use crate::rng::Rng;
use crate::scoring::{score_hand, ScoreResult};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Full game state
pub struct GameState {
    pub rng: Rng,
    pub deck_type: DeckType,
    pub stake: Stake,
    pub seed: String,

    // Run-level state
    pub ante: u32,
    pub round: u32, // 1=small, 2=big, 3=boss
    pub money: i32,
    pub state: GameStateKind,
    pub vouchers: Vec<VoucherKind>,
    pub tags: Vec<TagKind>,
    pub tarot_cards_used: u32,
    pub planet_cards_used: u32,

    // Blind state
    pub current_blind: BlindKind,
    pub boss_blind: Option<BossBlind>,
    pub score_goal: f64,
    pub skipped_blinds: Vec<(u32, u32)>, // (ante, round) of skipped blinds
    pub blind_defeated_this_ante: [bool; 3],

    // Round state
    pub deck: Vec<CardInstance>,  // full ordered deck
    pub draw_pile: Vec<usize>,    // indices into deck of remaining drawable cards
    pub hand: Vec<usize>,         // indices of cards currently in hand
    pub discard_pile: Vec<usize>, // indices of discarded cards this round
    pub jokers: Vec<JokerInstance>,
    pub consumables: Vec<ConsumableCard>,
    pub hands_remaining: u32,
    pub discards_remaining: u32,
    pub score_accumulated: f64,
    pub selected_indices: Vec<usize>, // selected from hand (hand-relative indices)

    // Hand levels
    pub hand_levels: HashMap<HandType, HandLevelData>,

    // Shop state
    pub shop_offers: Vec<ShopOffer>,
    pub shop_voucher: Option<VoucherKind>,
    pub reroll_cost: u32,
    pub free_rerolls: u32,

    // Pack state
    pub current_pack: Option<PackContents>,

    // Config
    pub hand_size: u32,
    pub max_hands: u32,
    pub max_discards: u32,
    pub joker_slots: u32,
    pub consumable_slots: u32,
    pub max_interest: i32,

    // History
    pub history: Vec<HistoryEvent>,
    pub next_id: u64,

    // For The Fool tarot: remembers the most recently used tarot or planet this run
    pub last_consumable_used: Option<LastConsumable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameStateKind {
    BlindSelect,
    Round,
    Shop,
    BoosterPack,
    GameOver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlindKind {
    Small,
    Big,
    Boss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LastConsumable {
    Tarot(TarotCard),
    Planet(PlanetCard),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub ante: u32,
    pub round: u32,
    pub event_type: String,
    pub data: serde_json::Value,
}

impl GameState {
    pub fn new(deck_type: DeckType, stake: Stake, seed: Option<String>) -> Self {
        let seed = seed.unwrap_or_default();
        let mut rng = Rng::new(&seed);

        // Initialize hand levels
        let mut hand_levels = HashMap::new();
        hand_levels.insert(HandType::FlushFive, HandLevelData::new(false));
        hand_levels.insert(HandType::FlushHouse, HandLevelData::new(false));
        hand_levels.insert(HandType::FiveOfAKind, HandLevelData::new(false));
        hand_levels.insert(HandType::StraightFlush, HandLevelData::new(true));
        hand_levels.insert(HandType::FourOfAKind, HandLevelData::new(true));
        hand_levels.insert(HandType::FullHouse, HandLevelData::new(true));
        hand_levels.insert(HandType::Flush, HandLevelData::new(true));
        hand_levels.insert(HandType::Straight, HandLevelData::new(true));
        hand_levels.insert(HandType::ThreeOfAKind, HandLevelData::new(true));
        hand_levels.insert(HandType::TwoPair, HandLevelData::new(true));
        hand_levels.insert(HandType::Pair, HandLevelData::new(true));
        hand_levels.insert(HandType::HighCard, HandLevelData::new(true));

        let mut gs = GameState {
            rng,
            deck_type,
            stake,
            seed: seed.clone(),
            ante: 1,
            round: 1,
            money: 4,
            state: GameStateKind::BlindSelect,
            vouchers: Vec::new(),
            tags: Vec::new(),
            tarot_cards_used: 0,
            planet_cards_used: 0,
            current_blind: BlindKind::Small,
            boss_blind: None,
            score_goal: 0.0,
            skipped_blinds: Vec::new(),
            blind_defeated_this_ante: [false; 3],
            deck: Vec::new(),
            draw_pile: Vec::new(),
            hand: Vec::new(),
            discard_pile: Vec::new(),
            jokers: Vec::new(),
            consumables: Vec::new(),
            hands_remaining: 4,
            discards_remaining: 3,
            score_accumulated: 0.0,
            selected_indices: Vec::new(),
            hand_levels,
            shop_offers: Vec::new(),
            shop_voucher: None,
            reroll_cost: 5,
            free_rerolls: 0,
            current_pack: None,
            hand_size: 8,
            max_hands: 4,
            max_discards: 3,
            joker_slots: 5,
            consumable_slots: 2,
            max_interest: 25,
            history: Vec::new(),
            next_id: 1,
            last_consumable_used: None,
        };

        // Apply deck-type modifications
        gs.apply_deck_init();

        // Build and shuffle the deck
        gs.build_deck();

        // Pick boss blind for ante 1
        gs.boss_blind = gs.pick_boss_blind();

        gs
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn apply_deck_init(&mut self) {
        match self.deck_type {
            DeckType::Red => {
                self.max_discards += 1;
            }
            DeckType::Blue => {
                self.max_hands += 1;
            }
            DeckType::Yellow => {
                self.money += 10;
            }
            DeckType::Black => {
                self.max_hands = self.max_hands.saturating_sub(1);
                self.joker_slots += 1;
            }
            DeckType::Painted => {
                self.hand_size += 2;
                self.joker_slots = self.joker_slots.saturating_sub(1);
            }
            DeckType::Abandoned => {
                // No face cards in deck (handled in build_deck)
            }
            DeckType::Magic => {
                // Start with Crystal Ball voucher + 2× The Fool tarot cards
                self.vouchers.push(VoucherKind::CrystalBall);
                self.consumable_slots += 1; // Crystal Ball gives +1 consumable slot
                self.consumables.push(ConsumableCard::Tarot(TarotCard::TheFool));
                self.consumables.push(ConsumableCard::Tarot(TarotCard::TheFool));
            }
            DeckType::Nebula => {
                // Start with Telescope voucher
                self.vouchers.push(VoucherKind::Telescope);
            }
            DeckType::Zodiac => {
                // Start with Tarot Merchant, Planet Merchant, Overstock vouchers
                self.vouchers.push(VoucherKind::TarotMerchant);
                self.vouchers.push(VoucherKind::PlanetMerchant);
                self.vouchers.push(VoucherKind::Overstock);
            }
            _ => {}
        }
        self.hands_remaining = self.max_hands;
        self.discards_remaining = self.max_discards;
    }

    pub fn build_deck(&mut self) {
        let mut cards = Vec::new();
        let suits = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
        let ranks = [
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
            Rank::Ace,
        ];

        for &suit in &suits {
            for &rank in &ranks {
                // Abandoned deck: skip face cards
                if self.deck_type == DeckType::Abandoned && rank.is_face() {
                    continue;
                }

                let effective_suit = match self.deck_type {
                    DeckType::Checkered => match suit {
                        Suit::Clubs => Suit::Spades,
                        Suit::Diamonds => Suit::Hearts,
                        s => s,
                    },
                    _ => suit,
                };

                let id = self.next_id();
                let mut card = CardInstance::new(id, rank, effective_suit);

                // Erratic deck: randomize rank and suit
                if self.deck_type == DeckType::Erratic {
                    let new_rank_idx = self.rng.range_u32(0, 12) as usize;
                    card.rank = ranks[new_rank_idx];
                    let new_suit_idx = self.rng.range_u32(0, 3) as usize;
                    card.suit = suits[new_suit_idx];
                }

                cards.push(card);
            }
        }

        // Shuffle
        self.rng.shuffle(&mut cards);
        self.deck = cards;
        self.draw_pile = (0..self.deck.len()).collect();
    }

    pub fn get_blind_chip_goal(&self) -> f64 {
        let base = get_base_blind_amount(self.ante);
        let mult = match self.current_blind {
            BlindKind::Small => 1.0,
            BlindKind::Big => 1.5,
            BlindKind::Boss => {
                if let Some(boss) = self.boss_blind {
                    boss.chip_multiplier()
                } else {
                    2.0
                }
            }
        };
        // Apply Violet Vessel (6x) or standard adjustments
        let scaling = match self.stake {
            Stake::White => 1.0,
            Stake::Red => 1.0,
            Stake::Green => 1.0,
            Stake::Black | Stake::Blue | Stake::Purple | Stake::Orange | Stake::Gold => 1.0,
        };
        (base as f64) * mult * scaling
    }

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

        // Record skip for tags / Throwback joker
        self.skipped_blinds.push((self.ante, self.round));
        for j in self.jokers.iter_mut() {
            if j.kind == JokerKind::Throwback {
                let skips = j.get_counter_i64("skips");
                j.set_counter_i64("skips", skips + 1);
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

        // Reset per-round hand played counters
        for data in self.hand_levels.values_mut() {
            data.played_this_round = 0;
        }

        // Reset draw pile
        self.draw_pile = (0..self.deck.len()).collect();
        self.rng.shuffle(&mut self.draw_pile);

        // Apply boss blind debuffs to cards
        self.apply_boss_blind_debuffs();

        // Set score goal
        self.score_goal = self.get_blind_chip_goal();

        // Draw initial hand
        self.draw_to_hand();

        // Notify jokers of blind selection
        self.notify_jokers_setting_blind();
    }

    fn effective_max_hands(&self) -> u32 {
        let mut hands = self.max_hands;
        for j in &self.jokers {
            if !j.active {
                continue;
            }
            match j.kind {
                JokerKind::Troubadour => {
                    hands = hands.saturating_sub(1);
                }
                JokerKind::BurntJoker => {
                    // +? hands (tracked per-joker)
                }
                _ => {}
            }
        }
        hands
    }

    fn effective_max_discards(&self) -> u32 {
        let mut discards = self.max_discards;
        // Black stake and above: -1 discard per round
        if self.stake as u8 >= Stake::Black as u8 {
            discards = discards.saturating_sub(1);
        }
        for j in &self.jokers {
            if !j.active {
                continue;
            }
            match j.kind {
                JokerKind::MerryAndy => discards += 3,
                JokerKind::Drunkard => discards += 1,
                _ => {}
            }
        }
        discards
    }

    pub fn effective_hand_size(&self) -> u32 {
        let mut size = self.hand_size;
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

    fn draw_to_hand(&mut self) {
        let hand_size = self.effective_hand_size() as usize;
        while self.hand.len() < hand_size && !self.draw_pile.is_empty() {
            let card_idx = self.draw_pile.remove(0);
            self.hand.push(card_idx);
        }
        // Apply Dusk joker: DNA copies first card of first hand of round
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
                _ => {}
            }
        }
    }

    pub fn select_card(&mut self, hand_index: usize) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        if hand_index >= self.hand.len() {
            return Err(BalatroError::IndexOutOfRange(hand_index, self.hand.len()));
        }
        if self.selected_indices.contains(&hand_index) {
            return Ok(()); // already selected
        }
        // Psychic boss: must play exactly 5
        // Max selected is 5 cards
        if self.selected_indices.len() >= 5 {
            return Err(BalatroError::TooManySelected);
        }
        self.selected_indices.push(hand_index);
        Ok(())
    }

    pub fn deselect_card(&mut self, hand_index: usize) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        if hand_index >= self.hand.len() {
            return Err(BalatroError::IndexOutOfRange(hand_index, self.hand.len()));
        }
        self.selected_indices.retain(|&x| x != hand_index);
        Ok(())
    }

    pub fn deselect_all(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        self.selected_indices.clear();
        Ok(())
    }

    pub fn select_cards_by_rank(&mut self, rank: Rank) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        for i in 0..self.hand.len() {
            let card_idx = self.hand[i];
            if self.deck[card_idx].rank == rank && !self.selected_indices.contains(&i) {
                if self.selected_indices.len() < 5 {
                    self.selected_indices.push(i);
                }
            }
        }
        Ok(())
    }

    pub fn deselect_cards_by_suit(&mut self, suit: Suit) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        let to_deselect: Vec<usize> = self
            .selected_indices
            .iter()
            .filter(|&&i| {
                let card_idx = self.hand[i];
                self.deck[card_idx].suit == suit
            })
            .copied()
            .collect();
        for i in to_deselect {
            self.selected_indices.retain(|&x| x != i);
        }
        Ok(())
    }

    pub fn play_hand(&mut self) -> Result<ScoreResult, BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        if self.selected_indices.is_empty() {
            return Err(BalatroError::NoCardsSelected);
        }
        if self.hands_remaining == 0 {
            return Err(BalatroError::NoHandsRemaining);
        }

        // Needle boss: only 1 hand
        if let Some(BossBlind::TheNeedle) = self.boss_blind {
            if !matches!(self.current_blind, BlindKind::Boss) {
                // Only apply to boss blind
            } else if self.hands_remaining < self.effective_max_hands() {
                return Err(BalatroError::BossBlindEffect("The Needle: only 1 hand allowed".to_string()));
            }
        }

        let played_hand_indices: Vec<usize> = self
            .selected_indices
            .iter()
            .map(|&hi| self.hand[hi])
            .collect();

        let played_cards: Vec<CardInstance> = played_hand_indices
            .iter()
            .map(|&i| self.deck[i].clone())
            .collect();

        let hand_card_indices: Vec<usize> = self
            .hand
            .iter()
            .filter(|&&i| !played_hand_indices.contains(&i))
            .copied()
            .collect();
        let hand_cards: Vec<CardInstance> = hand_card_indices
            .iter()
            .map(|&i| self.deck[i].clone())
            .collect();

        let result = score_hand(
            &played_cards,
            &hand_cards,
            &self.jokers,
            &self.hand_levels,
            self.hands_remaining - 1,
            self.discards_remaining,
            self.money,
            self.draw_pile.len(),
            self.deck.len(),
            self.boss_blind,
            self.joker_slots as usize,
            self.tarot_cards_used,
        );

        // Update hand level stats
        if let Some(level) = self.hand_levels.get_mut(&result.hand_type) {
            level.played += 1;
            level.played_this_round += 1;
        }

        // Plasma deck: balance chips and mult (replace both with their average)
        let final_score = if self.deck_type == DeckType::Plasma {
            let avg = (result.final_chips + result.final_mult) / 2.0;
            avg * avg
        } else {
            result.final_score
        };

        self.score_accumulated += final_score;
        self.hands_remaining -= 1;

        // Post-scoring joker updates
        self.post_play_joker_updates(&result, &played_cards, &hand_cards);

        // Earn dollars from scoring
        self.money += result.dollars_earned;

        // Tooth boss: -$1 per card played
        if let Some(BossBlind::TheTooth) = self.boss_blind {
            if matches!(self.current_blind, BlindKind::Boss) {
                self.money -= played_cards.len() as i32;
            }
        }

        // Discard played cards, draw new ones
        let played_sel = self.selected_indices.clone();
        // Remove played cards from hand (in reverse order to maintain indices)
        let mut sorted_sel = played_sel.clone();
        sorted_sel.sort_by(|a, b| b.cmp(a));
        for si in &sorted_sel {
            let card_idx = self.hand.remove(*si);
            self.discard_pile.push(card_idx);
        }
        self.selected_indices.clear();

        // Process glass cards: chance to destroy
        for card in &played_cards {
            if card.enhancement == Enhancement::Glass {
                // 1/4 chance to break
                if self.rng.next_bool_prob(0.25) {
                    // Remove card from deck (destroy_deck_card remaps all index collections)
                    self.destroy_deck_card(card.id);
                    self.notify_face_card_destroyed(card);
                    self.history.push(HistoryEvent {
                        ante: self.ante,
                        round: self.round,
                        event_type: "card_destroyed".to_string(),
                        data: serde_json::json!({
                            "reason": "glass_break",
                            "card": format!("{:?} of {:?}", card.rank, card.suit)
                        }),
                    });
                }
            }
        }

        // Dusk joker: retrigger scoring cards on last hand
        let is_last_hand = self.hands_remaining == 0;
        if is_last_hand && self.jokers.iter().any(|j| j.kind == JokerKind::Dusk && j.active) {
            // Would trigger retriggers (already handled in scoring via retrigger count)
        }

        // Plasma deck: if score_accumulated >= goal, balance chips/mult
        // (already handled in scoring)

        // Log the hand play
        self.history.push(HistoryEvent {
            ante: self.ante,
            round: self.round,
            event_type: "hand_played".to_string(),
            data: serde_json::json!({
                "hand_type": result.hand_name,
                "chips": result.final_chips,
                "mult": result.final_mult,
                "score": result.final_score,
                "accumulated": self.score_accumulated,
                "goal": self.score_goal,
            }),
        });

        // Draw up to hand size
        self.draw_to_hand();

        // Check for round win
        if self.score_accumulated >= self.score_goal {
            self.win_round();
        } else if self.hands_remaining == 0 {
            // Out of hands, didn't meet goal
            // Check Mr. Bones joker
            let mr_bones = self.jokers.iter().any(|j| j.kind == JokerKind::MrBones && !j.eternal);
            if mr_bones && self.score_accumulated >= self.score_goal / 4.0 {
                // Mr. Bones saves you (then gets destroyed)
                self.jokers.retain(|j| j.kind != JokerKind::MrBones);
                self.win_round();
            } else {
                self.state = GameStateKind::GameOver;
                self.history.push(HistoryEvent {
                    ante: self.ante,
                    round: self.round,
                    event_type: "game_over".to_string(),
                    data: serde_json::json!({
                        "score": self.score_accumulated,
                        "goal": self.score_goal,
                    }),
                });
            }
        }

        Ok(result)
    }

    fn post_play_joker_updates(&mut self, result: &ScoreResult, played: &[CardInstance], hand: &[CardInstance]) {
        let hand_type = result.hand_type;
        for i in 0..self.jokers.len() {
            let kind = self.jokers[i].kind;
            match kind {
                JokerKind::Runner => {
                    // +15 chips if straight
                    if hand_type == HandType::Straight || hand_type == HandType::StraightFlush {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 15);
                    }
                }
                JokerKind::IceCream => {
                    // -5 chips per hand played
                    let cur = self.jokers[i].get_counter_i64("chips");
                    let new = (cur - 5).max(0);
                    self.jokers[i].set_counter_i64("chips", new);
                    if new == 0 {
                        // Destroy joker
                        self.jokers[i].active = false;
                    }
                }
                JokerKind::Popcorn => {
                    let cur = self.jokers[i].get_counter_i64("mult");
                    let new = (cur - 4).max(0);
                    self.jokers[i].set_counter_i64("mult", new);
                    if new == 0 {
                        self.jokers[i].active = false;
                    }
                }
                JokerKind::SquareJoker => {
                    // +4 chips if 4 cards played exactly
                    if played.len() == 4 {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 4);
                    }
                }
                JokerKind::WeeJoker => {
                    // +8 chips for each 2 played
                    let twos = played.iter().filter(|c| c.rank == Rank::Two).count();
                    if twos > 0 {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 8 * twos as i64);
                    }
                }
                JokerKind::GreenJoker => {
                    // +1 mult per hand played, -1 per discard
                    let cur = self.jokers[i].get_counter_i64("mult");
                    self.jokers[i].set_counter_i64("mult", cur + 1);
                }
                JokerKind::SpareTrousers => {
                    if hand_type == HandType::TwoPair {
                        let cur = self.jokers[i].get_counter_i64("mult");
                        self.jokers[i].set_counter_i64("mult", cur + 2);
                    }
                }
                JokerKind::RideTheBus => {
                    let scoring_has_face = result
                        .scoring_card_indices
                        .iter()
                        .any(|&i| played[i].rank.is_face());
                    if !scoring_has_face {
                        let cur = self.jokers[i].get_counter_i64("mult");
                        self.jokers[i].set_counter_i64("mult", cur + 1);
                    } else {
                        self.jokers[i].set_counter_i64("mult", 0);
                    }
                }
                JokerKind::Hologram => {
                    // +0.25 Xmult for each playing card added to deck
                    // (tracked when cards are added to deck)
                }
                JokerKind::Vampire => {
                    // +0.1 Xmult for each enhanced card scored, remove enhancement
                    let enhanced: Vec<usize> = result
                        .scoring_card_indices
                        .iter()
                        .filter(|&&si| played[si].enhancement != Enhancement::None)
                        .copied()
                        .collect();
                    if !enhanced.is_empty() {
                        let cur = self.jokers[i].get_counter_f64("x_mult");
                        let new = cur + 0.1 * enhanced.len() as f64;
                        self.jokers[i].set_counter_f64("x_mult", new);
                        // Remove enhancements from those cards in deck
                        for si in enhanced {
                            let deck_idx = self.hand
                                .iter()
                                .find(|&&di| {
                                    self.deck[di].id == played[si].id
                                });
                            if let Some(&di) = deck_idx {
                                self.deck[di].enhancement = Enhancement::None;
                            }
                        }
                    }
                }
                JokerKind::Obelisk => {
                    // Xmult increases if you play different hand types
                    // Track which hand type was played most
                    // simplified: +0.2 if this hand type is NOT the most played
                    let this_plays = self
                        .hand_levels
                        .get(&hand_type)
                        .map(|h| h.played)
                        .unwrap_or(0);
                    let most_plays = self
                        .hand_levels
                        .values()
                        .map(|h| h.played)
                        .max()
                        .unwrap_or(0);
                    if this_plays < most_plays {
                        let cur = self.jokers[i].get_counter_f64("x_mult");
                        self.jokers[i].set_counter_f64("x_mult", cur + 0.2);
                    } else {
                        // Reset obelisk if this is the most played
                        self.jokers[i].set_counter_f64("x_mult", 1.0);
                    }
                }
                JokerKind::Madness => {
                    // +0.5 Xmult when blind is entered (done at begin_round)
                }
                JokerKind::Castle => {
                    // +3 chips for each card of the current suit discarded
                    // (handled in discard)
                }
                JokerKind::FlashCard => {
                    // +2 mult per reroll (handled in shop)
                }
                JokerKind::Campfire => {
                    // +0.25 Xmult for each joker sold
                    // (handled in sell_joker)
                }
                _ => {}
            }
        }
    }

    pub fn discard_hand(&mut self) -> Result<(), BalatroError> {
        if !matches!(self.state, GameStateKind::Round) {
            return Err(BalatroError::NotInRound);
        }
        if self.selected_indices.is_empty() {
            return Err(BalatroError::NoCardsSelected);
        }
        if self.discards_remaining == 0 {
            return Err(BalatroError::NoDiscardsRemaining);
        }

        // Water boss: reduce discards (tracked separately)
        let discarded_cards: Vec<CardInstance> = self
            .selected_indices
            .iter()
            .map(|&hi| self.deck[self.hand[hi]].clone())
            .collect();

        // Sort indices in reverse to maintain validity during removal
        let mut sorted_sel = self.selected_indices.clone();
        sorted_sel.sort_by(|a, b| b.cmp(a));
        for si in &sorted_sel {
            let card_idx = self.hand.remove(*si);
            self.discard_pile.push(card_idx);

            // Purple Seal: create tarot when discarded
            if self.deck[card_idx].seal == Seal::Purple {
                // Add a random tarot to consumables if space
                if self.consumables.len() < self.consumable_slots as usize {
                    let tarot = self.random_tarot();
                    self.consumables.push(ConsumableCard::Tarot(tarot));
                }
            }
        }
        self.selected_indices.clear();
        self.discards_remaining -= 1;

        // Post-discard joker updates
        for i in 0..self.jokers.len() {
            let kind = self.jokers[i].kind;
            match kind {
                JokerKind::GreenJoker => {
                    let cur = self.jokers[i].get_counter_i64("mult");
                    self.jokers[i].set_counter_i64("mult", (cur - 1).max(0));
                }
                JokerKind::Yorick => {
                    // Count individual cards discarded, not discard actions
                    let cards_this_discard = discarded_cards.len() as i64;
                    let prev = self.jokers[i].get_counter_i64("discards");
                    let new_total = prev + cards_this_discard;
                    self.jokers[i].set_counter_i64("discards", new_total);
                    // Gain +1 Xmult for every 23rd card discarded
                    let prev_milestones = prev / 23;
                    let new_milestones = new_total / 23;
                    if new_milestones > prev_milestones {
                        let gained = new_milestones - prev_milestones;
                        let cur = self.jokers[i].get_counter_f64("x_mult");
                        self.jokers[i].set_counter_f64("x_mult", cur + gained as f64);
                    }
                }
                JokerKind::Castle => {
                    let suit_str = self.jokers[i]
                        .counters
                        .get("suit")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Spades")
                        .to_string();
                    let target_suit = match suit_str.as_str() {
                        "Spades" => Suit::Spades,
                        "Hearts" => Suit::Hearts,
                        "Clubs" => Suit::Clubs,
                        "Diamonds" => Suit::Diamonds,
                        _ => Suit::Spades,
                    };
                    let count = discarded_cards
                        .iter()
                        .filter(|c| c.suit == target_suit)
                        .count();
                    if count > 0 {
                        let cur = self.jokers[i].get_counter_i64("chips");
                        self.jokers[i].set_counter_i64("chips", cur + 3 * count as i64);
                    }
                }
                _ => {}
            }
        }

        // Blue Seal: create planet when card held in hand at round end
        // (handled at round end, not here)

        // Hook boss blind: discard 2 random cards
        // (applied before player discards normally)

        // (Green deck end-of-round money is handled in win_round)

        self.history.push(HistoryEvent {
            ante: self.ante,
            round: self.round,
            event_type: "discarded".to_string(),
            data: serde_json::json!({
                "cards": discarded_cards.iter().map(|c| format!("{:?} of {:?}", c.rank, c.suit)).collect::<Vec<_>>(),
            }),
        });

        self.draw_to_hand();
        Ok(())
    }

    fn win_round(&mut self) {
        let blind_dollars = self.blind_reward_dollars();
        self.money += blind_dollars;

        if self.deck_type == DeckType::Green {
            // Green deck: +$1 per remaining hand, +$1 per remaining discard; no interest
            self.money += self.hands_remaining as i32;
            self.money += self.discards_remaining as i32;
        } else {
            // Interest: $1 per $5 held, up to max_interest
            let interest = (self.money / 5).min(self.max_interest / 5).max(0);
            self.money += interest;
        }

        // Perishable jokers: decrement rounds remaining; disable when expired
        for j in self.jokers.iter_mut() {
            if j.perishable && j.active {
                if j.perishable_rounds_left > 0 {
                    j.perishable_rounds_left -= 1;
                }
                if j.perishable_rounds_left == 0 {
                    j.active = false;
                }
            }
        }

        // Blue seal: create planet for cards in hand at round end
        let hand_copy: Vec<usize> = self.hand.clone();
        for &card_idx in &hand_copy {
            if self.deck[card_idx].seal == Seal::Blue {
                if self.consumables.len() < self.consumable_slots as usize {
                    let planet = self.random_planet();
                    self.consumables.push(ConsumableCard::Planet(planet));
                }
            }
        }

        // Log victory
        self.history.push(HistoryEvent {
            ante: self.ante,
            round: self.round,
            event_type: "round_won".to_string(),
            data: serde_json::json!({
                "score": self.score_accumulated,
                "goal": self.score_goal,
                "dollars_earned": blind_dollars,
            }),
        });

        // Mark blind as defeated
        match self.current_blind {
            BlindKind::Small => self.blind_defeated_this_ante[0] = true,
            BlindKind::Big => self.blind_defeated_this_ante[1] = true,
            BlindKind::Boss => self.blind_defeated_this_ante[2] = true,
        }

        // Anaglyph deck: gain a Double Tag (DoubleFun) after defeating each Boss Blind
        if matches!(self.current_blind, BlindKind::Boss) && self.deck_type == DeckType::Anaglyph {
            self.tags.push(TagKind::DoubleFun);
        }

        // Check if ante 8 boss beaten = game won
        if self.ante >= 8 && matches!(self.current_blind, BlindKind::Boss) {
            self.history.push(HistoryEvent {
                ante: self.ante,
                round: self.round,
                event_type: "game_won".to_string(),
                data: serde_json::json!({}),
            });
            self.state = GameStateKind::GameOver;
            return;
        }

        // If boss blind won, advance to next ante
        if matches!(self.current_blind, BlindKind::Boss) {
            // Advance to shop before going to next ante
            self.state = GameStateKind::Shop;
            self.generate_shop();
        } else {
            // Small/Big blind won → go to shop
            self.state = GameStateKind::Shop;
            self.generate_shop();
        }
    }

    fn blind_reward_dollars(&self) -> i32 {
        match (self.current_blind.clone(), self.boss_blind) {
            (BlindKind::Small, _) => 3,
            (BlindKind::Big, _) => 4,
            (BlindKind::Boss, Some(b)) => {
                // boss blinds give 5$ (showdowns give 8$)
                match b {
                    BossBlind::CeruleanBell
                    | BossBlind::VerdantLeaf
                    | BossBlind::VioletVessel
                    | BossBlind::AmberAcorn
                    | BossBlind::CrimsonHeart => 8,
                    _ => 5,
                }
            }
            _ => 5,
        }
    }

    fn advance_blind(&mut self) {
        match self.current_blind {
            BlindKind::Small => {
                self.current_blind = BlindKind::Big;
                self.round = 2;
            }
            BlindKind::Big => {
                self.current_blind = BlindKind::Boss;
                self.round = 3;
            }
            BlindKind::Boss => {
                self.ante += 1;
                self.round = 1;
                self.current_blind = BlindKind::Small;
                self.blind_defeated_this_ante = [false; 3];
                self.boss_blind = self.pick_boss_blind();
            }
        }
    }

    // =========================================================
    // Shop Actions
    // =========================================================

    fn generate_shop(&mut self) {
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
    }

    fn generate_random_joker(&mut self) -> Option<JokerInstance> {
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

    fn random_tarot(&mut self) -> TarotCard {
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

    fn random_planet(&mut self) -> PlanetCard {
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

        let price = self.calculate_shop_price(offer.price);
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

    // =========================================================
    // Pack Actions
    // =========================================================

    fn generate_pack_contents(&mut self, kind: PackKind) -> PackContents {
        let cards_shown = kind.cards_shown();
        let picks = kind.picks_allowed();
        let mut cards = Vec::new();

        match kind {
            PackKind::ArcanaPack
            | PackKind::ArcanaPackSmall
            | PackKind::ArcanaPackJumbo
            | PackKind::ArcanaPackMega => {
                for _ in 0..cards_shown {
                    let t = self.random_tarot();
                    cards.push(PackCard::Consumable(ConsumableCard::Tarot(t)));
                }
            }
            PackKind::CelestialPack
            | PackKind::CelestialPackSmall
            | PackKind::CelestialPackJumbo
            | PackKind::CelestialPackMega => {
                for _ in 0..cards_shown {
                    let p = self.random_planet();
                    cards.push(PackCard::Consumable(ConsumableCard::Planet(p)));
                }
            }
            PackKind::SpectralPack
            | PackKind::SpectralPackSmall
            | PackKind::SpectralPackJumbo
            | PackKind::SpectralPackMega => {
                let spectrals = vec![
                    SpectralCard::Familiar,
                    SpectralCard::Grim,
                    SpectralCard::Incantation,
                    SpectralCard::Talisman,
                    SpectralCard::Aura,
                    SpectralCard::Wraith,
                    SpectralCard::Ectoplasm,
                    SpectralCard::Immolate,
                    SpectralCard::Ankh,
                    SpectralCard::DejaVu,
                    SpectralCard::Hex,
                    SpectralCard::Trance,
                    SpectralCard::Medium,
                    SpectralCard::Cryptid,
                ];
                for _ in 0..cards_shown {
                    let idx = self.rng.range_usize(0, spectrals.len() - 1);
                    cards.push(PackCard::Consumable(ConsumableCard::Spectral(spectrals[idx])));
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
                    let card = CardInstance::new(id, ranks[rank_idx], suits[suit_idx]);
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
                    // Track planet/tarot usage
                    match c {
                        ConsumableCard::Planet(_) => self.planet_cards_used += 1,
                        ConsumableCard::Tarot(_) => self.tarot_cards_used += 1,
                        _ => {}
                    }
                    // Apply planet/tarot immediately? No - user uses it separately via use_consumable
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

    pub fn use_consumable(&mut self, consumable_index: usize, targets: Vec<usize>) -> Result<(), BalatroError> {
        if consumable_index >= self.consumables.len() {
            return Err(BalatroError::IndexOutOfRange(consumable_index, self.consumables.len()));
        }

        let consumable = self.consumables[consumable_index].clone();
        match &consumable {
            ConsumableCard::Planet(p) => {
                self.apply_planet(*p);
                self.planet_cards_used += 1;
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
        Ok(())
    }

    fn apply_planet(&mut self, planet: PlanetCard) {
        let hand_type = planet.hand_type();
        if let Some(level) = self.hand_levels.get_mut(&hand_type) {
            level.level += 1;
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
                // 1/4 chance to add random edition to random joker
                if !self.jokers.is_empty() && self.rng.next_bool_prob(0.25) {
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

    pub fn hand_cards(&self) -> Vec<(usize, &CardInstance)> {
        self.hand
            .iter()
            .enumerate()
            .map(|(hi, &di)| (hi, &self.deck[di]))
            .collect()
    }

    pub fn is_card_selected(&self, hand_index: usize) -> bool {
        self.selected_indices.contains(&hand_index)
    }

    /// Notify Canio joker when a card is destroyed — if it's a face card, Canio gains +1 Xmult.
    /// Remove a single card from `deck` by ID and remap all index collections.
    /// Use this instead of `deck.retain(...)` to avoid stale indices in hand/draw_pile/discard_pile.
    fn destroy_deck_card(&mut self, card_id: u64) {
        if let Some(pos) = self.deck.iter().position(|c| c.id == card_id) {
            self.deck.remove(pos);
            // Any stored index > pos shifts down by 1; index == pos is now gone (caller must have
            // already removed it from hand/discard_pile before calling this).
            for idx in self.hand.iter_mut() {
                if *idx > pos { *idx -= 1; }
            }
            for idx in self.draw_pile.iter_mut() {
                if *idx > pos { *idx -= 1; }
            }
            for idx in self.discard_pile.iter_mut() {
                if *idx > pos { *idx -= 1; }
            }
        }
    }

    /// Remove multiple cards from `deck` by IDs, remapping indices correctly.
    fn destroy_deck_cards(&mut self, card_ids: &[u64]) {
        for &id in card_ids {
            self.destroy_deck_card(id);
        }
    }

    fn notify_face_card_destroyed(&mut self, card: &CardInstance) {
        if !card.rank.is_face() {
            return;
        }
        for j in self.jokers.iter_mut() {
            if j.kind == JokerKind::Canio && j.active {
                let cur = j.get_counter_f64("x_mult");
                j.set_counter_f64("x_mult", cur + 1.0);
            }
        }
    }
}

fn rank_up(rank: Rank) -> Rank {
    match rank {
        Rank::Two => Rank::Three,
        Rank::Three => Rank::Four,
        Rank::Four => Rank::Five,
        Rank::Five => Rank::Six,
        Rank::Six => Rank::Seven,
        Rank::Seven => Rank::Eight,
        Rank::Eight => Rank::Nine,
        Rank::Nine => Rank::Ten,
        Rank::Ten => Rank::Jack,
        Rank::Jack => Rank::Queen,
        Rank::Queen => Rank::King,
        Rank::King => Rank::Ace,
        Rank::Ace => Rank::Ace, // Can't go higher
    }
}

fn upgraded_voucher(base: VoucherKind) -> VoucherKind {
    match base {
        VoucherKind::Overstock => VoucherKind::OverstockPlus,
        VoucherKind::ClearanceSale => VoucherKind::Liquidation,
        VoucherKind::Hone => VoucherKind::GlowUp,
        VoucherKind::RerollSurplus => VoucherKind::RerollGlut,
        VoucherKind::CrystalBall => VoucherKind::OmenGlobe,
        VoucherKind::Telescope => VoucherKind::Observatory,
        VoucherKind::Grabber => VoucherKind::NachoTong,
        VoucherKind::Wasteful => VoucherKind::Recyclomancy,
        VoucherKind::TarotMerchant => VoucherKind::TarotTycoon,
        VoucherKind::PlanetMerchant => VoucherKind::PlanetTycoon,
        VoucherKind::SeedMoney => VoucherKind::MoneyTree,
        VoucherKind::Blank => VoucherKind::Antimatter,
        VoucherKind::MagicTrick => VoucherKind::Illusion,
        VoucherKind::Hieroglyph => VoucherKind::Petroglyph,
        VoucherKind::DirectorsCut => VoucherKind::Retcon,
        VoucherKind::PaintBrush => VoucherKind::Palette,
        // Already top-tier — return self
        other => other,
    }
}

pub fn get_base_blind_amount(ante: u32) -> u64 {
    let amounts: [u64; 8] = [300, 800, 2000, 5000, 11000, 20000, 35000, 50000];
    if ante == 0 {
        return 100;
    }
    if ante <= 8 {
        return amounts[(ante - 1) as usize];
    }
    // Scale exponentially for ante > 8
    let k = 0.75_f64;
    let a = 50000_f64;
    let b = 1.6_f64;
    let c = (ante - 8) as f64;
    let d = 1.0 + 0.2 * c;
    let amount = (a * (b + (k * c).powf(d)).powf(c)).floor() as u64;
    // Round to significant figures
    if amount < 10 {
        return amount;
    }
    let log = (amount as f64).log10().floor() as u32;
    let factor = 10u64.pow(log.saturating_sub(1));
    (amount / factor) * factor
}

// Error types
#[derive(Debug, Clone)]
pub enum BalatroError {
    NotInBlindSelect,
    NotInRound,
    NotInShop,
    NotInPack,
    CannotSkipBoss,
    NoCardsSelected,
    TooManySelected,
    NoHandsRemaining,
    NoDiscardsRemaining,
    NoPicksRemaining,
    IndexOutOfRange(usize, usize),
    NotEnoughMoney(u32, u32),
    JokerSlotsFull,
    ConsumableSlotsFull,
    AlreadySold,
    WrongItemType(String),
    EternalCard,
    NoVoucherAvailable,
    BossBlindEffect(String),
}

impl std::fmt::Display for BalatroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalatroError::NotInBlindSelect => write!(f, "Not in blind selection phase"),
            BalatroError::NotInRound => write!(f, "Not in round"),
            BalatroError::NotInShop => write!(f, "Not in shop"),
            BalatroError::NotInPack => write!(f, "Not opening a pack"),
            BalatroError::CannotSkipBoss => write!(f, "Cannot skip the boss blind"),
            BalatroError::NoCardsSelected => write!(f, "No cards selected"),
            BalatroError::TooManySelected => write!(f, "Too many cards selected (max 5)"),
            BalatroError::NoHandsRemaining => write!(f, "No hands remaining"),
            BalatroError::NoDiscardsRemaining => write!(f, "No discards remaining"),
            BalatroError::NoPicksRemaining => write!(f, "No picks remaining in pack"),
            BalatroError::IndexOutOfRange(i, max) => {
                write!(f, "Index {} out of range (max {})", i, max)
            }
            BalatroError::NotEnoughMoney(need, have) => {
                write!(f, "This costs ${} but you have ${}", need, have)
            }
            BalatroError::JokerSlotsFull => write!(f, "Joker slots are full"),
            BalatroError::ConsumableSlotsFull => write!(f, "Consumable slots are full"),
            BalatroError::AlreadySold => write!(f, "This item has already been sold"),
            BalatroError::WrongItemType(msg) => write!(f, "Wrong item type: {}", msg),
            BalatroError::EternalCard => write!(f, "Cannot sell an Eternal card"),
            BalatroError::NoVoucherAvailable => write!(f, "No voucher available in shop"),
            BalatroError::BossBlindEffect(msg) => write!(f, "Boss blind effect: {}", msg),
        }
    }
}

impl std::error::Error for BalatroError {}
