/** Reconnect delay after `attempt` consecutive failures: exponential from 500 ms, capped at 10 s. */
export function backoffDelay(attempt: number, jitter = 0): number {
  const base = Math.min(10000, 500 * 2 ** Math.max(0, attempt));
  return Math.round(base * (1 + 0.25 * jitter));
}
