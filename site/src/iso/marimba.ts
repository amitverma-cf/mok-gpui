import { useSyncExternalStore } from "react";

/** A sixteenth at 132 beats a minute. */
export const BEAT = 60_000 / 132 / 4;

// C major pentatonic, C4 to E6.
const SCALE = [0, 2, 4, 7, 9, 12, 14, 16, 19, 21, 24, 26, 28];
const LISTENERS = new Set<(on: boolean) => void>();

let context: AudioContext | null = null;
let bus: GainNode | null = null;
let on = false;
let last = 0;

export function soundOn(): boolean {
  return on;
}

/** Turns sound on or off; browsers need a press first. */
export function setSound(next: boolean): void {
  on = next;
  if (on && !context) {
    context = new AudioContext();
    bus = context.createGain();
    bus.gain.value = 0.28;
    const squeeze = context.createDynamicsCompressor();
    bus.connect(squeeze).connect(context.destination);
    console.info("marimba: audio started");
  }
  if (on) void context!.resume();
  LISTENERS.forEach((listen) => listen(on));
}

export function onSound(listen: (on: boolean) => void): () => void {
  LISTENERS.add(listen);
  return () => LISTENERS.delete(listen);
}

/** Strikes the scale's `step`th note; strikes within 30 ms merge. */
export function strike(step: number, velocity = 0.7): void {
  if (!on || !context || !bus) return;
  const now = context.currentTime;
  if (now - last < 0.03) return;
  last = now;
  const index = ((step % SCALE.length) + SCALE.length) % SCALE.length;
  const hz = 261.63 * 2 ** (SCALE[index] / 12);
  // A struck bar: a fundamental, a partial, a click.
  for (const [ratio, gain, decay] of [
    [1, 1, 0.9],
    [3.93, 0.28, 0.18],
    [9.2, 0.06, 0.05],
  ]) {
    const tone = context.createOscillator();
    const env = context.createGain();
    tone.type = "sine";
    tone.frequency.value = hz * ratio;
    env.gain.setValueAtTime(0, now);
    env.gain.linearRampToValueAtTime(gain * velocity, now + 0.004);
    env.gain.exponentialRampToValueAtTime(0.0001, now + decay);
    tone.connect(env).connect(bus);
    tone.start(now);
    tone.stop(now + decay + 0.05);
  }
}

/** Whether sound is on, redrawn when it turns. */
export function useSound(): boolean {
  return useSyncExternalStore((listen) => onSound(listen), soundOn);
}
