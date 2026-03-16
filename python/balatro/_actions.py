"""Action types for the Balatro game engine."""

from dataclasses import dataclass, field
from typing import List


@dataclass(frozen=True)
class SelectBlind:
    """Select the current blind to begin the round."""


@dataclass(frozen=True)
class SkipBlind:
    """Skip the current (non-boss) blind."""


@dataclass(frozen=True)
class SelectCard:
    """Select a card in hand by hand-relative index."""

    index: int


@dataclass(frozen=True)
class DeselectCard:
    """Deselect a card in hand by hand-relative index."""

    index: int


@dataclass(frozen=True)
class DeselectAll:
    """Deselect all currently selected cards."""


@dataclass(frozen=True)
class SelectCardByRank:
    """Select all cards in hand with the given rank (0=Two … 12=Ace)."""

    rank: int


@dataclass(frozen=True)
class PlaySelectedHand:
    """Play the currently selected cards as a hand."""


@dataclass(frozen=True)
class DiscardSelected:
    """Discard the currently selected cards."""


@dataclass(frozen=True)
class BuyJoker:
    """Buy a joker from the shop by shop index."""

    index: int


@dataclass(frozen=True)
class SellJoker:
    """Sell an owned joker by joker-slot index."""

    index: int


@dataclass(frozen=True)
class BuyConsumable:
    """Buy a consumable card from the shop by shop index."""

    index: int


@dataclass(frozen=True)
class BuyPack:
    """Buy a booster pack from the shop by shop index."""

    index: int


@dataclass(frozen=True)
class BuyVoucher:
    """Buy the voucher available in the shop (there is at most one per roll)."""


@dataclass(frozen=True)
class RerollShop:
    """Reroll the current shop offerings."""


@dataclass(frozen=True)
class LeaveShop:
    """Leave the shop and advance to the next blind."""


@dataclass(frozen=True)
class TakePackCard:
    """Take a card from the open booster pack by pack index."""

    index: int


@dataclass(frozen=True)
class SkipPack:
    """Skip (close) the current booster pack without taking more cards."""


@dataclass(frozen=True)
class UseConsumable:
    """Use a consumable card from your collection.

    ``targets`` is a list of hand-relative indices for cards to apply the
    consumable to (e.g. tarot / spectral cards that modify cards in hand).
    """

    index: int
    targets: List[int] = field(default_factory=list)


@dataclass(frozen=True)
class SellConsumable:
    """Sell a consumable card from your collection by consumable index."""

    index: int


# Type alias for the union of all action types
Action = (
    SelectBlind
    | SkipBlind
    | SelectCard
    | DeselectCard
    | DeselectAll
    | SelectCardByRank
    | PlaySelectedHand
    | DiscardSelected
    | BuyJoker
    | SellJoker
    | BuyConsumable
    | BuyPack
    | BuyVoucher
    | RerollShop
    | LeaveShop
    | TakePackCard
    | SkipPack
    | UseConsumable
    | SellConsumable
)
