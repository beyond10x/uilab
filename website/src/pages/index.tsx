import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

import styles from './index.module.css';

type Line = {text: string; add?: boolean; key?: boolean};

const SPEC: Line[] = [
  {text: 'pages:', key: true},
  {text: '  overview:', key: true},
  {text: '    kind: dashboard_page'},
  {text: '    sections:', key: true},
  {text: '      - name: recent', key: true},
  {text: '        component: collection'},
  {text: '        reads: {view: loans.All}'},
  {text: '      - name: overdue', add: true},
  {text: '        component: collection', add: true},
  {text: '        reads:', add: true},
  {text: '          view: loans.All', add: true},
  {text: '          params: {state: overdue}', add: true},
  {text: '        columns:', add: true},
  {text: '        - field: title', add: true},
  {text: '        - field: member', add: true},
  {text: '        - field: due', add: true},
];

/**
 * The rows of `examples/library/fixtures/loans.yaml`, all of them: the canvas feeds a new
 * collection every fixture row of its view and does not apply `params`, so the real preview of
 * this proposal shows the same four.
 */
const ROWS = [
  ['The Left Hand of Darkness', 'Robin Example', '2026-10-14'],
  ['A Pattern Language', 'Kim Sample', '2026-10-02'],
  ['Middlemarch', 'Sam Placeholder', '2026-10-21'],
  ['Gödel, Escher, Bach', 'Alex Demo', '2026-10-09'],
];

