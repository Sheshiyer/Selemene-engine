// Minimal ambient typing for `@strudel/web`. Only types the surface we use.

declare module '@strudel/web' {
  /** Initialize Strudel + unlock the AudioContext. Call inside a user gesture. */
  export function initStrudel(opts?: { prebake?: () => void | Promise<void> }): Promise<void>;
  /** Evaluate a Strudel mini-notation/JS string. The transpiler resolves
   *  globals like `setcps`, `freq`, `s`, `slow`, etc. at parse time. */
  export function evaluate(code: string): Promise<unknown>;
  /** Stop all sound. */
  export function hush(): void;
}
