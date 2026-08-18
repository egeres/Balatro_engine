mod card;
mod game;
mod hand_eval;
mod rng;
mod scoring;
mod types;
#[cfg(test)]
mod tests;

use game::{BalatroError, GameState};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde_json::Value;
use types::*;

// ============================================================
// Error mapping
// ============================================================

fn balatro_err_to_py(err: BalatroError) -> PyErr {
    match &err {
        BalatroError::NotEnoughMoney(need, have) => {
            PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                "NotEnoughMoneyError: need ${need} but have ${have}"
            ))
        }
        BalatroError::IndexOutOfRange(idx, max) => PyErr::new::<pyo3::exceptions::PyException, _>(
            format!("IndexOutOfRangeError: index {idx} out of range (max {max})"),
        ),
        BalatroError::NotInRound
        | BalatroError::NotInBlindSelect
        | BalatroError::NotInShop
        | BalatroError::NotInPack
        | BalatroError::CannotSkipBoss
        | BalatroError::NoCardsSelected
        | BalatroError::TooManySelected
        | BalatroError::NoHandsRemaining
        | BalatroError::NoDiscardsRemaining
        | BalatroError::NoPicksRemaining
        | BalatroError::JokerSlotsFull
        | BalatroError::ConsumableSlotsFull
        | BalatroError::AlreadySold
        | BalatroError::EternalCard
        | BalatroError::NoVoucherAvailable => {
            PyErr::new::<pyo3::exceptions::PyException, _>(format!("InvalidStateError: {err}"))
        }
        BalatroError::WrongItemType(msg) => {
            PyValueError::new_err(format!("WrongItemType: {msg}"))
        }
        BalatroError::BossBlindEffect(msg) => PyRuntimeError::new_err(msg.clone()),
    }
}

// ============================================================
// Helper: convert serde_json::Value to a Python object
// ============================================================

fn json_to_py(py: Python<'_>, val: &Value) -> PyResult<PyObject> {
    match val {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok(b.into_py(py)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_py(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_py(py))
            } else {
                Ok(py.None())
            }
        }
        Value::String(s) => Ok(s.clone().into_py(py)),
        Value::Array(arr) => {
            let list = pyo3::types::PyList::empty_bound(py);
            for item in arr {
                list.append(json_to_py(py, item)?)?;
            }
            Ok(list.into())
        }
        Value::Object(map) => {
            let dict = PyDict::new_bound(py);
            for (k, v) in map {
                dict.set_item(k, json_to_py(py, v)?)?;
            }
            Ok(dict.into())
        }
    }
}

// ============================================================
// Shared JSON shapes
//
// These are the pieces that show up in more than one payload, so that a `run_info` tag and a
// `full_state` tag cannot drift into two different shapes.
// ============================================================

fn tag_json(tag: &TagKind) -> Value {
    serde_json::json!({
        "kind": format!("{:?}", tag),
        "name": tag.display_name(),
        "trigger": format!("{:?}", tag.trigger()),
    })
}

fn tags_json(tags: &[TagKind]) -> Value {
    Value::Array(tags.iter().map(tag_json).collect())
}

fn playing_card_json(c: &card::CardInstance) -> Value {
    serde_json::json!({
        "type": "PlayingCard",
        "rank": format!("{:?}", c.rank),
        "suit": format!("{:?}", c.suit),
        "enhancement": format!("{:?}", c.enhancement),
        "edition": format!("{:?}", c.edition),
        "seal": format!("{:?}", c.seal),
    })
}

fn consumable_json(c: &card::ConsumableCard) -> Value {
    serde_json::json!({
        "type": c.card_type(),
        "name": c.display_name(),
    })
}

