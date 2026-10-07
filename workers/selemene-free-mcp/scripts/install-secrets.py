#!/usr/bin/env python3
"""Install the sponsor credential without command-argument or stdout exposure."""
import json
import os
from pathlib import Path
import re
import secrets
import shlex
import shutil
import subprocess
import sys

WORKER = "selemene-free-mcp"
PROFILE = "9d9d"
ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path.home() / ".claude" / ".env"

def run(arguments, value=None):
    binary = shutil.which("wrangler")
    if not binary:
        raise RuntimeError("Wrangler is unavailable.")
    command = [binary, *arguments, "--config", str(ROOT / "wrangler.jsonc"),
               "--profile", PROFILE, "--name", WORKER]
    return subprocess.run(command, input=value, text=True, capture_output=True,
                          cwd=ROOT, env={**os.environ, "CI": "1"}, timeout=90)

def main():
    listed = run(["secret", "list"])
    if listed.returncode:
        raise RuntimeError("Scoped Worker secret inventory failed. Deploy the new Worker first.")
    existing = json.loads(listed.stdout)
    names = {item["name"] for item in existing}
    key = None
    for line in SOURCE.read_text().splitlines():
        match = re.match(r"^\s*(?:export\s+)?SELEMENE_API_KEY\s*=\s*(.*?)\s*$", line)
        if match:
            values = shlex.split(match.group(1), comments=True)
            key = values[0] if len(values) == 1 else None
    if not key or len(key) < 16 or len(key) > 256 or re.search(r"\s", key):
        raise RuntimeError("The Claude Selemene credential is missing or invalid.")
    to_install = {"SELEMENE_API_KEY": key}
    if "RATE_LIMIT_SALT" not in names:
        to_install["RATE_LIMIT_SALT"] = secrets.token_urlsafe(48)
    installed = []
    for name, value in to_install.items():
        result = run(["secret", "put", name], value + "\n")
        if result.returncode:
            raise RuntimeError("Scoped secret installation failed for " + name + ".")
        installed.append(name)
    print(json.dumps({"worker": WORKER, "profile": PROFILE, "installed_secret_names": installed,
                      "existing_salt_preserved": "RATE_LIMIT_SALT" in names}))
if __name__ == "__main__":
    try:
        main()
    except Exception:
        # Neither a subprocess diagnostic nor a credential-containing exception is exposed.
        print("Scoped secret installation failed; no secret values were displayed.", file=sys.stderr)
        sys.exit(1)
