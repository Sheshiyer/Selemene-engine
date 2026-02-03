# Quick Navigation - Reorganized Documentation

**Last Updated**: February 3, 2026  
**Purpose**: Quick reference for finding moved documentation and reports

---

## 📋 Most Common Files

### Recent Activity
- **Implementation Complete**: `.context/reports/completion/IMPLEMENTATION_COMPLETE.md`
- **Wave 3 Complete**: `.context/reports/phases/WAVE3_COMPLETION_REPORT.md`
- **GitHub Setup Guide**: `.context/documentation/guides/GITHUB_SETUP_GUIDE.md`
- **Release Notes v2.1.0**: `.context/documentation/guides/RELEASE_NOTES_v2.1.0.md`

### Key Architecture
- **System Architecture**: `.context/documentation/architecture/selemene_architecture.md`
- **Project Summary**: `.context/analysis/codebase/PROJECT_SUMMARY.md`
- **Codebase Summary**: `.context/analysis/codebase/CODEBASE_SUMMARY.md`

### Feature Documentation
- **Ghati API**: `.context/documentation/features/GHATI_API_DOCUMENTATION.md`
- **Ghati Standards**: `.context/documentation/features/GHATI_CALCULATION_STANDARDS.md`
- **Gene Keys Examples**: `.context/documentation/features/GENE_KEYS_EXAMPLE_INQUIRIES.md`
- **Human Design**: `.context/documentation/features/HUMAN_DESIGN_TIME_GATE_IMPLEMENTATION.md`

### Latest Agent Reports
- **Agent 34**: `.context/reports/agents/AGENT_34_COMPLETION_REPORT.md`
- **Agent 31**: `.context/reports/agents/AGENT_31_COMPLETION_REPORT.md`
- **Agent 30**: `.context/reports/agents/AGENT_30_COMPLETION_REPORT.md`

---

## 📂 Directory Guide

| Category | Location | Count | Description |
|----------|----------|-------|-------------|
| **Agent Reports** | `.context/reports/agents/` | 6 | Completion reports from development agents |
| **Phase Reports** | `.context/reports/phases/` | 3 | Wave and phase milestone reports |
| **Implementations** | `.context/reports/implementations/` | 15 | Implementation summaries and details |
| **Completion** | `.context/reports/completion/` | 1 | Final completion reports |
| **Architecture** | `.context/documentation/architecture/` | 1 | System architecture documentation |
| **Guides** | `.context/documentation/guides/` | 4+ | Setup guides and references |
| **API Docs** | `.context/documentation/api/` | 1+ | API documentation |
| **Features** | `.context/documentation/features/` | 7 | Feature-specific documentation |
| **Analysis** | `.context/analysis/codebase/` | 4 | Project and codebase analysis |
| **Test Scripts** | `.context/scripts/test/` | 13 | Test and verification scripts |

---

## 🔍 Search Examples

```bash
# Find all Ghati documentation
find .context -name "*GHATI*"

# Find all Gene Keys docs
find .context -name "*GENE_KEYS*"

# Find all agent reports
ls .context/reports/agents/

# Search for specific content
grep -r "Swiss Ephemeris" .context/

# List all test scripts
ls .context/scripts/test/

# Find implementation summaries
ls .context/reports/implementations/
```

---

## 🗂️ File Patterns

### Reports
- `AGENT_XX_*.md` → Agent completion reports
- `WAVE*_*.md` → Wave milestone reports
- `PHASE*_*.md` → Phase completion reports
- `*_COMPLETE.md` → Task completion reports

### Documentation
- `*_GUIDE.md` → Setup and configuration guides
- `*_DOCUMENTATION.md` → API and feature docs
- `*_STANDARDS.md` → Standards and conventions
- `RELEASE_NOTES_*.md` → Release documentation

### Scripts
- `test_*.sh` → Test execution scripts
- `verify_*.sh` → Verification scripts
- `validate_*.sh` → Validation scripts

---

## 📌 Root Files (Still in Repository Root)

Essential project files remain in root:
- `README.md` - Main project documentation
- `CHANGELOG.md` - Version history
- `DOCKER.md` - Docker deployment guide
- `claude.md` - AI agent instructions
- `memory.md` - Project memory
- `todo.md` - Active tasks
- `Cargo.toml` - Workspace configuration
- `package.json` - Node packages
- `Dockerfile*` - Container builds
- `docker-compose*.yml` - Service orchestration

---

For complete organization details, see: `.context/ORGANIZATION_SUMMARY.md`