fn gamestate_to_json(gs: &GameState) -> Value {
    serde_json::json!({
        "state": format!("{:?}", gs.state),
        "deck_type": format!("{:?}", gs.deck_type),
        "stake": format!("{:?}", gs.stake),
        "seed": gs.seed,
        "ante": gs.ante,
        "round": gs.round,
        "money": gs.money,
        "score_accumulated": gs.score_accumulated,
        "score_goal": gs.score_goal,
        "hands_remaining": gs.hands_remaining,
        "discards_remaining": gs.discards_remaining,
        "hand_size": gs.hand_size,
        "max_hands": gs.max_hands,
        "max_discards": gs.max_discards,
        "joker_slots": gs.joker_slots,
        "consumable_slots": gs.consumable_slots,
        "current_blind": format!("{:?}", gs.current_blind),
        "boss_blind": gs.boss_blind.map(|b| format!("{:?}", b)),
        "vouchers": gs.vouchers.iter().map(|v| format!("{:?}", v)).collect::<Vec<_>>(),
        "tags": tags_json(&gs.tags),
        "pending_free_pack": gs.pending_free_pack.map(|p| format!("{:?}", p)),
    })
}

fn round_info_json(gs: &GameState) -> Value {
    let hand_cards: Vec<Value> = gs
        .hand
        .iter()
        .enumerate()
        .map(|(hand_idx, &deck_idx)| {
            let c = &gs.deck[deck_idx];
            // The Fish, The Wheel, The House and The Mark deal cards face down. Their whole
            // effect is hidden information, so a face-down card reports no identity — it can
            // still be selected and played by hand_index.
            let hidden = |v: Value| if c.face_down { Value::Null } else { v };
            serde_json::json!({
                "hand_index": hand_idx,
                "id": c.id,
                "face_down": c.face_down,
                "rank": hidden(format!("{:?}", c.rank).into()),
                "suit": hidden(format!("{:?}", c.suit).into()),
                "enhancement": hidden(format!("{:?}", c.enhancement).into()),
                "edition": hidden(format!("{:?}", c.edition).into()),
                "seal": hidden(format!("{:?}", c.seal).into()),
                "debuffed": c.debuffed,
                "extra_chips": hidden(c.extra_chips.into()),
                "selected": gs.selected_indices.contains(&hand_idx),
            })
        })
        .collect();

    let jokers: Vec<Value> = gs
        .jokers
        .iter()
        .enumerate()
        .map(|(i, j)| {
            serde_json::json!({
                "index": i,
                "id": j.id,
                "kind": format!("{:?}", j.kind),
                "edition": format!("{:?}", j.edition),
                "eternal": j.eternal,
                "perishable": j.perishable,
                "perishable_rounds_left": j.perishable_rounds_left,
                "rental": j.rental,
                "active": j.active,
                "counters": j.counters,
            })
        })
        .collect();

    let consumables: Vec<Value> = gs
        .consumables
        .iter()
        .enumerate()
        .map(|(i, c)| {
            serde_json::json!({
                "index": i,
                "type": c.card_type(),
                "name": c.display_name(),
                "base_cost": c.base_cost(),
            })
        })
        .collect();


    let hand_levels: Vec<Value> = gs
        .hand_levels
        .iter()
        .map(|(ht, data)| {
            serde_json::json!({
                "hand_type": ht.display_name(),
                "level": data.level,
                "chips": data.chips(*ht),
                "mult": data.mult(*ht),
                "played": data.played,
                "played_this_round": data.played_this_round,
                "visible": data.visible,
            })
        })
        .collect();

    serde_json::json!({
        "hand": hand_cards,
        "jokers": jokers,
        "consumables": consumables,
        "hand_levels": hand_levels,
        "hands_remaining": gs.hands_remaining,
        "discards_remaining": gs.discards_remaining,
        "score_accumulated": gs.score_accumulated,
        "score_goal": gs.score_goal,
        "deck_remaining": gs.draw_pile.len(),
        "discard_pile_count": gs.discard_pile.len(),
        "selected_indices": gs.selected_indices.clone(),
        "effective_hand_size": gs.effective_hand_size(),
    })
}

