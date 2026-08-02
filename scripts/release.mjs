#!/usr/bin/env node
/**
 * Selemene Engine Release Script
 * 
 * Ported from Urania-137 release workflow.
 * Usage: node scripts/release.mjs [patch|minor|major|X.Y.Z] [--dry-run] [--yes]
 * 
 * Steps:
 *   1. Validate: clean tree, gh auth, no existing tag
 *   2. Bump: workspace Cargo.toml + API doc + baseline artifacts + satellites
 *   3. Generate grouped release notes from git log
 *   4. Commit, tag, push
 *   5. Create GitHub release
 */

import { execSync } from 'child_process';
import { readFileSync, writeFileSync } from 'fs';
import { resolve } from 'path';

const CWD = process.cwd();

// ── Helpers ─────────────────────────────────────────────────────────────────

function run(cmd, { silent = false } = {}) {
  const out = execSync(cmd, { encoding: 'utf-8', cwd: CWD });
  if (!silent) console.log(out.trimEnd());
  return out.trimEnd();
}

function runSilent(cmd) {
  try {
    return execSync(cmd, { encoding: 'utf-8', cwd: CWD }).trimEnd();
  } catch {
    return '';
  }
}

function bail(msg) {
  console.error(`\n❌  ${msg}`);
  process.exit(1);
}

function ok(msg) {
  console.log(`✅  ${msg}`);
}

function dry(msg) {
  console.log(`[DRY-RUN] ${msg}`);
}

function parseArgs(argv) {
  const pos = argv.filter(a => !a.startsWith('-'));
  const flags = argv.filter(a => a.startsWith('-'));
  return {
    bump: pos[0] || 'patch',
    dryRun: flags.includes('--dry-run'),
    yes: flags.includes('--yes'),
  };
}

function bumpVersion(current, bump) {
  if (/^\d+\.\d+\.\d+$/.test(bump)) return bump;
  const [maj, min, pat] = current.split('.').map(Number);
  switch (bump) {
    case 'major': return `${maj + 1}.0.0`;
    case 'minor': return `${maj}.${min + 1}.0`;
    case 'patch': return `${maj}.${min}.${pat + 1}`;
    default: bail(`Unknown bump level: ${bump}. Use patch, minor, major, or X.Y.Z`);
  }
}

function readWorkspaceVersion() {
  const cargo = readFileSync(resolve(CWD, 'Cargo.toml'), 'utf-8');
  const m = cargo.match(/version\s*=\s*"([\d.]+)"/);
  return m ? m[1] : null;
}

function setWorkspaceVersion(newVersion) {
  const path = resolve(CWD, 'Cargo.toml');
  let cargo = readFileSync(path, 'utf-8');
  cargo = cargo.replace(/(version\s*=\s*")([\d.]+)(")/, `$1${newVersion}$3`);
  writeFileSync(path, cargo);
}

function setApiDocVersion(newVersion) {
  const path = resolve(CWD, 'crates/noesis-api/src/lib.rs');
  let src = readFileSync(path, 'utf-8');
  src = src.replace(/(version\s*=\s*")([\d.]+)("\s*,)/, `$1${newVersion}$3`);
  writeFileSync(path, src);
}

/**
 * Satellite files that state the project version but are not Cargo manifests,
 * so nothing else keeps them honest.
 *
 * Each entry names a file and the exact pattern to rewrite. These drifted to
 * 3.0.0 (and 3.3.0 for the TS SDK) against a 3.3.1 workspace: no test asserts
 * them, so the only symptom was published images and a CLI reporting a version
 * from three releases ago.
 *
 * Deliberately excluded: python-services/** is its own version domain, and
 * anything matching `"openapi": "3.0.x"` is an OpenAPI format version rather
 * than a project version.
 */
