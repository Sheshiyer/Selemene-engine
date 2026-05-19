"""One-time interactive login to humdes.com.

Opens a real Chromium window, lets you sign in manually, then saves the
authenticated browser state (cookies + localStorage) to ./storageState.json.
Subsequent runs of capture.py replay this state without ever showing a login
screen — until the session expires, at which point you re-run this script.

Usage:
    python login.py
"""

from __future__ import annotations

import sys
from pathlib import Path

from playwright.sync_api import sync_playwright


STATE_FILE = Path(__file__).resolve().parent / "storageState.json"
LOGIN_URL = "https://www.humdes.com/en/personal/results/#/personal"


def main() -> int:
    print("Launching Chromium...")
    print(f"State will be saved to: {STATE_FILE}")
    print()
    print("INSTRUCTIONS:")
    print("  1. A browser window will open at the humdes login/results page.")
    print("  2. Log in normally (email + password).")
    print("  3. Make sure you reach the personal results page successfully.")
    print("  4. Return to THIS terminal and press Enter to save the session.")
    print("  5. The browser will close automatically.")
    print()

    with sync_playwright() as pw:
        browser = pw.chromium.launch(headless=False)
        context = browser.new_context(
            viewport={"width": 1400, "height": 900},
            locale="en-US",
        )
        page = context.new_page()
        page.goto(LOGIN_URL, wait_until="domcontentloaded")

        try:
            input("Press Enter here once you are logged in and on the results page... ")
        except (KeyboardInterrupt, EOFError):
            print("\nAborted.")
            browser.close()
            return 1

        context.storage_state(path=str(STATE_FILE))
        browser.close()

    size = STATE_FILE.stat().st_size if STATE_FILE.exists() else 0
    print(f"\nSaved authenticated session to {STATE_FILE} ({size} bytes).")
    print("Next: python capture.py")
    return 0


if __name__ == "__main__":
    sys.exit(main())
