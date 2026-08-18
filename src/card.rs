use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInstance {
    pub id: u64,
    pub rank: Rank,
    pub suit: Suit,
    pub enhancement: Enhancement,
    pub edition: Edition,
    pub seal: Seal,
    pub debuffed: bool,
    /// Card is face-down (hidden from player): TheFish draws all new cards face-down;
    /// TheWheel has a 1-in-7 chance per card. Cards still score normally.
    pub face_down: bool,
    /// For chip_modifier from Hiker/Wee Joker/etc
    pub extra_chips: i64,
    /// For flip cards (Certificate joker)
    pub extra_mult: i64,
    /// Pre-rolled x_mult bonus (e.g. Bloodstone 1/2 chance x1.5, pre-rolled by game loop)
    pub extra_x_mult: f64,
}

/// `Card:set_cost` up to and including the rental override (card.lua:375-381).
///
/// The edition surcharge rides on the base cost, the run's discount applies once, and a rental
/// is a flat dollar. Astronomer and Coupon land after this and are the shop's business — sell
/// value is derived from *this* number, which is why a couponed card still sells for its worth.
pub fn card_shop_cost(base_cost: u32, edition: Edition, rental: bool, discount_percent: f64) -> u32 {
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
    let cost = ((base_cost as f64 + extra + 0.5) * (100.0 - discount_percent) / 100.0).floor();
    cost.max(1.0) as u32
}

/// Hearts and Diamonds are the red pair, Spades and Clubs the black one. Smeared Joker treats
/// the two members of a pair as the same suit (card.lua:4084).
pub fn is_red(suit: Suit) -> bool {
    matches!(suit, Suit::Hearts | Suit::Diamonds)
}

impl CardInstance {
    pub fn new(id: u64, rank: Rank, suit: Suit) -> Self {
        Self {
            id,
            rank,
            suit,
            enhancement: Enhancement::None,
            edition: Edition::None,
            seal: Seal::None,
            debuffed: false,
            face_down: false,
            extra_chips: 0,
            extra_mult: 0,
            extra_x_mult: 1.0,
        }
    }

    /// Base chip value of this card (rank chips + enhancement bonus)
    pub fn base_chip_value(&self) -> i64 {
        if self.debuffed {
            return 0;
        }
        match self.enhancement {
            Enhancement::Stone => 50 + self.extra_chips,
            _ => self.rank.base_chips() + self.extra_chips,
        }
    }

    /// Additional flat mult from enhancement (when scoring).
    /// Also includes `extra_mult`, which is pre-rolled by the game loop for probabilistic
    /// effects (e.g. Lucky card +20 Mult on a 1/5 trigger).
    pub fn flat_mult_bonus(&self) -> i64 {
        if self.debuffed {
            return 0;
        }
        let base = match self.enhancement {
            Enhancement::Mult => 4,
            _ => 0,
        };
        base + self.extra_mult
    }

    /// Extra chips from enhancement (Bonus card)
    pub fn chip_bonus(&self) -> i64 {
        if self.debuffed {
            return 0;
        }
        match self.enhancement {
            Enhancement::Bonus => 30,
            _ => 0,
        }
    }

    /// X-mult multiplier from enhancement (Glass card: x2)
    pub fn x_mult_factor(&self) -> f64 {
        if self.debuffed {
            return 1.0;
        }
        match self.enhancement {
            Enhancement::Glass => 2.0,
            _ => 1.0,
        }
    }

    /// X-mult from Steel when held in hand
    pub fn steel_x_mult(&self) -> f64 {
        if self.debuffed {
            return 1.0;
        }
        match self.enhancement {
            Enhancement::Steel => 1.5,
            _ => 1.0,
        }
    }

    /// Whether this card counts as `suit`, mirroring `Card:is_suit` (card.lua:4064).
    ///
    /// Three rules ride on this, and they apply to *every* suit check in the game, not just
    /// flush detection: a Stone card has no suit at all, a Wild Card counts as all four, and
    /// Smeared Joker merges Hearts with Diamonds and Spades with Clubs.
    ///
    /// Debuff is deliberately not considered here — Balatro's `bypass_debuff` argument varies
    /// per caller, so each call site decides for itself.
    pub fn is_suit(&self, suit: Suit, smeared: bool) -> bool {
        if self.is_stone() {
            return false;
        }
        if self.enhancement == Enhancement::Wild {
            return true;
        }
        if smeared {
            return is_red(self.suit) == is_red(suit);
        }
        self.suit == suit
    }

