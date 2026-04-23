/**
 * `useMoney` — single currency composable for the whole app.
 *
 * House rule (plan.md §House rules): **money is BIGINT cents**. This
 * composable is the only approved bridge between the integer model and
 * human-readable German strings. Nothing else in the codebase should
 * call `Intl.NumberFormat` for currency directly.
 *
 * Negatives render with a leading minus (`-12,50 €`), never parentheses,
 * never italic. Sign + label always carry meaning in addition to color.
 */

const formatter = new Intl.NumberFormat('de-DE', {
  style: 'currency',
  currency: 'EUR',
});

export type Sign = 'positive' | 'negative' | 'neutral';

export interface UseMoney {
  format(cents: number): string;
  parse(input: string): number;
  sign(cents: number): Sign;
}

/**
 * Parse a user-entered string to integer cents.
 *
 * Accepts:
 *   - `"12"`           → 1200
 *   - `"12,50"`        → 1250
 *   - `"1.234,56"`     → 123456
 *   - `"1.234,56 €"`   → 123456
 *   - `"-12,50"`       → -1250
 *   - `"  "` or `""`   → 0
 *
 * To avoid float precision surprises we don't multiply by 100 on a
 * parsed float. Instead we split on the decimal separator and compose
 * cents as `whole * 100 + fractional`.
 */
function parseCents(input: string): number {
  const trimmed = (input ?? '').toString().trim();
  if (trimmed.length === 0) return 0;

  // Strip currency symbol, NBSPs, spaces.
  let cleaned = trimmed.replace(/[ \s€]/g, '');

  // Negative sign handling — only at start.
  let negative = false;
  if (cleaned.startsWith('-')) {
    negative = true;
    cleaned = cleaned.slice(1);
  }

  if (cleaned.length === 0) return 0;

  // German convention: `.` is thousands, `,` is decimal. Support a
  // lenient `.` as decimal too if the user typed US-style.
  let wholePart = cleaned;
  let fracPart = '';

  if (cleaned.includes(',')) {
    const [w, f = ''] = cleaned.split(',');
    wholePart = w.replace(/\./g, '');
    fracPart = f;
  } else if (cleaned.includes('.')) {
    // If there's exactly one `.` and the part after it is 1-2 digits, treat
    // as decimal separator. Otherwise treat all `.` as thousands.
    const parts = cleaned.split('.');
    if (parts.length === 2 && parts[1].length > 0 && parts[1].length <= 2) {
      wholePart = parts[0];
      fracPart = parts[1];
    } else {
      wholePart = parts.join('');
    }
  }

  if (!/^\d*$/.test(wholePart) || !/^\d*$/.test(fracPart)) {
    // Garbage → treat as zero rather than NaN, matches "tolerant input"
    // design.
    return 0;
  }

  const whole = wholePart.length > 0 ? parseInt(wholePart, 10) : 0;
  // Pad / truncate fractional to exactly two digits of cents.
  const fracNormalised = (fracPart + '00').slice(0, 2);
  const frac = parseInt(fracNormalised, 10) || 0;

  const cents = whole * 100 + frac;
  return negative ? -cents : cents;
}

export function useMoney(): UseMoney {
  return {
    format(cents: number) {
      // Guard against non-integer / NaN; treat as 0 so the UI never
      // shows `NaN €`.
      const safe = Number.isFinite(cents) ? Math.round(cents) : 0;
      return formatter.format(safe / 100);
    },
    parse(input: string) {
      return parseCents(input);
    },
    sign(cents: number): Sign {
      if (!Number.isFinite(cents) || cents === 0) return 'neutral';
      return cents > 0 ? 'positive' : 'negative';
    },
  };
}