function Workbench() {
  return (
    <div
      className={styles.bench}
      role="img"
      aria-label="A uilab session: the canvas previews a new overdue table in green while the sidebar shows the spoken instruction and the proposed YAML diff, waiting for Accept or Reject">
      <div className={styles.benchBar}>
        <span className={styles.dots} aria-hidden="true">
          <i />
          <i />
          <i />
        </span>
        <span className={clsx(styles.tab, styles.tabOn)}>UI 1</span>
        <span className={styles.tab}>YAML 2</span>
        <span className={styles.tab}>Docs 3</span>
        <span className={styles.tab}>Components 4</span>
        <span className={styles.benchPath}>library.ui.yaml</span>
      </div>
      <div className={styles.benchBody}>
        <div className={styles.canvas}>
          <div className={styles.canvasNav}>
            <span className={styles.navHead}>Circulation</span>
            <span className={clsx(styles.navItem, styles.navOn)}>Overview</span>
            <span className={styles.navItem}>Loans</span>
            <span className={styles.navHead}>People</span>
            <span className={styles.navItem}>Members</span>
          </div>
          <div className={styles.canvasPage}>
            <span className={styles.crumb}>overview · dashboard_page</span>
            <strong className={styles.pageTitle}>Overview</strong>
            <div className={styles.card}>
              <span className={styles.crumb}>on_loan · metric</span>
              <span className={styles.metric}>4</span>
            </div>
            <div className={clsx(styles.card, styles.cardNew)}>
              <span className={styles.crumb}>overdue · collection · loans.All</span>
              <table className={styles.miniTable}>
                <thead>
                  <tr>
                    <th>title</th>
                    <th>member</th>
                    <th>due</th>
                  </tr>
                </thead>
                <tbody>
                  {ROWS.map((row) => (
                    <tr key={row[0]}>
                      {row.map((cell) => (
                        <td key={cell}>{cell}</td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
        <div className={styles.side}>
          <div className={styles.utterance}>
            <span className={styles.mic} aria-hidden="true" />
            “add a table of overdue loans with title, member and due date”
          </div>
          <div className={styles.proposal}>
            <div className={styles.proposalHead}>
              <b>INSERT</b> <code>page:overview</code> → <code>section:overdue</code>
            </div>
            <pre className={styles.spec}>
              {SPEC.map((line, i) => (
                <span key={i} className={clsx(styles.specLine, line.add && styles.specAdd)}>
                  <span className={styles.gutter}>{line.add ? '+' : ' '}</span>
                  {line.text}
                </span>
              ))}
            </pre>
            <div className={styles.actions}>
              <span className={styles.accept}>
                Accept <kbd>Enter</kbd>
              </span>
              <span className={styles.reject}>
                Reject <kbd>Esc</kbd>
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Hero() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={styles.hero}>
      <div className={styles.heroGrid} aria-hidden="true" />
      <div className={styles.heroInner}>
        <div className={styles.heroCopy}>
          <span className={styles.eyebrow}>
            ess-ui/1 · <span className={styles.eyebrowWide}>browser workbench · </span>reviewed by you
          </span>
          <Heading as="h1" className={styles.heroTitle}>
            Point at the screen.
            <br />
            Say what it needs.
            <br />
            <span className={styles.gradientText}>Review the diff.</span>
          </Heading>
          <p className={styles.heroLead}>{siteConfig.tagline}</p>
          <div className={styles.buttons}>
            <Link className="button button--primary button--lg" to="/docs/getting-started">
              Get started
            </Link>
            <Link className={clsx('button button--lg', styles.ghost)} to="/docs">
              What uilab is
            </Link>
          </div>
          <ul className={styles.facts}>
            <li>Runs on your machine</li>
            <li>Writes one YAML file</li>
            <li>Nothing changes until you accept</li>
          </ul>
        </div>
        <Workbench />
      </div>
    </header>
  );
}

const LOOP = [
  {
    n: '01',
    verb: 'Select',
    keys: ['click'],
    text: 'Pick a node in the tree or on the canvas: a page, a table, a drawer. The agent works there and nowhere else.',
  },
  {
    n: '02',
    verb: 'Say',
    keys: ['Space'],
    text: 'Hold Space and speak, or type. Speech is transcribed on your own machine. For bigger work, give a goal.',
  },
  {
    n: '03',
    verb: 'Review',
    keys: ['diff'],
    text: 'One proposal, already checked: the YAML diff, a green preview on the canvas, and only the findings it brings.',
  },
  {
    n: '04',
    verb: 'Accept',
    keys: ['Enter', 'Esc', 'Ctrl+Z'],
    text: 'Accept writes the file. Reject leaves it untouched. Undo takes an accepted change back out.',
  },
];

function Loop() {
  return (
    <section className={styles.section}>
      <div className={styles.sectionInner}>
        <span className={styles.kicker}>The loop</span>
        <Heading as="h2" className={styles.sectionTitle}>
          Four steps, and you hold two of them
        </Heading>
        <p className={styles.sectionLead}>
          The agent proposes. Selecting and deciding stay with the person at the keyboard — that split
          is written into uilab's own session specification, not left to convention.
        </p>
        <ol className={styles.loop}>
          {LOOP.map((step) => (
            <li key={step.n} className={styles.loopStep}>
              <span className={styles.loopN}>{step.n}</span>
              <strong className={styles.loopVerb}>{step.verb}</strong>
              <span className={styles.loopKeys}>
                {step.keys.map((k) => (
                  <kbd key={k}>{k}</kbd>
                ))}
              </span>
              <p>{step.text}</p>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}

/**
 * A screenshot that opens at full size. The "Open full size" link sits under the image, beside
 * the caption, so it never covers part of the screen it shows.
 */
function Shot({src, alt, caption, className}: {src: string; alt: string; caption?: string; className: string}) {
  const url = useBaseUrl(src);
  return (
    <figure className={className}>
      <a className={styles.shotLink} href={url} target="_blank" rel="noopener" title="Open the full-size image">
        <img src={url} alt={alt} loading="lazy" />
      </a>
      <figcaption className={styles.shotCaption}>
        {caption && <span>{caption}</span>}
        <a
          className={styles.shotOpen}
          href={url}
          target="_blank"
          rel="noopener"
          aria-label={`Open full size: ${caption ?? alt}`}>
          Open full size
        </a>
      </figcaption>
    </figure>
  );
}

function RealScreens() {
  const shots = [
    {src: 'img/screens/canvas.png', label: 'Canvas: the Loans page rendered from the document'},
    {src: 'img/screens/components-proposal.png', label: 'Components: a widget proposed and previewed'},
    {src: 'img/screens/yaml.png', label: 'YAML: the file, with the selection highlighted'},
    {src: 'img/screens/docs.png', label: 'Docs: generated from the document'},
  ];
  return (
    <section className={clsx(styles.section, styles.sectionAlt)}>
      <div className={styles.sectionInner}>
        <span className={styles.kicker}>The real thing</span>
        <Heading as="h2" className={styles.sectionTitle}>
          Screenshots, not mock-ups
        </Heading>
        <p className={styles.sectionLead}>
          Taken from uilab running over the lending-library example that ships in the repository. The
          proposal below came from one typed instruction at <code>page:overview</code>.
        </p>
        <Shot
          className={styles.shotMain}
          src="img/screens/proposal.png"
          alt="uilab with a proposal waiting for review: the INSERT card and diff in the sidebar, the new overdue section highlighted in green on the canvas"
        />
        <div className={styles.shotRow}>
          {shots.map((shot) => (
            <Shot key={shot.src} className={styles.shotSmall} src={shot.src} alt={shot.label} caption={shot.label} />
          ))}
        </div>
      </div>
    </section>
  );
}

const FEATURES = [
  {
    title: 'One file is the record',
    text: 'An ess-ui/1 YAML document, the UI format ESS publishes, in your repository. Accepted changes are written back to it; review them in the pull request like any other diff.',
    to: '/docs/concepts/the-document',
  },
  {
    title: 'Checked before you see it',
    text: 'Names resolve, every page is reachable, opens names a real overlay, columns name real fields. A refused answer never reaches you.',
    to: '/docs/concepts/proposals-and-review',
  },
  {
    title: 'An agent without tools',
    text: 'Its only possible answer is a patch that fits the selected node. It cannot run commands, read your code or write files.',
    to: '/docs/working-with-the-agent',
  },
  {
    title: 'Goals, one step at a time',
    text: 'Type a larger request; the agent plans it in steps (eight at most by default) and proposes each one for your decision. Stop at any point.',
    to: '/docs/concepts/goals',
  },
  {
    title: 'Widgets and the Components tab',
    text: 'Declare reusable components with typed params and a body, preview them with sample data, and let the agent reuse them.',
    to: '/docs/concepts/widgets',
  },
  {
    title: 'Drafts show the data gap',
    text: 'When a screen needs data the model lacks, the agent writes a placeholder read instead of inventing a view. The list becomes your backend hand-off.',
    to: '/docs/concepts/drafts-and-sample-data',
  },
  {
    title: 'Local speech, optional',
    text: 'whisper.cpp on your own hardware, prompted with the names valid at the selected node. Or type — everything works without it.',
    to: '/docs/getting-started',
  },
  {
    title: 'Shared sessions, scriptable',
    text: 'Everyone connected sees the same selection and proposal. uilab op drives a session from a shell or a coding agent.',
    to: '/docs/faq',
  },
];

function Features() {
  return (
    <section className={styles.section}>
      <div className={styles.sectionInner}>
        <span className={styles.kicker}>What you get</span>
        <Heading as="h2" className={styles.sectionTitle}>
          Small, strict, and easy to inspect
        </Heading>
        <div className={styles.grid}>
          {FEATURES.map((f) => (
            <Link key={f.title} to={f.to} className={styles.feature}>
              <strong>{f.title}</strong>
              <p>{f.text}</p>
            </Link>
          ))}
        </div>
      </div>
    </section>
  );
}

function StaysYours() {
  return (
    <section className={clsx(styles.section, styles.sectionAlt)}>
      <div className={styles.sectionInner}>
        <span className={styles.kicker}>For cautious teams</span>
        <Heading as="h2" className={styles.sectionTitle}>
          It sits next to your stack, not inside it
        </Heading>
        <div className={styles.split}>
          <div className={styles.splitCol}>
            <h3>uilab adds</h3>
            <ul>
              <li>a reviewed specification of each screen, in the repository</li>
              <li>agreement on which data, fields and actions a screen shows</li>
              <li>a list of the views and commands the screens need</li>
              <li>a local journal of what was asked, proposed and accepted</li>
            </ul>
          </div>
          <div className={clsx(styles.splitCol, styles.splitKeep)}>
            <h3>You keep</h3>
            <ul>
              <li>your framework, components and design system</li>
              <li>all production code — uilab generates none</li>
              <li>styling, interaction details and accessibility work</li>
              <li>code review, CI and releases, unchanged</li>
            </ul>
          </div>
        </div>
        <p className={styles.more}>
          <Link to="/docs/adopting">Adopting uilab in a front-end team →</Link>
        </p>
      </div>
    </section>
  );
}

function Start() {
  return (
    <section className={styles.section}>
      <div className={clsx(styles.sectionInner, styles.start)}>
        <div>
          <span className={styles.kicker}>Try it</span>
          <Heading as="h2" className={styles.sectionTitle}>
            Five minutes on the example app
          </Heading>
          <p className={styles.sectionLead}>
            No GPU needed: run without speech and type your instructions. Work on a copy or a
            committed file — accepted changes are written back.
          </p>
          <Link className="button button--primary button--lg" to="/docs/getting-started">
            Getting started
          </Link>
        </div>
        <div className={styles.termWrap}>
          <pre className={styles.terminal} tabIndex={0} role="region" aria-label="Commands that run the example">
            <span className={styles.prompt}>$</span> task widget{'\n'}
            <span className={styles.prompt}>$</span> cargo run --release -p uilab-app -- serve \{'\n'}
            {'    '}--doc examples/library/library.ui.yaml --no-stt{'\n'}
            <span className={styles.out}>uilab: http://127.0.0.1:8740 editing examples/library/library.ui.yaml</span>
          </pre>
        </div>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout title="Build UI specifications with an agent, by voice or text" description={siteConfig.tagline as string}>
      <Hero />
      <main>
        <Loop />
        <RealScreens />
        <Features />
        <StaysYours />
        <Start />
      </main>
    </Layout>
  );
}
