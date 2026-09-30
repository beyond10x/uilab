// Development-only stand-in for the server, used with `pnpm dev` and `?mock`. It answers every
// client message from the fixture and fakes a proposal per instruction:
//   anything        → Insert an "overdue loans" collection on the page of the selection
//   "remove …"      → Remove the selected section
//   "rename …"      → Replace the selected section's title
//   "refuse …"      → refused;  "fail …" → failed
// A second operator, the agent "Claude", acts on instructions that start with "claude":
//   "claude …"        → thinks on the selection, then applies an Insert as `changed`
//   "claude remove …" → Remove of the selected section;  "claude refuse …" → refused
//   "claude gap …"    → a `changed` that skips a revision, so the app must resync
// window.__uilabMock records what the app sent, including the binary audio frames.
import type {
  UilabWireClientMessage as ClientMessage,
  UilabWireDocumentState as DocumentState,
  UilabWireFinding as Finding,
  UilabWireOperator as Operator,
  UilabWireOutlineNode as OutlineNode,
  UilabWireProposalShown as ProposalShown,
  UilabWireServerMessage as ServerMessage,
} from '../src/generated/types.ts';
import { findNode, pageOf, parentPath } from '../src/lib/outline.ts';
import type { Transport, TransportHandlers } from '../src/transport.ts';
import { libraryOutline, libraryRows } from './fixture.ts';

interface MockLog {
  sent: ClientMessage[];
  frames: number;
  samples: number;
  frameBytes: number[];
  /** performance.now() of each mic open and closed sent. */
  micAt: number[];
}

declare global {
  interface Window {
    __uilabMock?: MockLog;
  }
}

const SPOKEN = 'add a table of overdue loans under the loans page';
const HUMAN_ID = 'op-human-1';
const AGENT_ID = 'op-agent-1';

export class MockTransport implements Transport {
  private h: TransportHandlers | null = null;
  private doc: DocumentState = {
    document_id: 'mock-library',
    file: 'examples/library/library.ui.yaml',
    title: 'Lending library',
    selected: 'page:overview',
    outline: libraryOutline(),
    findings: [],
    revision: 0,
  };
  private operators: Operator[] = [{ id: AGENT_ID, name: 'Claude', kind: 'agent', last_seen_ms: 0 }];
  private selectedBy: string | undefined;
  private pending: { proposal: ProposalShown; outline: OutlineNode; findings: Finding[] } | null = null;
  private undoStack: { id: string; outline: OutlineNode; findings: Finding[] }[] = [];
  private seq = 0;
  private micSamples = 0;
  private readonly log: MockLog = { sent: [], frames: 0, samples: 0, frameBytes: [], micAt: [] };

  start(handlers: TransportHandlers): void {
    this.h = handlers;
    window.__uilabMock = this.log;
    handlers.state('connecting');
    setTimeout(() => {
      handlers.state('open');
      this.emit({ type: 'document', value: this.doc });
    }, 150);
  }

  sendBinary(frame: ArrayBuffer): boolean {
    this.log.frames++;
    this.log.samples += frame.byteLength / 4;
    this.log.frameBytes.push(frame.byteLength);
    this.micSamples += frame.byteLength / 4;
    return true;
  }

  send(msg: ClientMessage): boolean {
    this.log.sent.push(msg);
    if (msg.type === 'mic') this.log.micAt.push(performance.now());
    setTimeout(() => this.handle(msg), 30);
    return true;
  }

  private emit(msg: ServerMessage): void {
    this.h?.message(structuredClone(msg));
  }

  private emitDocument(): void {
    this.doc.undoable = this.undoStack.at(-1)?.id;
    this.emit({ type: 'document', value: this.doc });
  }

  private emitPresence(): void {
    this.emit({ type: 'presence', value: { operators: this.operators, selected_by: this.selectedBy } });
  }

