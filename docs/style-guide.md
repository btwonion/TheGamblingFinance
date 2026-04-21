# Style guide — TheGamblingFinance

This file is the **source of truth** for the design system. It is lifted
verbatim from `plan.md` §"Style guide" so agents can reference one canonical
location. The CSS custom properties below are mirrored in
`frontend/src/styles/tokens.css`; Tailwind reads them via
`theme.extend.colors` so components can use classes like `bg-surface` and
`text-positive` without hardcoding hex values.

**No hex colors outside `frontend/src/styles/`.** An ESLint
`no-restricted-syntax` rule enforces this in `.vue`/`.ts` sources.

---

## Design philosophy

Dark, high-contrast, deliberate. Casino-lounge warmth (gold accents,
saturated greens/reds) grafted onto modern fintech discipline (tabular
numerals, tight hierarchy, generous whitespace). **No gradients, no
glassmorphism, no "playful" animations on money values** — numbers must
feel authoritative.

Three principles:

1. **Numbers are sacred** — tabular, never italic, never truncated
   without a tooltip. Red/green only for deltas, never for labels.
2. **Touch first** — primary actions thumb-reachable on phones; ≥44×44 px
   hit targets; FABs sit above the bottom nav.
3. **State is legible** — open vs closed nights differ by pill color +
   border; sign of an amount, not just its color, conveys meaning.

---

## Color tokens (`frontend/src/styles/tokens.css`)

```css
:root {
  /* Backgrounds */
  --bg:           #0B0E11;  /* near-black, warm tilt */
  --surface:      #151A21;  /* cards, nav */
  --surface-2:    #1E2530;  /* raised, inputs, hover */
  --border:       #2A3340;
  --border-strong:#3B475A;  /* focused/selected */

  /* Text */
  --text:        #F1F5F9;   /* ≥15:1 on --bg, AAA */
  --text-muted:  #94A3B8;   /* ≥5.2:1,  AA */
  --text-dim:    #64748B;   /* placeholder, captions */

  /* Brand / action */
  --primary:        #10B981;
  --primary-hover:  #34D399;
  --primary-active: #059669;
  --primary-ink:    #0B1511; /* text on primary button, ~11:1 */

  /* Semantic */
  --positive:      #22C55E;  /* wins */
  --positive-soft: #0F2A1A;
  --negative:      #EF4444;  /* losses, danger */
  --negative-soft: #2A1111;
  --warning:       #F59E0B;  /* highlights, leaderboard #1 gold */
  --warning-soft:  #2A1E08;
  --info:          #3B82F6;

  --focus:         #7DD3FC;  /* ≥3:1 on all surfaces */

  /* Elevation */
  --shadow-1:     0 1px 2px rgba(0,0,0,0.40);
  --shadow-2:     0 4px 12px rgba(0,0,0,0.45);
  --shadow-sheet: 0 -8px 32px rgba(0,0,0,0.55);

  /* Radii */
  --radius-sm: 6px;  --radius-md: 8px;  --radius-lg: 12px;  --radius-xl: 16px;

  /* Motion */
  --ease-out: cubic-bezier(0.2, 0.8, 0.2, 1);
  --dur-fast: 150ms;  --dur-med: 220ms;
}
@media (prefers-reduced-motion: reduce) {
  :root { --dur-fast: 0ms; --dur-med: 0ms; }
}
```

Tailwind `theme.extend.colors` reads these via `var(--...)` — single
source of truth. No hardcoded hex in `.vue`/`.ts` (enforced by an ESLint
rule).

---

## Typography

- **UI**: Inter variable (self-hosted as woff2 under
  `frontend/public/fonts/`).
- **Money / chip counts / ranks**: Inter with
  `font-feature-settings: "tnum" 1;` → utility class `.tabular`. No
  JetBrains Mono shipped (Inter's `tnum` is sufficient and saves bundle
  size).
- **Scale** (rem @ 16 px root): `xs .75 / sm .875 / base 1 / lg 1.125 /
  xl 1.25 / 2xl 1.5 / 3xl 1.875 / 4xl 2.25`. Weights 500–800 per role.
- **Never italic.** Negatives render as `-12,50 €` (minus, not parenthesis).

---

## Spacing, radius, elevation

Tailwind 4-pt base grid. Cards `p-4` mobile / `p-6` desktop. Inputs `h-11`
(44 px). Radii: inputs `rounded-md`, cards `rounded-lg`, bottom sheets
`rounded-t-xl`.

Dark UIs can't lean on shadow for hierarchy. Combine: surface-color steps
(`bg` → `surface` → `surface-2`) + 1 px `border-border` hairline +
`shadow-1` on cards for tactility. Bottom sheets get `shadow-sheet`.

---

## Component patterns

- **Buttons**: `primary` (filled emerald, `--primary-ink` text), `ghost`
  (bordered), `danger` (outline red), `icon` (circular 44 px). All get
  `focus-visible` ring in `--focus`.
- **Inputs**: `bg-surface-2 border-border` → `focus:border-border-strong`,
  error `border-negative`, helper text `text-xs text-text-muted`.
- **Cards**: `bg-surface border border-border rounded-lg shadow-1`.
- **Stat tile**: tiny uppercase label, large tabular value, color by sign.
- **List row**: 48 px min height, border-b, tappable whole row.
- **Leaderboard row**: rank pill (#1 gets gold tint), avatar, name,
  trailing amount.
- **Bottom sheet**: drag handle, enters with `translateY 100% → 0` over
  `--dur-med`.
- **Status pill**: `open` uses `bg-positive-soft text-positive`; `closed`
  uses `bg-surface-2 text-text-muted`.

---

## Motion

No page transitions. Hover → `--dur-fast`. Sheet / toast / dialog enter →
`--dur-med` ease-out; exit is half that. 200 ms color crossfade only when
a money sign flips. All suppressed under `prefers-reduced-motion`.

---

## Accessibility

WCAG 2.1 **AA**. Focus rings always visible (no global `outline:none`).
All interactive elements keyboard-reachable in visual order; bottom sheets
trap focus. Icon-only buttons carry `aria-label`. Toasts are live regions
(`role="status"`); errors `role="alert"`. Sign + label always carry
meaning, not just color. Inputs `font-size: 16px` to disable iOS zoom.
`lang="de"` on root; dates/currency via `Intl`.
