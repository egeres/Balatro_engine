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

    pub fn is_face(&self, pareidolia: bool) -> bool {
        if pareidolia {
            return true;
        }
        self.rank.is_face()
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
}

impl JokerInstance {
    pub fn new(id: u64, kind: JokerKind, edition: Edition) -> Self {
        let mut counters = std::collections::HashMap::new();
        // Initialize joker-specific counters
        match kind {
            JokerKind::CeremonialDagger => {
                counters.insert("mult".to_string(), serde_json::json!(0));
            }
            JokerKind::Runner => {
                counters.insert("chips".to_string(), serde_json::json!(0));
            }
            JokerKind::SquareJoker => {
                counters.insert("chips".to_string(), serde_json::json!(0));
            }
            JokerKind::WeeJoker => {
                counters.insert("chips".to_string(), serde_json::json!(0));
            }
            JokerKind::IceCream => {
                counters.insert("chips".to_string(), serde_json::json!(100));
            }
            JokerKind::LoyaltyCard => {
                // Hands played since acquisition; X4 Mult on every 6th.
                counters.insert("hands".to_string(), serde_json::json!(0_i64));
            }
            JokerKind::Misprint => {
                // Re-rolled to 0..=23 before each hand scores.
                counters.insert("mult".to_string(), serde_json::json!(0_i64));
            }
            JokerKind::Popcorn => {
                counters.insert("mult".to_string(), serde_json::json!(20));
            }
            JokerKind::SpareTrousers => {
                counters.insert("mult".to_string(), serde_json::json!(0));
            }
            JokerKind::Castle => {
                // Target suit lives on GameState.round_targets: it is round-wide, not per-joker.
                counters.insert("chips".to_string(), serde_json::json!(0));
            }
            JokerKind::Hologram => {
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
            }
            JokerKind::Vampire => {
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
            }
            JokerKind::Obelisk => {
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
                counters.insert("hand_count".to_string(), serde_json::json!({}));
            }
            JokerKind::LuckyCat => {
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
            }
            JokerKind::Constellation => {
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
            }
            JokerKind::GlassJoker => {
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
            }
            JokerKind::Ramen => {
                counters.insert("x_mult".to_string(), serde_json::json!(2.0_f64));
            }
            JokerKind::HitTheRoad => {
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
            }
            JokerKind::FlashCard => {
                counters.insert("mult".to_string(), serde_json::json!(0));
            }
            JokerKind::Madness => {
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
            }
            JokerKind::GreenJoker => {
                counters.insert("mult".to_string(), serde_json::json!(0));
            }
            JokerKind::RideTheBus => {
                counters.insert("mult".to_string(), serde_json::json!(0));
            }
            JokerKind::Swashbuckler => {
                counters.insert("mult".to_string(), serde_json::json!(0));
            }
            JokerKind::TurtleBean => {
                counters.insert("h_size".to_string(), serde_json::json!(5));
            }
            JokerKind::Yorick => {
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
                counters.insert("discards".to_string(), serde_json::json!(0));
            }
            JokerKind::Campfire => {
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
            }
            JokerKind::Rocket => {
                counters.insert("dollars".to_string(), serde_json::json!(1));
            }
            JokerKind::Seltzer => {
                counters.insert("hands".to_string(), serde_json::json!(10_i64));
            }
            JokerKind::Satellite => {
                // tracks unique planet types used (as a set stored as count)
                counters.insert("planets_used".to_string(), serde_json::json!(0_i64));
            }
            JokerKind::ToDoList => {
                counters.insert("hand_type".to_string(), serde_json::json!("HighCard"));
            }
            JokerKind::RedCard => {
                counters.insert("mult".to_string(), serde_json::json!(0_i64));
            }
            JokerKind::Burglar => {
                // Gains +3 hands, discards = 0 for the round (tracked via effective_max_*)
            }
            JokerKind::Canio => {
                // Starts at X1 Mult, gains +1 Xmult per face card destroyed
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
            }
            JokerKind::InvisibleJoker => {
                counters.insert("rounds".to_string(), serde_json::json!(0));
            }
            JokerKind::Egg => {
                counters.insert("sell_bonus".to_string(), serde_json::json!(0));
            }
            _ => {}
        }
        Self {
            id,
            kind,
            edition,
            eternal: false,
            perishable: false,
            perishable_rounds_left: 5,
            rental: false,
            counters,
            active: true,
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

    pub fn sell_value(&self) -> u32 {
        // Balatro uses floor(buy_cost / 2), minimum $1
        let base = (self.kind.base_cost() / 2).max(1);
        let bonus = self.get_counter_i64("sell_bonus") as u32;
        base + bonus
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

    pub fn set_counter_f64(&mut self, key: &str, val: f64) {
        self.counters.insert(key.to_string(), serde_json::json!(val));
    }

    pub fn set_counter_i64(&mut self, key: &str, val: i64) {
        self.counters.insert(key.to_string(), serde_json::json!(val));
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
    /// Cumulative X1.5 multiplier stacked by Observatory voucher each time a Planet card is used
    pub observatory_x_mult: f64,
}

impl HandLevelData {
    pub fn new(visible: bool) -> Self {
        Self {
            level: 1,
            played: 0,
            played_this_round: 0,
            visible,
            observatory_x_mult: 1.0,
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
