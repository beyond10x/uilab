import type { UilabWireClientMessage, UilabWireServerMessage } from './generated/types.ts';
import { backoffDelay } from './lib/backoff.ts';

export type ConnState = 'connecting' | 'open' | 'closed';

/** The one channel to the server: JSON messages both ways, binary audio frames up. */
export interface Transport {
  start(handlers: TransportHandlers): void;
  /** Sends a message; `false` when the channel is not open. */
  send(message: UilabWireClientMessage): boolean;
  /** Sends one binary audio frame; `false` when the channel is not open. */
  sendBinary(frame: ArrayBuffer): boolean;
}

export interface TransportHandlers {
  message(message: UilabWireServerMessage): void;
  state(state: ConnState, retryInMs?: number): void;
}

/** `ws(s)://<page host>/ws`. */
export function socketUrl(loc: Location = window.location): string {
  return `${loc.protocol === 'https:' ? 'wss' : 'ws'}://${loc.host}/ws`;
}

/** A WebSocket that reconnects with exponential backoff. */
export class WsTransport implements Transport {
  private ws: WebSocket | null = null;
  private attempt = 0;
  private handlers: TransportHandlers | null = null;
  private readonly url: string;

  constructor(url: string) {
    this.url = url;
  }

  start(handlers: TransportHandlers): void {
    this.handlers = handlers;
    this.connect();
  }

  private connect(): void {
    const h = this.handlers!;
    h.state('connecting');
    const ws = new WebSocket(this.url);
    ws.binaryType = 'arraybuffer';
    this.ws = ws;
    ws.onopen = () => {
      this.attempt = 0;
      h.state('open');
    };
    ws.onmessage = (ev) => {
      if (typeof ev.data !== 'string') return;
      let parsed: UilabWireServerMessage;
      try {
        parsed = JSON.parse(ev.data) as UilabWireServerMessage;
      } catch {
        console.warn('uilab: a server message is not JSON', ev.data);
        return;
      }
      h.message(parsed);
    };
    ws.onclose = () => {
      if (this.ws !== ws) return;
      this.ws = null;
      const delay = backoffDelay(this.attempt++, Math.random());
      h.state('closed', delay);
      setTimeout(() => this.connect(), delay);
    };
  }

  send(message: UilabWireClientMessage): boolean {
    if (this.ws?.readyState !== WebSocket.OPEN) return false;
    this.ws.send(JSON.stringify(message));
    return true;
  }

  sendBinary(frame: ArrayBuffer): boolean {
    if (this.ws?.readyState !== WebSocket.OPEN) return false;
    this.ws.send(frame);
    return true;
  }
}
