#!/usr/bin/env node
/**
 * Post-deploy gate for the /assets/generate mode contract.
 *
 * Runs against a DEPLOYED engine, because `cargo test` green does not mean the
 * running service serves this — the two drifted before and nobody noticed: every
 * mode returned 200 with a generic "default: Reading", so a typo'd mode was
 * indistinguishable from a real reading and an entire fake surface looked healthy.
 *
 * Two gates, both of which must hold:
 *   1. An unknown mode is REJECTED (400 UNKNOWN_MODE) — not answered.
 *   2. Accepted modes have DISTINCT pass plans.
 *
 * Gate 2 compares pass plans, not assembled text: the text embeds time-varying
 * engine seeds (biorhythm), so hashing it false-PASSes. Weaker assertions that
 * do NOT work — `status === 200` (a typo passes) and `passes[0] !== 'default'`
 * (a whitelisted-but-unimplemented mode passes).
 *
 * Usage: node scripts/verify-mode-contract.mjs <baseUrl> [--token <jwt>]
 * Exit 0 = contract holds. Exit 1 = it doesn't. Exit 2 = couldn't check.
 */

const args = process.argv.slice(2)
const BASE = (args.find((a) => !a.startsWith('--')) || process.env.ENGINE_URL || '').replace(/\/+$/, '')
const TOKEN = process.env.ENGINE_TOKEN || (args.includes('--token') ? args[args.indexOf('--token') + 1] : '')

if (!BASE) {
  console.error('usage: node scripts/verify-mode-contract.mjs <baseUrl>   (or set ENGINE_URL)')
  process.exit(2)
}

const BIRTH = {
  date: '1990-01-15', time: '14:30',
  latitude: 12.9716, longitude: 77.5946,
  timezone: 'Asia/Kolkata', name: 'Contract Probe',
}

const call = async (mode) => {
  const res = await fetch(`${BASE}/api/v1/assets/generate`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(TOKEN ? { Authorization: `Bearer ${TOKEN}` } : {}),
    },
    body: JSON.stringify({ birth_data: BIRTH, mode, consciousness_level: 3 }),
  })
  const text = await res.text()
  let json = null
  try { json = JSON.parse(text) } catch {}
  return {
    mode,
    status: res.status,
    errorCode: json?.error_code ?? null,
    passes: (json?.passes ?? []).map((p) => p.id).join(','),
    body: text.slice(0, 120),
  }
}

// Declared aliases are legitimately identical — they're the same mode document.
const ALIASES = [
  ['integrated-reading', 'composite-dyad'],
  ['integrated-kundali-l0', 'kundali', 'kundali-l0'],
]
const canonical = (m) => (ALIASES.find((g) => g.includes(m)) ?? [m])[0]

const REAL = ['integrated-reading', 'integrated-kundali-l0']
const BOGUS = 'THIS-MODE-DOES-NOT-EXIST-xyz'

const main = async () => {
  console.log(`mode contract gate → ${BASE}`)

  let reachable
  try {
    reachable = await fetch(`${BASE}/health`).then((r) => r.ok)
  } catch {
    reachable = false
  }
  if (!reachable) {
    console.error(`  cannot reach ${BASE}/health — refusing to report a pass on an unverified engine`)
    process.exit(2)
  }

  const rows = []
  for (const m of [...REAL, BOGUS]) rows.push(await call(m))
  for (const r of rows) {
    console.log(`  ${String(r.status).padEnd(4)} ${r.mode.padEnd(30)} ${r.errorCode ?? r.passes ?? r.body}`)
  }

  // gate 1 — an unknown mode must be rejected, not answered
  const bogus = rows.find((r) => r.mode === BOGUS)
  const gate1 = bogus.status === 400 && bogus.errorCode === 'UNKNOWN_MODE'

  // gate 2 — accepted modes must have distinct pass plans
  const accepted = rows.filter((r) => r.status === 200 && r.passes)
  const byPlan = new Map()
  for (const r of accepted) byPlan.set(r.passes, [...(byPlan.get(r.passes) ?? []), r.mode])
  const collisions = [...byPlan.values()]
    .map((g) => [...new Set(g.map(canonical))])
    .filter((g) => g.length > 1)
  const gate2 = collisions.length === 0

  console.log('')
  console.log(`  gate 1 — unknown mode rejected 400/UNKNOWN_MODE : ${gate1 ? 'PASS' : `FAIL (got ${bogus.status}${bogus.errorCode ? '/' + bogus.errorCode : ''})`}`)
  console.log(`  gate 2 — accepted modes have distinct pass plans : ${gate2 ? 'PASS' : 'FAIL'}`)
  for (const c of collisions) console.log(`      share one pass plan: ${c.join(' == ')}`)

  if (!gate1 || !gate2) {
    console.error('\n  the deployed engine does not honour the mode contract')
    process.exit(1)
  }
  console.log('\n  contract holds on the deployed engine')
}

main().catch((e) => {
  console.error(`  gate errored: ${e.message}`)
  process.exit(2)
})