    /// This card's rank as the jokers see it — `None` for a Stone card.
    ///
    /// `Card:get_id` hands a Stone card a fresh random negative on every call (card.lua:957), so
    /// it answers no to every rank question there is: not Scholar's Ace, not Wee Joker's 2, not
    /// the King Baron wants held in hand, not the 9 Cloud 9 counts. A Stone card made from a King
    /// with The Tower keeps its King printed on it and none of the King's behaviour.
    pub fn scoring_rank(&self) -> Option<Rank> {
        (!self.is_stone()).then_some(self.rank)
    }

    /// Whether this card counts as `rank`. See [`Self::scoring_rank`].
    pub fn has_rank(&self, rank: Rank) -> bool {
        self.scoring_rank() == Some(rank)
    }

    /// Whether this card's rank satisfies `pred` — the Fibonacci / even / odd questions.
    /// A Stone card satisfies none of them.
    pub fn rank_is(&self, pred: impl Fn(&Rank) -> bool) -> bool {
        self.scoring_rank().is_some_and(|r| pred(&r))
    }

    /// Mirrors `Card:is_face` (card.lua:964), which reads the card's id — so a Stone card is not
    /// a face card, but Pareidolia still overrides everything and makes it one.
    pub fn is_face(&self, pareidolia: bool) -> bool {
        if pareidolia {
            return true;
        }
        self.rank_is(Rank::is_face)
    }

    /// Is this card a stone card?
    pub fn is_stone(&self) -> bool {
        self.enhancement == Enhancement::Stone
    }

    /// Chip bonus from edition (foil card)
    pub fn edition_chip_bonus(&self) -> i64 { self.edition.chip_bonus() }

    /// Mult bonus from edition (holographic card)
    pub fn edition_mult_bonus(&self) -> i64 { self.edition.mult_bonus() }

    /// X-mult from edition (polychrome card)
    pub fn edition_x_mult(&self) -> f64 { self.edition.x_mult() }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JokerInstance {
    pub id: u64,
    pub kind: JokerKind,
    pub edition: Edition,
    pub eternal: bool,
    pub perishable: bool,
    pub perishable_rounds_left: u32,
    pub rental: bool,
    /// Generic counter for scaling/tracking jokers (mult for CeremonialDagger, chips for Runner, etc.)
    pub counters: std::collections::HashMap<String, serde_json::Value>,
    /// Is this joker currently active/enabled? (perishable can disable)
    pub active: bool,
    /// Turned face down by Amber Acorn (blind.lua:189-203) and turned back over when that blind
    /// is beaten or disabled (blind.lua:338, :359).
    ///
    /// Purely a matter of what the *player* can see — a face-down joker scores exactly as it
    /// would face up. That is the whole point: Amber Acorn shuffles the row as well, and the
    /// shuffle only bites because you have lost track of which joker is which.
    pub face_down: bool,
}

/// Jokers whose Mult counter starts at zero and climbs.
const MULT_FROM_ZERO: [JokerKind; 8] = [
    JokerKind::CeremonialDagger,
    JokerKind::FlashCard,
    JokerKind::GreenJoker,
    JokerKind::Misprint,
    JokerKind::RedCard,
    JokerKind::RideTheBus,
    JokerKind::SpareTrousers,
    JokerKind::Swashbuckler,
];

/// Jokers whose Chips counter starts at zero and climbs.
const CHIPS_FROM_ZERO: [JokerKind; 4] = [
    JokerKind::Castle,
    JokerKind::Runner,
    JokerKind::SquareJoker,
    JokerKind::WeeJoker,
];

/// Jokers that start at X1 Mult and scale up from there.
const X_MULT_FROM_ONE: [JokerKind; 11] = [
    JokerKind::Campfire,
    JokerKind::Canio,
    JokerKind::Constellation,
    JokerKind::GlassJoker,
    JokerKind::HitTheRoad,
    JokerKind::Hologram,
    JokerKind::LuckyCat,
    JokerKind::Madness,
    JokerKind::Obelisk,
    JokerKind::Vampire,
    JokerKind::Yorick,
];

impl JokerInstance {
    pub fn new(id: u64, kind: JokerKind, edition: Edition) -> Self {
        let mut joker = Self {
            id,
            kind,
            edition,
            eternal: false,
            perishable: false,
            perishable_rounds_left: 5,
            rental: false,
            counters: std::collections::HashMap::new(),
            active: true,
            face_down: false,
        };
        joker.init_counters();
        joker
    }

