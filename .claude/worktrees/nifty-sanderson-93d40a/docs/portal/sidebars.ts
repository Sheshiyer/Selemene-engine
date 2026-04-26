import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  tutorialSidebar: [
    'index',
    'authentication',
    'rate-limits',
    'sdk-quickstarts',
    'openapi-explorer',
    {
      type: 'category',
      label: 'Engines',
      items: [
        'engines/index',
        'engines/panchanga',
        'engines/numerology',
        'engines/biorhythm',
        'engines/human-design',
        'engines/gene-keys',
        'engines/vimshottari',
        'engines/biofield',
        'engines/vedic-clock',
        'engines/face-reading',
        'engines/nadabrahman',
        'engines/transits',
        'engines/tarot',
        'engines/i-ching',
        'engines/enneagram',
        'engines/sacred-geometry',
        'engines/sigil-forge'
      ]
    },
    {
      type: 'category',
      label: 'Workflows',
      items: [
        'workflows/index',
        'workflows/birth-blueprint',
        'workflows/daily-practice',
        'workflows/decision-support',
        'workflows/self-inquiry',
        'workflows/creative-expression',
        'workflows/full-spectrum'
      ]
    }
  ],
};

export default sidebars;
