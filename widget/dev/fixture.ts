// Development fixture: the outline and view rows of examples/library, written out as the server
// would send them. Not part of the production bundle.
import type { UilabWireOutlineNode as OutlineNode, UilabWireRows as Rows } from '../src/generated/types.ts';

function node(path: string, layer: string, kind: string, extra: Partial<OutlineNode> = {}, children: OutlineNode[] = []): OutlineNode {
  const last = path === '/' || path === 'nav' ? '' : path.split('/').at(-1)!;
  return { path, layer, name: last.includes(':') ? last.split(':')[1] : '', kind, children, ...extra };
}

export function libraryOutline(): OutlineNode {
  return node('/', 'root', 'document', { title: 'Lending library' }, [
    node('shell:app', 'shell', 'shell', {}, [
      node('shell:app/region:nav', 'region', 'navigation'),
      node('shell:app/region:account', 'region', 'account_menu'),
      node('shell:app/region:main', 'region', 'page_outlet'),
      node('shell:app/region:overlay', 'region', 'overlay_outlet'),
      node('shell:app/region:notify', 'region', 'notifications'),
    ]),
    node('nav', 'nav', 'navigation', { props: { home: 'overview' } }, [
      node('nav/nav_section:circulation', 'nav_section', 'nav_section', { title: 'Circulation', props: { pages: ['overview', 'loans'] } }),
      node('nav/nav_section:people', 'nav_section', 'nav_section', { title: 'People', props: { pages: ['members'] } }),
    ]),
    node('page:overview', 'page', 'dashboard_page', { title: 'Overview', props: { shell: 'app' } }, [
      node('page:overview/section:on_loan', 'section', 'metric', {
        title: 'Copies on loan',
        view: 'loans.Summary',
        props: { title: 'Copies on loan', from: 'on_loan' },
      }),
      node('page:overview/section:recent', 'section', 'collection', {
        title: 'Recent loans',
        view: 'loans.All',
        props: { title: 'Recent loans', columns: [{ field: 'title' }, { field: 'member' }, { field: 'due' }] },
      }),
    ]),
    node('page:loans', 'page', 'list_page', { title: 'Loans', props: { shell: 'app' } }, [
      node('page:loans/section:list', 'section', 'collection', {
        view: 'loans.All',
        props: {
          columns: [{ field: 'title' }, { field: 'member' }, { field: 'due' }, { field: 'state', as: 'tag' }],
          row_actions: [{ opens: 'edit', label: 'Extend' }],
        },
      }),
      node('page:loans/overlay:edit', 'overlay', 'drawer form', {
        title: 'Extend loan',
        props: { title: 'Extend loan', does: 'loans.ExtendLoan', fields: ['due'] },
      }),
    ]),
    node('page:members', 'page', 'list_page', { title: 'Members', props: { shell: 'app' } }, [
      node('page:members/section:list', 'section', 'collection', {
        view: 'members.All',
        props: {
          columns: [{ field: 'name' }, { field: 'joined' }, { field: 'loans' }, { field: 'standing', as: 'tag' }],
        },
      }),
    ]),
  ]);
}

export const libraryRows: Record<string, Rows> = {
  'loans.All': {
    view: 'loans.All',
    total: 4,
    rows: [
      { id: 'l-1', title: 'The Left Hand of Darkness', member: 'Robin Example', due: '2026-10-14', state: 'on_loan' },
      { id: 'l-2', title: 'A Pattern Language', member: 'Kim Sample', due: '2026-10-02', state: 'overdue' },
      { id: 'l-3', title: 'Middlemarch', member: 'Sam Placeholder', due: '2026-10-21', state: 'on_loan' },
      { id: 'l-4', title: 'Gödel, Escher, Bach', member: 'Alex Demo', due: '2026-10-09', state: 'on_loan' },
    ],
  },
  'loans.Summary': { view: 'loans.Summary', rows: [{ on_loan: 4, overdue: 1 }] },
  'members.All': {
    view: 'members.All',
    total: 3,
    rows: [
      { name: 'Robin Example', joined: '2024-03-01', loans: 1, standing: 'good' },
      { name: 'Kim Sample', joined: '2025-06-12', loans: 1, standing: 'overdue' },
      { name: 'Sam Placeholder', joined: '2026-01-20', loans: 1, standing: 'good' },
    ],
  },
  'staff.Me': { view: 'staff.Me', rows: [{ id: 's-1', name: 'Example Librarian', email: 'librarian@example.com' }] },
};
