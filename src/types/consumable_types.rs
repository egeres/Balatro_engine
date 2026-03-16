use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use super::HandType;

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
