import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'Selemene Developer Portal',
  tagline: 'SDK, API, engines, workflows, and launch-ready integration docs',
  url: 'https://selemene-engine-docs.vercel.app',
  baseUrl: '/',
  organizationName: 'Sheshiyer',
  projectName: 'Selemene-engine',
  onBrokenLinks: 'throw',
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },
  i18n: {defaultLocale: 'en', locales: ['en']},
  presets: [
    [
      'classic',
      {
        docs: {
          routeBasePath: '/',
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/Sheshiyer/Selemene-engine/tree/main/docs/portal/'
        },
        blog: false,
        theme: {customCss: './src/css/custom.css'}
      } satisfies Preset.Options,
    ],
  ],
  themeConfig: {
    navbar: {
      title: 'Selemene Docs',
      items: [
        {to: '/', label: 'Docs', position: 'left'},
        {href: 'https://github.com/Sheshiyer/Selemene-engine', label: 'GitHub', position: 'right'}
      ]
    }
  } satisfies Preset.ThemeConfig,
};

export default config;
