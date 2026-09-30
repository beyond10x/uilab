import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import Heading from '@theme/Heading';

import styles from './styles.module.css';

const WAYS = [
  {to: '/docs', title: 'Introduction', text: 'What uilab is, and who it is for.'},
  {to: '/docs/getting-started', title: 'Getting started', text: 'Run the example and make a first change.'},
  {to: '/docs/concepts/the-document', title: 'Concepts', text: 'The document, nodes, proposals, goals.'},
  {to: '/docs/faq', title: 'FAQ', text: 'Short answers to the usual questions.'},
];

export default function NotFoundContent({className}: {className?: string}): ReactNode {
  return (
    <main className={clsx(styles.main, className)}>
      <div className={styles.inner}>
        <span className={styles.code}>404 · no node at this path</span>
        <Heading as="h1" className={styles.title}>
          This page does not exist
        </Heading>
        <p className={styles.lead}>
          The address may be old or mistyped. The documentation starts at one of these:
        </p>
        <ul className={styles.ways}>
          {WAYS.map((way) => (
            <li key={way.to}>
              <Link to={way.to} className={styles.way}>
                <strong>{way.title}</strong>
                <span>{way.text}</span>
              </Link>
            </li>
          ))}
        </ul>
        <p className={styles.home}>
          <Link to="/">Back to the uilab home page →</Link>
        </p>
      </div>
    </main>
  );
}
