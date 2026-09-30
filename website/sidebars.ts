import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'category',
      label: 'Start here',
      collapsed: false,
      items: ['index', 'getting-started'],
    },
    {
      type: 'category',
      label: 'Concepts',
      collapsed: false,
      items: [
        'concepts/the-document',
        'concepts/nodes-and-layers',
        'concepts/proposals-and-review',
        'concepts/widgets',
        'concepts/goals',
        'concepts/drafts-and-sample-data',
      ],
    },
    {
      type: 'category',
      label: 'Working with uilab',
      collapsed: false,
      items: ['working-with-the-agent', 'specification', 'adopting'],
    },
    {
      type: 'category',
      label: 'Questions',
      collapsed: false,
      items: ['faq'],
    },
  ],
};

export default sidebars;
