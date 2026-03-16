"""High-level Python Game wrapper around the Rust BalatroEngine."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from ._actions import (
    Action,
    BuyConsumable,
    BuyJoker,
    BuyPack,
    BuyVoucher,
    DeselectAll,
    DeselectCard,
    DiscardSelected,
    LeaveShop,
    PlaySelectedHand,
    RerollShop,
    SelectBlind,
    SelectCard,
    SelectCardByRank,
    SellConsumable,
    SellJoker,
    SkipBlind,
    SkipPack,
    TakePackCard,
    UseConsumable,
)
from ._errors import _parse_engine_error
from ._types import DeckType, Stake, State

try:
    from . import _engine
except ImportError as exc:
    raise ImportError(
        "balatro._engine not found. Did you build the Rust extension with "
        "'maturin develop'?"
    ) from exc


class Game:
    """A single Balatro run.

    Parameters
    ----------
    deck:
        The deck type to use (``DeckType`` enum value, default RED).
    stake:
        The stake level (``Stake`` enum value, default WHITE).
    seed:
        Optional string seed for deterministic RNG.
    """

    def __init__(
        self,
        deck: DeckType = DeckType.RED,
        stake: Stake = Stake.WHITE,
        seed: Optional[str] = None,
    ) -> None:
        self._engine = _engine.BalatroEngine(int(deck), int(stake), seed)

    # ------------------------------------------------------------------
    # State
    # ------------------------------------------------------------------

    @property
    def state(self) -> str:
        """Current game state string (e.g. ``'BlindSelect'``, ``'Round'``, …)."""
        return self._engine.state_str()

    # ------------------------------------------------------------------
    # Info snapshots
    # ------------------------------------------------------------------

    def run_info(self) -> Dict[str, Any]:
        """Return a snapshot of run-level info (ante, money, jokers, …)."""
        return self._engine.run_info()

    def round_info(self) -> Dict[str, Any]:
        """Return a snapshot of round-level info (hand, score, …).

        Only meaningful when ``state == 'Round'``.
        """
        return self._engine.round_info()

    def shop_info(self) -> Dict[str, Any]:
        """Return a snapshot of the current shop.

        Only meaningful when ``state == 'Shop'``.
        """
        return self._engine.shop_info()

    def pack_info(self) -> Optional[Dict[str, Any]]:
        """Return a snapshot of the open booster pack, or ``None``.

        Only meaningful when ``state == 'BoosterPack'``.
        """
        return self._engine.pack_info()

    def full_state(self) -> Dict[str, Any]:
        """Return a compact snapshot of the full game state."""
        return self._engine.full_state()

    def history(self) -> List[Dict[str, Any]]:
        """Return the full event history for this run."""
        return self._engine.history()

    def available_actions(self) -> List[Dict[str, Any]]:
        """Return a list of action descriptors valid in the current state.

        Each descriptor is a dict with at least an ``"action"`` key naming
        the action class, plus any relevant parameters (``index``, ``price``,
        etc.).
        """
        return self._engine.available_actions()

    # ------------------------------------------------------------------
    # Apply
    # ------------------------------------------------------------------

    def apply(self, action: Action) -> Any:
        """Apply an action to the game state.

        Returns the action result (e.g. a ``ScoreResult`` dict for
        ``PlaySelectedHand``), or ``None`` for actions with no result.

        Raises
        ------
        InvalidStateError
            If the action is not valid in the current state.
        NotEnoughMoneyError
            If a purchase cannot be afforded.
        IndexOutOfRangeError
            If an index is out of bounds.
        """
        try:
            return self._dispatch(action)
        except Exception as exc:
            typed = _parse_engine_error(exc)
            if typed is not exc:
                raise typed from exc
            raise

    def _dispatch(self, action: Action) -> Any:
        e = self._engine

        if isinstance(action, SelectBlind):
            return e.select_blind()
        if isinstance(action, SkipBlind):
            return e.skip_blind()
        if isinstance(action, SelectCard):
            return e.select_card(action.index)
        if isinstance(action, DeselectCard):
            return e.deselect_card(action.index)
        if isinstance(action, DeselectAll):
            return e.deselect_all()
        if isinstance(action, SelectCardByRank):
            return e.select_cards_by_rank(action.rank)
        if isinstance(action, PlaySelectedHand):
            return e.play_hand()
        if isinstance(action, DiscardSelected):
            return e.discard_hand()
        if isinstance(action, BuyJoker):
            return e.buy_joker(action.index)
        if isinstance(action, SellJoker):
            return e.sell_joker(action.index)
        if isinstance(action, BuyConsumable):
            return e.buy_consumable(action.index)
        if isinstance(action, BuyPack):
            return e.buy_pack(action.index)
        if isinstance(action, BuyVoucher):
            return e.buy_voucher()
        if isinstance(action, RerollShop):
            return e.reroll_shop()
        if isinstance(action, LeaveShop):
            return e.leave_shop()
        if isinstance(action, TakePackCard):
            return e.take_pack_card(action.index)
        if isinstance(action, SkipPack):
            return e.skip_pack()
        if isinstance(action, UseConsumable):
            return e.use_consumable(action.index, list(action.targets))
        if isinstance(action, SellConsumable):
            return e.sell_consumable(action.index)

        raise TypeError(f"Unknown action type: {type(action)!r}")

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    def close(self) -> None:
        """Release any resources (currently a no-op, provided for API compat)."""
        pass

    def __enter__(self) -> "Game":
        return self

    def __exit__(self, *_: Any) -> None:
        self.close()

    def __repr__(self) -> str:
        info = self.run_info()
        return (
            f"<Game state={self.state!r} ante={info.get('ante')} "
            f"round={info.get('round')} money={info.get('money')}>"
        )
