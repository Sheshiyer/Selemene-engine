import research from '../evidence/engine-truth-research.json';
import { nodes, koshas } from './data.js';

// The report preserves the source terminology and implementation boundaries.
// The scene interprets each responsibility as a sculptural part.
export const engineDetails = research.engines.map(engine => {
  const node = nodes.find(node => node.id === engine.id && node.kind === 'engine');
  const kosha = koshas.find(kosha => kosha.id === node?.kosha);
  return {
    ...engine,
    kosha: node?.kosha,
    color: kosha?.color || '#C5A017',
    artwork: `/assets/engines/${engine.id}.png`,
  };
});

export const engineResearch = research;
