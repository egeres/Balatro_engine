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
    /// Tracks which distinct PlanetCard types have been used this run (for Satellite joker)
    pub planet_types_used: std::collections::HashSet<PlanetCard>,

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

    // Showdown boss blind state
    /// CeruleanBell: ID of the card that is always forced-selected this draw.
    pub cerulean_forced_card_id: Option<u64>,
    /// VerdantLeaf: set to true once the first joker is sold this blind.
    pub verdant_leaf_joker_sold: bool,

    /// ThePillar: IDs of cards played in earlier rounds of the current Ante.
    /// Cleared when a new Ante begins. Used to debuff those cards during the Boss blind.
    pub played_card_ids_this_ante: Vec<u64>,
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
            planet_types_used: std::collections::HashSet::new(),
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
            cerulean_forced_card_id: None,
            verdant_leaf_joker_sold: false,
            played_card_ids_this_ante: Vec::new(),
        };

        // Apply deck-type modifications
        gs.apply_deck_init();

        // Build and shuffle the deck
        gs.build_deck();

        // Pick boss blind for ante 1
        gs.boss_blind = gs.pick_boss_blind();

        gs
    }

    pub(crate) fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Swap two jokers by position. Order matters for Blueprint/Brainstorm.
    /// Valid at any point in the game; returns `IndexOutOfRange` if either index is out of bounds.
    pub fn swap_jokers(&mut self, a: usize, b: usize) -> Result<(), BalatroError> {
        let len = self.jokers.len();
        if a >= len {
            return Err(BalatroError::IndexOutOfRange(a, len));
        }
        if b >= len {
            return Err(BalatroError::IndexOutOfRange(b, len));
        }
        self.jokers.swap(a, b);
        Ok(())
    }

    /// Swap two consumables (tarots, planets, spectrals) by position.
    /// Valid at any point in the game; returns `IndexOutOfRange` if either index is out of bounds.
    pub fn swap_consumables(&mut self, a: usize, b: usize) -> Result<(), BalatroError> {
        let len = self.consumables.len();
        if a >= len {
            return Err(BalatroError::IndexOutOfRange(a, len));
        }
        if b >= len {
            return Err(BalatroError::IndexOutOfRange(b, len));
        }
        self.consumables.swap(a, b);
        Ok(())
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
}

mod blind;
mod round;
mod shop;
mod pack;
mod consumable;

impl GameState {
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
    pub(crate) fn destroy_deck_card(&mut self, card_id: u64) {
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
    pub(crate) fn destroy_deck_cards(&mut self, card_ids: &[u64]) {
        for &id in card_ids {
            self.destroy_deck_card(id);
        }
    }

    pub(crate) fn notify_face_card_destroyed(&mut self, card: &CardInstance) {
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

pub(crate) fn rank_up(rank: Rank) -> Rank {
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

pub(crate) fn upgraded_voucher(base: VoucherKind) -> VoucherKind {
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
