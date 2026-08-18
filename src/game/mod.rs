use crate::card::*;
use crate::rng::Rng;
use crate::scoring::RoundTargets;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Full game state
pub struct GameState {
    pub rng: Rng,
    pub deck_type: DeckType,
    pub stake: Stake,
    pub seed: String,

    // Run-level state
    pub ante: u32,
    pub round: u32, // 1=small, 2=big, 3=boss
    pub money: i32,
    pub state: GameStateKind,
    pub vouchers: Vec<VoucherKind>,
    pub tags: Vec<TagKind>,
    pub tarot_cards_used: u32,
    pub planet_cards_used: u32,
    /// Tracks which distinct PlanetCard types have been used this run (for Satellite joker)
    pub planet_types_used: std::collections::HashSet<PlanetCard>,

    // Blind state
    pub current_blind: BlindKind,
    pub boss_blind: Option<BossBlind>,
    /// The tag sitting on each skippable blind this ante — `[Small, Big]`.
    ///
    /// Balatro draws both when the ante starts and shows them on the blind-select screen
    /// (`round_resets.blind_tags`, game.lua:2179), so skipping is an informed choice rather than
    /// a blind roll. Redrawn when the Boss falls (button_callbacks.lua:2951).
    pub blind_tags: [TagKind; 2],
    /// The hand each slot's Orbital Tag would level, drawn with the tag itself so it can be read
    /// off the blind-select screen. Meaningless unless that slot's tag *is* an Orbital Tag.
    pub blind_tag_orbital_hands: [HandType; 2],
    pub score_goal: f64,
    pub skipped_blinds: Vec<(u32, u32)>, // (ante, round) of skipped blinds
    pub blind_defeated_this_ante: [bool; 3],

    // Round state
    pub deck: Vec<CardInstance>,  // full ordered deck
    /// Number of cards the deck started the run with. Erosion measures the full deck against
    /// this, not against a hardcoded 52 (Abandoned Deck starts at 40).
    pub starting_deck_size: usize,
    pub draw_pile: Vec<usize>,    // indices into deck of remaining drawable cards
    pub hand: Vec<usize>,         // indices of cards currently in hand
    pub discard_pile: Vec<usize>, // indices of discarded cards this round
    pub jokers: Vec<JokerInstance>,
    pub consumables: Vec<HeldConsumable>,
    pub hands_remaining: u32,
    pub discards_remaining: u32,
    pub score_accumulated: f64,
    pub selected_indices: Vec<usize>, // selected from hand (hand-relative indices)

    // Hand levels
    pub hand_levels: HashMap<HandType, HandLevelData>,

    // Shop state
    pub shop_offers: Vec<ShopOffer>,
    pub shop_voucher: Option<VoucherKind>,
    pub reroll_cost: u32,
    pub free_rerolls: u32,

    // Pack state
    pub current_pack: Option<PackContents>,

    // Config
    pub hand_size: u32,
    pub max_hands: u32,
    pub max_discards: u32,
    pub joker_slots: u32,
    pub consumable_slots: u32,
    pub max_interest: i32,

    // History
    pub history: Vec<HistoryEvent>,
    pub next_id: u64,

    // For The Fool tarot: remembers the most recently used tarot or planet this run
    pub last_consumable_used: Option<LastConsumable>,

    // Showdown boss blind state
    /// CeruleanBell: ID of the card that is always forced-selected this draw.
    pub cerulean_forced_card_id: Option<u64>,
    /// VerdantLeaf: set to true once the first joker is sold this blind.
    pub verdant_leaf_joker_sold: bool,

    /// Set when Luchador is sold during an active Boss blind. Unlike Chicot (which disables
    /// passively while held), Luchador only disables the blind at the moment it is sold, so the
    /// effect has to be latched for the rest of the round. Cleared when a new round begins.
    pub boss_blind_manually_disabled: bool,

    /// The hand type most recently played this run. Blue Seal creates the Planet for it
    /// (`G.GAME.last_hand_played`, card.lua:1046).
    pub last_hand_played: Option<HandType>,

    /// Randomised targets for The Idol / Ancient Joker / Castle / Mail-In Rebate. Shared by all
    /// copies of a joker and re-rolled at the start of every round.
    pub round_targets: RoundTargets,

    /// Set once a Gros Michel has been destroyed. Gros Michel then leaves the pool and Cavendish
    /// enters it (`no_pool_flag` / `yes_pool_flag` in game.lua).
    pub gros_michel_extinct: bool,

    /// How many Ectoplasms have been used. Its hand-size cost escalates by 1 each time
    /// (`G.GAME.ecto_minus`, card.lua:1495).
    pub ectoplasm_uses: u32,

