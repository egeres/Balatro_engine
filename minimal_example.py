"""
Minimal demo script for the balatro-environment package.

It:
- Starts a new game
- Enters the first blind
- Tries one discard
- Then tries to play a hand (aiming for a flush if possible)
- Reports whether the blind was beaten or the run ended.
"""

from __future__ import annotations

from balatro import (
    DeckType,
    DiscardSelected,
    Game,
    PlaySelectedHand,
    SelectBlind,
    SelectCard,
    Stake,
)


def choose_flush_like_indices(round_info: dict) -> list[int]:
    """Pick up to 5 cards, preferring a single suit (flush-ish)."""
    hand = round_info["hand"]
    by_suit: dict[str, list[int]] = {}
    for card in hand:
        suit = card["suit"]
        by_suit.setdefault(suit, []).append(card["hand_index"])

    # Prefer any suit with at least 5 cards
    for suit, indices in by_suit.items():
        if len(indices) >= 5:
            return indices[:5]

    # Otherwise just take the first 5 cards
    return [c["hand_index"] for c in hand[:5]]


def main() -> None:
    game = Game(deck=DeckType.RED, stake=Stake.WHITE)

    print("Initial state:", game.state)
    print("Initial run info:", game.run_info())

    # ------------------------------------------------------------------
    # Step 1: select the first blind
    # ------------------------------------------------------------------
    actions = game.available_actions()
    print("Available actions in initial state:", actions)

    action_names = {a.get("action") for a in actions}
    if "SelectBlind" in action_names:
        print("\nSelecting first blind…")
        game.apply(SelectBlind())
    else:
        print("No SelectBlind action available; something is off.")
        return

    print("State after selecting blind:", game.state)
    print("Run info after blind selection:", game.run_info())

    # ------------------------------------------------------------------
    # Step 2: if we are in a round, try a discard and then play a hand
    # ------------------------------------------------------------------
    if game.state != "Round":
        print("Not in a Round state after selecting blind; stopping.")
        return

    rinfo = game.round_info()
    print("\nRound info at start of round:")
    print("  Hands remaining:", rinfo["hands_remaining"])
    print("  Discards remaining:", rinfo["discards_remaining"])
    print("  Current hand:", rinfo["hand"])

    # Try one discard: discard the first card if allowed
    actions = game.available_actions()
    print("\nActions at start of round:", actions[:10])
    first_index = rinfo["hand"][0]["hand_index"]
    print("Selecting first card (for potential discard):", first_index)
    game.apply(SelectCard(index=first_index))

    actions = game.available_actions()
    names = {a.get("action") for a in actions}
    if "DiscardSelected" in names:
        print("Discarding selected card…")
        game.apply(DiscardSelected())
        rinfo = game.round_info()
        print("Hand after discard:", rinfo["hand"])
    else:
        print("DiscardSelected not available; skipping discard.")

    # Now try to play a flush-like hand
    rinfo = game.round_info()
    indices = choose_flush_like_indices(rinfo)
    print("\nTrying to play cards at indices (hand positions):", indices)
    for idx in indices:
        game.apply(SelectCard(index=idx))

    actions = game.available_actions()
    names = {a.get("action") for a in actions}
    if "PlaySelectedHand" in names:
        print("Playing selected hand…")
        result = game.apply(PlaySelectedHand())
        print("Hand result:", result)
    else:
        print("PlaySelectedHand not available; ending demo.")
        return

    # ------------------------------------------------------------------
    # Step 3: check outcome of the blind
    # ------------------------------------------------------------------
    run = game.run_info()
    print("\nRun info after playing hand:", run)
    state = run.get("state")
    if state == "GameOver":
        print("Run ended: you lost the blind (GameOver).")
    else:
        # We don't inspect exact scoring logic here; if the run continues
        # after playing a hand, treat that as 'beat the blind' for this demo.
        print("Run is still alive (state =", state, ") – you likely beat the blind!")


if __name__ == "__main__":
    main()
