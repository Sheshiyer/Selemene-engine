import assert from 'node:assert/strict';
import { readFileSync, existsSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import { nodes, meta } from '../src/data.js';
import { readEngineRoomState, writeEngineRoomState } from '../src/engine-room-state.js';

const root = fileURLToPath(new URL('..', import.meta.url));
const repo = resolve(root, '../../..');
const readJSON = path => JSON.parse(readFileSync(resolve(root, path), 'utf8'));
const report = readJSON('evidence/engine-truth-research.json');
const artwork = readJSON('evidence/engine-artwork-manifest.json').artworks;
const canonical = nodes.filter(node => node.kind === 'engine').map(node => node.id).sort();
assert.equal(report.revision, meta.revision);
assert.deepEqual(report.engines.map(engine => engine.id).sort(), canonical);
assert.deepEqual(artwork.map(asset => asset.id).sort(), canonical);
const sources = new Set();
const statuses = new Set(['implemented', 'simplified', 'delegated', 'placeholder']);
let partCount = 0;
let connectionCount = 0;
let reconciliationCount = 0;
for (const engine of report.engines) {
  assert.ok(engine.parts.length >= 4 && engine.parts.length <= 7, `${engine.id}: part count`);
  const ids = new Set(engine.parts.map(part => part.id));
  assert.equal(ids.size, engine.parts.length, `${engine.id}: unique part IDs`);
  assert.ok(engine.summary && engine.statusNote && engine.profile);
  for (const part of engine.parts) {
    assert.ok(part.label && part.role && part.description && part.symbol, `${engine.id}/${part.id}: content`);
    assert.ok(statuses.has(part.status), `${engine.id}/${part.id}: status`);
    assert.ok(Number.isInteger(part.line) && part.line > 0, `${engine.id}/${part.id}: line`);
    sources.add(part.source);
    const state = { engineId: engine.id, explosion: 0.37, partId: part.id, isolated: true };
    const url = writeEngineRoomState('https://atlas.tryambakam.space/?view=biofield&time=15&kosha=all', state);
    assert.equal(url.searchParams.get('view'), 'biofield');
    assert.equal(url.searchParams.get('time'), '15');
    assert.deepEqual(readEngineRoomState(url.searchParams, report.engines), state);
    const closed = writeEngineRoomState(url, null);
    assert.equal(closed.searchParams.get('view'), 'biofield');
    assert.equal(readEngineRoomState(closed.searchParams, report.engines), null);
  }
  for (const edge of engine.connections) assert.ok(ids.has(edge.from) && ids.has(edge.to), `${engine.id}: dangling relationship`);
  for (const change of engine.reconciliations) {
    assert.ok(change.claim && change.truth && change.source);
    sources.add(change.source);
  }
  engine.docs.forEach(path => sources.add(path));
  const asset = artwork.find(asset => asset.id === engine.id);
  assert.equal(asset.path, `/assets/engines/${engine.id}.png`);
  const localAsset = resolve(root, 'public', asset.path.slice(1));
  assert.ok(existsSync(localAsset));
  assert.equal(createHash('sha256').update(readFileSync(localAsset)).digest('hex'), asset.sha256, `${engine.id}: artwork provenance`);
  partCount += engine.parts.length;
  connectionCount += engine.connections.length;
  reconciliationCount += engine.reconciliations.length;
}
for (const path of sources) {
  assert.ok(!path.startsWith('/') && !path.includes('..'), `unsafe source path: ${path}`);
  execFileSync('git', ['cat-file', '-e', `${report.revision}:${path}`], { cwd: repo, stdio: 'pipe' });
}
const first = report.engines[0].id;
const invalid = readEngineRoomState(new URLSearchParams(`engine=${first}&explode=NaN&part=missing&isolate=1`), report.engines);
assert.deepEqual(invalid, { engineId: first, explosion: 0, partId: null, isolated: false });
assert.equal(readEngineRoomState(new URLSearchParams('engine=unknown'), report.engines), null);
assert.equal(readEngineRoomState(new URLSearchParams(`engine=${first}&explode=99`), report.engines).explosion, 1);
assert.equal(readEngineRoomState(new URLSearchParams(`engine=${first}&explode=-3`), report.engines).explosion, 0);
console.log(`PASS: ${report.engines.length} engines, ${partCount} parts, ${connectionCount} relationships, ${reconciliationCount} reconciliations, ${sources.size} pinned sources, 19 artwork hashes and all part-link round trips.`);