    /// Seed the counters this joker starts life with.
    ///
    /// A missing counter reads as zero, but every scaling effect reads-then-writes, so anything
    /// that starts somewhere other than zero — an X-Mult joker starts at X1, not X0 — has to be
    /// seeded here or its very first upgrade would land on the wrong base.
    fn init_counters(&mut self) {
        let kind = self.kind;
        if MULT_FROM_ZERO.contains(&kind) {
            self.set_counter_i64("mult", 0);
        }
        if CHIPS_FROM_ZERO.contains(&kind) {
            self.set_counter_i64("chips", 0);
        }
        if X_MULT_FROM_ONE.contains(&kind) {
            self.set_counter_f64("x_mult", 1.0);
        }

        match kind {
            // Melts by 5 Chips a hand until there is nothing left.
            JokerKind::IceCream => self.set_counter_i64("chips", 100),
            // Hands played since acquisition; X4 Mult on every 6th.
            JokerKind::LoyaltyCard => self.set_counter_i64("hands", 0),
            // Loses 4 Mult a round; eaten at 0.
            JokerKind::Popcorn => self.set_counter_i64("mult", 20),
            // The one X-Mult joker that starts above X1 and shrinks.
            JokerKind::Ramen => self.set_counter_f64("x_mult", 2.0),
            // Hand size granted, shrinking by 1 a round.
            JokerKind::TurtleBean => self.set_counter_i64("h_size", 5),
            // Pays out its counter each round, +$2 per Boss beaten.
            JokerKind::Rocket => self.set_counter_i64("dollars", 1),
            // Retriggers every card for 10 hands, then destroys itself.
            JokerKind::Seltzer => self.set_counter_i64("hands", 10),
            JokerKind::Satellite => self.set_counter_i64("planets_used", 0),
            JokerKind::ToDoList => self.set_counter_str("hand_type", "HighCard"),
            JokerKind::InvisibleJoker => self.set_counter_i64("rounds", 0),
            // Gains $3 of sell value each round.
            JokerKind::Egg => self.set_counter_i64("sell_bonus", 0),
            // Every 23rd card discarded is worth +1 X-Mult.
            JokerKind::Yorick => self.set_counter_i64("discards", 0),
            JokerKind::Obelisk => self.set_counter_json("hand_count", serde_json::json!({})),
            _ => {}
        }
    }

    /// Whether this instance's Eternal sticker is legal for its kind. Only used by tests and
    /// tooling — the shop already refuses to apply an incompatible sticker.
    pub fn eternal_compat_ok(&self) -> bool {
        !self.eternal || self.kind.eternal_compat()
    }

    /// As above, for Perishable.
    pub fn perishable_compat_ok(&self) -> bool {
        !self.perishable || self.kind.perishable_compat()
    }

    /// `sell_cost = max(1, floor(cost/2)) + extra_value` (card.lua:382), where `cost` is the
    /// joker's *shop* price rather than its bare base cost.
    ///
    /// Two consequences that are easy to miss: an edition raises what a joker sells for, because
    /// its surcharge is part of that price, and a shop discount lowers it — under Liquidation
    /// your board is worth half as much. A rental sells for a dollar.
    pub fn sell_value(&self, discount_percent: f64) -> u32 {
        let cost = card_shop_cost(
            self.kind.base_cost(),
            self.edition,
            self.rental,
            discount_percent,
        );
        (cost / 2).max(1) + self.get_counter_i64("sell_bonus") as u32
    }

    /// Edition chip bonus (foil joker: +50 chips)
    pub fn edition_chip_bonus(&self) -> i64 { self.edition.chip_bonus() }

    /// Edition mult bonus (holographic joker: +10 mult)
    pub fn edition_mult_bonus(&self) -> i64 { self.edition.mult_bonus() }

    /// Edition x-mult (polychrome joker: x1.5)
    pub fn edition_x_mult(&self) -> f64 { self.edition.x_mult() }

    pub fn get_counter_f64(&self, key: &str) -> f64 {
        self.counters
            .get(key)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    }

    pub fn get_counter_i64(&self, key: &str) -> i64 {
        self.counters
            .get(key)
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    }

    pub fn get_counter_str(&self, key: &str) -> Option<&str> {
        self.counters.get(key).and_then(|v| v.as_str())
    }

    pub fn set_counter_f64(&mut self, key: &str, val: f64) {
        self.set_counter_json(key, serde_json::json!(val));
    }

    pub fn set_counter_i64(&mut self, key: &str, val: i64) {
        self.set_counter_json(key, serde_json::json!(val));
    }

    pub fn set_counter_str(&mut self, key: &str, val: &str) {
        self.set_counter_json(key, serde_json::json!(val));
    }

    pub fn set_counter_json(&mut self, key: &str, val: serde_json::Value) {
        self.counters.insert(key.to_string(), val);
    }

    /// Add to a counter, reading a missing one as zero. This is how nearly every scaling joker
    /// grows, so it saves the read-modify-write dance at each call site.
    pub fn add_counter_f64(&mut self, key: &str, delta: f64) {
        let new = self.get_counter_f64(key) + delta;
        self.set_counter_f64(key, new);
    }

