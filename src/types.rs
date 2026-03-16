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

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JokerKind {
    Joker,
    GreedyJoker,
    LustyJoker,
    WrathfulJoker,
    GluttonousJoker,
    JollyJoker,
    ZanyJoker,
    MadJoker,
    CrazyJoker,
    DrollJoker,
    SlyJoker,
    WilyJoker,
    CleverJoker,
    DeviousJoker,
    CraftyJoker,
    HalfJoker,
    JokerStencil,
    FourFingers,
    Mime,
    CreditCard,
    CeremonialDagger,
    Banner,
    MysticSummit,
    MarbleJoker,
    LoyaltyCard,
    EightBall,
    Misprint,
    Dusk,
    RaisedFist,
    ChaosTheClown,
    Fibonacci,
    SteelJoker,
    ScaryFace,
    AbstractJoker,
    DelayedGratification,
    Hack,
    Pareidolia,
    GrosMichel,
    EvenSteven,
    OddTodd,
    Scholar,
    BusinessCard,
    Supernova,
    RideTheBus,
    SpaceJoker,
    Egg,
    Burglar,
    Blackboard,
    Runner,
    IceCream,
    Dna,
    Splash,
    BlueJoker,
    SixthSense,
    Constellation,
    Hiker,
    FacelessJoker,
    GreenJoker,
    Superposition,
    ToDoList,
    Cavendish,
    CardSharp,
    RedCard,
    Madness,
    SquareJoker,
    Seance,
    RiffRaff,
    Vampire,
    Shortcut,
    Hologram,
    Vagabond,
    Baron,
    Cloud9,
    Rocket,
    Obelisk,
    MidasMask,
    Luchador,
    Photograph,
    GiftCard,
    TurtleBean,
    Erosion,
    ReservedParking,
    MailInRebate,
    ToTheMoon,
    Hallucination,
    FortuneTeller,
    Juggler,
    Drunkard,
    StoneJoker,
    GoldenJoker,
    LuckyCat,
    BaseballCard,
    Bull,
    DietCola,
    TradingCard,
    FlashCard,
    Popcorn,
    SpareTrousers,
    AncientJoker,
    Ramen,
    WalkieTalkie,
    Seltzer,
    Castle,
    SmileyFace,
    Campfire,
    GoldenTicket,
    MrBones,
    Acrobat,
    SockAndBuskin,
    Swashbuckler,
    Troubadour,
    Certificate,
    SmearedJoker,
    Throwback,
    HangingChad,
    RoughGem,
    Bloodstone,
    Arrowhead,
    OnyxAgate,
    GlassJoker,
    Showman,
    FlowerPot,
    Blueprint,
    WeeJoker,
    MerryAndy,
    OopsAll6s,
    TheIdol,
    SeeingDouble,
    Matador,
    HitTheRoad,
    TheDuo,
    TheTrio,
    TheFamily,
    TheOrder,
    TheTribe,
    Stuntman,
    InvisibleJoker,
    Brainstorm,
    Satellite,
    ShootTheMoon,
    DriversLicense,
    Cartomancer,
    Astronomer,
    BurntJoker,
    Bootstraps,
    Canio,
    Triboulet,
    Yorick,
    Chicot,
    Perkeo,
}

