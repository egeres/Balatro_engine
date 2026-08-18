use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// Everything static about a joker, in one row.
///
/// One table instead of three parallel `match self` arms: a joker's name, price and rarity are
/// read together far more often than apart, and keeping them on one line is what stops the three
/// from drifting when a joker is added or retuned.
struct JokerData {
    kind: JokerKind,
    name: &'static str,
    cost: u32,
    rarity: u8,
}

/// Every joker, in `order` from game.lua. Indexed by enum discriminant, so the order here is
/// load-bearing — `joker_table_is_indexed_by_discriminant` guards it.
const JOKERS: [JokerData; 150] = [
    JokerData { kind: JokerKind::Joker,                name: "Joker",                 cost:  2, rarity: 1 },
    JokerData { kind: JokerKind::GreedyJoker,          name: "Greedy Joker",          cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::LustyJoker,           name: "Lusty Joker",           cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::WrathfulJoker,        name: "Wrathful Joker",        cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::GluttonousJoker,      name: "Gluttonous Joker",      cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::JollyJoker,           name: "Jolly Joker",           cost:  3, rarity: 1 },
    JokerData { kind: JokerKind::ZanyJoker,            name: "Zany Joker",            cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::MadJoker,             name: "Mad Joker",             cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::CrazyJoker,           name: "Crazy Joker",           cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::DrollJoker,           name: "Droll Joker",           cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::SlyJoker,             name: "Sly Joker",             cost:  3, rarity: 1 },
    JokerData { kind: JokerKind::WilyJoker,            name: "Wily Joker",            cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::CleverJoker,          name: "Clever Joker",          cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::DeviousJoker,         name: "Devious Joker",         cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::CraftyJoker,          name: "Crafty Joker",          cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::HalfJoker,            name: "Half Joker",            cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::JokerStencil,         name: "Joker Stencil",         cost:  8, rarity: 2 },
    JokerData { kind: JokerKind::FourFingers,          name: "Four Fingers",          cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::Mime,                 name: "Mime",                  cost:  5, rarity: 2 },
    JokerData { kind: JokerKind::CreditCard,           name: "Credit Card",           cost:  1, rarity: 1 },
    JokerData { kind: JokerKind::CeremonialDagger,     name: "Ceremonial Dagger",     cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Banner,               name: "Banner",                cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::MysticSummit,         name: "Mystic Summit",         cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::MarbleJoker,          name: "Marble Joker",          cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::LoyaltyCard,          name: "Loyalty Card",          cost:  5, rarity: 2 },
    JokerData { kind: JokerKind::EightBall,            name: "8 Ball",                cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::Misprint,             name: "Misprint",              cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Dusk,                 name: "Dusk",                  cost:  5, rarity: 2 },
    JokerData { kind: JokerKind::RaisedFist,           name: "Raised Fist",           cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::ChaosTheClown,        name: "Chaos the Clown",       cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Fibonacci,            name: "Fibonacci",             cost:  8, rarity: 2 },
    JokerData { kind: JokerKind::SteelJoker,           name: "Steel Joker",           cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::ScaryFace,            name: "Scary Face",            cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::AbstractJoker,        name: "Abstract Joker",        cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::DelayedGratification, name: "Delayed Gratification", cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Hack,                 name: "Hack",                  cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Pareidolia,           name: "Pareidolia",            cost:  5, rarity: 2 },
    JokerData { kind: JokerKind::GrosMichel,           name: "Gros Michel",           cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::EvenSteven,           name: "Even Steven",           cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::OddTodd,              name: "Odd Todd",              cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Scholar,              name: "Scholar",               cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::BusinessCard,         name: "Business Card",         cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Supernova,            name: "Supernova",             cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::RideTheBus,           name: "Ride the Bus",          cost:  6, rarity: 1 },
    JokerData { kind: JokerKind::SpaceJoker,           name: "Space Joker",           cost:  5, rarity: 2 },
    JokerData { kind: JokerKind::Egg,                  name: "Egg",                   cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Burglar,              name: "Burglar",               cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Blackboard,           name: "Blackboard",            cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Runner,               name: "Runner",                cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::IceCream,             name: "Ice Cream",             cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::Dna,                  name: "DNA",                   cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::Splash,               name: "Splash",                cost:  3, rarity: 1 },
    JokerData { kind: JokerKind::BlueJoker,            name: "Blue Joker",            cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::SixthSense,           name: "Sixth Sense",           cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Constellation,        name: "Constellation",         cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Hiker,                name: "Hiker",                 cost:  5, rarity: 2 },
    JokerData { kind: JokerKind::FacelessJoker,        name: "Faceless Joker",        cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::GreenJoker,           name: "Green Joker",           cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Superposition,        name: "Superposition",         cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::ToDoList,             name: "To Do List",            cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Cavendish,            name: "Cavendish",             cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::CardSharp,            name: "Card Sharp",            cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::RedCard,              name: "Red Card",              cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::Madness,              name: "Madness",               cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::SquareJoker,          name: "Square Joker",          cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Seance,               name: "Seance",                cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::RiffRaff,             name: "Riff-raff",             cost:  6, rarity: 1 },
    JokerData { kind: JokerKind::Vampire,              name: "Vampire",               cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::Shortcut,             name: "Shortcut",              cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::Hologram,             name: "Hologram",              cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::Vagabond,             name: "Vagabond",              cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::Baron,                name: "Baron",                 cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::Cloud9,               name: "Cloud 9",               cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::Rocket,               name: "Rocket",                cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Obelisk,              name: "Obelisk",               cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::MidasMask,            name: "Midas Mask",            cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::Luchador,             name: "Luchador",              cost:  5, rarity: 2 },
    JokerData { kind: JokerKind::Photograph,           name: "Photograph",            cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::GiftCard,             name: "Gift Card",             cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::TurtleBean,           name: "Turtle Bean",           cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Erosion,              name: "Erosion",               cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::ReservedParking,      name: "Reserved Parking",      cost:  6, rarity: 1 },
    JokerData { kind: JokerKind::MailInRebate,         name: "Mail-In Rebate",        cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::ToTheMoon,            name: "To the Moon",           cost:  5, rarity: 2 },
    JokerData { kind: JokerKind::Hallucination,        name: "Hallucination",         cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::FortuneTeller,        name: "Fortune Teller",        cost:  6, rarity: 1 },
    JokerData { kind: JokerKind::Juggler,              name: "Juggler",               cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Drunkard,             name: "Drunkard",              cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::StoneJoker,           name: "Stone Joker",           cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::GoldenJoker,          name: "Golden Joker",          cost:  6, rarity: 1 },
    JokerData { kind: JokerKind::LuckyCat,             name: "Lucky Cat",             cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::BaseballCard,         name: "Baseball Card",         cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::Bull,                 name: "Bull",                  cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::DietCola,             name: "Diet Cola",             cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::TradingCard,          name: "Trading Card",          cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::FlashCard,            name: "Flash Card",            cost:  5, rarity: 2 },
    JokerData { kind: JokerKind::Popcorn,              name: "Popcorn",               cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::SpareTrousers,        name: "Spare Trousers",        cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::AncientJoker,         name: "Ancient Joker",         cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::Ramen,                name: "Ramen",                 cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::WalkieTalkie,         name: "Walkie Talkie",         cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Seltzer,              name: "Seltzer",               cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Castle,               name: "Castle",                cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::SmileyFace,           name: "Smiley Face",           cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Campfire,             name: "Campfire",              cost:  9, rarity: 3 },
    JokerData { kind: JokerKind::GoldenTicket,         name: "Golden Ticket",         cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::MrBones,              name: "Mr. Bones",             cost:  5, rarity: 2 },
    JokerData { kind: JokerKind::Acrobat,              name: "Acrobat",               cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::SockAndBuskin,        name: "Sock and Buskin",       cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Swashbuckler,         name: "Swashbuckler",          cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::Troubadour,           name: "Troubadour",            cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Certificate,          name: "Certificate",           cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::SmearedJoker,         name: "Smeared Joker",         cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::Throwback,            name: "Throwback",             cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::HangingChad,          name: "Hanging Chad",          cost:  4, rarity: 1 },
    JokerData { kind: JokerKind::RoughGem,             name: "Rough Gem",             cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::Bloodstone,           name: "Bloodstone",            cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::Arrowhead,            name: "Arrowhead",             cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::OnyxAgate,            name: "Onyx Agate",            cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::GlassJoker,           name: "Glass Joker",           cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Showman,              name: "Showman",               cost:  5, rarity: 2 },
    JokerData { kind: JokerKind::FlowerPot,            name: "Flower Pot",            cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Blueprint,            name: "Blueprint",             cost: 10, rarity: 3 },
    JokerData { kind: JokerKind::WeeJoker,             name: "Wee Joker",             cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::MerryAndy,            name: "Merry Andy",            cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::OopsAll6s,            name: "Oops! All 6s",          cost:  4, rarity: 2 },
    JokerData { kind: JokerKind::TheIdol,              name: "The Idol",              cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::SeeingDouble,         name: "Seeing Double",         cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Matador,              name: "Matador",               cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::HitTheRoad,           name: "Hit the Road",          cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::TheDuo,               name: "The Duo",               cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::TheTrio,              name: "The Trio",              cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::TheFamily,            name: "The Family",            cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::TheOrder,             name: "The Order",             cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::TheTribe,             name: "The Tribe",             cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::Stuntman,             name: "Stuntman",              cost:  7, rarity: 3 },
    JokerData { kind: JokerKind::InvisibleJoker,       name: "Invisible Joker",       cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::Brainstorm,           name: "Brainstorm",            cost: 10, rarity: 3 },
    JokerData { kind: JokerKind::Satellite,            name: "Satellite",             cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::ShootTheMoon,         name: "Shoot the Moon",        cost:  5, rarity: 1 },
    JokerData { kind: JokerKind::DriversLicense,       name: "Driver's License",      cost:  7, rarity: 3 },
    JokerData { kind: JokerKind::Cartomancer,          name: "Cartomancer",           cost:  6, rarity: 2 },
    JokerData { kind: JokerKind::Astronomer,           name: "Astronomer",            cost:  8, rarity: 2 },
    JokerData { kind: JokerKind::BurntJoker,           name: "Burnt Joker",           cost:  8, rarity: 3 },
    JokerData { kind: JokerKind::Bootstraps,           name: "Bootstraps",            cost:  7, rarity: 2 },
    JokerData { kind: JokerKind::Canio,                name: "Canio",                 cost: 20, rarity: 4 },
    JokerData { kind: JokerKind::Triboulet,            name: "Triboulet",             cost: 20, rarity: 4 },
    JokerData { kind: JokerKind::Yorick,               name: "Yorick",                cost: 20, rarity: 4 },
    JokerData { kind: JokerKind::Chicot,               name: "Chicot",                cost: 20, rarity: 4 },
    JokerData { kind: JokerKind::Perkeo,               name: "Perkeo",                cost: 20, rarity: 4 },
];

