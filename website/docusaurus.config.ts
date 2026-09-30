import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import {themes as prismThemes} from 'prism-react-renderer';
import docsSystemPlugin, {ecosystemFooterGroup, ecosystemNavbarItems} from '@beyond10x/docs-system/docusaurus';

const config: Config = {
  title: 'uilab',
  tagline:
    'A browser workbench where an agent builds your UI specification with you: select a node, say what you want, review the diff, accept.',
  favicon: 'img/mark.svg',

  future: {v4: true},
  url: 'https://beyond10x.github.io',
  baseUrl: '/uilab/',
  organizationName: 'beyond10x',
  projectName: 'uilab',
  deploymentBranch: 'gh-pages',
  trailingSlash: false,
  onBrokenLinks: 'throw',

  markdown: {
    hooks: {onBrokenMarkdownLinks: 'throw'},
    mermaid: true,
  },
  themes: ['@docusaurus/theme-mermaid'],
  plugins: [docsSystemPlugin],
  i18n: {defaultLocale: 'en', locales: ['en']},

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          routeBasePath: 'docs',
          editUrl: 'https://github.com/beyond10x/uilab/tree/main/website/',
        },
        blog: false,
        theme: {customCss: './src/css/custom.css'},
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    colorMode: {defaultMode: 'dark', respectPrefersColorScheme: true},
    navbar: {
      title: 'uilab',
      logo: {alt: 'uilab', src: 'img/mark.svg', width: 26, height: 26},
      items: [
        ...ecosystemNavbarItems(),
        {type: 'docSidebar', sidebarId: 'docsSidebar', position: 'left', label: 'Documentation'},
        {to: '/docs/getting-started', label: 'Getting started', position: 'left'},
        {to: '/docs/adopting', label: 'Adopting', position: 'left'},
        {
          href: 'https://github.com/beyond10x/uilab',
          label: 'GitHub',
          position: 'right',
          className: 'navbar-github-link',
          'aria-label': 'GitHub repository',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        ecosystemFooterGroup(),
        {
          title: 'Documentation',
          items: [
            {label: 'Introduction', to: '/docs'},
            {label: 'Getting started', to: '/docs/getting-started'},
            {label: 'Concepts', to: '/docs/concepts/the-document'},
            {label: 'FAQ', to: '/docs/faq'},
          ],
        },
        {
          title: 'Working with uilab',
          items: [
            {label: 'Working with the agent', to: '/docs/working-with-the-agent'},
            {label: 'The specification', to: '/docs/specification'},
            {label: 'Adopting in a front-end team', to: '/docs/adopting'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'Source', href: 'https://github.com/beyond10x/uilab'},
            {label: 'ESS', href: 'https://github.com/beyond10x/ess'},
          ],
        },
      ],
      logo: {alt: 'uilab', src: 'img/mark.svg', href: '/', width: 22, height: 22},
      copyright:
        '<span class="footer__claim">You decide. The agent proposes. The file is the record.</span>' +
        'uilab · built with Docusaurus.',
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'yaml', 'json', 'bash'],
    },
    mermaid: {theme: {light: 'neutral', dark: 'dark'}},
  } satisfies Preset.ThemeConfig,
};

export default config;
