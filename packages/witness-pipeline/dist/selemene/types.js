// ─── Engine identifiers, routing, and typed result map ───────────────
// Ported from witness-agents/src/types/engine.ts
export const SELEMENE_ENGINE_IDS = [
    'panchanga', 'vimshottari', 'human-design', 'gene-keys', 'numerology',
    'biorhythm', 'vedic-clock', 'biofield', 'face-reading', 'nadabrahman',
    'transits', 'tarot', 'i-ching', 'enneagram', 'sacred-geometry', 'sigil-forge',
];
export const ENGINE_ID_MAP = {
    'panchanga': 'temporal-grammar',
    'vimshottari': 'chronofield',
    'human-design': 'energetic-authority',
    'gene-keys': 'gift-shadow-spectrum',
    'numerology': 'numeric-architecture',
    'biorhythm': 'three-wave-cycle',
    'vedic-clock': 'circadian-cartography',
    'biofield': 'bioelectric-field',
    'face-reading': 'physiognomic-mapping',
    'nadabrahman': 'resonance-architecture',
    'transits': 'active-planetary-weather',
    'tarot': 'archetypal-mirror',
    'i-ching': 'hexagram-navigation',
    'enneagram': 'nine-point-architecture',
    'sacred-geometry': 'geometric-resonance',
    'sigil-forge': 'sigil-forge',
};
export const REVERSE_ENGINE_MAP = Object.fromEntries(Object.entries(ENGINE_ID_MAP).map(([k, v]) => [v, k]));
export const ENGINE_ROUTING = {
    'vimshottari': 'aletheios-primary',
    'human-design': 'aletheios-primary',
    'enneagram': 'aletheios-primary',
    'i-ching': 'aletheios-primary',
    'numerology': 'aletheios-primary',
    'biorhythm': 'pichet-primary',
    'vedic-clock': 'pichet-primary',
    'biofield': 'pichet-primary',
    'face-reading': 'pichet-primary',
    'nadabrahman': 'pichet-primary',
    'panchanga': 'dyad-synthesis',
    'gene-keys': 'dyad-synthesis',
    'tarot': 'dyad-synthesis',
    'sacred-geometry': 'dyad-synthesis',
    'sigil-forge': 'dyad-synthesis',
    'transits': 'dyad-synthesis',
};
//# sourceMappingURL=types.js.map