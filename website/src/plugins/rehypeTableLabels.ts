/**
 * Gives every body cell of a Markdown table its column header as `data-label`, so narrow screens
 * can show a table as a stacked list (see `custom.css`) without losing what each value means.
 */

type Node = {
  type: string;
  tagName?: string;
  value?: string;
  properties?: Record<string, unknown>;
  children?: Node[];
};

function textOf(node: Node): string {
  if (node.type === 'text') return node.value ?? '';
  return (node.children ?? []).map(textOf).join('');
}

function elements(node: Node, tagName: string): Node[] {
  const found: Node[] = [];
  const walk = (n: Node) => {
    if (n.type === 'element' && n.tagName === tagName) found.push(n);
    (n.children ?? []).forEach(walk);
  };
  (node.children ?? []).forEach(walk);
  return found;
}

function cells(row: Node): Node[] {
  return (row.children ?? []).filter((c) => c.type === 'element' && (c.tagName === 'td' || c.tagName === 'th'));
}

function label(table: Node): void {
  const head = elements(table, 'thead')[0];
  const headRow = head && elements(head, 'tr')[0];
  if (!headRow) return;
  const labels = cells(headRow).map((th) => textOf(th).trim());
  for (const body of elements(table, 'tbody')) {
    for (const row of elements(body, 'tr')) {
      cells(row).forEach((cell, i) => {
        cell.properties = {...cell.properties, dataLabel: labels[i] ?? ''};
      });
    }
  }
}

export default function rehypeTableLabels() {
  return (tree: Node) => {
    for (const table of elements(tree, 'table')) label(table);
    if (tree.type === 'element' && tree.tagName === 'table') label(tree);
  };
}
