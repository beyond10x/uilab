/** The sample rate the server transcribes at. */
export const WIRE_RATE = 16000;

/** Samples per binary frame: 100 ms at 16 kHz. */
export const FRAME_SAMPLES = 1600;

/**
 * Streaming downsampler. Each output sample is the mean of the input samples it covers, which
 * low-passes enough for speech at the 48 kHz → 16 kHz ratio. Chunk boundaries do not change the
 * result: pushing a signal in pieces yields the same samples as pushing it whole.
 */
export class Downsampler {
  private readonly ratio: number;
  private pending: number[] = [];
  /** Absolute input index of `pending[0]`. */
  private base = 0;
  /** Output samples emitted so far. */
  private emitted = 0;

  constructor(inRate: number, outRate = WIRE_RATE) {
    if (!(inRate > 0) || !(outRate > 0)) throw new RangeError('sample rates must be positive');
    if (outRate > inRate) throw new RangeError('cannot upsample');
    this.ratio = inRate / outRate;
  }

  push(input: Float32Array): Float32Array {
    for (let k = 0; k < input.length; k++) this.pending.push(input[k]);
    const out: number[] = [];
    const available = this.base + this.pending.length;
    for (;;) {
      const lo = Math.floor(this.emitted * this.ratio);
      const hi = Math.floor((this.emitted + 1) * this.ratio);
      if (hi > available) break;
      let sum = 0;
      for (let idx = lo; idx < hi; idx++) sum += this.pending[idx - this.base];
      out.push(clamp(sum / (hi - lo)));
      this.emitted++;
    }
    const keepFrom = Math.floor(this.emitted * this.ratio) - this.base;
    if (keepFrom > 0) {
      this.pending = this.pending.slice(keepFrom);
      this.base += keepFrom;
    }
    return Float32Array.from(out);
  }
}

function clamp(x: number): number {
  return x > 1 ? 1 : x < -1 ? -1 : x;
}

/** Collects samples into fixed-size frames. */
export class Framer {
  private buf: number[] = [];
  private readonly size: number;

  constructor(size = FRAME_SAMPLES) {
    this.size = size;
  }

  /** Every whole frame the new samples complete. */
  push(samples: Float32Array): Float32Array[] {
    for (let k = 0; k < samples.length; k++) this.buf.push(samples[k]);
    const frames: Float32Array[] = [];
    while (this.buf.length >= this.size) {
      frames.push(Float32Array.from(this.buf.slice(0, this.size)));
      this.buf = this.buf.slice(this.size);
    }
    return frames;
  }

  /** The partial frame left over, or `null` when none; empties the buffer. */
  flush(): Float32Array | null {
    if (this.buf.length === 0) return null;
    const rest = Float32Array.from(this.buf);
    this.buf = [];
    return rest;
  }
}

/** A frame as the wire carries it: little-endian IEEE-754 float32, independent of the host. */
export function encodeFrame(samples: Float32Array): ArrayBuffer {
  const buf = new ArrayBuffer(samples.length * 4);
  const view = new DataView(buf);
  for (let k = 0; k < samples.length; k++) view.setFloat32(k * 4, samples[k], true);
  return buf;
}

/** The inverse of {@link encodeFrame}. */
export function decodeFrame(buf: ArrayBuffer): Float32Array {
  const view = new DataView(buf);
  const out = new Float32Array(buf.byteLength / 4);
  for (let k = 0; k < out.length; k++) out[k] = view.getFloat32(k * 4, true);
  return out;
}

/** Root-mean-square level of a block. */
export function rms(samples: Float32Array): number {
  if (samples.length === 0) return 0;
  let sum = 0;
  for (let k = 0; k < samples.length; k++) sum += samples[k] * samples[k];
  return Math.sqrt(sum / samples.length);
}

/** A level in [0, 1] for a meter: -60 dBFS and below is 0, 0 dBFS is 1. */
export function meterLevel(rmsValue: number): number {
  if (rmsValue <= 0) return 0;
  const db = 20 * Math.log10(rmsValue);
  return Math.min(1, Math.max(0, (db + 60) / 60));
}
