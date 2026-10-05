/** Shared report titles: generation and final verification use one contract. */
const FRENCH_TITLES: Readonly<Record<string,string>> = {
  opening: 'Ouverture et systèmes disponibles',
  part1: 'Partie I — Carte des convergences',
  part2: 'Partie II — Fondements védiques',
  part3: 'Partie III — Architecture karmique',
  part4: 'Partie IV — Carrière et dharma',
  part5: 'Partie V — Richesse et argent',
  part6: 'Partie VI — Amour et vie de couple',
  part7: 'Partie VII — Santé et corps énergétique',
  part8: 'Partie VIII — Famille et racines',
  part9: 'Partie IX — Chronologie de référence',
  part10: 'Partie X — Pratiques et autonomie',
  part11: 'Partie XI — Synthèse finale',
};
export function localizeReferenceTitles(mode: {frontmatter:{pass_plan:Array<{id:string;title:string}>}}, language: string): void {
  if (language !== 'en' && language !== 'fr') throw new Error('Unsupported reference language: '+language);
  if (language === 'en') return;
  for (const pass of mode.frontmatter.pass_plan) {
    if (!FRENCH_TITLES[pass.id]) throw new Error('Unknown reference section: '+pass.id);
    pass.title = FRENCH_TITLES[pass.id];
  }
}