fn shop_info_json(gs: &GameState) -> Value {
    let offers: Vec<Value> = gs
        .shop_offers
        .iter()
        .enumerate()
        .map(|(i, offer)| {
            let item_json = match &offer.kind {
                card::ShopItem::Joker(j) => serde_json::json!({
                    "type": "Joker",
                    "kind": format!("{:?}", j.kind),
                    "edition": format!("{:?}", j.edition),
                    "eternal": j.eternal,
                    "perishable": j.perishable,
                    "rental": j.rental,
                }),
                card::ShopItem::Consumable(c) => consumable_json(c),
                card::ShopItem::PlayingCard(c) => playing_card_json(c),
                card::ShopItem::Pack(p) => serde_json::json!({
                    "type": "Pack",
                    "kind": format!("{:?}", p),
                }),
                card::ShopItem::Voucher(v) => serde_json::json!({
                    "type": "Voucher",
                    "kind": format!("{:?}", v),
                }),
            };
            serde_json::json!({
                "index": i,
                "price": offer.price,
                "sold": offer.sold,
                "item": item_json,
            })
        })
        .collect();

    serde_json::json!({
        "offers": offers,
        "voucher": gs.shop_voucher.map(|v| format!("{:?}", v)),
        "reroll_cost": gs.reroll_cost,
        "free_rerolls": gs.free_rerolls,
        "money": gs.money,
    })
}

fn pack_info_json(gs: &GameState) -> Value {
    match &gs.current_pack {
        None => serde_json::json!(null),
        Some(pack) => {
            let cards: Vec<Value> = pack
                .cards
                .iter()
                .enumerate()
                .map(|(i, pc)| {
                    let card_json = match pc {
                        card::PackCard::PlayingCard(c) => playing_card_json(c),
                        card::PackCard::Consumable(c) => consumable_json(c),
                        card::PackCard::Joker(j) => serde_json::json!({
                            "type": "Joker",
                            "kind": format!("{:?}", j.kind),
                            "edition": format!("{:?}", j.edition),
                            "eternal": j.eternal,
                            "perishable": j.perishable,
                        }),
                    };
                    serde_json::json!({ "index": i, "card": card_json })
                })
                .collect();

            serde_json::json!({
                "kind": format!("{:?}", pack.kind),
                "cards": cards,
                "picks_remaining": pack.picks_remaining,
            })
        }
    }
}

fn run_info_json(gs: &GameState) -> Value {
    serde_json::json!({
        "ante": gs.ante,
        "round": gs.round,
        "money": gs.money,
        "deck_type": format!("{:?}", gs.deck_type),
        "stake": format!("{:?}", gs.stake),
        "seed": gs.seed,
        "state": format!("{:?}", gs.state),
        "jokers": gs.jokers.iter().map(|j| format!("{:?}", j.kind)).collect::<Vec<_>>(),
        "vouchers": gs.vouchers.iter().map(|v| format!("{:?}", v)).collect::<Vec<_>>(),
        "tags": tags_json(&gs.tags),
        "pending_free_pack": gs.pending_free_pack.map(|p| format!("{:?}", p)),
        // The tag you would get for skipping the blind currently up. Balatro shows this before
        // you commit, so it is observable state rather than a surprise roll.
        "tag_on_offer": gs.tag_on_offer().as_ref().map(tag_json),
        "blind_tags": {
            "small": gs.blind_tags[0].display_name(),
            "big": gs.blind_tags[1].display_name(),
        },
        "shop_voucher": gs.shop_voucher.map(|v| format!("{:?}", v)),
        "history_len": gs.history.len(),
        "boss_blind": gs.boss_blind.map(|b| b.display_name()),
    })
}

fn history_json(gs: &GameState) -> Value {
    let events: Vec<Value> = gs
        .history
        .iter()
        .map(|e| {
            serde_json::json!({
                "ante": e.ante,
                "round": e.round,
                "event_type": e.event_type,
                "data": e.data,
            })
        })
        .collect();
    Value::Array(events)
}