  private handle(msg: ClientMessage): void {
    switch (msg.type) {
      case 'hello':
        this.operators = [{ id: HUMAN_ID, name: msg.value.name, kind: msg.value.kind, last_seen_ms: 0 }, ...this.operators.filter((o) => o.id !== HUMAN_ID)];
        this.emitPresence();
        break;
      case 'resync':
        this.emitDocument();
        break;
      case 'select':
        this.doc.selected = msg.value.path;
        this.selectedBy = HUMAN_ID;
        this.emitDocument();
        this.emitPresence();
        break;
      case 'rows': {
        const rows = libraryRows[msg.value.view] ?? { view: msg.value.view, total: 0, rows: [] };
        this.emit({ type: 'rows', value: rows });
        break;
      }
      case 'mic':
        if (msg.value.state === 'open') this.micSamples = 0;
        else {
          const audioMs = Math.round(this.micSamples / 16);
          setTimeout(() => {
            this.emit({ type: 'transcript', value: { text: audioMs > 0 ? SPOKEN : '', audio_ms: audioMs, took_ms: 180 } });
            if (audioMs > 0) this.instruct(SPOKEN);
          }, 400);
        }
        break;
      case 'say':
        if (/^claude\b/i.test(msg.value.text)) this.agent(msg.value.text.toLowerCase(), msg.value.target ?? this.doc.selected);
        else this.instruct(msg.value.text);
        break;
      case 'accept':
        if (this.pending?.proposal.proposal_id === msg.value.proposal_id) {
          this.undoStack.push({ id: msg.value.proposal_id, outline: this.doc.outline, findings: this.doc.findings });
          this.doc.outline = this.pending.outline;
          this.doc.revision++;
          this.doc.findings = this.pending.findings;
          this.doc.selected = findNode(this.doc.outline, this.pending.proposal.changed) ? this.pending.proposal.changed : this.pending.proposal.target;
          this.pending = null;
        }
        this.emitDocument();
        break;
      case 'reject':
        this.pending = null;
        this.emitDocument();
        break;
      case 'undo': {
        const top = this.undoStack.at(-1);
        if (top && top.id === msg.value.proposal_id) {
          this.undoStack.pop();
          this.doc.outline = top.outline;
          this.doc.revision++;
          this.doc.findings = top.findings;
          if (!findNode(this.doc.outline, this.doc.selected)) this.doc.selected = '/';
        }
        this.emitDocument();
        break;
      }
    }
  }

  /** The agent operator: thinks on `target`, then changes the document directly. */
  private agent(text: string, target: string): void {
    this.emit({ type: 'transcript', value: { by: AGENT_ID, text, audio_ms: 0, took_ms: 0 } });
    this.emit({ type: 'thinking', value: { by: AGENT_ID, target } });
    setTimeout(() => {
      if (text.includes('refuse')) {
        this.emit({ type: 'refused', value: { by: AGENT_ID, check: 'layer.misplaced', message: `a section cannot sit under ${target}` } });
        return;
      }
      const outline = structuredClone(this.doc.outline);
      let change: { op: 'Insert' | 'Remove'; changed: string; parent: string; node?: OutlineNode };
      if (text.includes('remove')) {
        const node = findNode(outline, target);
        const parent = node && node.layer === 'section' ? findNode(outline, parentPath(node.path) ?? '/') : null;
        if (!node || !parent) {
          this.emit({ type: 'failed', value: { by: AGENT_ID, message: 'select a section for the agent to remove' } });
          return;
        }
        parent.children = parent.children.filter((c) => c.path !== node.path);
        change = { op: 'Remove', changed: node.path, parent: parent.path };
      } else {
        const pagePath = pageOf(target) ?? 'page:overview';
        const page = findNode(outline, pagePath)!;
        let name = 'agent_note';
        for (let k = 2; page.children.some((c) => c.name === name); k++) name = `agent_note_${k}`;
        const node: OutlineNode = { path: `${pagePath}/section:${name}`, layer: 'section', name, kind: 'text', title: 'Added by the agent', children: [] };
        page.children.push(node);
        change = { op: 'Insert', changed: node.path, parent: pagePath, node };
      }
      this.doc.outline = outline;
      this.doc.revision += text.includes('gap') ? 2 : 1;
      this.emit({ type: 'changed', value: { by: AGENT_ID, revision: this.doc.revision, findings: this.doc.findings, ...change } });
    }, 1200);
  }

  private instruct(text: string): void {
    const target = this.doc.selected;
    this.emit({ type: 'thinking', value: { target } });
    setTimeout(() => {
      const lower = text.toLowerCase();
      if (lower.includes('refuse')) {
        this.emit({ type: 'refused', value: { check: 'layer.misplaced', message: `a section cannot sit under ${target}` } });
      } else if (lower.includes('fail')) {
        this.emit({ type: 'failed', value: { message: 'the agent returned no patch' } });
      } else if (lower.includes('remove') || lower.includes('delete')) {
        this.propose(text, 'Remove');
      } else if (lower.includes('rename') || lower.includes('replace')) {
        this.propose(text, 'Replace');
      } else {
        this.propose(text, 'Insert');
      }
    }, 700);
  }

