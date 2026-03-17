use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub fn base_chips(&self) -> i64 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 10,
            Rank::Queen => 10,
            Rank::King => 10,
            Rank::Ace => 11,
        }
    }

    pub fn is_face(&self) -> bool {
        matches!(self, Rank::Jack | Rank::Queen | Rank::King)
    }

    pub fn numeric_value(&self) -> u8 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            Rank::Queen => 12,
            Rank::King => 13,
            Rank::Ace => 14,
        }
    }

    pub fn is_even(&self) -> bool {
        matches!(
            self,
            Rank::Two | Rank::Four | Rank::Six | Rank::Eight | Rank::Ten
        )
    }

    pub fn is_odd(&self) -> bool {
        matches!(
            self,
            Rank::Three | Rank::Five | Rank::Seven | Rank::Nine | Rank::Ace
        )
    }

    pub fn is_fibonacci(&self) -> bool {
        matches!(
            self,
            Rank::Ace | Rank::Two | Rank::Three | Rank::Five | Rank::Eight
        )
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Rank::Two),
            1 => Some(Rank::Three),
            2 => Some(Rank::Four),
            3 => Some(Rank::Five),
            4 => Some(Rank::Six),
            5 => Some(Rank::Seven),
            6 => Some(Rank::Eight),
            7 => Some(Rank::Nine),
            8 => Some(Rank::Ten),
            9 => Some(Rank::Jack),
            10 => Some(Rank::Queen),
            11 => Some(Rank::King),
            12 => Some(Rank::Ace),
            _ => None,
        }
    }
}

#[pymethods]
impl Rank {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds,
}

#[pymethods]
impl Suit {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Enhancement {
    None,
    Bonus,   // +30 chips
    Mult,    // +4 mult
    Wild,    // counts as all suits
    Glass,   // x2 mult, 1/4 chance to break
    Steel,   // x1.5 mult while in hand
    Stone,   // +50 chips, no rank/suit
    Gold,    // $3 at end of round
    Lucky,   // 1/5 chance +20 mult, 1/15 chance $20
}

#[pymethods]
impl Enhancement {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Edition {
    None,
    Foil,        // +50 chips
    Holographic, // +10 mult
    Polychrome,  // x1.5 mult
    Negative,    // +1 joker slot (jokers only)
}

#[pymethods]
impl Edition {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Seal {
    None,
    Gold,   // $3 when played
    Red,    // retrigger once
    Blue,   // create planet card when held in hand at end of round
    Purple, // create tarot card when discarded
}

#[pymethods]
impl Seal {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeckType {
    Red,
    Blue,
    Yellow,
    Green,
    Black,
    Magic,
    Nebula,
    Ghost,
    Abandoned,
    Checkered,
    Zodiac,
    Painted,
    Anaglyph,
    Plasma,
    Erratic,
}

impl DeckType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(DeckType::Red),
            1 => Some(DeckType::Blue),
            2 => Some(DeckType::Yellow),
            3 => Some(DeckType::Green),
            4 => Some(DeckType::Black),
            5 => Some(DeckType::Magic),
            6 => Some(DeckType::Nebula),
            7 => Some(DeckType::Ghost),
            8 => Some(DeckType::Abandoned),
            9 => Some(DeckType::Checkered),
            10 => Some(DeckType::Zodiac),
            11 => Some(DeckType::Painted),
            12 => Some(DeckType::Anaglyph),
            13 => Some(DeckType::Plasma),
            14 => Some(DeckType::Erratic),
            _ => None,
        }
    }
}

#[pymethods]
impl DeckType {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stake {
    White,
    Red,
    Green,
    Black,
    Blue,
    Purple,
    Orange,
    Gold,
}

impl Stake {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Stake::White),
            1 => Some(Stake::Red),
            2 => Some(Stake::Green),
            3 => Some(Stake::Black),
            4 => Some(Stake::Blue),
            5 => Some(Stake::Purple),
            6 => Some(Stake::Orange),
            7 => Some(Stake::Gold),
            _ => None,
        }
    }
}

