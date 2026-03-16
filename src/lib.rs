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
        "tags": gs.tags.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>(),
    })
}

fn round_info_json(gs: &GameState) -> Value {
    let hand_cards: Vec<Value> = gs
        .hand
        .iter()
        .enumerate()
        .map(|(hand_idx, &deck_idx)| {
            let c = &gs.deck[deck_idx];
            serde_json::json!({
                "hand_index": hand_idx,
                "id": c.id,
                "rank": format!("{:?}", c.rank),
                "suit": format!("{:?}", c.suit),
                "enhancement": format!("{:?}", c.enhancement),
                "edition": format!("{:?}", c.edition),
                "seal": format!("{:?}", c.seal),
                "debuffed": c.debuffed,
                "extra_chips": c.extra_chips,
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
                card::ShopItem::Consumable(c) => serde_json::json!({
                    "type": c.card_type(),
                    "name": c.display_name(),
                }),
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
                        card::PackCard::PlayingCard(c) => serde_json::json!({
                            "type": "PlayingCard",
                            "rank": format!("{:?}", c.rank),
                            "suit": format!("{:?}", c.suit),
                            "enhancement": format!("{:?}", c.enhancement),
                            "edition": format!("{:?}", c.edition),
                            "seal": format!("{:?}", c.seal),
                        }),
                        card::PackCard::Joker(j) => serde_json::json!({
                            "type": "Joker",
                            "kind": format!("{:?}", j.kind),
                            "edition": format!("{:?}", j.edition),
                            "eternal": j.eternal,
                            "perishable": j.perishable,
                        }),
                        card::PackCard::Consumable(c) => serde_json::json!({
                            "type": c.card_type(),
                            "name": c.display_name(),
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
        "tags": gs.tags.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>(),
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

fn available_actions_json(gs: &GameState) -> Value {
    use game::GameStateKind;
    let mut actions: Vec<Value> = Vec::new();

    match gs.state {
        GameStateKind::BlindSelect => {
            actions.push(serde_json::json!({ "action": "SelectBlind" }));
            if !matches!(gs.current_blind, game::BlindKind::Boss) {
                actions.push(serde_json::json!({ "action": "SkipBlind" }));
            }
            // Consumables can be used in blind select
            for (i, c) in gs.consumables.iter().enumerate() {
                actions.push(serde_json::json!({
                    "action": "UseConsumable",
                    "index": i,
                    "name": c.display_name(),
                    "type": c.card_type(),
                }));
                actions.push(serde_json::json!({
                    "action": "SellConsumable",
                    "index": i,
                    "name": c.display_name(),
                }));
            }
        }
        GameStateKind::Round => {
            // Card selection
            for i in 0..gs.hand.len() {
                if gs.selected_indices.contains(&i) {
                    actions.push(serde_json::json!({ "action": "DeselectCard", "index": i }));
                } else {
                    actions.push(serde_json::json!({ "action": "SelectCard", "index": i }));
                }
            }
            if !gs.selected_indices.is_empty() {
                if gs.hands_remaining > 0 && gs.selected_indices.len() <= 5 {
                    actions.push(serde_json::json!({ "action": "PlaySelectedHand" }));
                }
                if gs.discards_remaining > 0 {
                    actions.push(serde_json::json!({ "action": "DiscardSelected" }));
                }
            }
            // Consumables
            for (i, c) in gs.consumables.iter().enumerate() {
                actions.push(serde_json::json!({
                    "action": "UseConsumable",
                    "index": i,
                    "name": c.display_name(),
                    "type": c.card_type(),
                }));
            }
            // Sell jokers
            for (i, j) in gs.jokers.iter().enumerate() {
                if !j.eternal {
                    actions.push(serde_json::json!({
                        "action": "SellJoker",
                        "index": i,
                        "kind": format!("{:?}", j.kind),
                        "sell_value": j.sell_value(),
                    }));
                }
            }
        }
        GameStateKind::Shop => {
            for (i, offer) in gs.shop_offers.iter().enumerate() {
                if !offer.sold {
                    actions.push(serde_json::json!({
                        "action": match &offer.kind {
                            card::ShopItem::Joker(_) => "BuyJoker",
                            card::ShopItem::Pack(_) => "BuyPack",
                            card::ShopItem::Consumable(_) => "BuyConsumable",
                            card::ShopItem::Voucher(_) => "BuyVoucher",
                        },
                        "price": offer.price,
                    }));
                }
            }
            if gs.free_rerolls > 0 || gs.money >= gs.reroll_cost as i32 {
                actions.push(serde_json::json!({
                    "action": "RerollShop",
                    "cost": if gs.free_rerolls > 0 { 0 } else { gs.reroll_cost },
                }));
            }
            actions.push(serde_json::json!({ "action": "LeaveShop" }));
            for (i, j) in gs.jokers.iter().enumerate() {
                if !j.eternal {
                    actions.push(serde_json::json!({
                        "action": "SellJoker",
                        "index": i,
                        "kind": format!("{:?}", j.kind),
                        "sell_value": j.sell_value(),
                    }));
                }
            }
            for (i, c) in gs.consumables.iter().enumerate() {
                actions.push(serde_json::json!({
                    "action": "UseConsumable",
                    "index": i,
                    "name": c.display_name(),
                }));
                actions.push(serde_json::json!({
                    "action": "SellConsumable",
                    "index": i,
                    "name": c.display_name(),
                }));
            }
        }
        GameStateKind::BoosterPack => {
            if let Some(pack) = &gs.current_pack {
                for (i, _) in pack.cards.iter().enumerate() {
                    actions.push(serde_json::json!({
                        "action": "TakePackCard",
                        "index": i,
                    }));
                }
                actions.push(serde_json::json!({ "action": "SkipPack" }));
            }
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
        self.gs.deselect_all();
        Ok(())
    }

    fn select_cards_by_rank(&mut self, rank_u8: u8) -> PyResult<()> {
        let rank = Rank::from_u8(rank_u8)
            .ok_or_else(|| PyValueError::new_err(format!("Invalid rank: {rank_u8}")))?;
        self.gs.select_cards_by_rank(rank);
        Ok(())
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

    fn buy_consumable(&mut self, index: usize) -> PyResult<()> {
        self.gs.buy_consumable(index).map_err(balatro_err_to_py)
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

    // Expose numeric constants for enum mapping
    // DeckType
    m.add("DECK_RED", 0u8)?;
    m.add("DECK_BLUE", 1u8)?;
    m.add("DECK_YELLOW", 2u8)?;
    m.add("DECK_GREEN", 3u8)?;
    m.add("DECK_BLACK", 4u8)?;
    m.add("DECK_MAGIC", 5u8)?;
    m.add("DECK_NEBULA", 6u8)?;
    m.add("DECK_GHOST", 7u8)?;
    m.add("DECK_ABANDONED", 8u8)?;
    m.add("DECK_CHECKERED", 9u8)?;
    m.add("DECK_ZODIAC", 10u8)?;
    m.add("DECK_PAINTED", 11u8)?;
    m.add("DECK_ANAGLYPH", 12u8)?;
    m.add("DECK_PLASMA", 13u8)?;
    m.add("DECK_ERRATIC", 14u8)?;

    // Stake
    m.add("STAKE_WHITE", 0u8)?;
    m.add("STAKE_RED", 1u8)?;
    m.add("STAKE_GREEN", 2u8)?;
    m.add("STAKE_BLACK", 3u8)?;
    m.add("STAKE_BLUE", 4u8)?;
    m.add("STAKE_PURPLE", 5u8)?;
    m.add("STAKE_ORANGE", 6u8)?;
    m.add("STAKE_GOLD", 7u8)?;

    Ok(())
}
