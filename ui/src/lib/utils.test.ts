import { describe, it, expect } from 'vitest';
import { cn } from './utils';

describe('cn utility', () => {
  it('merges class names correctly', () => {
    expect(cn('bg-red-500', 'text-white')).toBe('bg-red-500 text-white');
  });

  it('handles conditional classes', () => {
    const isTrue = true;
    const isFalse = false;
    expect(cn('base', isTrue && 'active', isFalse && 'inactive')).toBe('base active');
  });

  it('merges tailwind classes (overriding)', () => {
    // tailwind-merge should resolve conflicts, e.g., p-4 overrides p-2
    expect(cn('p-2', 'p-4')).toBe('p-4');
  });

  it('handles arrays and objects (clsx behavior)', () => {
    expect(cn(['a', 'b'], { c: true, d: false })).toBe('a b c');
  });
});