    /// Tags collected by skipping blinds, waiting for their trigger point.
    /// `tags` holds the ones still pending; consumed tags are removed.
    pub hands_played_this_run: u32,
    /// Discards left unused at the end of each round, summed over the run (Garbage Tag).
    pub unused_discards_this_run: u32,
    /// How many blinds have been skipped this run (Skip Tag pays $5 per skip).
    pub skips_this_run: u32,
    /// Set by Coupon Tag: everything already stocked in the next shop is free.
    pub shop_is_free: bool,
    /// Set by D6 Tag: rerolls in the next shop start at $0.
    pub shop_rerolls_free: bool,
    /// Free booster packs waiting to be opened, from Charm/Meteor/Standard/Buffoon/Ethereal
    /// Tags. Balatro queues them one behind the other rather than dropping all but one, so
    /// skipping both blinds of an ante for two pack tags really does give you two packs.
    pub pending_free_packs: Vec<PackKind>,
    /// Extra hand size granted by Juggle Tag, for the current round only.
    pub juggle_hand_size: u32,

    /// Relative weights for what a shop card slot turns into (game.lua:1901-1905).
    /// Vouchers rewrite these: Tarot Merchant/Tycoon set `tarot_rate`, Magic Trick/Illusion set
    /// `playing_card_rate`, and the Ghost Deck raises `spectral_rate`.
    pub joker_rate: f64,
    pub tarot_rate: f64,
    pub planet_rate: f64,
    pub spectral_rate: f64,
    pub playing_card_rate: f64,

    /// Multiplier on Foil/Holographic/Polychrome chances (`G.GAME.edition_rate`, base 1).
    /// Hone sets it to 2, Glow Up to 4.
    pub edition_rate: f64,

    /// Reroll price at the start of a shop, before the +$1-per-reroll escalation.
    /// Reroll Surplus and Reroll Glut each knock $2 off it.
    pub base_reroll_cost: u32,

    /// How much the reroll price has escalated this round (`reroll_cost_increase`,
    /// common_events.lua:2267). Reset when a round begins, +$1 per paid reroll.
    pub reroll_cost_increase: u32,

    /// Whether the Boss blind has already been rerolled this ante (Director's Cut allows one).
    pub boss_rerolled_this_ante: bool,

    /// How many times each Boss blind has been used this run. Balatro draws from the
    /// least-used eligible bosses so a run cycles the roster (common_events.lua:2363).
    pub bosses_used: HashMap<BossBlind, u32>,

    /// ThePillar: IDs of cards played in earlier rounds of the current Ante.
    /// Cleared when a new Ante begins. Used to debuff those cards during the Boss blind.
    pub played_card_ids_this_ante: Vec<u64>,

    /// The Fish only hides the cards drawn right after a hand is *played* (`self.prepped`,
    /// blind.lua:487), not the ones drawn after a discard.
    pub fish_prepped: bool,

