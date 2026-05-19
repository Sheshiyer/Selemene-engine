"""Extract cookies for a given domain from Chrome's encrypted cookie store on macOS.

Reads `~/Library/Application Support/Google/Chrome/<Profile>/Cookies` (SQLite)
and decrypts encrypted values using the AES key derived from the
"Chrome Safe Storage" entry in the macOS Keychain.

Chrome must be CLOSED (or at least not holding a write-lock on the DB) during
extraction. The script copies the DB to a temp file before reading to minimise
lock collisions.
"""

from __future__ import annotations

import os
import shutil
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path

try:
    from Crypto.Cipher import AES
    from Crypto.Protocol.KDF import PBKDF2
except ImportError:
    print("Missing dependency: pip install pycryptodome", file=sys.stderr)
    raise


CHROME_PROFILES_ROOT = Path.home() / "Library/Application Support/Google/Chrome"
SALT = b"saltysalt"
IV = b" " * 16
ITERATIONS = 1003
KEY_LENGTH = 16


# Known Chromium-based browser Keychain "Safe Storage" entry names on macOS.
# Service name is the reliable match key; account is browser-name only.
SAFE_STORAGE_SERVICES = [
    "Chrome Safe Storage",
    "Chromium Safe Storage",
    "Brave Safe Storage",
    "Microsoft Edge Safe Storage",
    "Arc Safe Storage",
    "Opera Safe Storage",
    "Vivaldi Safe Storage",
]


def _security(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["security", *args], capture_output=True, text=True)


def list_safe_storage_entries() -> list[str]:
    """Return the names of every '* Safe Storage' service present in the Keychain."""
    found: list[str] = []
    for svc in SAFE_STORAGE_SERVICES:
        result = _security("find-generic-password", "-s", svc)
        if result.returncode == 0:
            found.append(svc)
    return found


def _get_keychain_password(service: str = "Chrome Safe Storage") -> bytes:
    # 1) Match by service name (the correct lookup for "Chrome Safe Storage").
    result = _security("find-generic-password", "-w", "-s", service)
    if result.returncode == 0 and result.stdout.strip():
        return result.stdout.strip().encode("utf-8")

    # 2) Fall back to account-name match (older pycookiecheat convention).
    short_name = service.replace(" Safe Storage", "")
    result = _security("find-generic-password", "-w", "-a", short_name)
    if result.returncode == 0 and result.stdout.strip():
        return result.stdout.strip().encode("utf-8")

    # 3) Diagnose: list which * Safe Storage entries DO exist so the user
    #    knows whether Chrome is even tracked there.
    available = list_safe_storage_entries()
    hint = (
        f"Available 'Safe Storage' entries in your Keychain: {available}"
        if available else
        "No '* Safe Storage' entries found in the default Keychain at all.\n"
        "  This usually means Chrome has not yet stored an encrypted cookie\n"
        "  on this machine. Try: open Chrome, log in to humdes.com (so it\n"
        "  writes new session cookies), quit Chrome, then re-run."
    )
    raise RuntimeError(
        f"Could not read '{service}' from Keychain.\n"
        f"  {hint}\n"
        f"  Diagnostic: python chrome_cookies.py --diagnose"
    )


def _derive_key(password: bytes) -> bytes:
    return PBKDF2(password, SALT, dkLen=KEY_LENGTH, count=ITERATIONS)


def _decrypt(encrypted_value: bytes, key: bytes) -> str:
    if not encrypted_value:
        return ""
    if encrypted_value[:3] in (b"v10", b"v11"):
        encrypted_value = encrypted_value[3:]
    cipher = AES.new(key, AES.MODE_CBC, IV)
    decrypted = cipher.decrypt(encrypted_value)
    pad_len = decrypted[-1]
    if 1 <= pad_len <= 16:
        decrypted = decrypted[:-pad_len]
    return decrypted.decode("utf-8", errors="replace")


