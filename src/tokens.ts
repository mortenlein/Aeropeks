// Design tokens — Terminal Precision theme
// Source: aeropeks_design/design_handoff_aeropeks_theme/README.md

export const HUE = {
  media:   'oklch(0.74 0.14 55)',
  amber:   'oklch(0.78 0.13 85)',
  ok:      'oklch(0.74 0.14 152)',
  phone:   'oklch(0.75 0.12 172)',
  weather: 'oklch(0.75 0.12 218)',
  vacuum:  'oklch(0.72 0.12 248)',
  cal:     'oklch(0.71 0.12 276)',
  mower:   'oklch(0.71 0.13 302)',
  red:     'oklch(0.66 0.19 25)',
} as const;

export const T = {
  panelBg:     '#0e1013',
  panelBorder: '1px solid rgba(255,255,255,0.11)',
  panelR:      8,
  shadow:      '0 12px 32px rgba(0,0,0,0.45)',
  pad:         14,
  cardBg:      'rgba(255,255,255,0.02)',
  cardBorder:  '1px solid rgba(255,255,255,0.08)',
  cardR:       5,
  t1:          'rgba(228,232,236,0.92)',
  t2:          'rgba(160,170,182,0.7)',
  t3:          'rgba(118,128,144,0.5)',
  divider:     'rgba(255,255,255,0.08)',
  ctlBg:       'rgba(255,255,255,0.05)',
  ctlR:        4,
  pillR:       4,
  inputBg:     'rgba(0,0,0,0.32)',
  inputBorder: '1px solid rgba(255,255,255,0.1)',
  barBg:       'rgba(12,14,17,0.98)',
} as const;

/** Severity by what's LEFT — quota bars (green default, amber ≤25%, red ≤10%) */
export function sevLeft(pct: number): string {
  if (pct <= 10) return HUE.red;
  if (pct <= 25) return HUE.amber;
  return HUE.ok;
}

/** Severity by how full — device usage (red ≥90%, amber ≥75%) */
export function sev(pct: number): string {
  if (pct >= 90) return HUE.red;
  if (pct >= 75) return HUE.amber;
  return HUE.ok;
}