  private propose(utterance: string, op: 'Insert' | 'Replace' | 'Remove'): void {
    const outline = structuredClone(this.doc.outline);
    const selected = this.doc.selected;
    let target: string;
    let changed: string;
    let findings: Finding[] = [...this.doc.findings];
    let beforeNode: unknown;
    let afterNode: unknown;
    if (op === 'Insert') {
      target = pageOf(selected) ?? 'page:loans';
      const page = findNode(outline, target)!;
      beforeNode = pageYaml(page);
      let name = 'overdue';
      for (let k = 2; page.children.some((c) => c.name === name); k++) name = `overdue_${k}`;
      changed = `${target}/section:${name}`;
      const props = { title: 'Overdue loans', columns: [{ field: 'title' }, { field: 'member' }, { field: 'due' }] };
      const at = page.children.findIndex((c) => c.layer !== 'section');
      page.children.splice(at < 0 ? page.children.length : at, 0, {
        path: changed,
        layer: 'section',
        name,
        kind: 'collection',
        title: 'Overdue loans',
        view: 'draft.loans.Overdue',
        props,
        children: [],
      });
      afterNode = pageYaml(page);
      findings.push({ check: 'view.draft', severity: 'warning', path: changed, message: 'reads draft.loans.Overdue, which no model view backs yet' });
    } else {
      const node = findNode(outline, selected);
      const section = node && ['section', 'overlay', 'widget', 'item'].includes(node.layer) ? node : findNode(outline, 'page:overview/section:recent')!;
      target = section.path;
      changed = section.path;
      if (op === 'Replace') {
        beforeNode = composite(section);
        section.title = `${section.title ?? section.name} (renamed)`;
        section.props = { ...(section.props as object), title: section.title };
        afterNode = composite(section);
      } else {
        const parent = findNode(outline, parentPath(section.path) ?? '/')!;
        beforeNode = pageYaml(parent);
        parent.children = parent.children.filter((c) => c.path !== section.path);
        afterNode = pageYaml(parent);
        findings = findings.filter((f) => !f.path.startsWith(section.path));
      }
    }
    const proposal: ProposalShown = {
      proposal_id: `p-${++this.seq}`,
      op,
      target,
      changed,
      utterance,
      before: toYaml(beforeNode),
      after: toYaml(afterNode),
      findings: findings.filter((f) => !this.doc.findings.includes(f)),
      outline,
    };
    this.pending = { proposal, outline, findings };
    this.emit({ type: 'proposal', value: proposal });
  }
}

function composite(n: OutlineNode): Record<string, unknown> {
  const kind = n.layer === 'overlay' ? n.kind.split(' ') : [n.kind];
  const out: Record<string, unknown> = kind.length > 1 ? { kind: kind[0], component: kind[1] } : { component: kind[0] };
  if (n.view) out.reads = { view: n.view };
  return { ...out, ...(n.props as Record<string, unknown> | undefined) };
}

function pageYaml(page: OutlineNode): Record<string, unknown> {
  const sections: Record<string, unknown> = {};
  const overlays: Record<string, unknown> = {};
  for (const c of page.children) (c.layer === 'overlay' ? overlays : sections)[c.name] = composite(c);
  const out: Record<string, unknown> = { kind: page.kind, title: page.title, sections };
  if (Object.keys(overlays).length) out.overlays = overlays;
  return out;
}

/** Block YAML for plain data; flow style for arrays of scalars and small objects. */
function toYaml(value: unknown, indent = ''): string {
  const flow = (v: unknown): string =>
    Array.isArray(v)
      ? `[${v.map(flow).join(', ')}]`
      : v && typeof v === 'object'
        ? `{${Object.entries(v).map(([k, x]) => `${k}: ${flow(x)}`).join(', ')}}`
        : String(v);
  if (!value || typeof value !== 'object' || Array.isArray(value)) return `${flow(value)}\n`;
  let out = '';
  for (const [k, v] of Object.entries(value)) {
    if (v === undefined) continue;
    if (v && typeof v === 'object' && !Array.isArray(v) && Object.values(v).some((x) => x && typeof x === 'object' && !Array.isArray(x))) {
      out += `${indent}${k}:\n${toYaml(v, indent + '  ')}`;
    } else if (v && typeof v === 'object' && !Array.isArray(v) && Object.keys(v).length > 2) {
      out += `${indent}${k}:\n${toYaml(v, indent + '  ')}`;
    } else {
      out += `${indent}${k}: ${flow(v)}\n`;
    }
  }
  return out;
}
