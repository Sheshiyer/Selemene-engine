"""Authenticated HTTP client for humdes.com (Bitrix CMS).

Reuses cookies extracted by chrome_cookies.py to impersonate a logged-in
browser session. After the HAR file is captured and inspected, fill in the
ENDPOINTS dict with the real paths from har_inspector.py output.

This module is intentionally tolerant: if the endpoints are not yet known,
it exposes raw `.get_json()` / `.post_json()` helpers and a `.probe()` method
so we can iterate before locking the endpoint names down.
"""

from __future__ import annotations

import json
import re
import sys
from typing import Any
from urllib.parse import urljoin

import requests

import chrome_cookies


BASE_URL = "https://www.humdes.com"
RESULTS_PAGE = "/en/personal/results/"

DEFAULT_UA = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    "AppleWebKit/537.36 (KHTML, like Gecko) "
    "Chrome/148.0.0.0 Safari/537.36"
)

# Filled in after running har_inspector.py against ~/Downloads/humdes.har.
# Keep the structure: each entry is (METHOD, PATH-template, body-template-or-None).
ENDPOINTS: dict[str, tuple[str, str, dict | None]] = {
    # "list_readings":  ("GET",  "/local/ajax/readings/list.php", None),
    # "get_reading":    ("POST", "/local/ajax/reading/get.php",   {"id": "{reading_id}"}),
}


class AuthError(RuntimeError):
    pass


class EndpointNotConfigured(RuntimeError):
    pass


class HumdesClient:
    def __init__(self, user_agent: str = DEFAULT_UA) -> None:
        self.session = requests.Session()
        self.session.headers.update({
            "User-Agent": user_agent,
            "Accept": "application/json, text/plain, */*",
            "Accept-Language": "en-US,en;q=0.9",
            "Referer": urljoin(BASE_URL, RESULTS_PAGE),
            "Origin": BASE_URL,
            "X-Requested-With": "XMLHttpRequest",
        })
        self.sessid: str | None = None
        self._load_cookies()

    # ----- setup ----------------------------------------------------------

    def _load_cookies(self) -> None:
        raw = chrome_cookies.load_auto("humdes.com")
        bitrix = chrome_cookies.filter_bitrix(raw)
        if not bitrix:
            raise AuthError(
                "No Bitrix cookies found for humdes.com.\n"
                "Quick fix: create `cookies.env` next to this script and paste\n"
                "cookie name=value lines copied from DevTools -> Application ->\n"
                "Cookies -> https://www.humdes.com (see cookies.env.example)."
            )
        for name, value in raw.items():
            self.session.cookies.set(name, value, domain=".humdes.com")
        # Also accept www.humdes.com host_key cookies if present
        for name, value in raw.items():
            self.session.cookies.set(name, value, domain="www.humdes.com")

    def warm_up(self) -> None:
        """Hit the results page to pick up any inline sessid/CSRF token."""
        resp = self.session.get(urljoin(BASE_URL, RESULTS_PAGE), allow_redirects=True)
        resp.raise_for_status()
        # Bitrix typically exposes the session id as: BX.message({'bitrix_sessid':'...'})
        # or as <meta name="csrf-token" content="...">.
        for pattern in (
            r"bitrix_sessid['\"]\s*:\s*['\"]([a-f0-9]{16,})['\"]",
            r"['\"]sessid['\"]\s*:\s*['\"]([a-f0-9]{16,})['\"]",
            r'name="csrf-token"\s+content="([^"]+)"',
        ):
            m = re.search(pattern, resp.text, re.IGNORECASE)
            if m:
                self.sessid = m.group(1)
                self.session.headers["X-Bitrix-Csrf-Token"] = self.sessid
                break

        if "personal/auth" in resp.url or "login" in resp.url.lower():
            raise AuthError(
                f"Server redirected to login page ({resp.url}). "
                "Cookies are stale or invalid — re-login in Chrome."
            )

    # ----- low-level helpers ---------------------------------------------

    def get_json(self, path: str, **kwargs: Any) -> Any:
        url = urljoin(BASE_URL, path)
        resp = self.session.get(url, **kwargs)
        return self._parse(resp)

    def post_json(self, path: str, data: dict | None = None, json_body: dict | None = None,
                  **kwargs: Any) -> Any:
        url = urljoin(BASE_URL, path)
        payload = dict(data or {})
        if self.sessid and "sessid" not in payload and json_body is None:
            payload["sessid"] = self.sessid
        resp = self.session.post(url, data=payload or None, json=json_body, **kwargs)
        return self._parse(resp)

    @staticmethod
    def _parse(resp: requests.Response) -> Any:
        ct = resp.headers.get("Content-Type", "")
        if "json" not in ct.lower():
            preview = resp.text[:200].replace("\n", " ")
            raise AuthError(
                f"Expected JSON from {resp.url}, got {ct}. "
                f"HTTP {resp.status_code}. Body preview: {preview!r}"
            )
        resp.raise_for_status()
        return resp.json()

    # ----- domain methods (filled in after HAR inspection) ----------------

    def _endpoint(self, name: str) -> tuple[str, str, dict | None]:
        if name not in ENDPOINTS:
            raise EndpointNotConfigured(
                f"Endpoint '{name}' not yet configured. "
                f"Run `python har_inspector.py ~/Downloads/humdes.har` and add it to ENDPOINTS in humdes_client.py."
            )
        return ENDPOINTS[name]

    def list_readings(self) -> list[dict]:
        method, path, body_tpl = self._endpoint("list_readings")
        if method == "GET":
            data = self.get_json(path)
        else:
            data = self.post_json(path, data=body_tpl or {})
        if isinstance(data, dict):
            for key in ("items", "data", "result", "readings"):
                if key in data and isinstance(data[key], list):
                    return data[key]
        if isinstance(data, list):
            return data
        raise RuntimeError(f"Unexpected list_readings shape: {type(data).__name__}")

    def get_reading(self, reading_id: str | int) -> dict:
        method, path, body_tpl = self._endpoint("get_reading")
        path = path.format(reading_id=reading_id)
        body: dict | None = None
        if body_tpl:
            body = {k: str(v).format(reading_id=reading_id) for k, v in body_tpl.items()}
        if method == "GET":
            return self.get_json(path)
        return self.post_json(path, data=body)

    # ----- diagnostics ----------------------------------------------------

    def probe(self) -> dict:
        """Quick sanity check: cookies present, results page reachable, sessid found."""
        return {
            "cookies_loaded": [c.name for c in self.session.cookies if "humdes" in (c.domain or "")],
            "sessid": self.sessid,
            "endpoints_configured": list(ENDPOINTS.keys()),
        }


if __name__ == "__main__":
    client = HumdesClient()
    try:
        client.warm_up()
        print("Warm-up OK.")
    except AuthError as e:
        print(f"Auth error: {e}", file=sys.stderr)
        sys.exit(1)
    print(json.dumps(client.probe(), indent=2))
