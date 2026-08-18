use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// Give each of these enums a Python `__repr__` that prints its Rust name.
///
/// One `#[pymethods]` block per class is all pyo3 allows without the `multiple-pymethods`
/// feature, so this only covers the enums whose Python surface is *just* `__repr__`; the ones
/// that also expose real methods spell it out in their own block.
macro_rules! debug_repr {
    ($($ty:ty),* $(,)?) => {
        $(
            #[pymethods]
            impl $ty {
                fn __repr__(&self) -> String {
                    format!("{:?}", self)
                }
            }
        )*
    };
}
pub(crate) use debug_repr;

debug_repr!(Rank, Suit, Enhancement, Edition, Seal, DeckType, Stake, HandType, GameState);

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
    /// Every rank, low to high. The index into this table *is* the wire value used by
    /// [`Rank::from_u8`], and card generators draw from it, so the order is load-bearing.
    pub const ALL: [Rank; 13] = [
        Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven, Rank::Eight,
        Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
    ];

    /// The nine numbered ranks — what Incantation creates.
    pub const NUMBERS: [Rank; 9] = [
        Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven, Rank::Eight,
        Rank::Nine, Rank::Ten,
    ];

    /// The three face ranks — what Familiar creates.
    pub const FACES: [Rank; 3] = [Rank::Jack, Rank::Queen, Rank::King];

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
        Self::ALL.get(v as usize).copied()
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

impl Suit {
    /// The four suits, in the order every card generator and suit scan walks them.
    pub const ALL: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds];
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

impl Enhancement {
    /// The eight real enhancements, in the order the shop and Standard packs roll them.
    /// `None` is excluded: nothing ever *rolls* a plain card.
    pub const ALL: [Enhancement; 8] = [
        Enhancement::Bonus, Enhancement::Mult, Enhancement::Wild, Enhancement::Glass,
        Enhancement::Steel, Enhancement::Stone, Enhancement::Gold, Enhancement::Lucky,
    ];
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

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Seal {
    None,
    Gold,   // $3 when played
    Red,    // retrigger once
    Blue,   // create planet card when held in hand at end of round
    Purple, // create tarot card when discarded
}

impl Seal {
    /// The four real seals, in roll order. Certificate draws from these.
    pub const REAL: [Seal; 4] = [Seal::Gold, Seal::Red, Seal::Blue, Seal::Purple];

    /// As [`Seal::REAL`], but with "no seal" as an equally likely outcome — what the Illusion
    /// voucher rolls for a shop playing card.
    pub const ALL: [Seal; 5] = [Seal::None, Seal::Gold, Seal::Red, Seal::Blue, Seal::Purple];
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
    /// Every deck, in the order the `DECK_*` constants exposed to Python number them.
    pub const ALL: [DeckType; 15] = [
        DeckType::Red, DeckType::Blue, DeckType::Yellow, DeckType::Green, DeckType::Black,
        DeckType::Magic, DeckType::Nebula, DeckType::Ghost, DeckType::Abandoned,
        DeckType::Checkered, DeckType::Zodiac, DeckType::Painted, DeckType::Anaglyph,
        DeckType::Plasma, DeckType::Erratic,
    ];