/// The actions that would actually succeed right now.
///
/// This is a legal-action mask, so an action only appears if applying it returns `Ok`: prices
/// are the live amounts (`offer_price`), affordability goes through `can_afford` so Credit Card
/// debt counts, and anything needing a free slot is gated on there being one.
fn available_actions_json(gs: &GameState) -> Value {
    use game::GameStateKind;
    let mut actions: Vec<Value> = Vec::new();

    let consumable_actions = |actions: &mut Vec<Value>, sellable: bool| {
        for (i, c) in gs.consumables.iter().enumerate() {
            actions.push(serde_json::json!({
                "action": "UseConsumable",
                "index": i,
                "name": c.display_name(),
                "type": c.card_type(),
                "negative": c.negative,
            }));
            if sellable {
                actions.push(serde_json::json!({
                    "action": "SellConsumable",
                    "index": i,
                    "name": c.display_name(),
                    "type": c.card_type(),
                }));
            }
        }
    };

    let sell_joker_actions = |actions: &mut Vec<Value>| {
        for (i, j) in gs.jokers.iter().enumerate() {
            if !j.eternal {
                actions.push(serde_json::json!({
                    "action": "SellJoker",
                    "index": i,
                    "kind": format!("{:?}", j.kind),
                    "sell_value": j.sell_value(gs.discount_percent()),
                }));
            }
        }
    };

    match gs.state {
        GameStateKind::BlindSelect => {
            actions.push(serde_json::json!({ "action": "SelectBlind" }));
            if !matches!(gs.current_blind, game::BlindKind::Boss) {
                actions.push(serde_json::json!({
                    "action": "SkipBlind",
                    "tag": gs.tag_on_offer().map(|t| t.display_name()),
                }));
            }
            if gs.pending_free_pack.is_some() {
                actions.push(serde_json::json!({ "action": "OpenPendingFreePack" }));
            }
            // Director's Cut / Retcon put a Boss reroll on this screen.
            if gs.can_reroll_boss_blind() {
                actions.push(serde_json::json!({ "action": "RerollBossBlind", "cost": 10 }));
            }
            consumable_actions(&mut actions, true);
            sell_joker_actions(&mut actions);
        }
        GameStateKind::Round => {
            for i in 0..gs.hand.len() {
                if gs.selected_indices.contains(&i) {
                    actions.push(serde_json::json!({ "action": "DeselectCard", "index": i }));
                } else if gs.selected_indices.len() < 5 {
                    actions.push(serde_json::json!({ "action": "SelectCard", "index": i }));
                }
            }
            if !gs.selected_indices.is_empty() {
                actions.push(serde_json::json!({ "action": "DeselectAll" }));
                if gs.hands_remaining > 0 {
                    actions.push(serde_json::json!({ "action": "PlaySelectedHand" }));
                }
                if gs.discards_remaining > 0 {
                    actions.push(serde_json::json!({ "action": "DiscardSelected" }));
                }
            }
            consumable_actions(&mut actions, true);
            sell_joker_actions(&mut actions);
        }
        GameStateKind::Shop => {
            for (i, offer) in gs.shop_offers.iter().enumerate() {
                if offer.sold {
                    continue;
                }
                let price = gs.offer_price(i).unwrap_or(0);
                if !gs.can_afford(price as i32) {
                    continue;
                }
                let (action, room) = match &offer.kind {
                    card::ShopItem::Joker(_) => {
                        ("BuyJoker", gs.jokers.len() < gs.effective_joker_slots())
                    }
                    card::ShopItem::Consumable(_) => ("BuyConsumable", gs.has_room_for_consumable()),
                    card::ShopItem::Pack(_) => ("BuyPack", true),
                    card::ShopItem::PlayingCard(_) => ("BuyPlayingCard", true),
                    card::ShopItem::Voucher(_) => ("BuyVoucher", true),
                };
                if !room {
                    continue;
                }
                actions.push(serde_json::json!({
                    "action": action,
                    "index": i,
                    "price": price,
                }));
            }
            // The ante's voucher lives in its own slot, not among the card offers.
            if gs.shop_voucher.is_some() {
                let price = gs.voucher_price();
                if gs.can_afford(price as i32) {
                    actions.push(serde_json::json!({
                        "action": "BuyVoucher",
                        "voucher": gs.shop_voucher.map(|v| format!("{:?}", v)),
                        "price": price,
                    }));
                }
            }
            if gs.can_afford(gs.reroll_cost as i32) {
                actions.push(serde_json::json!({
                    "action": "RerollShop",
                    "cost": gs.reroll_cost,
                }));
            }
            actions.push(serde_json::json!({ "action": "LeaveShop" }));
            consumable_actions(&mut actions, true);
            sell_joker_actions(&mut actions);
        }
        GameStateKind::BoosterPack => {
            if let Some(pack) = &gs.current_pack {
                for (i, c) in pack.cards.iter().enumerate() {
                    // Taking a card needs somewhere to put it.
                    let room = match c {
                        card::PackCard::PlayingCard(_) => true,
                        card::PackCard::Joker(_) => gs.jokers.len() < gs.effective_joker_slots(),
                        card::PackCard::Consumable(_) => gs.has_room_for_consumable(),
                    };
                    if room {
                        actions.push(serde_json::json!({ "action": "TakePackCard", "index": i }));
                    }
                }
                actions.push(serde_json::json!({ "action": "SkipPack" }));
            }
            consumable_actions(&mut actions, false);
        }
        GameStateKind::GameOver => {
            // No actions available
        }
    }

    Value::Array(actions)
}

