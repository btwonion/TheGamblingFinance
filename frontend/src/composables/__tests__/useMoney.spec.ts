import { describe, it, expect } from 'vitest';
import { useMoney } from '../useMoney';

// `Intl.NumberFormat('de-DE', {currency:'EUR'})` uses a non-breaking
// space between number and symbol. Matchers below are tolerant of that.
const NBSP = ' ';

describe('useMoney', () => {
  const money = useMoney();

  describe('format', () => {
    it('formats zero as 0,00 €', () => {
      expect(money.format(0)).toBe(`0,00${NBSP}€`);
    });

    it('formats positive cents as de-DE currency', () => {
      expect(money.format(1250)).toBe(`12,50${NBSP}€`);
      expect(money.format(123456)).toBe(`1.234,56${NBSP}€`);
    });

    it('renders negative with a leading minus', () => {
      expect(money.format(-1250)).toBe(`-12,50${NBSP}€`);
    });

    it('rounds non-integer input to nearest cent', () => {
      expect(money.format(1249.6)).toBe(`12,50${NBSP}€`);
    });

    it('treats NaN / Infinity as zero rather than crashing', () => {
      expect(money.format(NaN)).toBe(`0,00${NBSP}€`);
      expect(money.format(Infinity)).toBe(`0,00${NBSP}€`);
    });
  });

  describe('parse', () => {
    it('parses an empty / whitespace string as 0', () => {
      expect(money.parse('')).toBe(0);
      expect(money.parse('   ')).toBe(0);
    });

    it('parses plain integers as euros (× 100)', () => {
      expect(money.parse('12')).toBe(1200);
      expect(money.parse('1234')).toBe(123400);
    });

    it('parses de-DE decimals with comma', () => {
      expect(money.parse('12,50')).toBe(1250);
      expect(money.parse('0,01')).toBe(1);
    });

    it('strips € and thousands separators', () => {
      expect(money.parse('1.234,56 €')).toBe(123456);
      expect(money.parse(`1.234,56${NBSP}€`)).toBe(123456);
    });

    it('parses a leading minus as a negative number', () => {
      expect(money.parse('-12,50')).toBe(-1250);
      expect(money.parse('-1.234,56 €')).toBe(-123456);
    });

    it('accepts dot as decimal when the fractional part is 1-2 digits', () => {
      expect(money.parse('12.50')).toBe(1250);
    });

    it('treats multiple dots as thousands separators', () => {
      expect(money.parse('1.234.567')).toBe(123456700);
    });

    it('returns 0 for garbage input rather than NaN', () => {
      expect(money.parse('not a number')).toBe(0);
    });
  });

  describe('format ∘ parse round-trips', () => {
    it('round-trips common values', () => {
      for (const cents of [0, 1, 50, 100, 1234, -1, -12_50, 999_99]) {
        const formatted = money.format(cents);
        expect(money.parse(formatted)).toBe(cents);
      }
    });
  });

  describe('sign', () => {
    it('classifies positive / negative / neutral', () => {
      expect(money.sign(1)).toBe('positive');
      expect(money.sign(-1)).toBe('negative');
      expect(money.sign(0)).toBe('neutral');
      expect(money.sign(NaN)).toBe('neutral');
    });
  });
});