#[pyclass(eq, eq_int, from_py_object)]
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
        self.data().name
    }

    pub fn base_cost(&self) -> u32 {
        self.data().cost
    }

    /// Whether this joker can be given the Eternal sticker (`eternal_compat` in game.lua).
    /// The jokers that consume themselves cannot — an Eternal one could never finish.
    pub fn eternal_compat(&self) -> bool {
        !matches!(
            self,
            JokerKind::GrosMichel
                | JokerKind::IceCream
                | JokerKind::Cavendish
                | JokerKind::Luchador
                | JokerKind::TurtleBean
                | JokerKind::DietCola
                | JokerKind::Popcorn
                | JokerKind::Ramen
                | JokerKind::Seltzer
                | JokerKind::MrBones
                | JokerKind::InvisibleJoker
        )
    }

    /// Whether this joker can be given the Perishable sticker (`perishable_compat` in game.lua).
    /// The scaling jokers are exempt: being disabled after five rounds would waste the scaling.
    pub fn perishable_compat(&self) -> bool {
        !matches!(
            self,
            JokerKind::CeremonialDagger
                | JokerKind::RideTheBus
                | JokerKind::Runner
                | JokerKind::Constellation
                | JokerKind::GreenJoker
                | JokerKind::RedCard
                | JokerKind::Madness
                | JokerKind::SquareJoker
                | JokerKind::Vampire
                | JokerKind::Hologram
                | JokerKind::Rocket
                | JokerKind::Obelisk
                | JokerKind::LuckyCat
                | JokerKind::FlashCard
                | JokerKind::SpareTrousers
                | JokerKind::Castle
                | JokerKind::GlassJoker
                | JokerKind::WeeJoker
        )
    }

    /// Whether Blueprint / Brainstorm can copy this joker (`blueprint_compat` in game.lua).
    /// The incompatible ones are mostly passive rule-changers and once-per-round economy jokers
    /// whose effects have no meaning when duplicated.
    pub fn blueprint_compat(&self) -> bool {
        !matches!(
            self,
            JokerKind::FourFingers
                | JokerKind::CreditCard
                | JokerKind::ChaosTheClown
                | JokerKind::DelayedGratification
                | JokerKind::Pareidolia
                | JokerKind::Egg
                | JokerKind::Splash
                | JokerKind::SixthSense
                | JokerKind::Shortcut
                | JokerKind::Cloud9
                | JokerKind::Rocket
                | JokerKind::MidasMask
                | JokerKind::GiftCard
                | JokerKind::TurtleBean
                | JokerKind::ToTheMoon
                | JokerKind::Juggler
                | JokerKind::Drunkard
                | JokerKind::GoldenJoker
                | JokerKind::TradingCard
                | JokerKind::MrBones
                | JokerKind::Troubadour
                | JokerKind::SmearedJoker
                | JokerKind::Showman
                | JokerKind::MerryAndy
                | JokerKind::OopsAll6s
                | JokerKind::InvisibleJoker
                | JokerKind::Satellite
                | JokerKind::Astronomer
                | JokerKind::Chicot
        )
    }

    pub fn rarity(&self) -> u8 {
        self.data().rarity
    }
}

impl JokerKind {
    /// Every joker, in `order` from game.lua.
    ///
    /// Derived from [`JOKERS`] rather than written out a second time, so a joker cannot be added
    /// to the table and then quietly left out of every pool.
    pub const ALL: [JokerKind; JOKERS.len()] = {
        let mut all = [JokerKind::Joker; JOKERS.len()];
        let mut i = 0;
        while i < JOKERS.len() {
            all[i] = JOKERS[i].kind;
            i += 1;
        }
        all
    };

    /// This joker's row in [`JOKERS`].
    fn data(&self) -> &'static JokerData {
        &JOKERS[*self as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `JokerKind::data` indexes [`JOKERS`] by discriminant, so a row inserted out of order would
    /// silently hand every joker after it the wrong name, price and rarity.
    #[test]
    fn joker_table_is_indexed_by_discriminant() {
        for (i, entry) in JOKERS.iter().enumerate() {
            assert_eq!(
                entry.kind as usize, i,
                "JOKERS[{i}] holds {:?}, whose discriminant is {}",
                entry.kind, entry.kind as usize
            );
        }
    }
}
