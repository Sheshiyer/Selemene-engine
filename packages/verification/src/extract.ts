import type { SelemeneEngineId } from '@noesis/witness-pipeline';

export function extract(result: unknown, path: string): unknown {
  if (!result || typeof result !== 'object') return undefined;
  return path.split('.').reduce<unknown>((acc, key) => {
    if (acc === undefined || acc === null) return undefined;
    if (Array.isArray(acc)) {
      const idx = Number(key);
      return Number.isNaN(idx) ? undefined : acc[idx];
    }
    return (acc as Record<string, unknown>)[key];
  }, result);
}

export const ENGINE_FIELD_EXTRACTORS: Partial<Record<SelemeneEngineId, Record<string, string>>> = {
  panchanga: {
    vara: 'vara_name',
    tithi: 'tithi_name',
    nakshatra: 'nakshatra_name',
    yoga: 'yoga_name',
    karana: 'karana_name',
  },
  'human-design': {
    type: 'type',
    authority: 'authority',
    profile: 'profile',
    definition: 'definition',
    cross: 'cross',
    defined_centers: 'defined_centers',
    active_channels: 'active_channels',
  },
  'gene-keys': {
    life_work: 'hologenetic_profile.life_work',
    evolution: 'hologenetic_profile.evolution',
    radiance: 'hologenetic_profile.radiance',
    purpose: 'hologenetic_profile.purpose',
    pearl: 'hologenetic_profile.pearl',
    vocation: 'hologenetic_profile.vocation',
    culture: 'hologenetic_profile.culture',
    sq: 'hologenetic_profile.sq',
    eq: 'hologenetic_profile.eq',
    iq: 'hologenetic_profile.iq',
    attraction: 'hologenetic_profile.attraction',
    reaction_shadow: 'hologenetic_profile.eq.shadow',
  },
  'vedic-clock': {
    lagna_sign: 'd1.ascendant.sign',
    lagna_nakshatra: 'd1.ascendant.nakshatra',
    moon_sign: 'd1.planets.Moon.sign',
    moon_nakshatra: 'd1.planets.Moon.nakshatra',
    mars_dignity: 'd1.planets.Mars.dignity',
    mercury_dignity: 'd1.planets.Mercury.dignity',
    saturn_dignity: 'd1.planets.Saturn.dignity',
  },
  vimshottari: {
    current_mahadasha: 'current_mahadasha.planet',
    current_antardasha: 'current_antardasha.planet',
  },
  numerology: {
    life_path_number: 'life_path_number',
    expression: 'expression',
    soul_urge: 'soul_urge',
    destiny_number: 'destiny_number',
  },
};
