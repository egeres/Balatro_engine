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
    /// For chip_modifier from Hiker/Wee Joker/etc
    pub extra_chips: i64,
    /// For flip cards (Certificate joker)
    pub extra_mult: i64,
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
            extra_chips: 0,
            extra_mult: 0,
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

    /// Additional flat mult from enhancement (when scoring)
    pub fn flat_mult_bonus(&self) -> i64 {
        if self.debuffed {
            return 0;
        }
        match self.enhancement {
            Enhancement::Mult => 4,
            _ => 0,
        }
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

    /// Effective suit (Wild counts as all suits)
    pub fn effective_suits(&self) -> Vec<Suit> {
        match self.enhancement {
            Enhancement::Wild => vec![Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds],
            _ => vec![self.suit],
        }
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
            JokerKind::Popcorn => {
                counters.insert("mult".to_string(), serde_json::json!(20));
            }
            JokerKind::SpareTrousers => {
                counters.insert("mult".to_string(), serde_json::json!(0));
            }
            JokerKind::Castle => {
                counters.insert("chips".to_string(), serde_json::json!(0));
                counters.insert("suit".to_string(), serde_json::json!("Spades"));
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
            JokerKind::AncientJoker => {
                counters.insert(
                    "suit".to_string(),
                    serde_json::json!("Hearts"),
                );
            }
            JokerKind::Rocket => {
                counters.insert("dollars".to_string(), serde_json::json!(1));
            }
            JokerKind::Canio => {
                // Starts at X1 Mult, gains +1 Xmult per face card destroyed
                counters.insert("x_mult".to_string(), serde_json::json!(1.0_f64));
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

    pub fn sell_value(&self) -> u32 {
        // Base sell value is roughly half of cost
        (self.kind.base_cost() + 1) / 2
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub price: u32,
    pub sold: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShopItem {
    Joker(JokerInstance),
    Consumable(ConsumableCard),
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
