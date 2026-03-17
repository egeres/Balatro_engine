use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

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
            JokerKind::SlyJoker => 3,
            JokerKind::WilyJoker
            | JokerKind::CleverJoker
            | JokerKind::DeviousJoker
            | JokerKind::CraftyJoker => 4,
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
            | JokerKind::Satellite
            | JokerKind::Cartomancer
            | JokerKind::Astronomer
            | JokerKind::Bootstraps
            | JokerKind::TheIdol
            | JokerKind::OopsAll6s
            | JokerKind::Popcorn => 2,

            JokerKind::Dna
            | JokerKind::Stuntman
            | JokerKind::Vagabond
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