    pub fn from_u8(v: u8) -> Option<Self> {
        Self::ALL.get(v as usize).copied()
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
    /// Every stake, ascending. The index is the wire value of the `STAKE_*` constants, and the
    /// ordering also drives the `stake as u8 >= Stake::X as u8` difficulty gates.
    pub const ALL: [Stake; 8] = [
        Stake::White, Stake::Red, Stake::Green, Stake::Black, Stake::Blue, Stake::Purple,
        Stake::Orange, Stake::Gold,
    ];

    pub fn from_u8(v: u8) -> Option<Self> {
        Self::ALL.get(v as usize).copied()
    }

    /// Whether this stake is at least as hard as `other`.
    pub fn at_least(self, other: Stake) -> bool {
        self as u8 >= other as u8
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

impl HandType {
    /// Every hand type, best to worst.
    pub const ALL: [HandType; 12] = [
        HandType::FlushFive, HandType::FlushHouse, HandType::FiveOfAKind,
        HandType::StraightFlush, HandType::FourOfAKind, HandType::FullHouse, HandType::Flush,
        HandType::Straight, HandType::ThreeOfAKind, HandType::TwoPair, HandType::Pair,
        HandType::HighCard,
    ];

    /// Parse the `{:?}` spelling of a hand type, which is how To Do List stores its target.
    pub fn from_debug_name(name: &str) -> Option<HandType> {
        HandType::ALL.into_iter().find(|h| format!("{h:?}") == name)
    }

    /// The three hands that stay hidden until you first play one (`visible = false`).
    pub fn is_secret(&self) -> bool {
        matches!(
            self,
            HandType::FlushFive | HandType::FlushHouse | HandType::FiveOfAKind
        )
    }

    // There used to be `contains_pair()` / `contains_flush()` / … helpers here, deriving what a
    // hand holds from its *name*. That is not how Balatro asks the question: the hand-shape
    // jokers read `context.poker_hands`, built from the played cards themselves
    // (misc_functions.lua:376), so a Flush that happens to hold a pair pays Jolly Joker. Use
    // `HandEvalResult::contained` — the name alone cannot answer it.

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

    /// Every Boss blind, so pool building cannot silently drop one.
    pub const ALL: [BossBlind; 28] = [
        BossBlind::TheOx, BossBlind::TheHook, BossBlind::TheMouth, BossBlind::TheFish,
        BossBlind::TheClub, BossBlind::TheManacle, BossBlind::TheTooth, BossBlind::TheWall,
        BossBlind::TheHouse, BossBlind::TheMark, BossBlind::CeruleanBell, BossBlind::TheWheel,
        BossBlind::TheArm, BossBlind::ThePsychic, BossBlind::TheGoad, BossBlind::TheWater,
        BossBlind::TheEye, BossBlind::ThePlant, BossBlind::TheNeedle, BossBlind::TheHead,
        BossBlind::VerdantLeaf, BossBlind::VioletVessel, BossBlind::TheWindow,
        BossBlind::TheSerpent, BossBlind::ThePillar, BossBlind::TheFlint, BossBlind::AmberAcorn,
        BossBlind::CrimsonHeart,
    ];

    /// Earliest ante this boss can appear at (`boss = {min = N}` in game.lua:266-290).
    /// Showdown bosses report 0 — they are gated on `ante % 8 == 0` instead.
    pub fn min_ante(&self) -> u32 {
        match self {
            BossBlind::TheHook
            | BossBlind::TheClub
            | BossBlind::TheManacle
            | BossBlind::ThePsychic
            | BossBlind::TheGoad
            | BossBlind::TheHead
            | BossBlind::TheWindow
            | BossBlind::ThePillar => 1,
            BossBlind::TheMouth
            | BossBlind::TheFish
            | BossBlind::TheWall
            | BossBlind::TheHouse
            | BossBlind::TheMark
            | BossBlind::TheWheel
            | BossBlind::TheArm
            | BossBlind::TheWater
            | BossBlind::TheNeedle
            | BossBlind::TheFlint => 2,
            BossBlind::TheTooth | BossBlind::TheEye => 3,
            BossBlind::ThePlant => 4,
            BossBlind::TheSerpent => 5,
            BossBlind::TheOx => 6,
            BossBlind::CeruleanBell
            | BossBlind::VerdantLeaf
            | BossBlind::VioletVessel
            | BossBlind::AmberAcorn
            | BossBlind::CrimsonHeart => 0,
        }
    }

    /// Showdown bosses appear only on the winning ante and its multiples.
    pub fn is_showdown(&self) -> bool {
        matches!(
            self,
            BossBlind::CeruleanBell
                | BossBlind::VerdantLeaf
                | BossBlind::VioletVessel
                | BossBlind::AmberAcorn
                | BossBlind::CrimsonHeart
        )
    }

    pub fn chip_multiplier(&self) -> f64 {
        match self {
            BossBlind::TheWall => 4.0,
            BossBlind::TheNeedle => 1.0,
            BossBlind::VioletVessel => 6.0,
            _ => 2.0,
        }
    }

    /// The requirement once the blind's ability has been switched off.
    ///
    /// `Blind:disable()` halves The Wall's chips and cuts Violet Vessel's to a third
    /// (blind.lua:377, :393). Nothing else is touched, so a disabled Needle keeps its easy 1x —
    /// the small requirement is the blind, not the ability.
    pub fn chip_multiplier_disabled(&self) -> f64 {
        match self {
            BossBlind::TheWall | BossBlind::VioletVessel => 2.0,
            other => other.chip_multiplier(),
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

pub mod joker_kind;
pub use joker_kind::*;

pub mod consumable_types;
pub use consumable_types::*;
