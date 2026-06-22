import { describe, it, expect } from 'vitest';

describe('witness-pipeline workspace', () => {
  it('has the expected package name', () => {
    const pkg = { name: '@noesis/witness-pipeline' };
    expect(pkg.name).toBe('@noesis/witness-pipeline');
  });
});
