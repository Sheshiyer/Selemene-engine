# Selemene contract authority v1

This directory is the language-neutral authority for Selemene engine wire contracts. The manifest names the six canonical schemas and binds each golden fixture to its schema. Runtime-specific Rust, OpenAPI, and TypeScript types are compatibility adapters and must prove parity against these files.

Validate locally with `python3 scripts/validate_contracts.py`. Contract changes require updated schemas, fixtures, cross-language parity tests, and the full repository gate. This v1 slice is additive: it does not migrate engine algorithms, providers, routing, authentication, persistence, or live services.
