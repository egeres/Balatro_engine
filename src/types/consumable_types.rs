use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use super::{Edition, HandType};

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
    // Pair 3: how often editions show up
    Hone,           // Foil/Holographic/Polychrome appear 2X as often
    GlowUp,         // Foil/Holographic/Polychrome appear 4X as often
    // Pair 4: reroll cost
    RerollSurplus,  // -$2 reroll cost
    RerollGlut,     // -$2 more reroll cost
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
    // Pair 9: how often tarots are stocked
    TarotMerchant,  // Tarot cards appear 2X as often in the shop
    TarotTycoon,    // Tarot cards appear 4X as often in the shop
    // Pair 10: how often planets are stocked
    PlanetMerchant, // Planet cards appear 2X as often in the shop
    PlanetTycoon,   // Planet cards appear 4X as often in the shop
    // Pair 11: interest
    SeedMoney,      // +$10 max interest
    MoneyTree,      // +$10 more max interest
    // Pair 12: joker slots
    Blank,          // Does nothing; exists only to unlock Antimatter
    Antimatter,     // +1 joker slot
    // Pair 13: playing cards in shop
    MagicTrick,     // Playing cards can appear in the shop
    Illusion,       // Shop playing cards may carry an enhancement, edition and/or seal
    // Pair 14: ante
    Hieroglyph,     // -1 Ante, -1 hand each round (the winning ante is unchanged)
    Petroglyph,     // -1 Ante, -1 discard each round
    // Pair 15: boss reroll
    DirectorsCut,   // Reroll the Boss blind once per ante for $10
    Retcon,         // Reroll the Boss blind as often as you like, $10 a go
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
            // Arcana/Celestial/Standard: Normal=3, Jumbo=5, Mega=5
            PackKind::ArcanaPackSmall | PackKind::ArcanaPack => 3,
            PackKind::CelestialPackSmall | PackKind::CelestialPack => 3,
            PackKind::StandardPackSmall | PackKind::StandardPack => 3,
            PackKind::ArcanaPackJumbo | PackKind::ArcanaPackMega => 5,
            PackKind::CelestialPackJumbo | PackKind::CelestialPackMega => 5,
            PackKind::StandardPackJumbo | PackKind::StandardPackMega => 5,
            // Buffoon/Spectral: Normal=2, Jumbo=4, Mega=4
            PackKind::BuffoonPackSmall | PackKind::BuffoonPack => 2,
            PackKind::SpectralPackSmall | PackKind::SpectralPack => 2,
            PackKind::BuffoonPackJumbo | PackKind::BuffoonPackMega => 4,
            PackKind::SpectralPackJumbo | PackKind::SpectralPackMega => 4,
        }
    }

    pub fn picks_allowed(&self) -> usize {
        match self {
            // Normal and Jumbo: choose 1; Mega: choose up to 2
            PackKind::ArcanaPackSmall | PackKind::ArcanaPack | PackKind::ArcanaPackJumbo => 1,
            PackKind::CelestialPackSmall | PackKind::CelestialPack | PackKind::CelestialPackJumbo => 1,
            PackKind::SpectralPackSmall | PackKind::SpectralPack | PackKind::SpectralPackJumbo => 1,
            PackKind::StandardPackSmall | PackKind::StandardPack | PackKind::StandardPackJumbo => 1,
            PackKind::BuffoonPackSmall | PackKind::BuffoonPack | PackKind::BuffoonPackJumbo => 1,
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

/// When a tag's effect fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagTrigger {
    /// Pays out the moment the blind is skipped.
    Immediate,
    /// Waits for the next shop.
    Shop,
    /// Waits for the next blind-select screen (free booster packs, Boss reroll).
    BlindSelect,
    /// Waits for the start of the next round.
    RoundStart,
    /// Waits until the next Boss blind is defeated.
    BossDefeated,
    /// Copies the next tag acquired.
    CopyNextTag,
}

#[pymethods]
impl TagKind {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TagKind::Uncommon => "Uncommon Tag",
            TagKind::Rare => "Rare Tag",
            TagKind::Negative => "Negative Tag",
            TagKind::Foil => "Foil Tag",
            TagKind::Holographic => "Holographic Tag",
            TagKind::Polychrome => "Polychrome Tag",
            TagKind::Investment => "Investment Tag",
            TagKind::Voucher => "Voucher Tag",
            TagKind::Boss => "Boss Tag",
            TagKind::Standard => "Standard Tag",
            TagKind::Charm => "Charm Tag",
            TagKind::Meteor => "Meteor Tag",
            TagKind::Buffoon => "Buffoon Tag",
            TagKind::Handy => "Handy Tag",
            TagKind::Garbage => "Garbage Tag",
            TagKind::Ethereal => "Ethereal Tag",
            TagKind::Coupon => "Coupon Tag",
            TagKind::DoubleFun => "Double Tag",
            TagKind::Juggle => "Juggle Tag",
            TagKind::D6 => "D6 Tag",
            TagKind::TopUp => "Top-up Tag",
            TagKind::Skip => "Skip Tag",
            TagKind::Orbital => "Orbital Tag",
            TagKind::Economy => "Economy Tag",
        }
    }

    /// Earliest ante this tag can be offered at (`min_ante` in game.lua:225-248).
    pub fn min_ante(&self) -> u32 {
        match self {
            TagKind::Negative
            | TagKind::Standard
            | TagKind::Meteor
            | TagKind::Buffoon
            | TagKind::Handy
            | TagKind::Garbage
            | TagKind::Ethereal
            | TagKind::TopUp
            | TagKind::Orbital => 2,
            _ => 1,
        }
    }
}

