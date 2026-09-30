/** The part of a key event's target these rules read; a window or a bare object has none of it. */
export interface KeyTarget {
  tagName?: string;
  isContentEditable?: boolean;
  getAttribute?(name: string): string | null;
}

const OWN_ENTER_TAGS = ['BUTTON', 'A', 'INPUT', 'TEXTAREA', 'SELECT'];
const OWN_ENTER_ROLES = ['button', 'link'];

/**
 * Whether Enter, with a proposal waiting, accepts it. Not when keyboard focus is on a button, a
 * link, a form field or an editable region: Enter there belongs to that element (a focused Reject
 * rejects, a focused findings badge opens its list).
 */
export function enterAccepts(target: KeyTarget | null): boolean {
  if (!target?.tagName) return true;
  if (target.isContentEditable || OWN_ENTER_TAGS.includes(target.tagName.toUpperCase())) return false;
  const role = target.getAttribute?.('role');
  return !(role && OWN_ENTER_ROLES.includes(role));
}
