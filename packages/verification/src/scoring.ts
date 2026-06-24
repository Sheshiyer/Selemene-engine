import type { FieldOutcome } from './types.js';

export function compareField(expected: unknown, actual: unknown, weight: number): FieldOutcome {
  if (!Number.isFinite(weight) || weight < 0) {
    throw new Error(`weight must be a finite non-negative number, got ${weight}`);
  }
  return {
    pass: deepEqual(expected, actual),
    expected,
    actual,
    weight,
  };
}

export function calculateAccuracy(fields: Record<string, FieldOutcome>): number {
  const totalWeight = Object.values(fields).reduce((sum, f) => sum + f.weight, 0);
  if (totalWeight === 0) return 0;
  const correctWeight = Object.values(fields)
    .filter((f) => f.pass)
    .reduce((sum, f) => sum + f.weight, 0);
  return correctWeight / totalWeight;
}

function deepEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (typeof a !== typeof b) return false;
  if (a === null || b === null) return false;
  if (a instanceof Date || b instanceof Date) {
    return a instanceof Date && b instanceof Date && a.getTime() === b.getTime();
  }
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false;
    return a.every((v, i) => deepEqual(v, b[i]));
  }
  if (typeof a === 'object' && typeof b === 'object') {
    const aKeys = Object.keys(a as object);
    const bKeys = Object.keys(b as object);
    if (aKeys.length !== bKeys.length) return false;
    return aKeys.every((k) => deepEqual((a as Record<string, unknown>)[k], (b as Record<string, unknown>)[k]));
  }
  return false;
}