    /// As [`Self::add_counter_f64`], for the integer counters.
    pub fn add_counter_i64(&mut self, key: &str, delta: i64) {
        let new = self.get_counter_i64(key) + delta;
        self.set_counter_i64(key, new);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConsumableCard {
    Tarot(TarotCard),
    Planet(PlanetCard),
    Spectral(SpectralCard),
}

impl ConsumableCard {
    pub fn display_name(&self) -> String {
        match self {
            ConsumableCard::Tarot(t) => format!("{:?}", t),
            ConsumableCard::Planet(p) => format!("{:?}", p),
            ConsumableCard::Spectral(s) => format!("{:?}", s),
        }
    }

    pub fn base_cost(&self) -> u32 {
        match self {
            ConsumableCard::Tarot(_) => 3,
            ConsumableCard::Planet(_) => 3,
            ConsumableCard::Spectral(_) => 4,
        }
    }

    pub fn card_type(&self) -> &'static str {
        match self {
            ConsumableCard::Tarot(_) => "Tarot",
            ConsumableCard::Planet(_) => "Planet",
            ConsumableCard::Spectral(_) => "Spectral",
        }
    }
}

/// A consumable sitting in your consumable slots, with the edition it carries.
///
/// Kept apart from [`ConsumableCard`] because only a *held* card can have one: the shop and the
/// booster packs never roll editions onto consumables (`create_card` only polls them for jokers,
/// common_events.lua:2149). In a vanilla run Perkeo is the sole source of a Negative one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldConsumable {
    pub card: ConsumableCard,
    /// A Negative consumable carries its own consumable slot, which it takes with it when used
    /// or sold (`card.lua:687`, the same rule that gives Negative jokers a joker slot).
    pub negative: bool,
}

impl HeldConsumable {
    pub fn new(card: ConsumableCard) -> Self {
        Self { card, negative: false }
    }

    pub fn negative(card: ConsumableCard) -> Self {
        Self { card, negative: true }
    }
}

impl From<ConsumableCard> for HeldConsumable {
    fn from(card: ConsumableCard) -> Self {
        Self::new(card)
    }
}

/// Reading a held consumable as the card it is, so `display_name()` / `base_cost()` and friends
/// stay reachable without spelling out `.card` every time.
impl std::ops::Deref for HeldConsumable {
    type Target = ConsumableCard;
    fn deref(&self) -> &ConsumableCard {
        &self.card
    }
}

/// Comparing a held consumable directly against a bare card, ignoring the edition.
impl PartialEq<ConsumableCard> for HeldConsumable {
    fn eq(&self, other: &ConsumableCard) -> bool {
        self.card == *other
    }
}

/// Hand level data - tracks levels and play counts for each hand type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandLevelData {
    pub level: u32,
    pub played: u32,
    pub played_this_round: u32,
    pub visible: bool,
}

impl HandLevelData {
    pub fn new(visible: bool) -> Self {
        Self {
            level: 1,
            played: 0,
            played_this_round: 0,
            visible,
        }
    }

    pub fn chips(&self, hand_type: HandType) -> i64 {
        hand_type.base_chips() + hand_type.level_chip_bonus() * (self.level as i64 - 1)
    }

    pub fn mult(&self, hand_type: HandType) -> i64 {
        hand_type.base_mult() + hand_type.level_mult_bonus() * (self.level as i64 - 1)
    }
}

/// A shop offer: what's being sold, its price, and whether it's been bought
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopOffer {
    pub kind: ShopItem,
    /// The item's **base** cost, before editions, discounts and the free-item overrides.
    /// What the player is actually charged is `GameState::offer_price`, which applies those
    /// once — Balatro recomputes from the base every time (`Card:set_cost`, card.lua:369).
    pub price: u32,
    pub sold: bool,
    /// Coupon Tag: this one is free. Held as a flag rather than a price of 0 because the
    /// override lands *after* the `max(1)` floor, so it really is free and not a dollar.
    pub free: bool,
}

impl ShopOffer {
    pub fn new(kind: ShopItem, price: u32) -> Self {
        Self { kind, price, sold: false, free: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShopItem {
    Joker(JokerInstance),
    Consumable(ConsumableCard),
    /// A loose playing card. Only stocked once Magic Trick has been redeemed.
    PlayingCard(CardInstance),
    Pack(PackKind),
    Voucher(VoucherKind),
}

/// Contents of a booster pack being opened
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackContents {
    pub kind: PackKind,
    pub cards: Vec<PackCard>,
    pub picks_remaining: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackCard {
    PlayingCard(CardInstance),
    Joker(JokerInstance),
    Consumable(ConsumableCard),
}
