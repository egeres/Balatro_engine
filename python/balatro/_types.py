"""Python enums mirroring the Rust type definitions."""

from enum import IntEnum


class DeckType(IntEnum):
    RED = 0
    BLUE = 1
    YELLOW = 2
    GREEN = 3
    BLACK = 4
    MAGIC = 5
    NEBULA = 6
    GHOST = 7
    ABANDONED = 8
    CHECKERED = 9
    ZODIAC = 10
    PAINTED = 11
    ANAGLYPH = 12
    PLASMA = 13
    ERRATIC = 14


class Stake(IntEnum):
    WHITE = 0
    RED = 1
    GREEN = 2
    BLACK = 3
    BLUE = 4
    PURPLE = 5
    ORANGE = 6
    GOLD = 7


class State(str):
    """Game state string constants (matches Rust Debug output)."""

    BLIND_SELECT = "BlindSelect"
    ROUND = "Round"
    SHOP = "Shop"
    BOOSTER_PACK = "BoosterPack"
    GAME_OVER = "GameOver"


class Rank(IntEnum):
    TWO = 0
    THREE = 1
    FOUR = 2
    FIVE = 3
    SIX = 4
    SEVEN = 5
    EIGHT = 6
    NINE = 7
    TEN = 8
    JACK = 9
    QUEEN = 10
    KING = 11
    ACE = 12


class Suit(IntEnum):
    SPADES = 0
    HEARTS = 1
    CLUBS = 2
    DIAMONDS = 3
