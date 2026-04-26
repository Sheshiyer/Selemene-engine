# Versioning Strategy

## Semantic Versioning

Selemene Engine follows [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH
```

### Version Components

#### MAJOR (Breaking Changes)
Increment when making incompatible API changes:
- Breaking changes to REST API endpoints
- Removal of deprecated features
- Fundamental architecture changes
- Database schema breaking changes

**Examples:**
- `v2.0.0 → v3.0.0`: Restructure API response format
- `v3.0.0 → v4.0.0`: Remove deprecated v1 endpoints

#### MINOR (New Features)
Increment when adding functionality in a backward-compatible manner:
- New consciousness engines
- New API endpoints
- New features to existing engines
- Integration with new external APIs

**Examples:**
- `v2.0.0 → v2.1.0`: Add Integration Layer + Vedic Clock enhancements
- `v2.1.0 → v2.2.0`: Add Numerology + Biorhythm engines
- `v2.2.0 → v2.3.0`: Add GraphQL API layer

#### PATCH (Bug Fixes)
Increment when making backward-compatible bug fixes:
- Bug fixes
- Performance improvements
- Documentation updates
- Internal refactoring without API changes

**Examples:**
- `v2.1.0 → v2.1.1`: Fix calculation error in Vimshottari engine
- `v2.1.1 → v2.1.2`: Improve caching performance

## Version History

### v2.x Series (Current)

#### v2.1.0 - Integration Layer (Feb 3, 2026)
- **Type**: MINOR
- **Focus**: External API integration, Vedic Clock enhancements
- **Key Features**:
  - Integration layer (`noesis-integration`)
  - Vedic API client (`noesis-vedic-api`)
  - 4 new Vedic Clock integrations
  - JSON task management system

#### v2.0.0 - Wave 2 Complete (Jan 2026)
- **Type**: MAJOR
- **Focus**: Human Design, Gene Keys, Vimshottari
- **Key Features**:
  - Complete Human Design engine (100% validated)
  - Gene Keys engine (Shadow-Gift-Siddhi framework)
  - Vimshottari Dasha engine (120-year timeline)
  - Multi-engine orchestration

### v1.x Series (Foundation)

#### v1.0.0 - Wave 1 Complete (Dec 2025)
- **Type**: MAJOR
- **Focus**: Foundation and infrastructure
- **Key Features**:
  - Core API with Axum
  - JWT + API key authentication
  - Docker deployment
  - CORS and rate limiting
  - Health checks and monitoring

## Release Workflow

### 1. Development Phase
- Work on `develop` branch or feature branches
- Create PRs to `develop`
- Continuous integration testing

### 2. Release Candidate
```bash
# Create release branch
git checkout -b release/v2.2.0 develop

# Update version in Cargo.toml
# Update CHANGELOG.md
# Create RELEASE_NOTES_v2.2.0.md

# Final testing
cargo test --all
cargo clippy --all
./scripts/integration-tests.sh

# Commit
git commit -am "chore: Prepare release v2.2.0"
```

### 3. Create Release
```bash
# Merge to main
git checkout main
git merge --no-ff release/v2.2.0

# Tag
git tag -a v2.2.0 -m "Release v2.2.0 - Specialized Engines"

# Push
git push origin main --tags

# GitHub Release will auto-create via Actions
```

### 4. Post-Release
```bash
# Merge back to develop
git checkout develop
git merge --no-ff main

# Delete release branch
git branch -d release/v2.2.0

# Push develop
git push origin develop
```

## Version Planning

### Planned Releases

#### v2.2.0 (Q2 2026) - Specialized Engines
- **Type**: MINOR
- **Engines**: Numerology, Biorhythm, Biofield
- **Focus**: Complete Wave 4 engine implementations

#### v2.3.0 (Q2 2026) - Advanced Features
- **Type**: MINOR
- **Features**: GraphQL API, WebSocket real-time, enhanced synthesis
- **Focus**: Platform capabilities

#### v3.0.0 (Q3 2026) - Platform Launch
- **Type**: MAJOR
- **Breaking**: New API v2, database schema changes
- **Features**: Complete platform with UI, mobile SDKs, analytics

### Long-term Roadmap

#### v3.x Series (Platform Era)
- v3.0.0: Platform launch
- v3.1.0: Mobile SDKs
- v3.2.0: Web dashboard
- v3.3.0: Advanced analytics

#### v4.x Series (AI Era)
- v4.0.0: AI-powered interpretations
- v4.1.0: Predictive modeling
- v4.2.0: Natural language interface

## Version Compatibility

### API Versioning
- **v1 API**: Deprecated, remove in v3.0.0
- **v2 API**: Current, stable
- **v3 API**: Planned for v3.0.0

### Database Schema
- **v1 Schema**: Legacy
- **v2 Schema**: Current (since v2.0.0)
- **v3 Schema**: Planned (v3.0.0)

### Docker Images
- **Latest**: Always points to latest stable
- **vX.Y.Z**: Specific version
- **vX.Y**: Latest patch in minor version
- **vX**: Latest minor in major version

## Deprecation Policy

### Timeline
1. **Announce**: Document in release notes
2. **Mark**: Add deprecation warnings in code/docs
3. **Remove**: Delete in next MAJOR version

**Minimum deprecation period**: 2 minor versions

### Example
```
v2.1.0: Feature X marked deprecated
v2.2.0: Feature X still present, warnings active
v2.3.0: Feature X still present, warnings active
v3.0.0: Feature X removed
```

## Hotfix Process

For critical bugs in production:

```bash
# Create hotfix branch from main
git checkout -b hotfix/v2.1.1 main

# Fix the bug
# Test thoroughly

# Update version to v2.1.1
# Update CHANGELOG.md

# Commit
git commit -am "fix: Critical bug in Vedic Clock calculation"

# Merge to main
git checkout main
git merge --no-ff hotfix/v2.1.1

# Tag
git tag -a v2.1.1 -m "Hotfix v2.1.1"

# Push
git push origin main --tags

# Merge back to develop
git checkout develop
git merge --no-ff hotfix/v2.1.1

# Clean up
git branch -d hotfix/v2.1.1
```

## Version Metadata

### Cargo.toml
```toml
[package]
version = "2.1.0"  # Semantic version
```

### Git Tags
```bash
# Format: vX.Y.Z
v2.1.0  # Release tag
v2.1.1  # Patch tag
```

### Docker Tags
```bash
# Multiple tags per release
ghcr.io/sheshiyer/selemene-engine:v2.1.0
ghcr.io/sheshiyer/selemene-engine:v2.1
ghcr.io/sheshiyer/selemene-engine:v2
ghcr.io/sheshiyer/selemene-engine:latest
```

## Changelog Format

See [CHANGELOG.md](CHANGELOG.md) for full history.

Each version entry includes:
- **Version & Date**: `## [2.1.0] - 2026-02-03`
- **Categories**:
  - Added (new features)
  - Changed (changes to existing)
  - Deprecated (soon-to-be removed)
  - Removed (removed features)
  - Fixed (bug fixes)
  - Security (security improvements)