#[pymethods]
impl Stake {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandType {
    FlushFive,
    FlushHouse,
    FiveOfAKind,
    StraightFlush,
    FourOfAKind,
    FullHouse,
    Flush,
    Straight,
    ThreeOfAKind,
    TwoPair,
    Pair,
    HighCard,
}

#[pymethods]
impl HandType {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl HandType {
    /// True if this hand type contains at least one pair (same rank × 2+).
    pub fn contains_pair(&self) -> bool {
        matches!(
            self,
            HandType::Pair
                | HandType::TwoPair
                | HandType::ThreeOfAKind
                | HandType::FullHouse
                | HandType::FourOfAKind
                | HandType::FiveOfAKind
                | HandType::FlushHouse
                | HandType::FlushFive
        )
    }

    /// True if this hand type contains two distinct same-rank pairs.
    pub fn contains_two_pair(&self) -> bool {
        matches!(
            self,
            HandType::TwoPair | HandType::FullHouse | HandType::FlushHouse
        )
    }

    /// True if this hand type contains a three-of-a-kind (same rank × 3+).
    pub fn contains_three_of_a_kind(&self) -> bool {
        matches!(
            self,
            HandType::ThreeOfAKind
                | HandType::FullHouse
                | HandType::FourOfAKind
                | HandType::FiveOfAKind
                | HandType::FlushHouse
                | HandType::FlushFive
        )
    }

    /// True if this hand type contains a four-of-a-kind (same rank × 4+).
    pub fn contains_four_of_a_kind(&self) -> bool {
        matches!(
            self,
            HandType::FourOfAKind | HandType::FiveOfAKind | HandType::FlushFive
        )
    }

    /// True if this hand type contains a straight (consecutive ranks).
    pub fn contains_straight(&self) -> bool {
        matches!(self, HandType::Straight | HandType::StraightFlush)
    }

    /// True if this hand type contains a flush (all same suit).
    pub fn contains_flush(&self) -> bool {
        matches!(
            self,
            HandType::Flush | HandType::StraightFlush | HandType::FlushHouse | HandType::FlushFive
        )
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            HandType::FlushFive => "Flush Five",
            HandType::FlushHouse => "Flush House",
            HandType::FiveOfAKind => "Five of a Kind",
            HandType::StraightFlush => "Straight Flush",
            HandType::FourOfAKind => "Four of a Kind",
            HandType::FullHouse => "Full House",
            HandType::Flush => "Flush",
            HandType::Straight => "Straight",
            HandType::ThreeOfAKind => "Three of a Kind",
            HandType::TwoPair => "Two Pair",
            HandType::Pair => "Pair",
            HandType::HighCard => "High Card",
        }
    }

    pub fn base_chips(&self) -> i64 {
        match self {
            HandType::FlushFive => 160,
            HandType::FlushHouse => 140,
            HandType::FiveOfAKind => 120,
            HandType::StraightFlush => 100,
            HandType::FourOfAKind => 60,
            HandType::FullHouse => 40,
            HandType::Flush => 35,
            HandType::Straight => 30,
            HandType::ThreeOfAKind => 30,
            HandType::TwoPair => 20,
            HandType::Pair => 10,
            HandType::HighCard => 5,
        }
    }

    pub fn base_mult(&self) -> i64 {
        match self {
            HandType::FlushFive => 16,
            HandType::FlushHouse => 14,
            HandType::FiveOfAKind => 12,
            HandType::StraightFlush => 8,
            HandType::FourOfAKind => 7,
            HandType::FullHouse => 4,
            HandType::Flush => 4,
            HandType::Straight => 4,
            HandType::ThreeOfAKind => 3,
            HandType::TwoPair => 2,
            HandType::Pair => 2,
            HandType::HighCard => 1,
        }
    }

    pub fn level_chip_bonus(&self) -> i64 {
        match self {
            HandType::FlushFive => 50,
            HandType::FlushHouse => 40,
            HandType::FiveOfAKind => 35,
            HandType::StraightFlush => 40,
            HandType::FourOfAKind => 30,
            HandType::FullHouse => 25,
            HandType::Flush => 15,
            HandType::Straight => 30,
            HandType::ThreeOfAKind => 20,
            HandType::TwoPair => 20,
            HandType::Pair => 15,
            HandType::HighCard => 10,
        }
    }