#[pymethods]
impl JokerKind {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
    pub fn display_name(&self) -> &'static str {
        match self {
            JokerKind::Joker => "Joker",
            JokerKind::GreedyJoker => "Greedy Joker",
            JokerKind::LustyJoker => "Lusty Joker",
            JokerKind::WrathfulJoker => "Wrathful Joker",
            JokerKind::GluttonousJoker => "Gluttonous Joker",
            JokerKind::JollyJoker => "Jolly Joker",
            JokerKind::ZanyJoker => "Zany Joker",
            JokerKind::MadJoker => "Mad Joker",
            JokerKind::CrazyJoker => "Crazy Joker",
            JokerKind::DrollJoker => "Droll Joker",
            JokerKind::SlyJoker => "Sly Joker",
            JokerKind::WilyJoker => "Wily Joker",
            JokerKind::CleverJoker => "Clever Joker",
            JokerKind::DeviousJoker => "Devious Joker",
            JokerKind::CraftyJoker => "Crafty Joker",
            JokerKind::HalfJoker => "Half Joker",
            JokerKind::JokerStencil => "Joker Stencil",
            JokerKind::FourFingers => "Four Fingers",
            JokerKind::Mime => "Mime",
            JokerKind::CreditCard => "Credit Card",
            JokerKind::CeremonialDagger => "Ceremonial Dagger",
            JokerKind::Banner => "Banner",
            JokerKind::MysticSummit => "Mystic Summit",
            JokerKind::MarbleJoker => "Marble Joker",
            JokerKind::LoyaltyCard => "Loyalty Card",
            JokerKind::EightBall => "8 Ball",
            JokerKind::Misprint => "Misprint",
            JokerKind::Dusk => "Dusk",
            JokerKind::RaisedFist => "Raised Fist",
            JokerKind::ChaosTheClown => "Chaos the Clown",
            JokerKind::Fibonacci => "Fibonacci",
            JokerKind::SteelJoker => "Steel Joker",
            JokerKind::ScaryFace => "Scary Face",
            JokerKind::AbstractJoker => "Abstract Joker",
            JokerKind::DelayedGratification => "Delayed Gratification",
            JokerKind::Hack => "Hack",
            JokerKind::Pareidolia => "Pareidolia",
            JokerKind::GrosMichel => "Gros Michel",
            JokerKind::EvenSteven => "Even Steven",
            JokerKind::OddTodd => "Odd Todd",
            JokerKind::Scholar => "Scholar",
            JokerKind::BusinessCard => "Business Card",
            JokerKind::Supernova => "Supernova",
            JokerKind::RideTheBus => "Ride the Bus",
            JokerKind::SpaceJoker => "Space Joker",
            JokerKind::Egg => "Egg",
            JokerKind::Burglar => "Burglar",
            JokerKind::Blackboard => "Blackboard",
            JokerKind::Runner => "Runner",
            JokerKind::IceCream => "Ice Cream",
            JokerKind::Dna => "DNA",
            JokerKind::Splash => "Splash",
            JokerKind::BlueJoker => "Blue Joker",
            JokerKind::SixthSense => "Sixth Sense",
            JokerKind::Constellation => "Constellation",
            JokerKind::Hiker => "Hiker",
            JokerKind::FacelessJoker => "Faceless Joker",
            JokerKind::GreenJoker => "Green Joker",
            JokerKind::Superposition => "Superposition",
            JokerKind::ToDoList => "To Do List",
            JokerKind::Cavendish => "Cavendish",
            JokerKind::CardSharp => "Card Sharp",
            JokerKind::RedCard => "Red Card",
            JokerKind::Madness => "Madness",
            JokerKind::SquareJoker => "Square Joker",
            JokerKind::Seance => "Seance",
            JokerKind::RiffRaff => "Riff-raff",
            JokerKind::Vampire => "Vampire",
            JokerKind::Shortcut => "Shortcut",
            JokerKind::Hologram => "Hologram",
            JokerKind::Vagabond => "Vagabond",
            JokerKind::Baron => "Baron",
            JokerKind::Cloud9 => "Cloud 9",
            JokerKind::Rocket => "Rocket",
            JokerKind::Obelisk => "Obelisk",
            JokerKind::MidasMask => "Midas Mask",
            JokerKind::Luchador => "Luchador",
            JokerKind::Photograph => "Photograph",
            JokerKind::GiftCard => "Gift Card",
            JokerKind::TurtleBean => "Turtle Bean",
            JokerKind::Erosion => "Erosion",
            JokerKind::ReservedParking => "Reserved Parking",
            JokerKind::MailInRebate => "Mail-In Rebate",
            JokerKind::ToTheMoon => "To the Moon",
            JokerKind::Hallucination => "Hallucination",
            JokerKind::FortuneTeller => "Fortune Teller",
            JokerKind::Juggler => "Juggler",
            JokerKind::Drunkard => "Drunkard",
            JokerKind::StoneJoker => "Stone Joker",
            JokerKind::GoldenJoker => "Golden Joker",
            JokerKind::LuckyCat => "Lucky Cat",
            JokerKind::BaseballCard => "Baseball Card",
            JokerKind::Bull => "Bull",
            JokerKind::DietCola => "Diet Cola",
            JokerKind::TradingCard => "Trading Card",
            JokerKind::FlashCard => "Flash Card",
            JokerKind::Popcorn => "Popcorn",
            JokerKind::SpareTrousers => "Spare Trousers",
            JokerKind::AncientJoker => "Ancient Joker",
            JokerKind::Ramen => "Ramen",
            JokerKind::WalkieTalkie => "Walkie Talkie",
            JokerKind::Seltzer => "Seltzer",
            JokerKind::Castle => "Castle",
            JokerKind::SmileyFace => "Smiley Face",
            JokerKind::Campfire => "Campfire",
            JokerKind::GoldenTicket => "Golden Ticket",
            JokerKind::MrBones => "Mr. Bones",
            JokerKind::Acrobat => "Acrobat",
            JokerKind::SockAndBuskin => "Sock and Buskin",
            JokerKind::Swashbuckler => "Swashbuckler",
            JokerKind::Troubadour => "Troubadour",
            JokerKind::Certificate => "Certificate",
            JokerKind::SmearedJoker => "Smeared Joker",
            JokerKind::Throwback => "Throwback",
            JokerKind::HangingChad => "Hanging Chad",
            JokerKind::RoughGem => "Rough Gem",
            JokerKind::Bloodstone => "Bloodstone",
            JokerKind::Arrowhead => "Arrowhead",
            JokerKind::OnyxAgate => "Onyx Agate",
            JokerKind::GlassJoker => "Glass Joker",
            JokerKind::Showman => "Showman",
            JokerKind::FlowerPot => "Flower Pot",
            JokerKind::Blueprint => "Blueprint",
            JokerKind::WeeJoker => "Wee Joker",
            JokerKind::MerryAndy => "Merry Andy",
            JokerKind::OopsAll6s => "Oops! All 6s",
            JokerKind::TheIdol => "The Idol",
            JokerKind::SeeingDouble => "Seeing Double",
            JokerKind::Matador => "Matador",
            JokerKind::HitTheRoad => "Hit the Road",
            JokerKind::TheDuo => "The Duo",
            JokerKind::TheTrio => "The Trio",
            JokerKind::TheFamily => "The Family",
            JokerKind::TheOrder => "The Order",
            JokerKind::TheTribe => "The Tribe",
            JokerKind::Stuntman => "Stuntman",
            JokerKind::InvisibleJoker => "Invisible Joker",
            JokerKind::Brainstorm => "Brainstorm",
            JokerKind::Satellite => "Satellite",
            JokerKind::ShootTheMoon => "Shoot the Moon",
            JokerKind::DriversLicense => "Driver's License",
            JokerKind::Cartomancer => "Cartomancer",
            JokerKind::Astronomer => "Astronomer",
            JokerKind::BurntJoker => "Burnt Joker",
            JokerKind::Bootstraps => "Bootstraps",
            JokerKind::Canio => "Canio",
            JokerKind::Triboulet => "Triboulet",
            JokerKind::Yorick => "Yorick",
            JokerKind::Chicot => "Chicot",
            JokerKind::Perkeo => "Perkeo",
        }
    }

    pub fn base_cost(&self) -> u32 {
        match self {
            JokerKind::Joker => 2,
            JokerKind::GreedyJoker
            | JokerKind::LustyJoker
            | JokerKind::WrathfulJoker
            | JokerKind::GluttonousJoker => 5,
            JokerKind::JollyJoker => 3,
            JokerKind::ZanyJoker
            | JokerKind::MadJoker
            | JokerKind::CrazyJoker
            | JokerKind::DrollJoker => 4,
            JokerKind::SlyJoker
            | JokerKind::WilyJoker
            | JokerKind::CleverJoker
            | JokerKind::DeviousJoker
            | JokerKind::CraftyJoker => 3,
            JokerKind::HalfJoker => 5,
            JokerKind::JokerStencil => 8,
            JokerKind::FourFingers => 7,
            JokerKind::Mime => 5,
            JokerKind::CreditCard => 1,
            JokerKind::CeremonialDagger => 6,
            JokerKind::Banner => 5,
            JokerKind::MysticSummit => 5,
            JokerKind::MarbleJoker => 6,
            JokerKind::LoyaltyCard => 5,
            JokerKind::EightBall => 5,
            JokerKind::Misprint => 4,
            JokerKind::Dusk => 5,
            JokerKind::RaisedFist => 5,
            JokerKind::ChaosTheClown => 4,
            JokerKind::Fibonacci => 8,
            JokerKind::SteelJoker => 7,
            JokerKind::ScaryFace => 4,
            JokerKind::AbstractJoker => 4,
            JokerKind::DelayedGratification => 4,
            JokerKind::Hack => 6,
            JokerKind::Pareidolia => 5,
            JokerKind::GrosMichel => 5,
            JokerKind::EvenSteven | JokerKind::OddTodd | JokerKind::Scholar => 4,
            JokerKind::BusinessCard => 4,
            JokerKind::Supernova => 5,
            JokerKind::RideTheBus => 6,
            JokerKind::SpaceJoker => 5,
            JokerKind::Egg => 4,
            JokerKind::Burglar => 6,
            JokerKind::Blackboard => 6,
            JokerKind::Runner => 5,
            JokerKind::IceCream => 5,
            JokerKind::Dna => 8,
            JokerKind::Splash => 3,
            JokerKind::BlueJoker => 5,
            JokerKind::SixthSense => 6,
            JokerKind::Constellation => 6,
            JokerKind::Hiker => 5,
            JokerKind::FacelessJoker => 4,
            JokerKind::GreenJoker => 4,
            JokerKind::Superposition => 4,
            JokerKind::ToDoList => 4,
            JokerKind::Cavendish => 4,
            JokerKind::CardSharp => 6,
            JokerKind::RedCard => 5,
            JokerKind::Madness => 7,
            JokerKind::SquareJoker => 4,
            JokerKind::Seance => 6,
            JokerKind::RiffRaff => 6,
            JokerKind::Vampire => 7,
            JokerKind::Shortcut => 7,
            JokerKind::Hologram => 7,
            JokerKind::Vagabond => 8,
            JokerKind::Baron => 8,
            JokerKind::Cloud9 => 7,
            JokerKind::Rocket => 6,
            JokerKind::Obelisk => 8,
            JokerKind::MidasMask => 7,
            JokerKind::Luchador => 5,
            JokerKind::Photograph => 5,
            JokerKind::GiftCard => 6,
            JokerKind::TurtleBean => 6,
            JokerKind::Erosion => 6,
            JokerKind::ReservedParking => 6,
            JokerKind::MailInRebate => 4,
            JokerKind::ToTheMoon => 5,
            JokerKind::Hallucination => 4,
            JokerKind::FortuneTeller => 6,
            JokerKind::Juggler => 4,
            JokerKind::Drunkard => 4,
            JokerKind::StoneJoker => 6,
            JokerKind::GoldenJoker => 6,
            JokerKind::LuckyCat => 6,
            JokerKind::BaseballCard => 8,
            JokerKind::Bull => 6,
            JokerKind::DietCola => 6,
            JokerKind::TradingCard => 6,
            JokerKind::FlashCard => 5,
            JokerKind::Popcorn => 5,
            JokerKind::SpareTrousers => 6,
            JokerKind::AncientJoker => 8,
            JokerKind::Ramen => 6,
            JokerKind::WalkieTalkie => 4,
            JokerKind::Seltzer => 6,
            JokerKind::Castle => 6,
            JokerKind::SmileyFace => 4,
            JokerKind::Campfire => 9,
            JokerKind::GoldenTicket => 5,
            JokerKind::MrBones => 5,
            JokerKind::Acrobat => 6,
            JokerKind::SockAndBuskin => 6,
            JokerKind::Swashbuckler => 4,
            JokerKind::Troubadour => 6,
            JokerKind::Certificate => 6,
            JokerKind::SmearedJoker => 7,
            JokerKind::Throwback => 6,
            JokerKind::HangingChad => 4,
            JokerKind::RoughGem => 7,
            JokerKind::Bloodstone => 7,
            JokerKind::Arrowhead => 7,
            JokerKind::OnyxAgate => 7,
            JokerKind::GlassJoker => 6,
            JokerKind::Showman => 5,
            JokerKind::FlowerPot => 6,
            JokerKind::Blueprint => 10,
            JokerKind::WeeJoker => 8,
            JokerKind::MerryAndy => 7,
            JokerKind::OopsAll6s => 4,
            JokerKind::TheIdol => 6,
            JokerKind::SeeingDouble => 6,
            JokerKind::Matador => 7,
            JokerKind::HitTheRoad => 8,
            JokerKind::TheDuo
            | JokerKind::TheTrio
            | JokerKind::TheFamily
            | JokerKind::TheOrder
            | JokerKind::TheTribe => 8,
            JokerKind::Stuntman => 7,
            JokerKind::InvisibleJoker => 8,
            JokerKind::Brainstorm => 10,
            JokerKind::Satellite => 6,
            JokerKind::ShootTheMoon => 5,
            JokerKind::DriversLicense => 7,
            JokerKind::Cartomancer => 6,
            JokerKind::Astronomer => 8,
            JokerKind::BurntJoker => 8,
            JokerKind::Bootstraps => 7,
            JokerKind::Canio
            | JokerKind::Triboulet
            | JokerKind::Yorick
            | JokerKind::Chicot
            | JokerKind::Perkeo => 20,
        }
    }

    pub fn rarity(&self) -> u8 {
        match self {
            JokerKind::Joker
            | JokerKind::GreedyJoker
            | JokerKind::LustyJoker
            | JokerKind::WrathfulJoker
            | JokerKind::GluttonousJoker
            | JokerKind::JollyJoker
            | JokerKind::ZanyJoker
            | JokerKind::MadJoker
            | JokerKind::CrazyJoker
            | JokerKind::DrollJoker
            | JokerKind::SlyJoker
            | JokerKind::WilyJoker
            | JokerKind::CleverJoker
            | JokerKind::DeviousJoker
            | JokerKind::CraftyJoker
            | JokerKind::HalfJoker
            | JokerKind::CreditCard
            | JokerKind::Banner
            | JokerKind::MysticSummit
            | JokerKind::EightBall
            | JokerKind::Misprint
            | JokerKind::RaisedFist
            | JokerKind::ChaosTheClown
            | JokerKind::ScaryFace
            | JokerKind::AbstractJoker
            | JokerKind::DelayedGratification
            | JokerKind::GrosMichel
            | JokerKind::EvenSteven
            | JokerKind::OddTodd
            | JokerKind::Scholar
            | JokerKind::BusinessCard
            | JokerKind::Supernova
            | JokerKind::RideTheBus
            | JokerKind::Egg
            | JokerKind::Runner
            | JokerKind::IceCream
            | JokerKind::Splash
            | JokerKind::BlueJoker
            | JokerKind::FacelessJoker
            | JokerKind::GreenJoker
            | JokerKind::Superposition
            | JokerKind::ToDoList
            | JokerKind::Cavendish
            | JokerKind::RedCard
            | JokerKind::SquareJoker
            | JokerKind::RiffRaff
            | JokerKind::Photograph
            | JokerKind::MailInRebate
            | JokerKind::Hallucination
            | JokerKind::FortuneTeller
            | JokerKind::Juggler
            | JokerKind::Drunkard
            | JokerKind::GoldenJoker
            | JokerKind::WalkieTalkie
            | JokerKind::SmileyFace
            | JokerKind::GoldenTicket
            | JokerKind::Swashbuckler
            | JokerKind::HangingChad
            | JokerKind::ReservedParking
            | JokerKind::ShootTheMoon => 1,

            JokerKind::JokerStencil
            | JokerKind::FourFingers
            | JokerKind::Mime
            | JokerKind::CeremonialDagger
            | JokerKind::MarbleJoker
            | JokerKind::LoyaltyCard
            | JokerKind::Dusk
            | JokerKind::Fibonacci
            | JokerKind::SteelJoker
            | JokerKind::Hack
            | JokerKind::Pareidolia
            | JokerKind::SpaceJoker
            | JokerKind::Burglar
            | JokerKind::Blackboard
            | JokerKind::Dna
            | JokerKind::SixthSense
            | JokerKind::Constellation
            | JokerKind::Hiker
            | JokerKind::CardSharp
            | JokerKind::Madness
            | JokerKind::Seance
            | JokerKind::Vampire
            | JokerKind::Shortcut
            | JokerKind::Hologram
            | JokerKind::Cloud9
            | JokerKind::Rocket
            | JokerKind::MidasMask
            | JokerKind::Luchador
            | JokerKind::GiftCard
            | JokerKind::TurtleBean
            | JokerKind::Erosion
            | JokerKind::ToTheMoon
            | JokerKind::LuckyCat
            | JokerKind::Bull
            | JokerKind::DietCola
            | JokerKind::TradingCard
            | JokerKind::FlashCard
            | JokerKind::SpareTrousers
            | JokerKind::Ramen
            | JokerKind::Seltzer
            | JokerKind::Castle
            | JokerKind::MrBones
            | JokerKind::Acrobat
            | JokerKind::SockAndBuskin
            | JokerKind::Troubadour
            | JokerKind::Certificate
            | JokerKind::SmearedJoker
            | JokerKind::Throwback
            | JokerKind::RoughGem
            | JokerKind::Bloodstone
            | JokerKind::Arrowhead
            | JokerKind::OnyxAgate
            | JokerKind::GlassJoker
            | JokerKind::Showman
            | JokerKind::FlowerPot
            | JokerKind::SeeingDouble
            | JokerKind::Matador
            | JokerKind::Stuntman
            | JokerKind::Satellite
            | JokerKind::Cartomancer
            | JokerKind::Astronomer
            | JokerKind::Bootstraps
            | JokerKind::TheIdol
            | JokerKind::OopsAll6s
            | JokerKind::Popcorn => 2,

            JokerKind::Vagabond
            | JokerKind::Baron
            | JokerKind::Obelisk
            | JokerKind::BaseballCard
            | JokerKind::AncientJoker
            | JokerKind::Campfire
            | JokerKind::Blueprint
            | JokerKind::WeeJoker
            | JokerKind::HitTheRoad
            | JokerKind::TheDuo
            | JokerKind::TheTrio
            | JokerKind::TheFamily
            | JokerKind::TheOrder
            | JokerKind::TheTribe
            | JokerKind::InvisibleJoker
            | JokerKind::Brainstorm
            | JokerKind::DriversLicense
            | JokerKind::BurntJoker => 3,

            JokerKind::Canio
            | JokerKind::Triboulet
            | JokerKind::Yorick
            | JokerKind::Chicot
            | JokerKind::Perkeo => 4,

            JokerKind::StoneJoker
            | JokerKind::MerryAndy => 2,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TarotCard {
    TheFool,
    TheMagician,
    TheHighPriestess,
    TheEmpress,
    TheEmperor,
    TheHierophant,
    TheLovers,
    TheChariot,
    Justice,
    TheHermit,
    TheWheelOfFortune,
    Strength,
    TheHangedMan,
    Death,
    Temperance,
    TheDevil,
    TheTower,
    TheStar,
    TheMoon,
    TheSun,
    Judgement,
    TheWorld,
}

#[pymethods]
impl TarotCard {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlanetCard {
    Mercury,  // Pair
    Venus,    // Three of a Kind
    Earth,    // Full House
    Mars,     // Four of a Kind
    Jupiter,  // Flush
    Saturn,   // Straight
    Uranus,   // Two Pair
    Neptune,  // Straight Flush
    Pluto,    // High Card
    PlanetX,  // Five of a Kind  (SECRET: only after playing Five of a Kind)
    Ceres,    // Flush House     (SECRET: only after playing Flush House)
    Eris,     // Flush Five      (SECRET: only after playing Flush Five)
}

#[pymethods]
impl PlanetCard {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
    pub fn hand_type(&self) -> HandType {
        match self {
            PlanetCard::Mercury => HandType::Pair,
            PlanetCard::Venus => HandType::ThreeOfAKind,
            PlanetCard::Earth => HandType::FullHouse,
            PlanetCard::Mars => HandType::FourOfAKind,
            PlanetCard::Jupiter => HandType::Flush,
            PlanetCard::Saturn => HandType::Straight,
            PlanetCard::Uranus => HandType::TwoPair,
            PlanetCard::Neptune => HandType::StraightFlush,
            PlanetCard::Pluto => HandType::HighCard,
            PlanetCard::PlanetX => HandType::FiveOfAKind,
            PlanetCard::Ceres => HandType::FlushHouse,
            PlanetCard::Eris => HandType::FlushFive,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpectralCard {
    Familiar,
    Grim,
    Incantation,
    Talisman,
    Aura,
    Wraith,
    Sigil,
    Ouija,
    Ectoplasm,
    Immolate,
    Ankh,
    DejaVu,
    Hex,
    Trance,
    Medium,
    Cryptid,
    TheSoul,
    BlackHole,
}

#[pymethods]
impl SpectralCard {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoucherKind {
    // Pair 1: shop card count
    Overstock,      // +1 card slot in shop
    OverstockPlus,  // +1 more card slot in shop
    // Pair 2: discounts
    ClearanceSale,  // -25% all shop prices
    Liquidation,    // -50% all shop prices (replaces ClearanceSale bonus)
    // Pair 3: card editions in packs
    Hone,           // Playing cards in packs can have Foil edition
    GlowUp,         // Playing cards in packs can have Holo/Poly edition too
    // Pair 4: reroll cost
    RerollSurplus,  // -$1 reroll cost
    RerollGlut,     // -$1 more reroll cost
    // Pair 5: consumable slots
    CrystalBall,    // +1 consumable slot
    OmenGlobe,      // Spectral cards can appear in Arcana packs
    // Pair 6: celestial packs
    Telescope,      // Celestial packs contain 1 extra card
    Observatory,    // Each planet used gives +0.5 Xmult to that hand type
    // Pair 7: hands per round
    Grabber,        // +1 hand per round
    NachoTong,      // +1 more hand per round
    // Pair 8: discards per round
    Wasteful,       // +1 discard per round
    Recyclomancy,   // +1 more discard per round
    // Pair 9: tarot card prices
    TarotMerchant,  // Tarot cards cost $1 less
    TarotTycoon,    // Tarot cards cost $0
    // Pair 10: planet card prices
    PlanetMerchant, // Planet cards cost $1 less
    PlanetTycoon,   // Planet cards cost $0
    // Pair 11: interest
    SeedMoney,      // +$10 max interest
    MoneyTree,      // +$10 more max interest
    // Pair 12: joker slots
    Blank,          // +1 joker slot
    Antimatter,     // +1 more joker slot
    // Pair 13: playing cards in shop
    MagicTrick,     // Playing cards can appear in the shop
    Illusion,       // Playing cards in packs can have editions
    // Pair 14: ante skip
    Hieroglyph,     // -1 ante required to win
    Petroglyph,     // -1 more ante required to win
    // Pair 15: free rerolls / boss reroll
    DirectorsCut,   // +1 free reroll per shop
    Retcon,         // Can reroll the boss blind once per ante
    // Pair 16: hand size
    PaintBrush,     // +1 hand size
    Palette,        // +1 more hand size
}

#[pymethods]
impl VoucherKind {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackKind {
    ArcanaPackSmall,
    ArcanaPack,
    ArcanaPackJumbo,
    ArcanaPackMega,
    CelestialPackSmall,
    CelestialPack,
    CelestialPackJumbo,
    CelestialPackMega,
    SpectralPackSmall,
    SpectralPack,
    SpectralPackJumbo,
    SpectralPackMega,
    StandardPackSmall,
    StandardPack,
    StandardPackJumbo,
    StandardPackMega,
    BuffoonPackSmall,
    BuffoonPack,
    BuffoonPackJumbo,
    BuffoonPackMega,
}

#[pymethods]
impl PackKind {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    pub fn cards_shown(&self) -> usize {
        match self {
            PackKind::ArcanaPackSmall
            | PackKind::CelestialPackSmall
            | PackKind::SpectralPackSmall
            | PackKind::StandardPackSmall
            | PackKind::BuffoonPackSmall => 3,
            PackKind::ArcanaPack
            | PackKind::CelestialPack
            | PackKind::SpectralPack
            | PackKind::StandardPack
            | PackKind::BuffoonPack => 3,
            PackKind::ArcanaPackJumbo
            | PackKind::CelestialPackJumbo
            | PackKind::SpectralPackJumbo
            | PackKind::StandardPackJumbo
            | PackKind::BuffoonPackJumbo => 5,
            PackKind::ArcanaPackMega
            | PackKind::CelestialPackMega
            | PackKind::SpectralPackMega
            | PackKind::StandardPackMega
            | PackKind::BuffoonPackMega => 5,
        }
    }

    pub fn picks_allowed(&self) -> usize {
        match self {
            PackKind::ArcanaPackSmall
            | PackKind::CelestialPackSmall
            | PackKind::SpectralPackSmall
            | PackKind::StandardPackSmall
            | PackKind::BuffoonPackSmall => 1,
            PackKind::ArcanaPack
            | PackKind::CelestialPack
            | PackKind::SpectralPack
            | PackKind::StandardPack
            | PackKind::BuffoonPack => 1,
            PackKind::ArcanaPackJumbo
            | PackKind::CelestialPackJumbo
            | PackKind::SpectralPackJumbo
            | PackKind::StandardPackJumbo
            | PackKind::BuffoonPackJumbo => 2,
            PackKind::ArcanaPackMega
            | PackKind::CelestialPackMega
            | PackKind::SpectralPackMega
            | PackKind::StandardPackMega
            | PackKind::BuffoonPackMega => 2,
        }
    }

    pub fn base_cost(&self) -> u32 {
        match self {
            PackKind::ArcanaPackSmall | PackKind::CelestialPackSmall => 4,
            PackKind::SpectralPackSmall | PackKind::StandardPackSmall => 4,
            PackKind::BuffoonPackSmall => 4,
            PackKind::ArcanaPack | PackKind::CelestialPack => 4,
            PackKind::SpectralPack | PackKind::StandardPack | PackKind::BuffoonPack => 4,
            PackKind::ArcanaPackJumbo
            | PackKind::CelestialPackJumbo
            | PackKind::SpectralPackJumbo => 6,
            PackKind::StandardPackJumbo | PackKind::BuffoonPackJumbo => 6,
            PackKind::ArcanaPackMega
            | PackKind::CelestialPackMega
            | PackKind::SpectralPackMega => 8,
            PackKind::StandardPackMega | PackKind::BuffoonPackMega => 8,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TagKind {
    Uncommon,
    Rare,
    Negative,
    Foil,
    Holographic,
    Polychrome,
    Investment,
    Voucher,
    Boss,
    Standard,
    Charm,
    Meteor,
    Buffoon,
    Handy,
    Garbage,
    Ethereal,
    Coupon,
    DoubleFun,
    Juggle,
    D6,
    TopUp,
    Skip,
    Orbital,
    Economy,
}

#[pymethods]
impl TagKind {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
