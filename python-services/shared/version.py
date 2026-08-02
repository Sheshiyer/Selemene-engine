"""Single source of truth for the python-services version.

The version lives in ``pyproject.toml``. It used to be copied by hand into the
Pydantic health model, both service entrypoints, both health routes and two
tests -- six literals that a bump had to hit all at once or leave the services
reporting a version they were not.

Resolution order:

1. Installed package metadata, which is what ``pyproject.toml`` produced.
2. ``pyproject.toml`` itself, so a source checkout that was never ``pip
   install``-ed still reports the truth.
3. ``"0.0.0+unknown"`` -- a clearly wrong value, chosen so a broken lookup is
   visible rather than silently plausible.
"""

from __future__ import annotations

import tomllib
from importlib.metadata import PackageNotFoundError, version as _dist_version
from pathlib import Path

DISTRIBUTION_NAME = "selemene-python-services"

#: Sentinel for "the version could not be resolved". Deliberately not a
#: plausible release number.
UNKNOWN_VERSION = "0.0.0+unknown"


def _from_pyproject() -> str | None:
    """Read the version straight out of pyproject.toml, for source checkouts."""
    pyproject = Path(__file__).resolve().parent.parent / "pyproject.toml"
    try:
        with pyproject.open("rb") as fh:
            return tomllib.load(fh).get("project", {}).get("version")
    except (OSError, tomllib.TOMLDecodeError):
        return None


def resolve_version() -> str:
    """Return the services version, preferring installed metadata."""
    try:
        return _dist_version(DISTRIBUTION_NAME)
    except PackageNotFoundError:
        pass
    return _from_pyproject() or UNKNOWN_VERSION


#: The resolved version. Import this rather than writing a literal.
SERVICE_VERSION = resolve_version()