    pub fn level_mult_bonus(&self) -> i64 {
        match self {
            HandType::FlushFive => 3,
            HandType::FlushHouse => 4,
            HandType::FiveOfAKind => 3,
            HandType::StraightFlush => 4,
            HandType::FourOfAKind => 3,
            HandType::FullHouse => 2,
            HandType::Flush => 2,
            HandType::Straight => 3,
            HandType::ThreeOfAKind => 2,
            HandType::TwoPair => 1,
            HandType::Pair => 1,
            HandType::HighCard => 1,
        }
    }

    /// Planet card index for level-up
    pub fn planet_key(&self) -> &'static str {
        match self {
            HandType::FlushFive => "c_eris",
            HandType::FlushHouse => "c_ceres",
            HandType::FiveOfAKind => "c_planet_x",
            HandType::StraightFlush => "c_neptune",
            HandType::FourOfAKind => "c_mars",
            HandType::FullHouse => "c_jupiter",
            HandType::Flush => "c_venus",
            HandType::Straight => "c_saturn",
            HandType::ThreeOfAKind => "c_earth",
            HandType::TwoPair => "c_uranus",
            HandType::Pair => "c_mercury",
            HandType::HighCard => "c_pluto",
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BossBlind {
    TheOx,
    TheHook,
    TheMouth,
    TheFish,
    TheClub,
    TheManacle,
    TheTooth,
    TheWall,
    TheHouse,
    TheMark,
    CeruleanBell,
    TheWheel,
    TheArm,
    ThePsychic,
    TheGoad,
    TheWater,
    TheEye,
    ThePlant,
    TheNeedle,
    TheHead,
    VerdantLeaf,
    VioletVessel,
    TheWindow,
    TheSerpent,
    ThePillar,
    TheFlint,
    AmberAcorn,
    CrimsonHeart,
}

#[pymethods]
impl BossBlind {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            BossBlind::TheOx => "The Ox",
            BossBlind::TheHook => "The Hook",
            BossBlind::TheMouth => "The Mouth",
            BossBlind::TheFish => "The Fish",
            BossBlind::TheClub => "The Club",
            BossBlind::TheManacle => "The Manacle",
            BossBlind::TheTooth => "The Tooth",
            BossBlind::TheWall => "The Wall",
            BossBlind::TheHouse => "The House",
            BossBlind::TheMark => "The Mark",
            BossBlind::CeruleanBell => "Cerulean Bell",
            BossBlind::TheWheel => "The Wheel",
            BossBlind::TheArm => "The Arm",
            BossBlind::ThePsychic => "The Psychic",
            BossBlind::TheGoad => "The Goad",
            BossBlind::TheWater => "The Water",
            BossBlind::TheEye => "The Eye",
            BossBlind::ThePlant => "The Plant",
            BossBlind::TheNeedle => "The Needle",
            BossBlind::TheHead => "The Head",
            BossBlind::VerdantLeaf => "Verdant Leaf",
            BossBlind::VioletVessel => "Violet Vessel",
            BossBlind::TheWindow => "The Window",
            BossBlind::TheSerpent => "The Serpent",
            BossBlind::ThePillar => "The Pillar",
            BossBlind::TheFlint => "The Flint",
            BossBlind::AmberAcorn => "Amber Acorn",
            BossBlind::CrimsonHeart => "Crimson Heart",
        }
    }

    pub fn chip_multiplier(&self) -> f64 {
        match self {
            BossBlind::TheWall => 4.0,
            BossBlind::TheNeedle => 1.0,
            BossBlind::VioletVessel => 6.0,
            _ => 2.0,
        }
    }
}

impl Edition {
    pub fn chip_bonus(&self) -> i64 {
        match self {
            Edition::Foil => 50,
            _ => 0,
        }
    }

    pub fn mult_bonus(&self) -> i64 {
        match self {
            Edition::Holographic => 10,
            _ => 0,
        }
    }

    pub fn x_mult(&self) -> f64 {
        match self {
            Edition::Polychrome => 1.5,
            _ => 1.0,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameState {
    BlindSelect,
    Round,
    Shop,
    BoosterPack,
    GameOver,
}

#[pymethods]
impl GameState {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

pub mod joker_kind;
pub use joker_kind::*;

pub mod consumable_types;
pub use consumable_types::*;
