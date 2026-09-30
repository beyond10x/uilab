/** The part of a key event's target these rules read; a window or a bare object has none of it. */
export interface KeyTarget {
  tagName?: string;
  isContentEditable?: boolean;
  getAttribute?(name: string): string | null;
}

const FIELD_TAGS = ['INPUT', 'TEXTAREA', 'SELECT'];
const CONTROL_TAGS = ['BUTTON', 'A', 'SUMMARY'];
const CONTROL_ROLES = ['button', 'link'];

/**
 * Whether Enter, with a proposal waiting, accepts it. A form field or an editable region always
 * keeps Enter. A button, a link or a summary keeps it when the keyboard put focus there (a focused
 * Reject rejects, a focused findings badge opens its list, a summary toggles its details), but not
 * when a mouse click did (`byPointer`) or when it is out of the tab order (`tabindex="-1"`, the
 * canvas preview's inert controls): then focus is incidental and Enter goes to the proposal.
 */
export function enterAccepts(target: KeyTarget | null, byPointer = false): boolean {
  if (!target?.tagName) return true;
  const tag = target.tagName.toUpperCase();
  if (target.isContentEditable || FIELD_TAGS.includes(tag)) return false;
  const role = target.getAttribute?.('role');
  const control = CONTROL_TAGS.includes(tag) || (!!role && CONTROL_ROLES.includes(role));
  if (!control) return true;
  return byPointer || target.getAttribute?.('tabindex') === '-1';
}
