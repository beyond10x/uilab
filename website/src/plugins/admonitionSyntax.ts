import {readdirSync, readFileSync, statSync} from 'node:fs';
import {join, relative} from 'node:path';

/**
 * Fails the build on an admonition title written in the MDX 1 form `:::note Title`. With
 * `future.v4` that form is not parsed, and the page shows the colons as text; the title belongs in
 * brackets, `:::note[Title]`.
 */

const OLD_TITLE = /^\s*:::[a-z]+[ \t]+\S/;

function markdownFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return markdownFiles(path);
    return /\.mdx?$/.test(name) ? [path] : [];
  });
}

export function oldAdmonitionTitles(dir: string): string[] {
  return markdownFiles(dir).flatMap((file) =>
    readFileSync(file, 'utf8')
      .split('\n')
      .flatMap((line, i) => (OLD_TITLE.test(line) ? [`${relative(dir, file)}:${i + 1}: ${line.trim()}`] : [])),
  );
}

export default function admonitionSyntax(context: {siteDir: string}) {
  return {
    name: 'uilab-admonition-syntax',
    async loadContent() {
      const found = oldAdmonitionTitles(join(context.siteDir, 'docs'));
      if (found.length) {
        throw new Error(
          `admonition titles in the MDX 1 form render as raw text; write :::type[Title]:\n  ${found.join('\n  ')}`,
        );
      }
    },
  };
}