def _find_cookie_dbs() -> list[Path]:
    if not CHROME_PROFILES_ROOT.exists():
        raise RuntimeError(f"Chrome data dir not found: {CHROME_PROFILES_ROOT}")
    candidates: list[Path] = []
    for profile in CHROME_PROFILES_ROOT.iterdir():
        if not profile.is_dir():
            continue
        if profile.name in {"System Profile", "Crashpad", "GrShaderCache"}:
            continue
        db = profile / "Cookies"
        if not db.exists():
            db = profile / "Network" / "Cookies"
        if db.exists():
            candidates.append(db)
    return candidates


def _copy_to_temp(db_path: Path) -> Path:
    tmp = Path(tempfile.mkdtemp(prefix="humdes_cookies_")) / "Cookies"
    shutil.copy2(db_path, tmp)
    return tmp


def load(
    domain_suffix: str = "humdes.com",
    profile: str | None = None,
    keychain_service: str = "Chrome Safe Storage",
) -> dict[str, str]:
    """Return {cookie_name: value} for all cookies whose host ends in domain_suffix.

    If multiple Chrome profiles have matching cookies, the most recently
    updated one wins per cookie name. Use `keychain_service` to switch
    browsers (e.g. "Brave Safe Storage").
    """
    password = _get_keychain_password(keychain_service)
    key = _derive_key(password)

    dbs = _find_cookie_dbs()
    if profile:
        dbs = [d for d in dbs if d.parent.name == profile or d.parent.parent.name == profile]
    if not dbs:
        raise RuntimeError("No Chrome cookie DB found.")

    merged: dict[str, tuple[int, str]] = {}

    for db_path in dbs:
        try:
            tmp = _copy_to_temp(db_path)
        except PermissionError as e:
            print(f"[warn] Cannot copy {db_path}: {e}", file=sys.stderr)
            continue

        try:
            conn = sqlite3.connect(f"file:{tmp}?mode=ro", uri=True)
            try:
                rows = conn.execute(
                    "SELECT host_key, name, value, encrypted_value, last_access_utc "
                    "FROM cookies WHERE host_key LIKE ?",
                    (f"%{domain_suffix}",),
                ).fetchall()
            finally:
                conn.close()
        except sqlite3.DatabaseError as e:
            print(f"[warn] Cannot read {db_path} ({e}). Is Chrome running?", file=sys.stderr)
            continue
        finally:
            shutil.rmtree(tmp.parent, ignore_errors=True)

        for host_key, name, value, encrypted_value, last_access in rows:
            if value:
                decoded = value
            else:
                try:
                    decoded = _decrypt(encrypted_value, key)
                except Exception as e:  # noqa: BLE001 - log + skip
                    print(f"[warn] Failed to decrypt {name}@{host_key}: {e}", file=sys.stderr)
                    continue
            prev = merged.get(name)
            if prev is None or last_access > prev[0]:
                merged[name] = (last_access, decoded)

    return {name: val for name, (_, val) in merged.items()}


def filter_bitrix(cookies: dict[str, str]) -> dict[str, str]:
    """Keep only the cookies needed for Bitrix authentication."""
    wanted_prefixes = ("BITRIX_SM_",)
    wanted_exact = {"PHPSESSID"}
    return {
        k: v
        for k, v in cookies.items()
        if k.startswith(wanted_prefixes) or k in wanted_exact
    }


