# noesis-tui

Interactive Ratatui terminal client for the Selemene Engine API.

The TUI uses the first-party `noesis-sdk` for authenticated engine and
workflow calls. It can run against a local server or the configured Noesis
API, and supports profile setup, engine/workflow selection, report rendering,
history, and export.

## Install

The public registry package is published only after `noesis-core` and
`noesis-sdk` have matching registry receipts. Until that gate is green, run it
from a source checkout:

```bash
cargo run -p noesis-tui --bin noesis-tui
```

When installed from crates.io:

```bash
cargo install noesis-tui
noesis-tui
```

## Billing modes

Billing is controlled by the API release, not by the TUI. `free` is the safe
default when Dodo Payments is unavailable; `disabled` is a hard ceiling. The
checkout and portal routes remain part of the API contract but return
`503 BILLING_DISABLED` unless a future release explicitly enables a verified
provider.

## License

MIT
