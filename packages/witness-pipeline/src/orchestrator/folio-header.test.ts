import { describe, it, expect } from 'vitest';
import { renderFolioRelationshipHeader } from './folio-header.js';

describe('folio-header renderer (Phase 6 B-surface)', () => {
  it('produces declarative non-presumptive Folio header for mother-son', () => {
    const header = renderFolioRelationshipHeader({
      subjectRoles: [
        { role: 'mother', name: 'Aarav', label: undefined },
        { role: 'son', name: 'Vikram', label: undefined },
      ],
      relationshipContext: {
        type: 'family',
        mapping_goal: 'understand transmission patterns without outcome prediction',
        sensitivity_level: 'high',
      },
    });
    // Failing initially: current logic produces "mother-son family — ..." not the Folio declarative form
    expect(header).toContain('Mother-Son Lineage Mapping — non-predictive pattern witness');
    expect(header).toContain('Subjects: Aarav (mother), Vikram (son)');
    expect(header).toContain('Mapping goal: understand transmission patterns without outcome prediction');
    expect(header).toContain('Sensitivity: high');
    // Basic Folio markdown typography cue: starts with # heading (ink-iron style heading)
    expect(header.startsWith('# ')).toBe(true);
  });

  it('produces declarative non-presumptive Folio header for business-partners', () => {
    const header = renderFolioRelationshipHeader({
      subjectRoles: [
        { role: 'business-partner', name: 'Priya', label: 'CEO' },
        { role: 'business-partner', name: 'Rahul', label: 'CTO' },
      ],
      relationshipContext: {
        type: 'business-partners',
        mapping_goal: 'map decisions and synergy without guarantees',
        sensitivity_level: 'medium',
      },
    });
    expect(header).toContain('Business-Partners Synergy Audit — non-predictive pattern witness');
    expect(header).toContain('Subjects: Priya (business-partner, CEO), Rahul (business-partner, CTO)');
    expect(header).toContain('Mapping goal: map decisions and synergy without guarantees');
    expect(header).toContain('Sensitivity: medium');
    expect(header.startsWith('# ')).toBe(true);
  });

  it('localizes the relationship header for French reports', () => {
    const header = renderFolioRelationshipHeader({
      subjectRoles: [
        { role: 'business-partner', name: 'Priya' },
        { role: 'business-partner', name: 'Rahul' },
        { role: 'business-partner', name: 'Sam' },
      ],
      relationshipContext: {
        type: 'business-partners',
        mapping_goal: 'Décrire les motifs symboliques entre trois partenaires.',
        sensitivity_level: 'high',
      },
      language: 'fr',
    });
    expect(header).toContain('Partenaires commerciaux — synastrie professionnelle — lecture descriptive, sans prédiction');
    expect(header).toContain('Partenaires: Priya (partenaire commercial), Rahul (partenaire commercial), Sam (partenaire commercial)');
    expect(header).toContain('Objectif de lecture: Décrire les motifs symboliques entre trois partenaires.');
    expect(header).toContain('Sensibilité: élevée');
    expect(header).not.toContain('Mapping goal:');
  });
});