def load_from_file(path: Path) -> dict[str, str]:
    """Load cookies from a simple KEY=VALUE file (one per line).

    Lines starting with '#' or blank lines are ignored. Values can be wrapped
    in single or double quotes. Use this when Keychain decryption fails or
    cookies live only in Chrome's memory (e.g. auth cookies that haven't
    been flushed to disk yet).

    Example file contents:
        # Paste from DevTools -> Application -> Cookies -> https://www.humdes.com
        BITRIX_SM_UIDH=1gataW2WS3s7Np7AvWU5x2B9Ja3iv1Y1
        BITRIX_SM_UIDL=sheshnarayan.iyer%40gmail.com
        BITRIX_SM_UIDD=j1cvdn6hpehsr6pa2qc9bsskepvsw67q
        BITRIX_SM_SALE_UID=9a8b97619356363a4921e87bb859b38a
        PHPSESSID=<copy from devtools>
    """
    cookies: dict[str, str] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue
        name, _, value = line.partition("=")
        name = name.strip()
        value = value.strip()
        if (value.startswith('"') and value.endswith('"')) or (
            value.startswith("'") and value.endswith("'")
        ):
            value = value[1:-1]
        if name:
            cookies[name] = value
    return cookies


def load_auto(domain_suffix: str = "humdes.com") -> dict[str, str]:
    """Try a manual cookies file first, then fall back to Chrome auto-extract.

    Looks for `cookies.env` next to this script. If that file exists and has
    any entries, it wins. Otherwise it falls back to decrypting Chrome's DB.
    """
    manual = Path(__file__).resolve().parent / "cookies.env"
    if manual.exists():
        loaded = load_from_file(manual)
        if loaded:
            return loaded
    return load(domain_suffix)


def _diagnose(verbose: bool = False) -> int:
    print("=== Keychain diagnostic ===")
    available = list_safe_storage_entries()
    if available:
        print("'* Safe Storage' entries found:")
        for svc in available:
            print(f"  - {svc}")
    else:
        print("  (none found in default Keychain)")

    print("\n=== Chrome cookie DB locations + humdes.com cookies ===")
    try:
        dbs = _find_cookie_dbs()
    except RuntimeError as e:
        print(f"  {e}")
        return 1
    if not dbs:
        print("  (no cookie DBs found)")
        return 1
    for db in dbs:
        try:
            tmp = _copy_to_temp(db)
            conn = sqlite3.connect(f"file:{tmp}?mode=ro", uri=True)
            rows = conn.execute(
                "SELECT host_key, name, value, encrypted_value FROM cookies "
                "WHERE host_key LIKE ?",
                ("%humdes.com",),
            ).fetchall()
            conn.close()
            shutil.rmtree(tmp.parent, ignore_errors=True)

            profile_name = db.parent.name if db.parent.name not in ("Network",) else db.parent.parent.name
            print(f"\n  PROFILE: {profile_name}  ({db})")
            print(f"  -> {len(rows)} humdes.com cookies")
            if verbose:
                for host, name, value, enc in rows:
                    prefix = bytes(enc[:3]).decode("ascii", errors="replace") if enc else ""
                    enc_len = len(enc) if enc else 0
                    plain = value or "(none)"
                    print(f"     [{host:25s}] {name:35s}  plain={plain!r:30s}"
                          f"  enc_prefix={prefix!r}  enc_len={enc_len}")
        except Exception as e:  # noqa: BLE001
            print(f"  {db}  → ERROR: {e}")
    return 0


if __name__ == "__main__":
    if "--diagnose" in sys.argv:
        sys.exit(_diagnose(verbose="--verbose" in sys.argv or "-v" in sys.argv))

    # Allow `python chrome_cookies.py "Brave Safe Storage"` to switch browser.
    service = next((a for a in sys.argv[1:] if a.endswith("Safe Storage")), "Chrome Safe Storage")

    cookies = load("humdes.com", keychain_service=service)
    bitrix = filter_bitrix(cookies)
    print(f"Found {len(cookies)} cookies for humdes.com (via {service!r})")
    print(f"Of those, {len(bitrix)} are Bitrix/PHP session cookies:")
    for name in sorted(bitrix):
        val = bitrix[name]
        preview = val if len(val) < 40 else f"{val[:30]}...({len(val)} chars)"
        print(f"  {name:30s} = {preview}")
    if not bitrix:
        print("\nNo Bitrix cookies found. Are you logged in to humdes.com in Chrome?")
        sys.exit(1)