    /// The single hand type The Ox punishes, fixed when the Boss round begins
    /// (`G.GAME.current_round.most_played_poker_hand`, state_events.lua:137).
    ///
    /// Pinned rather than recomputed per hand: under Balatro's rule you can play a *different*
    /// hand into a tie for most-played all round and The Ox will not notice.
    pub ox_target_hand: Option<HandType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameStateKind {
    BlindSelect,
    Round,
    Shop,
    BoosterPack,
    GameOver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlindKind {
    Small,
    Big,
    Boss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LastConsumable {
    Tarot(TarotCard),
    Planet(PlanetCard),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub ante: u32,
    pub round: u32,
    pub event_type: String,
    pub data: serde_json::Value,
}

impl GameState {
    pub fn new(deck_type: DeckType, stake: Stake, seed: Option<String>) -> Self {
        let seed = seed.unwrap_or_default();
        let rng = Rng::new(&seed);

        // Every hand starts at level 1; the three secret ones start hidden.
        let hand_levels: HashMap<HandType, HandLevelData> = HandType::ALL
            .into_iter()
            .map(|ht| (ht, HandLevelData::new(!ht.is_secret())))
            .collect();

        let mut gs = GameState {
            rng,
            deck_type,
            stake,
            seed: seed.clone(),
            ante: 1,
            round: 1,
            money: 4,
            state: GameStateKind::BlindSelect,
            vouchers: Vec::new(),
            tags: Vec::new(),
            tarot_cards_used: 0,
            planet_cards_used: 0,
            planet_types_used: std::collections::HashSet::new(),
            current_blind: BlindKind::Small,
            boss_blind: None,
            blind_tags: [TagKind::Uncommon; 2],
            blind_tag_orbital_hands: [HandType::HighCard; 2],
            score_goal: 0.0,
            skipped_blinds: Vec::new(),
            blind_defeated_this_ante: [false; 3],
            deck: Vec::new(),
            starting_deck_size: 52,
            draw_pile: Vec::new(),
            hand: Vec::new(),
            discard_pile: Vec::new(),
            jokers: Vec::new(),
            consumables: Vec::new(),
            hands_remaining: 4,
            discards_remaining: 3,
            score_accumulated: 0.0,
            selected_indices: Vec::new(),
            hand_levels,
            shop_offers: Vec::new(),
            shop_voucher: None,
            reroll_cost: 5,
            free_rerolls: 0,
            current_pack: None,
            hand_size: 8,
            max_hands: 4,
            max_discards: 3,
            joker_slots: 5,
            consumable_slots: 2,
            max_interest: 25,
            history: Vec::new(),
            next_id: 1,
            last_consumable_used: None,
            cerulean_forced_card_id: None,
            verdant_leaf_joker_sold: false,
            boss_blind_manually_disabled: false,
            last_hand_played: None,
            round_targets: RoundTargets::default(),
            gros_michel_extinct: false,
            ectoplasm_uses: 0,
            bosses_used: HashMap::new(),
            hands_played_this_run: 0,
            unused_discards_this_run: 0,
            skips_this_run: 0,
            shop_is_free: false,
            shop_rerolls_free: false,
            pending_free_packs: Vec::new(),
            juggle_hand_size: 0,
            joker_rate: 20.0,
            tarot_rate: 4.0,
            planet_rate: 4.0,
            spectral_rate: 0.0,
            playing_card_rate: 0.0,
            edition_rate: 1.0,
            base_reroll_cost: 5,
            reroll_cost_increase: 0,
            boss_rerolled_this_ante: false,
            played_card_ids_this_ante: Vec::new(),
            fish_prepped: false,
            ox_target_hand: None,
        };

        // Apply deck-type modifications
        gs.apply_deck_init();

        // Build and shuffle the deck
        gs.build_deck();

        // Pick boss blind for ante 1
        gs.boss_blind = gs.pick_boss_blind();

        // The first ante's voucher and blind tags (game.lua:2178-2180).
        gs.shop_voucher = Some(gs.random_voucher());
        gs.reroll_blind_tags();

        gs
    }

    pub(crate) fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Swap two jokers by position. Order matters for Blueprint/Brainstorm.
    /// Valid at any point in the game; returns `IndexOutOfRange` if either index is out of bounds.
    pub fn swap_jokers(&mut self, a: usize, b: usize) -> Result<(), BalatroError> {
        let len = self.jokers.len();
        if a >= len {
            return Err(BalatroError::IndexOutOfRange(a, len));
        }
        if b >= len {
            return Err(BalatroError::IndexOutOfRange(b, len));
        }
        self.jokers.swap(a, b);
        Ok(())
    }

    /// Swap two consumables (tarots, planets, spectrals) by position.
    /// Valid at any point in the game; returns `IndexOutOfRange` if either index is out of bounds.
    pub fn swap_consumables(&mut self, a: usize, b: usize) -> Result<(), BalatroError> {
        let len = self.consumables.len();
        if a >= len {
            return Err(BalatroError::IndexOutOfRange(a, len));
        }
        if b >= len {
            return Err(BalatroError::IndexOutOfRange(b, len));
        }
        self.consumables.swap(a, b);
        Ok(())
    }

    fn apply_deck_init(&mut self) {
        match self.deck_type {
            DeckType::Red => {
                self.max_discards += 1;
            }
            DeckType::Blue => {
                self.max_hands += 1;
            }
            DeckType::Yellow => {
                self.money += 10;
            }
            DeckType::Black => {
                self.max_hands = self.max_hands.saturating_sub(1);
                self.joker_slots += 1;
            }
            DeckType::Painted => {
                self.hand_size += 2;
                self.joker_slots = self.joker_slots.saturating_sub(1);
            }
            DeckType::Abandoned => {
                // No face cards in deck (handled in build_deck)
            }
            DeckType::Magic => {
                // Start with Crystal Ball voucher + 2× The Fool tarot cards
                self.vouchers.push(VoucherKind::CrystalBall);
                self.consumable_slots += 1; // Crystal Ball gives +1 consumable slot
                self.consumables.push(HeldConsumable::new(ConsumableCard::Tarot(TarotCard::TheFool)));
                self.consumables.push(HeldConsumable::new(ConsumableCard::Tarot(TarotCard::TheFool)));
            }
            DeckType::Nebula => {
                // Telescope voucher, at the cost of a consumable slot (game.lua:634)
                self.vouchers.push(VoucherKind::Telescope);
                self.consumable_slots = self.consumable_slots.saturating_sub(1);
            }
            DeckType::Ghost => {
                // Spectral cards appear in the shop, and the run starts with a Hex
                // (`spectral_rate = 2, consumables = {'c_hex'}`, game.lua:635)
                self.spectral_rate = 2.0;
                self.consumables.push(HeldConsumable::new(ConsumableCard::Spectral(SpectralCard::Hex)));
            }
            DeckType::Zodiac => {
                // Start with Tarot Merchant, Planet Merchant, Overstock vouchers
                self.vouchers.push(VoucherKind::TarotMerchant);
                self.vouchers.push(VoucherKind::PlanetMerchant);
                self.vouchers.push(VoucherKind::Overstock);
            }
            _ => {}
        }
        self.hands_remaining = self.max_hands;
        self.discards_remaining = self.max_discards;
    }

    pub fn build_deck(&mut self) {
        let mut cards = Vec::new();

        for suit in Suit::ALL {
            for rank in Rank::ALL {
                // Abandoned deck: skip face cards
                if self.deck_type == DeckType::Abandoned && rank.is_face() {
                    continue;
                }

                let effective_suit = match self.deck_type {
                    DeckType::Checkered => match suit {
                        Suit::Clubs => Suit::Spades,
                        Suit::Diamonds => Suit::Hearts,
                        s => s,
                    },
                    _ => suit,
                };

                let id = self.next_id();
                let mut card = CardInstance::new(id, rank, effective_suit);

                // Erratic deck: randomize rank and suit
                if self.deck_type == DeckType::Erratic {
                    let rank_idx = self.rng.range_u32("erratic", 0, Rank::ALL.len() as u32 - 1);
                    let suit_idx = self.rng.range_u32("erratic", 0, Suit::ALL.len() as u32 - 1);
                    card.rank = Rank::ALL[rank_idx as usize];
                    card.suit = Suit::ALL[suit_idx as usize];
                }

                cards.push(card);
            }
        }

        // Shuffle
        self.rng.shuffle("shuffle", &mut cards);
        self.deck = cards;
        self.starting_deck_size = self.deck.len();
        self.draw_pile = (0..self.deck.len()).collect();
    }

    pub fn get_blind_chip_goal(&self) -> f64 {
        let base = get_base_blind_amount_scaled(self.ante, blind_scaling_tier(self.stake));
        // Plasma Deck doubles every blind requirement (`ante_scaling = 2`, game.lua:641;
        // applied in UI_definitions.lua:1548).
        let ante_scaling = if self.deck_type == DeckType::Plasma { 2.0 } else { 1.0 };
        let mult = match self.current_blind {
            BlindKind::Small => 1.0,
            BlindKind::Big => 1.5,
            BlindKind::Boss => match self.boss_blind {
                Some(boss) if self.boss_blind_disabled() => boss.chip_multiplier_disabled(),
                Some(boss) => boss.chip_multiplier(),
                None => 2.0,
            },
        };
        (base as f64) * mult * ante_scaling
    }
}

mod blind;
mod round;
mod shop;
mod pack;
mod consumable;

impl GameState {
    // ---------------------------------------------------------------------
    // Asking about the board
    // ---------------------------------------------------------------------

    /// Whether an active copy of `kind` is on the board.
    ///
    /// "Active" is the important half: a Perishable joker that has run out still occupies its
    /// slot and still counts for Abstract Joker, but its ability is switched off.
    pub(crate) fn has_joker(&self, kind: JokerKind) -> bool {
        self.jokers.iter().any(|j| j.kind == kind && j.active)
    }

    /// How many active copies of `kind` are on the board. Showman makes duplicates reachable,
    /// and the effects that pay per copy have to count them.
    pub(crate) fn count_joker(&self, kind: JokerKind) -> usize {
        self.jokers.iter().filter(|j| j.kind == kind && j.active).count()
    }

    /// Every active joker of `kind`, by index — for the effects that mutate their own counters.
    pub(crate) fn joker_indices(&self, kind: JokerKind) -> Vec<usize> {
        self.jokers
            .iter()
            .enumerate()
            .filter(|(_, j)| j.kind == kind && j.active)
            .map(|(i, _)| i)
            .collect()
    }

    /// The multiplier Oops! All 6s puts on every *listed* probability.
    ///
    /// Each copy doubles the table again on the way in and halves it on the way out
    /// (card.lua:608, :665), so two copies quadruple and three multiply by eight.
    fn probability_mult(&self) -> f64 {
        2f64.powi(self.count_joker(JokerKind::OopsAll6s) as i32)
    }

    /// Roll one of Balatro's listed probabilities — "1 in 4 chance to…" and friends — from the
    /// `key` stream, with Oops! All 6s applied. A doubled certainty is still just certain.
    pub(crate) fn roll_chance(&mut self, key: &str, probability: f64) -> bool {
        let p = (probability * self.probability_mult()).min(1.0);
        self.rng.next_bool_prob(key, p)
    }

    /// The Boss blind in force *with its ability switched on*, if any.
    ///
    /// Three things have to line up: it is the Boss round, a Boss was drawn, and nothing has
    /// disabled it (Chicot passively, a sold Luchador by latch). Every boss effect asks this.
    pub(crate) fn active_boss(&self) -> Option<BossBlind> {
        if !matches!(self.current_blind, BlindKind::Boss) || self.boss_blind_disabled() {
            return None;
        }
        self.boss_blind
    }

    /// Whether this specific Boss blind's ability is in force right now.
    pub(crate) fn boss_effect_active(&self, boss: BossBlind) -> bool {
        self.active_boss() == Some(boss)
    }

    /// Record something that happened, stamped with the ante and round it happened in.
    pub(crate) fn log_event(&mut self, event_type: &str, data: serde_json::Value) {
        self.history.push(HistoryEvent {
            ante: self.ante,
            round: self.round,
            event_type: event_type.to_string(),
            data,
        });
    }

    /// Remove a single card from `deck` by ID and remap all index collections.
    /// Use this instead of `deck.retain(...)` to avoid stale indices in hand/draw_pile/discard_pile.
    pub(crate) fn destroy_deck_card(&mut self, card_id: u64) {
        if let Some(pos) = self.deck.iter().position(|c| c.id == card_id) {
            self.deck.remove(pos);
            // Any stored index > pos shifts down by 1; index == pos is now gone (caller must have
            // already removed it from hand/discard_pile before calling this).
            for idx in self.hand.iter_mut() {
                if *idx > pos { *idx -= 1; }
            }
            for idx in self.draw_pile.iter_mut() {
                if *idx > pos { *idx -= 1; }
            }
            for idx in self.discard_pile.iter_mut() {
                if *idx > pos { *idx -= 1; }
            }
        }
    }

    /// Remove multiple cards from `deck` by IDs, remapping indices correctly.
    pub(crate) fn destroy_deck_cards(&mut self, card_ids: &[u64]) {
        for &id in card_ids {
            self.destroy_deck_card(id);
        }
    }

    /// The lowest balance the player is allowed to reach (`G.GAME.bankrupt_at`, game.lua:1922).
    /// Normally $0; each Credit Card lowers it by $20 (card.lua:594).
    pub fn bankrupt_at(&self) -> i32 {
        -20 * self.count_joker(JokerKind::CreditCard) as i32
    }

    /// Whether a purchase of `cost` is allowed. Balatro's shop buttons test
    /// `cost > G.GAME.dollars - G.GAME.bankrupt_at` (button_callbacks.lua:56).
    pub fn can_afford(&self, cost: i32) -> bool {
        cost <= self.money - self.bankrupt_at()
    }

    /// Joker capacity, counting the extra slot each Negative joker brings with it
    /// (card.lua:687). Deriving it means the slot arrives and leaves with the joker no matter
    /// how it was acquired — bought, pulled from a Buffoon pack, or turned Negative by Ectoplasm.
    pub fn effective_joker_slots(&self) -> usize {
        let negatives = self
            .jokers
            .iter()
            .filter(|j| j.edition == Edition::Negative)
            .count();
        self.joker_slots as usize + negatives
    }

    /// Jokers that react to playing cards being added to the deck (`playing_card_joker_effects`,
    /// misc_functions.lua:1580). Every source calls it: booster packs, a bought playing card,
    /// Marble Joker, Certificate, DNA and the card-creating spectrals.
    pub(crate) fn notify_playing_cards_added(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        for j in self.jokers.iter_mut() {
            if j.kind == JokerKind::Hologram && j.active {
                j.add_counter_f64("x_mult", 0.25 * count as f64);
            }
        }
    }

    /// Jokers that react to any card being sold (`context.selling_card`, card.lua:2394).
    /// Consumables count, not just jokers.
    ///
    /// `sold_joker_id` is the joker being sold, if it is one: Balatro skips it when broadcasting
    /// (`if G.jokers.cards[i] ~= card`, button_callbacks.lua:2322), so selling a Campfire does
    /// not let it upgrade itself on the way out.
    pub(crate) fn notify_card_sold(&mut self, sold_joker_id: Option<u64>) {
        for j in self.jokers.iter_mut() {
            if Some(j.id) == sold_joker_id {
                continue;
            }
            if j.kind == JokerKind::Campfire {
                j.add_counter_f64("x_mult", 0.25);
            }
        }
    }

    /// Switch the current Boss blind's ability off for the rest of the round, undoing what it
    /// had already taken (`Blind:disable()`, blind.lua:356).
    ///
    /// Chicot does this passively and every `boss_blind_disabled()` check follows it for free.
    /// Luchador does it at the moment it is sold, part-way through a round that the blind has
    /// already shaped, so the damage has to be handed back: the discards The Water took, the
    /// hands The Needle took, the cards turned face down, Cerulean Bell's forced selection, The
    /// Manacle's hand slot, and the inflated requirement of The Wall or Violet Vessel.
    pub(crate) fn disable_boss_blind(&mut self) {
        if self.boss_blind_disabled() {
            return;
        }
        let hands_before = self.effective_max_hands();
        let discards_before = self.effective_max_discards();

        self.boss_blind_manually_disabled = true;

        // Amber Acorn's jokers are turned back over (blind.lua:358). This one is worth doing even
        // outside a round — a Luchador sold in the shop still reveals the row.
        for j in self.jokers.iter_mut() {
            j.face_down = false;
        }

        // Nothing else to undo outside a round — selling Luchador in the shop still latches the
        // blind off, but there is no hand to redraw or requirement to restate.
        if !matches!(self.state, GameStateKind::Round) {
            return;
        }

        self.hands_remaining += self.effective_max_hands().saturating_sub(hands_before);
        self.discards_remaining += self.effective_max_discards().saturating_sub(discards_before);

        for card in self.deck.iter_mut() {
            card.debuffed = false;
            card.face_down = false;
        }
        self.fish_prepped = false;

        // Cerulean Bell stops forcing its card, which can then be deselected like any other.
        self.cerulean_forced_card_id = None;

        // The Wall and Violet Vessel drop back to an ordinary Boss requirement.
        self.score_goal = self.get_blind_chip_goal();

        // The Manacle gives its hand slot back, and a card is drawn into it.
        self.draw_to_hand();
    }

    /// Draw the tags for the ante about to start. Called at run start and once the Boss falls.
    pub(crate) fn reroll_blind_tags(&mut self) {
        let ante = self.ante;
        self.draw_blind_tags_for_ante(ante);
    }

    /// Draw both blind tags for `ante`, along with the hand an Orbital Tag among them would
    /// level — Balatro settles that when the tag is drawn, not when it is redeemed
    /// (`G.GAME.orbital_choices`, tag.lua:104).
    pub(crate) fn draw_blind_tags_for_ante(&mut self, ante: u32) {
        self.blind_tags = [
            self.random_tag_for_ante(ante),
            self.random_tag_for_ante(ante),
        ];
        self.blind_tag_orbital_hands = [self.random_orbital_hand(), self.random_orbital_hand()];
    }

    /// The tag on offer for skipping the blind currently up, if it can be skipped at all.
    pub fn tag_on_offer(&self) -> Option<TagKind> {
        self.blind_tag_slot().map(|slot| self.blind_tags[slot])
    }

    /// Which of the two `blind_tags` slots the blind currently up draws from — `None` for a Boss,
    /// which cannot be skipped.
    pub(crate) fn blind_tag_slot(&self) -> Option<usize> {
        match self.current_blind {
            BlindKind::Small => Some(0),
            BlindKind::Big => Some(1),
            BlindKind::Boss => None,
        }
    }

    /// The hand type an Orbital Tag on offer would level, if that is the tag on offer.
    ///
    /// Balatro fixes this when the tag is drawn and prints it on the blind-select screen
    /// (`G.GAME.orbital_choices`, tag.lua:104), so skipping for it is an informed choice.
    pub fn orbital_hand_on_offer(&self) -> Option<HandType> {
        let slot = self.blind_tag_slot()?;
        (self.blind_tags[slot] == TagKind::Orbital).then(|| self.blind_tag_orbital_hands[slot])
    }

    /// The shop discount in force, as a percentage (`G.GAME.discount_percent`). It reaches sell
    /// values as well as prices, so it is not purely a shop concern.
    pub fn discount_percent(&self) -> f64 {
        if self.has_voucher(VoucherKind::Liquidation) {
            50.0
        } else if self.has_voucher(VoucherKind::ClearanceSale) {
            25.0
        } else {
            0.0
        }
    }

    /// Whether Pareidolia is out. `Card:is_face` answers yes to every card while it is
    /// (card.lua:964), so this gates every face-card check in the game, not just the scoring ones.
    pub(crate) fn has_pareidolia(&self) -> bool {
        self.has_joker(JokerKind::Pareidolia)
    }

    /// Whether Smeared Joker is out. It lives inside `Card:is_suit` (card.lua:4084), so it merges
    /// Hearts with Diamonds and Spades with Clubs for *every* suit check, not only flushes.
    pub(crate) fn has_smeared(&self) -> bool {
        self.has_joker(JokerKind::SmearedJoker)
    }

    /// How many consumables can be held right now: the slot count plus one for each Negative
    /// consumable, since a Negative card brings its own slot and takes it away again when spent
    /// (card.lua:687).
    pub fn effective_consumable_slots(&self) -> usize {
        let negatives = self.consumables.iter().filter(|c| c.negative).count();
        self.consumable_slots as usize + negatives
    }

    /// Whether there is room for one more consumable.
    pub fn has_room_for_consumable(&self) -> bool {
        self.consumables.len() < self.effective_consumable_slots()
    }

    /// Put a consumable into the slots. Callers gate on [`Self::has_room_for_consumable`] first.
    pub(crate) fn add_consumable(&mut self, card: ConsumableCard) {
        self.consumables.push(HeldConsumable::new(card));
    }

    /// Put a freshly created card into the deck, ready to be drawn this round.
    ///
    /// Deliberately does not notify the jokers that react to a card being added: several of the
    /// effects that call this add a handful at once, and Hologram counts the batch, not the card.
    pub(crate) fn add_card_to_draw_pile(&mut self, card: CardInstance) {
        let deck_idx = self.deck.len();
        self.deck.push(card);
        self.draw_pile.push(deck_idx);
    }

    /// Hand the player a random Tarot, if a slot is free. A great many effects do exactly this.
    ///
    /// No slot means no card is *rolled* at all, not one rolled and thrown away, so a full
    /// consumable tray leaves the tarot stream untouched.
    pub(crate) fn create_tarot(&mut self) {
        if !self.has_room_for_consumable() {
            return;
        }
        let tarot = self.random_tarot();
        self.add_consumable(ConsumableCard::Tarot(tarot));
    }

    /// As [`Self::create_tarot`], drawing a Spectral from an effect-specific pool: the jokers
    /// that conjure one each have their own shortlist.
    pub(crate) fn create_spectral_from(&mut self, key: &str, pool: &[SpectralCard]) {
        if !self.has_room_for_consumable() || pool.is_empty() {
            return;
        }
        let pick = pool[self.rng.range_usize(key, 0, pool.len() - 1)];
        self.add_consumable(ConsumableCard::Spectral(pick));
    }

    /// The ante that ends the run (`G.GAME.win_ante`).
    ///
    /// It is a flat 8 in a vanilla run. Hieroglyph and Petroglyph do *not* lower it — they lower
    /// the ante you are currently on (`ease_ante(-1)`, card.lua:1958), which is why they make a
    /// run longer and its blinds smaller rather than shorter. Only Challenges move this.
    pub fn win_ante(&self) -> u32 {
        8
    }

    /// Returns `true` if the current Boss blind's ability is disabled.
    ///
    /// Chicot disables it passively while held (`card.lua:596`), whereas Luchador only disables it
    /// when sold (`card.lua:2355`, `context.selling_self`) — that case latches
    /// `boss_blind_manually_disabled` instead.
    pub(crate) fn boss_blind_disabled(&self) -> bool {
        self.boss_blind_manually_disabled || self.has_joker(JokerKind::Chicot)
    }

    /// Notify jokers that a playing card has been destroyed, by any means (Glass shatter, a
    /// spectral, a joker, The Hanged Man, …). Call this *before* `destroy_deck_card`.
    ///
    /// - Canio gains +1 Xmult per destroyed face card (`card.lua:2673`).
    /// - Glass Joker gains +0.75 Xmult per destroyed Glass card. Balatro keys this off the
    ///   `shattered` flag, which `Card:shatter()` sets for every Glass card that is removed
    ///   (`state_events.lua:988`, `:413`), so "destroyed Glass card" is the right rule.
    pub(crate) fn notify_card_destroyed(&mut self, card: &CardInstance) {
        let is_face = card.is_face(self.has_pareidolia());
        let is_glass = card.enhancement == Enhancement::Glass;
        if !is_face && !is_glass {
            return;
        }
        for j in self.jokers.iter_mut() {
            if !j.active {
                continue;
            }
            match j.kind {
                JokerKind::Canio if is_face => j.add_counter_f64("x_mult", 1.0),
                JokerKind::GlassJoker if is_glass => j.add_counter_f64("x_mult", 0.75),
                _ => {}
            }
        }
    }
}

/// The next rank up, as Strength gives it.
///
/// An Ace wraps back round to a Two rather than sticking — `card.base.id == 14 and 2 or
/// math.min(id+1, 14)` (card.lua:1126). Strength on a hand of Aces is a downgrade, and that is
/// the intended trap.
pub(crate) fn rank_up(rank: Rank) -> Rank {
    if rank == Rank::Ace {
        return Rank::Two;
    }
    let pos = Rank::ALL.iter().position(|&r| r == rank).unwrap_or(0);
    Rank::ALL.get(pos + 1).copied().unwrap_or(Rank::Ace)
}

pub(crate) fn upgraded_voucher(base: VoucherKind) -> VoucherKind {
    match base {
        VoucherKind::Overstock => VoucherKind::OverstockPlus,
        VoucherKind::ClearanceSale => VoucherKind::Liquidation,
        VoucherKind::Hone => VoucherKind::GlowUp,
        VoucherKind::RerollSurplus => VoucherKind::RerollGlut,
        VoucherKind::CrystalBall => VoucherKind::OmenGlobe,
        VoucherKind::Telescope => VoucherKind::Observatory,
        VoucherKind::Grabber => VoucherKind::NachoTong,
        VoucherKind::Wasteful => VoucherKind::Recyclomancy,
        VoucherKind::TarotMerchant => VoucherKind::TarotTycoon,
        VoucherKind::PlanetMerchant => VoucherKind::PlanetTycoon,
        VoucherKind::SeedMoney => VoucherKind::MoneyTree,
        VoucherKind::Blank => VoucherKind::Antimatter,
        VoucherKind::MagicTrick => VoucherKind::Illusion,
        VoucherKind::Hieroglyph => VoucherKind::Petroglyph,
        VoucherKind::DirectorsCut => VoucherKind::Retcon,
        VoucherKind::PaintBrush => VoucherKind::Palette,
        // Already top-tier — return self
        other => other,
    }
}

/// Blind scaling tier, driven by stake (`game.lua:2053`, `:2056`). Green and above use a steeper
/// curve, Purple and above steeper still.
pub fn blind_scaling_tier(stake: Stake) -> u8 {
    if stake.at_least(Stake::Purple) {
        3
    } else if stake.at_least(Stake::Green) {
        2
    } else {
        1
    }
}

pub fn get_base_blind_amount(ante: u32) -> u64 {
    get_base_blind_amount_scaled(ante, 1)
}

/// `misc_functions.lua:919` — one table per scaling tier, with a shared formula past ante 8.
pub fn get_base_blind_amount_scaled(ante: u32, scaling: u8) -> u64 {
    let amounts: [u64; 8] = match scaling {
        2 => [300, 900, 2600, 8000, 20000, 36000, 60000, 100000],
        3 => [300, 1000, 3200, 9000, 25000, 60000, 110000, 200000],
        _ => [300, 800, 2000, 5000, 11000, 20000, 35000, 50000],
    };
    if ante == 0 {
        return 100;
    }
    if ante <= 8 {
        return amounts[(ante - 1) as usize];
    }
    // Scale exponentially for ante > 8, anchored on the tier's ante-8 value
    let k = 0.75_f64;
    let a = amounts[7] as f64;
    let b = 1.6_f64;
    let c = (ante - 8) as f64;
    let d = 1.0 + 0.2 * c;
    let amount = (a * (b + (k * c).powf(d)).powf(c)).floor() as u64;
    // Round to significant figures
    if amount < 10 {
        return amount;
    }
    let log = (amount as f64).log10().floor() as u32;
    let factor = 10u64.pow(log.saturating_sub(1));
    (amount / factor) * factor
}

// Error types
#[derive(Debug, Clone)]
pub enum BalatroError {
    NotInBlindSelect,
    NotInRound,
    NotInShop,
    NotInPack,
    CannotSkipBoss,
    NoCardsSelected,
    TooManySelected,
    NoHandsRemaining,
    NoDiscardsRemaining,
    NoPicksRemaining,
    IndexOutOfRange(usize, usize),
    NotEnoughMoney(u32, u32),
    JokerSlotsFull,
    ConsumableSlotsFull,
    AlreadySold,
    WrongItemType(String),
    EternalCard,
    NoVoucherAvailable,
    BossBlindEffect(String),
}

impl std::fmt::Display for BalatroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalatroError::NotInBlindSelect => write!(f, "Not in blind selection phase"),
            BalatroError::NotInRound => write!(f, "Not in round"),
            BalatroError::NotInShop => write!(f, "Not in shop"),
            BalatroError::NotInPack => write!(f, "Not opening a pack"),
            BalatroError::CannotSkipBoss => write!(f, "Cannot skip the boss blind"),
            BalatroError::NoCardsSelected => write!(f, "No cards selected"),
            BalatroError::TooManySelected => write!(f, "Too many cards selected (max 5)"),
            BalatroError::NoHandsRemaining => write!(f, "No hands remaining"),
            BalatroError::NoDiscardsRemaining => write!(f, "No discards remaining"),
            BalatroError::NoPicksRemaining => write!(f, "No picks remaining in pack"),
            BalatroError::IndexOutOfRange(i, max) => {
                write!(f, "Index {} out of range (max {})", i, max)
            }
            BalatroError::NotEnoughMoney(need, have) => {
                write!(f, "This costs ${} but you have ${}", need, have)
            }
            BalatroError::JokerSlotsFull => write!(f, "Joker slots are full"),
            BalatroError::ConsumableSlotsFull => write!(f, "Consumable slots are full"),
            BalatroError::AlreadySold => write!(f, "This item has already been sold"),
            BalatroError::WrongItemType(msg) => write!(f, "Wrong item type: {}", msg),
            BalatroError::EternalCard => write!(f, "Cannot sell an Eternal card"),
            BalatroError::NoVoucherAvailable => write!(f, "No voucher available in shop"),
            BalatroError::BossBlindEffect(msg) => write!(f, "Boss blind effect: {}", msg),
        }
    }
}

impl std::error::Error for BalatroError {}
