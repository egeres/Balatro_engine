"""Take a screenshot of a single monitor and save it under ``screenshots/``.

Requires ``typer`` and ``mss``::

    pip install typer mss

Usage::

    python scripts/screenshot_monitor.py --monitor 1
    python scripts/screenshot_monitor.py --list-monitors
"""

from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path
from typing import Annotated

import mss
import mss.tools
import typer

PROJECT_ROOT = Path(__file__).resolve().parent.parent
SCREENSHOTS_DIR = PROJECT_ROOT / "screenshots"


def _utc_filename(directory: Path) -> Path:
    """Return an unused ``<UTC timestamp>.png`` path inside ``directory``.

    Colons are illegal in Windows filenames, so the ISO-ish stamp uses dashes
    for the time part. A counter is appended if several shots land in the same
    second.
    """
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H-%M-%SZ")
    path = directory / f"{stamp}.png"
    counter = 1
    while path.exists():
        path = directory / f"{stamp}-{counter}.png"
        counter += 1
    return path


def main(
    monitor: Annotated[
        int,
        typer.Option(
            "--monitor",
            "-m",
            help="Monitor to capture: 1 is the primary one, 0 is all monitors combined.",
        ),
    ] = 1,
    list_monitors: Annotated[
        bool,
        typer.Option("--list-monitors", "-l", help="List available monitors and exit."),
    ] = False,
) -> None:
    """Screenshot a single monitor into the project's ``screenshots/`` directory."""
    with mss.MSS() as sct:
        monitors = sct.monitors

        if list_monitors:
            for index, mon in enumerate(monitors):
                label = "all monitors" if index == 0 else f"monitor {index}"
                typer.echo(
                    f"{index}: {label} - {mon['width']}x{mon['height']} "
                    f"at ({mon['left']}, {mon['top']})"
                )
            return

        if not 0 <= monitor < len(monitors):
            raise typer.BadParameter(
                f"Monitor {monitor} does not exist. Valid values are 0..{len(monitors) - 1} "
                f"(run with --list-monitors to see them).",
                param_hint="--monitor",
            )

        SCREENSHOTS_DIR.mkdir(parents=True, exist_ok=True)
        path = _utc_filename(SCREENSHOTS_DIR)

        shot = sct.grab(monitors[monitor])
        mss.tools.to_png(shot.rgb, shot.size, output=str(path))

    typer.echo(f"Saved {shot.width}x{shot.height} screenshot to {path}")


if __name__ == "__main__":
    typer.run(main)
