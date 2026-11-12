// Type definitions for weave.js
export type Preset = 'fast' | 'default' | 'expressive' | 'gentle';

export interface SpringHandle {
  /** Current value. */ readonly value: number;
  /** Destination. */ readonly target: number;
  /** Units per second. */ readonly velocity: number;
  /** True while moving. */ readonly moving: boolean;
  /** Animate to `target`; calling again mid-flight keeps momentum. Resolves at rest. */
  to(target: number): Promise<number>;
  /** Jump without animating. */ set(value: number): this;
  /** Listen to every new value. */ onChange(fn: (value: number) => void): this;
  /** Stop where it is. */ stop(): this;
  /** Free it. */ destroy(): void;
}

export interface SpringOptions { preset?: Preset; onChange?: (value: number) => void; }

export interface AnimateProps { x?: number; y?: number; scale?: number; rotate?: number; opacity?: number; }
export interface AnimateOptions { preset?: Preset; from?: AnimateProps; delay?: number; stagger?: number; }
type Targets = Element | string | ArrayLike<Element> | { style: object } | ArrayLike<{ style: object }>;

/** Low-level handle API, identical to the wasm `Motion` of weave-web. */
export class Motion {
  constructor(reduced?: boolean);
  setReduced(reduced: boolean): void;
  spring(value: number, preset: Preset | string): number;
  release(id: number): void;
  setTarget(id: number, target: number): void;
  snap(id: number, value: number): void;
  value(id: number): number;
  velocity(id: number): number;
  target(id: number): number;
  isActive(id: number): boolean;
  advance(dt: number): boolean;
}
export interface MotionEngine extends Pick<Motion, 'setReduced' | 'spring' | 'release' | 'setTarget' | 'snap' | 'value' | 'velocity' | 'target' | 'isActive' | 'advance'> {}

export interface SpringConfig { mass: number; stiffness: number; dampingRatio: number; positionEpsilon: number; velocityEpsilon: number; }
export class Spring {
  constructor(value: number, config: SpringConfig, target?: number, velocity?: number);
  value: number; velocity: number; target: number; config: SpringConfig;
  setTarget(target: number): void; snap(value: number): void; settle(): void;
  isSettled(): boolean; isActive(): boolean; advance(dt: number): boolean;
}

export function spring(value?: number, onChange?: ((value: number) => void) | SpringOptions, options?: SpringOptions): SpringHandle;
export function animate(targets: Targets, props: AnimateProps, options?: AnimateOptions): Promise<void>;
export function enter(targets: Targets, options?: AnimateOptions & { distance?: number }): Promise<void>;
export function after(seconds: number, fn: () => void): void;
export function tick(dt: number): void;
export function reducedMotion(value: boolean | 'auto'): void;
export function isReducedMotion(): boolean;
export function manual(on: boolean): void;
export function useEngine<E extends MotionEngine>(engine: E): E;
export function solve(config: SpringConfig, value: number, velocity: number, target: number, dt: number): [number, number];
export function springConfig(mass: number, stiffness: number, dampingRatio: number): SpringConfig;
export function presetConfig(name: Preset | string): SpringConfig;
export function applyPolicy(config: SpringConfig, policy: 'full' | 'reduced' | 'off'): SpringConfig;
export const PRESETS: readonly Preset[];
export const version: string;