// ============================================================
// BalatroEngine PyO3 class
// ============================================================

#[pyclass(name = "BalatroEngine")]
struct BalatroEngine {
    gs: GameState,
}

#[pymethods]
impl BalatroEngine {
    #[new]
    #[pyo3(signature = (deck_type, stake, seed=None))]
    fn new(deck_type: u8, stake: u8, seed: Option<String>) -> PyResult<Self> {
        let deck = DeckType::from_u8(deck_type)
            .ok_or_else(|| PyValueError::new_err(format!("Invalid deck_type: {deck_type}")))?;
        let stk = Stake::from_u8(stake)
            .ok_or_else(|| PyValueError::new_err(format!("Invalid stake: {stake}")))?;
        Ok(Self {
            gs: GameState::new(deck, stk, seed),
        })
    }

    // ---- State queries ----

    fn state_str(&self) -> String {
        format!("{:?}", self.gs.state)
    }

    fn run_info(&self, py: Python<'_>) -> PyResult<PyObject> {
        json_to_py(py, &run_info_json(&self.gs))
    }

    fn round_info(&self, py: Python<'_>) -> PyResult<PyObject> {
        json_to_py(py, &round_info_json(&self.gs))
    }

    fn shop_info(&self, py: Python<'_>) -> PyResult<PyObject> {
        json_to_py(py, &shop_info_json(&self.gs))
    }

    fn pack_info(&self, py: Python<'_>) -> PyResult<PyObject> {
        json_to_py(py, &pack_info_json(&self.gs))
    }

    fn full_state(&self, py: Python<'_>) -> PyResult<PyObject> {
        json_to_py(py, &gamestate_to_json(&self.gs))
    }

    fn history(&self, py: Python<'_>) -> PyResult<PyObject> {
        json_to_py(py, &history_json(&self.gs))
    }

    fn available_actions(&self, py: Python<'_>) -> PyResult<PyObject> {
        json_to_py(py, &available_actions_json(&self.gs))
    }

    // ---- Actions ----

    fn select_blind(&mut self) -> PyResult<()> {
        self.gs.select_blind().map_err(balatro_err_to_py)
    }

    fn skip_blind(&mut self) -> PyResult<()> {
        self.gs.skip_blind().map_err(balatro_err_to_py)
    }

    fn select_card(&mut self, index: usize) -> PyResult<()> {
        self.gs.select_card(index).map_err(balatro_err_to_py)
    }

    fn deselect_card(&mut self, index: usize) -> PyResult<()> {
        self.gs.deselect_card(index).map_err(balatro_err_to_py)
    }

    fn deselect_all(&mut self) -> PyResult<()> {
        self.gs.deselect_all().map_err(balatro_err_to_py)
    }

    fn select_cards_by_rank(&mut self, rank_u8: u8) -> PyResult<()> {
        let rank = Rank::from_u8(rank_u8)
            .ok_or_else(|| PyValueError::new_err(format!("Invalid rank: {rank_u8}")))?;
        self.gs.select_cards_by_rank(rank).map_err(balatro_err_to_py)
    }

