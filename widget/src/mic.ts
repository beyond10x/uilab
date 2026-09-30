import { Downsampler, Framer, encodeFrame, meterLevel, rms } from './lib/audio.ts';

/** Copies the first input channel to the main thread in blocks of 1024 samples. */
const TAP_SOURCE = `
class UilabTap extends AudioWorkletProcessor {
  constructor() { super(); this.buf = new Float32Array(1024); this.n = 0; }
  process(inputs) {
    const ch = inputs[0] && inputs[0][0];
    if (ch) {
      for (let i = 0; i < ch.length; i++) {
        this.buf[this.n++] = ch[i];
        if (this.n === this.buf.length) {
          this.port.postMessage(this.buf, [this.buf.buffer]);
          this.buf = new Float32Array(1024);
          this.n = 0;
        }
      }
    }
    return true;
  }
}
registerProcessor('uilab-tap', UilabTap);
`;

/** How long the worklet module may take to load before capture falls back. */
const WORKLET_LOAD_MS = 2000;

/** Loads the tap worklet; `false` when the browser refuses it or never finishes loading it. */
async function loadTap(ctx: AudioContext): Promise<boolean> {
  if (!ctx.audioWorklet) return false;
  const url = URL.createObjectURL(new Blob([TAP_SOURCE], { type: 'application/javascript' }));
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    const timeout = new Promise<never>((_, reject) => {
      timer = setTimeout(() => reject(new Error('worklet load timed out')), WORKLET_LOAD_MS);
    });
    await Promise.race([ctx.audioWorklet.addModule(url), timeout]);
    return true;
  } catch (err) {
    console.warn('uilab: audio worklet unavailable, capturing with a script processor', err);
    return false;
  } finally {
    clearTimeout(timer);
    URL.revokeObjectURL(url);
  }
}

/** Release the microphone after this long without a hold. */
const IDLE_RELEASE_MS = 30_000;

export interface MicSink {
  /** One binary frame of 16 kHz mono little-endian float32. */
  frame(buf: ArrayBuffer): void;
  /** Input level in [0, 1], while streaming. */
  level(value: number): void;
}

/**
 * Microphone capture. The stream stays open between holds so the first word of the next hold is
 * not lost to device start-up, and is released after 30 s idle.
 */
export class MicCapture {
  private ctx: AudioContext | null = null;
  private stream: MediaStream | null = null;
  private downsampler: Downsampler | null = null;
  private framer = new Framer();
  private streaming = false;
  private idleTimer: ReturnType<typeof setTimeout> | undefined;
  private readonly sink: MicSink;

  constructor(sink: MicSink) {
    this.sink = sink;
  }

  /** Opens the device if needed. Throws when the browser or the operator refuses. */
  async ensure(): Promise<void> {
    clearTimeout(this.idleTimer);
    if (this.ctx && this.stream) {
      if (this.ctx.state === 'suspended') await this.ctx.resume();
      return;
    }
    const ctx = new AudioContext();
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: { channelCount: 1, echoCancellation: true, noiseSuppression: true, autoGainControl: true },
      });
      const source = ctx.createMediaStreamSource(stream);
      const mute = ctx.createGain();
      mute.gain.value = 0;
      mute.connect(ctx.destination);
      if (await loadTap(ctx)) {
        const tap = new AudioWorkletNode(ctx, 'uilab-tap', { numberOfOutputs: 1 });
        tap.port.onmessage = (ev: MessageEvent<Float32Array>) => this.onBlock(ev.data);
        source.connect(tap).connect(mute);
      } else {
        // Deprecated but universally available; used only when the worklet cannot load.
        const proc = ctx.createScriptProcessor(1024, 1, 1);
        proc.onaudioprocess = (ev) => this.onBlock(ev.inputBuffer.getChannelData(0).slice());
        source.connect(proc).connect(mute);
      }
      if (ctx.state === 'suspended') await ctx.resume();
      this.ctx = ctx;
      this.stream = stream;
    } catch (err) {
      await ctx.close().catch(() => {});
      throw err;
    }
  }

  /** Starts forwarding frames. */
  begin(): void {
    clearTimeout(this.idleTimer);
    this.downsampler = new Downsampler(this.ctx!.sampleRate);
    this.framer = new Framer();
    this.streaming = true;
  }

  /** Stops forwarding and sends the partial last frame. */
  end(): void {
    if (!this.streaming) return;
    this.streaming = false;
    const rest = this.framer.flush();
    if (rest) this.sink.frame(encodeFrame(rest));
    this.sink.level(0);
    this.idleTimer = setTimeout(() => this.release(), IDLE_RELEASE_MS);
  }

  private onBlock(block: Float32Array): void {
    if (!this.streaming || !this.downsampler) return;
    this.sink.level(meterLevel(rms(block)));
    for (const frame of this.framer.push(this.downsampler.push(block))) this.sink.frame(encodeFrame(frame));
  }

  private release(): void {
    this.stream?.getTracks().forEach((t) => t.stop());
    this.ctx?.close().catch(() => {});
    this.stream = null;
    this.ctx = null;
  }
}
