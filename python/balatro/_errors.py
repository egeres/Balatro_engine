"""Balatro error types."""

from enum import Enum


class InvalidStateError(Exception):
    """Raised when an action is not valid in the current game state."""


class NotEnoughMoneyError(Exception):
    """Raised when a purchase cannot be afforded."""

    def __init__(self, need: int, have: int):
        super().__init__(f"This costs ${need} but you have ${have}")
        self.need = need
        self.have = have


class IndexOutOfRangeError(IndexError):
    """Raised when an index is out of range."""

    def __init__(self, index: int, max_len: int):
        super().__init__(f"Index {index} out of range (max {max_len})")
        self.index = index
        self.max_len = max_len


def _parse_engine_error(exc: Exception) -> Exception:
    """Convert a raw engine exception string into a typed Python error."""
    msg = str(exc)
    if msg.startswith("NotEnoughMoneyError:"):
        # "NotEnoughMoneyError: need $X but have $Y"
        try:
            parts = msg.split("$")
            need = int(parts[1].split()[0])
            have = int(parts[2].split()[0])
            return NotEnoughMoneyError(need, have)
        except Exception:
            return NotEnoughMoneyError(0, 0)
    if msg.startswith("IndexOutOfRangeError:"):
        # "IndexOutOfRangeError: index X out of range (max Y)"
        try:
            words = msg.split()
            idx = int(words[2])
            mx = int(words[-1].rstrip(")"))
            return IndexOutOfRangeError(idx, mx)
        except Exception:
            return IndexOutOfRangeError(0, 0)
    if msg.startswith("InvalidStateError:"):
        return InvalidStateError(msg[len("InvalidStateError:"):].strip())
    return exc
