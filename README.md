balatro-environment
====================

**balatro-environment** is a Python/Rust simulation engine for the game Balatro. It exposes a fast Rust core as a Python package so you can programmatically run games, test strategies, and build tooling around Balatro.

## Installation

### Prerequisites

- **Rust** and **Cargo** installed (recommended via `rustup`)
- **Python** ≥ 3.12 (your `313` environment is using Python 3.13)
- `maturin` available in the environment that will build/install the package

If you are using Conda and already have an environment called `313`, you can install `maturin` there with:

```bash
conda run -n 313 pip install maturin
```

### Install into an existing environment

From the project root (where `pyproject.toml` lives), run:

```bash
conda run -n 313 maturin develop --release
```

This will:

- Build the Rust extension in release mode
- Install the `balatro-environment` package into your Conda environment `313`

After this, you can import `balatro` from Python in that environment.

If you want to install into whatever environment is currently active instead of `313`, activate it and run:

```bash
maturin develop --release
```

## Minimal Python example

Once the package is installed in your Conda env (e.g. `313`), you can run a minimal example like this:

```python
from balatro import Game, DeckType, Stake, SelectBlind, PlaySelectedHand

# Create a new game with a specific deck and stake
game = Game.new(deck_type=DeckType.RED, stake=Stake.ONE)

# Choose the first available blind
state = game.state
first_blind = state.available_blinds[0]
game.step(SelectBlind(id=first_blind.id))

# Play a single hand
game.step(PlaySelectedHand())

# Inspect the resulting state
state = game.state
print("Current round:", state.round)
print("Money:", state.money)
print("Score this hand:", state.last_hand_score)
```

Save this as, for example, `minimal_example.py` and run it in the `313` environment:

```bash
conda run -n 313 python minimal_example.py
```

## Development

- The Rust crate and core game logic live under `src/`.
- The Python package is under `python/balatro/`.
- The Python package is configured via `pyproject.toml` and built with `maturin`.

To run tests (if added) or to rebuild after making changes, simply re-run:

```bash
conda run -n 313 maturin develop --release
```

