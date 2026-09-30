import { test } from 'node:test';
import assert from 'node:assert/strict';
import { Downsampler, Framer, encodeFrame, decodeFrame, meterLevel, rms, FRAME_SAMPLES } from './audio.ts';
import { backoffDelay } from './backoff.ts';

function sine(freq: number, rate: number, n: number, amp = 0.5): Float32Array {
  const out = new Float32Array(n);
  for (let k = 0; k < n; k++) out[k] = amp * Math.sin((2 * Math.PI * freq * k) / rate);
  return out;
}

test('48 kHz to 16 kHz yields a third of the samples', () => {
  const out = new Downsampler(48000).push(new Float32Array(48000));
  assert.equal(out.length, 16000);
});

test('each output sample is the mean of the three inputs it covers', () => {
  const out = new Downsampler(48000).push(Float32Array.from([0, 0.3, 0.6, 1, 1, 1, -0.3, -0.3, 0]));
  assert.deepEqual(Array.from(out).map((x) => Math.round(x * 1000) / 1000), [0.3, 1, -0.2]);
});

test('chunk boundaries do not change the output', () => {
  const signal = sine(440, 48000, 4800);
  const whole = new Downsampler(48000).push(signal);
  const pieces = new Downsampler(48000);
  const parts: number[] = [];
  for (let at = 0; at < signal.length; at += 128) parts.push(...pieces.push(signal.subarray(at, at + 128)));
  assert.equal(parts.length, whole.length);
  for (let k = 0; k < whole.length; k++) assert.ok(Math.abs(parts[k] - whole[k]) < 1e-7, `sample ${k}`);
});

test('a speech-band tone keeps its level; 44.1 kHz input is also handled', () => {
  for (const rate of [48000, 44100]) {
    const out = new Downsampler(rate).push(sine(300, rate, rate));
    assert.ok(Math.abs(out.length - 16000) <= 1, `length at ${rate}: ${out.length}`);
    const level = rms(out.subarray(100));
    assert.ok(Math.abs(level - 0.5 / Math.SQRT2) < 0.01, `rms at ${rate}: ${level}`);
  }
});

test('output is clamped to [-1, 1]', () => {
  const out = new Downsampler(48000).push(Float32Array.from([2, 2, 2, -3, -3, -3]));
  assert.deepEqual(Array.from(out), [1, -1]);
});

test('upsampling is refused', () => {
  assert.throws(() => new Downsampler(8000));
});

test('framer emits 1600-sample frames and flushes the rest', () => {
  const framer = new Framer();
  assert.equal(framer.push(new Float32Array(1000)).length, 0);
  const frames = framer.push(new Float32Array(2500));
  assert.equal(frames.length, 2);
  assert.ok(frames.every((f) => f.length === FRAME_SAMPLES));
  assert.equal(framer.flush()?.length, 300);
  assert.equal(framer.flush(), null);
});

test('frames are little-endian float32', () => {
  const buf = encodeFrame(Float32Array.from([1, -0.5, 0]));
  assert.equal(buf.byteLength, 12);
  assert.deepEqual(Array.from(new Uint8Array(buf)), [0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x00, 0xbf, 0, 0, 0, 0]);
  assert.deepEqual(Array.from(decodeFrame(buf)), [1, -0.5, 0]);
});

test('meter level maps -60..0 dBFS to 0..1', () => {
  assert.equal(meterLevel(0), 0);
  assert.equal(meterLevel(1), 1);
  assert.ok(Math.abs(meterLevel(0.001) - 0) < 1e-9);
  assert.ok(Math.abs(meterLevel(0.0316227766) - 0.5) < 1e-6);
});

test('reconnect backoff doubles from 500 ms and caps at 10 s', () => {
  assert.deepEqual([0, 1, 2, 3, 4, 5, 9].map((n) => backoffDelay(n)), [500, 1000, 2000, 4000, 8000, 10000, 10000]);
});
