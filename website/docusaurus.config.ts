import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import {themes as prismThemes, type PrismTheme} from 'prism-react-renderer';
import docsSystemPlugin, {ecosystemFooterGroup} from '@beyond10x/docs-system/docusaurus';
import admonitionSyntax from './src/plugins/admonitionSyntax';
import rehypeTableLabels from './src/plugins/rehypeTableLabels';

const lightCode: PrismTheme = {
  plain: {color: '#393a34', backgroundColor: '#f6f8fa'},
  styles: [
    {types: ['comment', 'prolog', 'doctype', 'cdata'], style: {color: '#66665a', fontStyle: 'italic'}},
    {types: ['namespace'], style: {opacity: 0.85}},
    {types: ['string', 'attr-value'], style: {color: '#b8005a'}},
    {types: ['punctuation', 'operator'], style: {color: '#393a34'}},
    {
      types: ['entity', 'url', 'symbol', 'number', 'boolean', 'variable', 'constant', 'property', 'regex', 'inserted'],
      style: {color: '#0a6f6d'},
    },
    {types: ['atrule', 'keyword', 'attr-name', 'selector'], style: {color: '#0063a3'}},
    {types: ['function', 'deleted', 'tag'], style: {color: '#b31d30'}},
    {types: ['function-variable'], style: {color: '#6f42c1'}},
    {types: ['tag', 'selector', 'keyword'], style: {color: '#00009f'}},
  ],
};

const darkCode: PrismTheme = {
  ...prismThemes.dracula,
  styles: prismThemes.dracula.styles.map((entry) => {
    if (entry.types.includes('comment')) return {...entry, style: {...entry.style, color: '#909dc8'}};
    if (entry.types.includes('deleted')) return {...entry, style: {...entry.style, color: '#ff6e6e'}};
    return entry;
  }),
};

/**
 * The shared organisation column, with its first link named after the site it opens: "Start here"
 * is also this site's first sidebar category, and the footer link leaves the site.
 */
function organisationFooterGroup() {
  const group = ecosystemFooterGroup();
  return {
    ...group,
    items: group.items.map((item) =>
      item.href === 'https://beyond10x.github.io/' ? {...item, label: 'beyond10x'} : item,
    ),
  };
}

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
  plugins: [docsSystemPlugin, admonitionSyntax],
  i18n: {defaultLocale: 'en', locales: ['en']},

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          routeBasePath: 'docs',
          editUrl: 'https://github.com/beyond10x/uilab/tree/main/website/',
          rehypePlugins: [rehypeTableLabels],
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
        {
          href: 'https://beyond10x.github.io/',
          label: 'beyond10x',
          position: 'right',
          'aria-label': 'beyond10x, the organisation site',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
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
        organisationFooterGroup(),
      ],
      logo: {alt: 'uilab', src: 'img/mark-footer.svg', href: '/', width: 36, height: 36},
      copyright:
        '<span class="footer__claim">You decide. The agent proposes. The file is the record.</span>' +
        'uilab · built with Docusaurus.',
    },
    prism: {
      theme: lightCode,
      darkTheme: darkCode,
      additionalLanguages: ['rust', 'yaml', 'json', 'bash'],
    },
    mermaid: {theme: {light: 'neutral', dark: 'dark'}},
  } satisfies Preset.ThemeConfig,
};

export default config;