    fn play_hand(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        let result = self.gs.play_hand().map_err(balatro_err_to_py)?;
        let v = serde_json::json!({
            "hand_type": result.hand_name,
            "scoring_card_indices": result.scoring_card_indices,
            "base_chips": result.base_chips,
            "base_mult": result.base_mult,
            "final_chips": result.final_chips,
            "final_mult": result.final_mult,
            "final_score": result.final_score,
            "dollars_earned": result.dollars_earned,
            "events": result.events.iter().map(|e| serde_json::json!({
                "source": e.source,
                "kind": format!("{:?}", e.kind),
                "value": e.value,
            })).collect::<Vec<_>>(),
        });
        json_to_py(py, &v)
    }

    fn discard_hand(&mut self) -> PyResult<()> {
        self.gs.discard_hand().map_err(balatro_err_to_py)
    }

    fn buy_joker(&mut self, index: usize) -> PyResult<()> {
        self.gs.buy_joker(index).map_err(balatro_err_to_py)
    }

    fn sell_joker(&mut self, index: usize) -> PyResult<()> {
        self.gs.sell_joker(index).map_err(balatro_err_to_py)
    }

    fn swap_jokers(&mut self, a: usize, b: usize) -> PyResult<()> {
        self.gs.swap_jokers(a, b).map_err(balatro_err_to_py)
    }

    fn swap_consumables(&mut self, a: usize, b: usize) -> PyResult<()> {
        self.gs.swap_consumables(a, b).map_err(balatro_err_to_py)
    }

    fn buy_consumable(&mut self, index: usize) -> PyResult<()> {
        self.gs.buy_consumable(index).map_err(balatro_err_to_py)
    }

    fn buy_playing_card(&mut self, index: usize) -> PyResult<()> {
        self.gs.buy_playing_card(index).map_err(balatro_err_to_py)
    }

    fn open_pending_free_pack(&mut self) -> PyResult<()> {
        self.gs.open_pending_free_pack().map_err(balatro_err_to_py)
    }

    fn reroll_boss_blind(&mut self) -> PyResult<()> {
        self.gs.reroll_boss_blind().map_err(balatro_err_to_py)
    }

    fn buy_pack(&mut self, index: usize) -> PyResult<()> {
        self.gs.buy_pack(index).map_err(balatro_err_to_py)
    }

    fn buy_voucher(&mut self) -> PyResult<()> {
        self.gs.buy_voucher().map_err(balatro_err_to_py)
    }

    fn reroll_shop(&mut self) -> PyResult<()> {
        self.gs.reroll_shop().map_err(balatro_err_to_py)
    }

    fn leave_shop(&mut self) -> PyResult<()> {
        self.gs.leave_shop().map_err(balatro_err_to_py)
    }

    fn take_pack_card(&mut self, index: usize) -> PyResult<()> {
        self.gs.take_pack_card(index).map_err(balatro_err_to_py)
    }

    fn skip_pack(&mut self) -> PyResult<()> {
        self.gs.skip_pack().map_err(balatro_err_to_py)
    }

    fn use_consumable(&mut self, index: usize, targets: Vec<usize>) -> PyResult<()> {
        self.gs
            .use_consumable(index, targets)
            .map_err(balatro_err_to_py)
    }

    fn sell_consumable(&mut self, index: usize) -> PyResult<()> {
        self.gs.sell_consumable(index).map_err(balatro_err_to_py)
    }
}

// ============================================================
// Module
// ============================================================

#[pymodule]
fn _engine(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<BalatroEngine>()?;

    // The numeric constants Python passes back to `BalatroEngine::new`. Both are derived from
    // the same tables `DeckType::from_u8` / `Stake::from_u8` read, so a new deck or stake gets
    // its constant automatically and the numbering cannot drift out of sync.
    for (value, deck) in DeckType::ALL.iter().enumerate() {
        m.add(
            pyo3::types::PyString::new_bound(m.py(), &format!("DECK_{deck:?}").to_uppercase()),
            value as u8,
        )?;
    }
    for (value, stake) in Stake::ALL.iter().enumerate() {
        m.add(
            pyo3::types::PyString::new_bound(m.py(), &format!("STAKE_{stake:?}").to_uppercase()),
            value as u8,
        )?;
    }

    Ok(())
}
