export declare const SELEMENE_ENGINE_IDS: readonly ["panchanga", "vimshottari", "human-design", "gene-keys", "numerology", "biorhythm", "vedic-clock", "biofield", "face-reading", "nadabrahman", "transits", "tarot", "i-ching", "enneagram", "sacred-geometry", "sigil-forge"];
export type SelemeneEngineId = (typeof SELEMENE_ENGINE_IDS)[number];
export type WitnessEngineAlias = 'temporal-grammar' | 'chronofield' | 'energetic-authority' | 'gift-shadow-spectrum' | 'numeric-architecture' | 'three-wave-cycle' | 'circadian-cartography' | 'bioelectric-field' | 'physiognomic-mapping' | 'resonance-architecture' | 'active-planetary-weather' | 'archetypal-mirror' | 'hexagram-navigation' | 'nine-point-architecture' | 'geometric-resonance' | 'sigil-forge';
export declare const ENGINE_ID_MAP: Record<SelemeneEngineId, WitnessEngineAlias>;
export declare const REVERSE_ENGINE_MAP: Record<WitnessEngineAlias, SelemeneEngineId>;
export type RoutingMode = 'aletheios-primary' | 'pichet-primary' | 'dyad-synthesis';
export declare const ENGINE_ROUTING: Record<SelemeneEngineId, RoutingMode>;
export interface CalculationMetadata {
    calculation_time_ms: number;
    backend: string;
    precision_achieved: string;
    cached: boolean;
    timestamp: string;
    engine_version: string;
}
export interface SelemeneEngineOutput {
    engine_id: SelemeneEngineId;
    result: unknown;
    witness_prompt: string;
    consciousness_level: number;
    metadata: CalculationMetadata;
    envelope_version: string;
    _error?: string;
}
export interface BirthData {
    date: string;
    time?: string;
    timezone?: string;
    latitude?: number;
    longitude?: number;
    name?: string;
}
//# sourceMappingURL=types.d.ts.map