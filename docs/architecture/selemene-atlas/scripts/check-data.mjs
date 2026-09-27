import assert from 'node:assert/strict';
import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
import { nodes, edges, meta, families, koshas } from '../src/data.js';

const root = fileURLToPath(new URL('../../../../', import.meta.url));
const source = readFileSync(resolve(root, 'crates/noesis-orchestrator/src/lib.rs'), 'utf8');
const declared = [...source.match(/SUPPORTED_ENGINE_IDS:[\s\S]*?= \[([\s\S]*?)\];/)[1].matchAll(/"([\w-]+)"/g)].map(m => m[1]).sort();
const actual = nodes.filter(n => n.kind === 'engine').map(n => n.id).sort();
assert.deepEqual(actual, declared, 'Engine atlas must match the source registry exactly');
assert.equal(meta.engineCount, declared.length);
assert.equal(koshas.length, 5, 'There must be five Kosha lenses');
assert.equal(new Set(koshas.map(k => k.id)).size, 5, 'Kosha identities must be unique');
assert.deepEqual(koshas.flatMap(k => k.engines).sort(), declared, 'Every canonical engine must appear in exactly one Kosha');
for (const kosha of koshas) {
  for (const id of kosha.engines) {
    const node = nodes.find(n => n.id === id);
    assert.equal(node.kosha, kosha.id, `Inconsistent Kosha assignment: ${id}`);
    assert.equal(node.family, kosha.id, `Inconsistent visual family: ${id}`);
    assert.ok(node.koshaRationale, `Missing interpretive rationale: ${id}`);
  }
}
assert.equal(new Set(nodes.map(n => n.id)).size, nodes.length, 'Duplicate node identity');
const ids = new Set(nodes.map(n => n.id));
const familyIds = new Set(families.map(f => f.id));
for (const path of new Set([...nodes, ...edges].map(n => n.source))) {
  execFileSync('git', ['cat-file', '-e', `${meta.revision}:${path}`], { cwd: root, stdio: 'pipe' });
}
for (const node of nodes) {
  assert.ok(existsSync(resolve(root, node.source)), `Missing source for ${node.id}`);
  assert.ok(familyIds.has(node.family), `Unknown family for ${node.id}`);
  for (const key of ['body', 'field']) assert.ok(node[key].length === 3 && node[key].every(Number.isFinite), `Invalid ${key} position: ${node.id}`);
  assert.ok(node.description && node.evidence && node.anatomy, `Missing context: ${node.id}`);
}
for (const edge of edges) {
  assert.ok(ids.has(edge.from) && ids.has(edge.to), `Dangling edge ${edge.from} -> ${edge.to}`);
  assert.notEqual(edge.from, edge.to, 'Unexpected self loop');
  assert.ok(existsSync(resolve(root, edge.source)), `Missing edge source: ${edge.source}`);
}
assert.ok(!edges.some(e => e.from === 'biofield' && e.to === 'biofield-cv'), 'Native Biofield must not masquerade as the live CV boundary');
assert.ok(edges.some(e => e.from === 'api' && e.to === 'biofield-cv'), 'API to live CV boundary missing');
assert.ok(nodes.find(n => n.id === 'pattern-memory').description.includes('Placeholder'), 'Scaffold status must remain explicit');
console.log(`PASS: ${actual.length} canonical engines across ${koshas.length} Koshas, ${nodes.length} unique components, ${edges.length} valid relationships; source paths, exact coverage and Biofield boundaries verified.`);