const VERSIONED_SATELLITES = [
  {
    file: 'Dockerfile.prod',
    pattern: /(org\.opencontainers\.image\.version=")[\d.]+(")/,
  },
  { file: 'bridges/cli/package.json', pattern: /("version"\s*:\s*")[\d.]+(")/ },
  { file: 'bridges/cli/src/cli.ts', pattern: /(\.version\(")[\d.]+("\))/ },
  { file: 'apps/admin-web/package.json', pattern: /("version"\s*:\s*")[\d.]+(")/ },
  { file: 'packages/noesis-sdk-ts/package.json', pattern: /("version"\s*:\s*")[\d.]+(")/ },
  {
    file: 'crates/noesis-sdk/Cargo.toml',
    pattern: /(noesis-core\s*=\s*\{\s*version\s*=\s*")[\d.]+(")/,
  },
];

function setSatelliteVersions(newVersion) {
  const updated = [];
  for (const { file, pattern } of VERSIONED_SATELLITES) {
    const path = resolve(CWD, file);
    let src;
    try {
      src = readFileSync(path, 'utf-8');
    } catch {
      bail(`Versioned file missing: ${file}. Update VERSIONED_SATELLITES in release.mjs.`);
    }
    const next = src.replace(pattern, `$1${newVersion}$2`);
    if (next === src) {
      bail(`No version field rewritten in ${file}. Its shape changed; update release.mjs.`);
    }
    writeFileSync(path, next);
    updated.push(file);
  }
  return updated;
}

/**
 * Baseline artifacts that record a crate version per entry.
 *
 * `crates/noesis-orchestrator/tests/baseline_artifact_tests.rs` compares every
 * one of these against live `cargo metadata`, so a release that bumps
 * Cargo.toml without updating them turns CI red. That is exactly what happened
 * at v3.3.1: the baselines sat at 3.1.0 and two tests failed from the release
 * commit onward, unnoticed because test.yml was separately broken.
 *
 * Every crate under `crates/` sets `version.workspace = true`, so a single
 * workspace version applies to all entries. The fields are rewritten in place
 * rather than regenerated, because engine-matrix.json also carries
 * hand-authored provenance notes that a regeneration would discard.
 */
const VERSIONED_BASELINES = [
  'docs/baseline/engine-matrix.json',
  'docs/baseline/dependency-graph.json',
];

function setBaselineVersions(newVersion) {
  const updated = [];
  for (const rel of VERSIONED_BASELINES) {
    const path = resolve(CWD, rel);
    let src;
    try {
      src = readFileSync(path, 'utf-8');
    } catch {
      bail(`Baseline artifact missing: ${rel}. baseline_artifact_tests will fail after release.`);
    }
    const before = src;
    src = src.replace(/("version"\s*:\s*")[\d.]+(")/g, `$1${newVersion}$2`);
    if (src === before) {
      bail(`No version fields rewritten in ${rel}. Its shape changed; update release.mjs.`);
    }
    // Fail loudly rather than commit a baseline that no longer parses.
    try {
      JSON.parse(src);
    } catch (e) {
      bail(`Rewriting versions in ${rel} produced invalid JSON: ${e.message}`);
    }
    writeFileSync(path, src);
    const count = (before.match(/"version"\s*:\s*"[\d.]+"/g) || []).length;
    updated.push(`${rel} (${count} entries)`);
  }
  return updated;
}

function getCommitsSince(lastTag) {
  const range = lastTag ? `${lastTag}..HEAD` : 'HEAD~20..HEAD';
  const log = runSilent(`git log ${range} --pretty=format:"%s"`);
  return log ? log.split('\n').filter(Boolean) : [];
}

function categorizeCommits(commits) {
  const cats = { feat: [], fix: [], docs: [], chore: [], test: [], other: [] };
  for (const c of commits) {
    if (/^(feat|feature)\b/i.test(c)) cats.feat.push(c);
    else if (/^fix\b/i.test(c)) cats.fix.push(c);
    else if (/^docs\b/i.test(c)) cats.docs.push(c);
    else if (/^(chore|refactor|style|ci|build)\b/i.test(c)) cats.chore.push(c);
    else if (/^test\b/i.test(c)) cats.test.push(c);
    else cats.other.push(c);
  }
  return cats;
}

function buildNotes(version, cats, commitCount) {
  const sections = [];
  if (cats.feat.length) sections.push(`### ✨ Features\n\n${cats.feat.map(c => `- ${c}`).join('\n')}`);
  if (cats.fix.length) sections.push(`### 🐛 Fixes\n\n${cats.fix.map(c => `- ${c}`).join('\n')}`);
  if (cats.docs.length) sections.push(`### 📝 Documentation\n\n${cats.docs.map(c => `- ${c}`).join('\n')}`);
  if (cats.test.length) sections.push(`### 🧪 Tests\n\n${cats.test.map(c => `- ${c}`).join('\n')}`);
  if (cats.chore.length) sections.push(`### 🔧 Chores\n\n${cats.chore.map(c => `- ${c}`).join('\n')}`);
  if (cats.other.length) sections.push(`### Other\n\n${cats.other.map(c => `- ${c}`).join('\n')}`);

  return [
    `## Selemene Engine v${version}`,
    '',
    `**${commitCount} commits** since last tag.`,
    '',
    ...sections,
    '',
    '---',
    `Docker: \`ghcr.io/Sheshiyer/selemene-engine:${version}\``,
  ].join('\n');
}

// ── Main ────────────────────────────────────────────────────────────────────

async function main() {
  const { bump, dryRun, yes } = parseArgs(process.argv.slice(2));

  console.log('🔧  Selemene Engine Release\n');

  // 1. Dirty tree check
  const dirty = runSilent('git status --porcelain');
  if (dirty) {
    console.log('Uncommitted changes:\n' + dirty);
    bail('Working tree is dirty. Commit or stash before releasing.');
  }
  ok('Working tree is clean');

  // 2. gh auth check
  try {
    runSilent('gh auth status');
    ok('gh CLI authenticated');
  } catch {
    bail('gh CLI not authenticated. Run: gh auth login');
  }

  // 3. Read current version
  const current = readWorkspaceVersion();
  if (!current) bail('Could not read version from Cargo.toml');
  console.log(`Current version: ${current}`);

  const next = bumpVersion(current, bump);
  console.log(`Next version:    ${next}`);

  // 4. Existing tag check
  const existing = runSilent(`git tag -l v${next}`);
  if (existing) bail(`Tag v${next} already exists.`);
  ok(`Tag v${next} is available`);

  // 5. Collect commits
  const lastTag = runSilent('git describe --tags --abbrev=0 2>/dev/null') || '';
  const commits = getCommitsSince(lastTag);
  const cats = categorizeCommits(commits);
  const total = commits.length;
  console.log(`\nCommits since ${lastTag || 'beginning'}: ${total}`);

  // 6. Dry-run plan
  if (dryRun) {
    console.log('\n─── DRY-RUN PLAN ───');
    dry(`Bump Cargo.toml workspace version: ${current} → ${next}`);
    dry(`Bump API doc version in noesis-api/src/lib.rs: ${current} → ${next}`);
    for (const rel of VERSIONED_BASELINES) {
      dry(`Rewrite version fields in ${rel}: ${current} → ${next}`);
    }
    for (const { file } of VERSIONED_SATELLITES) {
      dry(`Rewrite version in ${file}: ${current} → ${next}`);
    }
    dry(`Commit: "chore(release): v${next}"`);
    dry(`Tag: v${next}`);
    dry(`Push: git push origin main --tags`);
    dry(`Create GitHub release with ${total} commits`);
    console.log('─────────────────────\n');
    return;
  }

  // 7. Confirm
  if (!yes) {
    process.stdout.write('\nProceed? [y/N] ');
    const answer = await new Promise(r => process.stdin.once('data', d => r(d.toString().trim())));
    if (answer.toLowerCase() !== 'y') bail('Aborted.');
  }

  // 8. Bump versions
  setWorkspaceVersion(next);
  setApiDocVersion(next);
  const baselines = setBaselineVersions(next);
  const satellites = setSatelliteVersions(next);
  ok(`Bumped version to ${next}`);
  for (const b of baselines) ok(`Baseline synced: ${b}`);
  for (const f of satellites) ok(`Version synced: ${f}`);

  // 9. Commit
  run(`git add Cargo.toml crates/noesis-api/src/lib.rs ${VERSIONED_BASELINES.join(' ')} ${VERSIONED_SATELLITES.map(v => v.file).join(' ')}`);
  run(`git commit -m "chore(release): v${next}"`);
  ok('Version commit created');

  // 10. Tag
  run(`git tag -a v${next} -m "Selemene Engine v${next}"`);
  ok(`Annotated tag v${next} created`);

  // 11. Push
  run(`git push origin main --tags`);
  ok('Pushed to origin');

  // 12. GitHub release
  const notes = buildNotes(next, cats, total);
  const notesPath = resolve(CWD, '.release-notes-tmp.md');
  writeFileSync(notesPath, notes);
  run(`gh release create v${next} --title "Selemene Engine v${next}" --notes-file "${notesPath}"`);
  runSilent(`rm "${notesPath}"`);
  ok(`GitHub release v${next} published`);

  console.log(`\n🚀  Selemene Engine v${next} is live!`);
}

main().catch(e => {
  console.error(e);
  process.exit(1);
});