impl TagKind {
    pub const ALL: [TagKind; 24] = [
        TagKind::Uncommon, TagKind::Rare, TagKind::Negative, TagKind::Foil,
        TagKind::Holographic, TagKind::Polychrome, TagKind::Investment, TagKind::Voucher,
        TagKind::Boss, TagKind::Standard, TagKind::Charm, TagKind::Meteor,
        TagKind::Buffoon, TagKind::Handy, TagKind::Garbage, TagKind::Ethereal,
        TagKind::Coupon, TagKind::DoubleFun, TagKind::Juggle, TagKind::D6,
        TagKind::TopUp, TagKind::Skip, TagKind::Orbital, TagKind::Economy,
    ];

    /// When this tag's effect fires (the `config.type` field in game.lua:225-248).
    pub fn trigger(&self) -> TagTrigger {
        match self {
            TagKind::Skip
            | TagKind::Handy
            | TagKind::Garbage
            | TagKind::Economy
            | TagKind::Orbital
            | TagKind::TopUp => TagTrigger::Immediate,

            TagKind::Uncommon
            | TagKind::Rare
            | TagKind::Negative
            | TagKind::Foil
            | TagKind::Holographic
            | TagKind::Polychrome
            | TagKind::Coupon
            | TagKind::D6
            | TagKind::Voucher => TagTrigger::Shop,

            TagKind::Boss
            | TagKind::Standard
            | TagKind::Charm
            | TagKind::Meteor
            | TagKind::Buffoon
            | TagKind::Ethereal => TagTrigger::BlindSelect,

            TagKind::Juggle => TagTrigger::RoundStart,
            TagKind::Investment => TagTrigger::BossDefeated,
            TagKind::DoubleFun => TagTrigger::CopyNextTag,
        }
    }

    /// The free booster pack this tag opens at the next blind select, if any (tag.lua:206-260).
    pub fn free_pack(&self) -> Option<PackKind> {
        match self {
            TagKind::Charm => Some(PackKind::ArcanaPackMega),
            TagKind::Meteor => Some(PackKind::CelestialPackMega),
            TagKind::Standard => Some(PackKind::StandardPackMega),
            TagKind::Buffoon => Some(PackKind::BuffoonPackMega),
            TagKind::Ethereal => Some(PackKind::SpectralPack),
            _ => None,
        }
    }

    /// The edition this tag forces onto the next shop joker, if any.
    pub fn forced_edition(&self) -> Option<Edition> {
        match self {
            TagKind::Foil => Some(Edition::Foil),
            TagKind::Holographic => Some(Edition::Holographic),
            TagKind::Polychrome => Some(Edition::Polychrome),
            TagKind::Negative => Some(Edition::Negative),
            _ => None,
        }
    }

    /// The rarity this tag forces onto the next shop joker, if any.
    pub fn forced_rarity(&self) -> Option<u8> {
        match self {
            TagKind::Uncommon => Some(2),
            TagKind::Rare => Some(3),
            _ => None,
        }
    }
}

/// The Planet card that levels `hand_type`, if one exists.
pub fn planet_for_hand(hand_type: HandType) -> Option<PlanetCard> {
    [
        PlanetCard::Mercury, PlanetCard::Venus, PlanetCard::Earth, PlanetCard::Mars,
        PlanetCard::Jupiter, PlanetCard::Saturn, PlanetCard::Uranus, PlanetCard::Neptune,
        PlanetCard::Pluto, PlanetCard::PlanetX, PlanetCard::Ceres, PlanetCard::Eris,
    ]
    .into_iter()
    .find(|p| p.hand_type() == hand_type)
}
