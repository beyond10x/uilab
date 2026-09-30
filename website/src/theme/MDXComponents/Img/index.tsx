import type {ComponentProps, ReactNode} from 'react';
import Img from '@theme-original/MDXComponents/Img';

/**
 * Screenshots are taken at 2880 px and shown at the width of the text column, so each one links to
 * itself at full size.
 */
export default function MDXImg(props: ComponentProps<typeof Img>): ReactNode {
  const src = typeof props.src === 'string' ? props.src : undefined;
  if (!src) return <Img {...props} />;
  return (
    <a className="ul-shot" href={src} target="_blank" rel="noopener" title="Open the full-size image">
      <Img {...props} />
      <span className="ul-shot__open" aria-hidden="true">
        Open full size
      </span>
    </a>
  );
}
