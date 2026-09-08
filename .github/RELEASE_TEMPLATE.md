# Release vX.Y.Z

## 🎯 Overview
<!-- Brief summary of this release -->

## ✨ New Features
<!-- List new features and capabilities -->

### New Engines
<!-- If any new consciousness engines were added -->

### Engine Enhancements
<!-- Improvements to existing engines -->

### API Changes
<!-- New or modified API endpoints -->

### Integration Features
<!-- New integrations or bridge capabilities -->

## 🐛 Bug Fixes
<!-- List of bugs fixed -->

## ⚡ Performance Improvements
<!-- Any performance enhancements -->

## 📚 Documentation
<!-- Documentation updates -->

## 🔧 Infrastructure
<!-- DevOps, CI/CD, deployment improvements -->

## 🧭 External Release Gates

- [ ] Canonical CI Gate is green on the exact release commit.
- [ ] Rust SDK/TUI package dry-runs and registry receipts are attached.
- [ ] Railway source, schema, effective billing mode, health, and image digest are attested.
- [ ] Cloudflare account/source bindings are verified with the scoped account.
- [ ] Vercel protection and same-origin admin proxy checks are verified.
- [ ] Rollback target, schema rollback path, and operator are recorded.
- [ ] Dodo remains disabled/free unless current credentials and durable one-use receipts are proven.
- [ ] Tag, publication, deployment, promotion, and merge are separately approved.

## ⚠️ Breaking Changes
<!-- List any breaking changes and migration instructions -->

## 📦 Dependencies
<!-- Notable dependency updates -->

Package publication must follow dependency order: `noesis-core`, then
`noesis-sdk`, then `noesis-tui`. A local `cargo package` result is not a
registry receipt.

## 🧪 Testing
<!-- Test coverage improvements -->

## 📈 Metrics
<!-- Performance metrics, if available -->
- Response times:
- Test coverage:
- Engine count:

## 🙏 Contributors
<!-- Thank contributors -->

## 📥 Installation

### Docker
```bash
docker pull ghcr.io/sheshiyer/selemene-engine:vX.Y.Z
```

### From Source
```bash
git clone https://github.com/Sheshiyer/Selemene-engine.git
cd Selemene-engine
git checkout vX.Y.Z
cargo build --release
```

## 🔗 Links
- [Full Changelog](CHANGELOG.md)
- [Documentation](docs/)
- [API Reference](docs/api-docs.md)

## ⏭️ What's Next
<!-- Preview of upcoming features -->
