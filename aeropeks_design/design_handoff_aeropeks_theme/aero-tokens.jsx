// aero-tokens.jsx — Aeropeks design system: palette, themes, shared atoms
/* eslint-disable */

// ───────────────────────── Domain palette ─────────────────────────
// Rule: fixed lightness + chroma, hue varies. One hue per popup.
// Dev tools (AI, projects, terminal) ride the user accent instead.
const HUE = {
  media:   'oklch(0.74 0.14 55)',
  amber:   'oklch(0.78 0.13 85)',
  ok:      'oklch(0.74 0.14 152)',
  phone:   'oklch(0.75 0.12 172)',
  weather: 'oklch(0.75 0.12 218)',
  vacuum:  'oklch(0.72 0.12 248)',
  cal:     'oklch(0.71 0.12 276)',
  mower:   'oklch(0.71 0.13 302)',
  red:     'oklch(0.66 0.19 25)',
};
const ACCENT = 'var(--accent)';

// Severity: progress bars + counters color by load, not by domain.
function sev(pct) {
  if (pct >= 90) return HUE.red;
  if (pct >= 75) return HUE.amber;
  return HUE.ok;
}

// Quota remaining: green while plenty left, amber ≤25%, red ≤10%.
function sevLeft(pct) {
  if (pct <= 10) return HUE.red;
  if (pct <= 25) return HUE.amber;
  return HUE.ok;
}

const FONT_PAIRS = {
  grotesk: { label: 'Space Grotesk + JetBrains Mono', ui: "'Space Grotesk', sans-serif", mono: "'JetBrains Mono', monospace" },
  plex:    { label: 'IBM Plex Sans + IBM Plex Mono',  ui: "'IBM Plex Sans', sans-serif", mono: "'IBM Plex Mono', monospace" },
  sora:    { label: 'Sora + Spline Sans Mono',        ui: "'Sora', sans-serif",          mono: "'Spline Sans Mono', monospace" },
};

// ───────────────────────── Directions ─────────────────────────
const THEMES = {
  soft: {
    key: 'A', name: 'Soft Industrial', header: 'titled',
    panelBg: '#15171c', panelBorder: '1px solid rgba(255,255,255,0.07)', panelR: 16,
    shadow: '0 18px 48px rgba(0,0,0,0.5)', pad: 16,
    cardBg: 'rgba(255,255,255,0.035)', cardBorder: '1px solid rgba(255,255,255,0.05)', cardR: 10,
    t1: 'rgba(236,239,243,0.92)', t2: 'rgba(170,178,192,0.72)', t3: 'rgba(126,134,152,0.48)',
    divider: 'rgba(255,255,255,0.06)', rowDivider: false,
    ctlBg: 'rgba(255,255,255,0.055)', ctlR: 8, pillR: 999,
    inputBg: 'rgba(0,0,0,0.28)', inputBorder: '1px solid rgba(255,255,255,0.07)',
    barBg: 'rgba(18,20,25,0.97)',
  },
  term: {
    key: 'B', name: 'Terminal Precision', header: 'mono',
    panelBg: '#0e1013', panelBorder: '1px solid rgba(255,255,255,0.11)', panelR: 8,
    shadow: '0 12px 32px rgba(0,0,0,0.45)', pad: 14,
    cardBg: 'rgba(255,255,255,0.02)', cardBorder: '1px solid rgba(255,255,255,0.08)', cardR: 5,
    t1: 'rgba(228,232,236,0.92)', t2: 'rgba(160,170,182,0.7)', t3: 'rgba(118,128,144,0.5)',
    divider: 'rgba(255,255,255,0.08)', rowDivider: true,
    ctlBg: 'rgba(255,255,255,0.05)', ctlR: 4, pillR: 4,
    inputBg: 'rgba(0,0,0,0.32)', inputBorder: '1px solid rgba(255,255,255,0.1)',
    barBg: 'rgba(12,14,17,0.98)',
  },
  glass: {
    key: 'C', name: 'Quiet Glass', header: 'chromeless',
    panelBg: 'linear-gradient(180deg, rgba(36,40,48,0.94), rgba(23,26,32,0.97))',
    panelBorder: '1px solid rgba(255,255,255,0.09)', panelR: 22,
    shadow: 'inset 0 1px 0 rgba(255,255,255,0.06), 0 24px 60px rgba(0,0,0,0.55)', pad: 18,
    cardBg: 'rgba(255,255,255,0.05)', cardBorder: '1px solid transparent', cardR: 14,
    t1: 'rgba(238,240,245,0.94)', t2: 'rgba(172,180,194,0.7)', t3: 'rgba(128,136,154,0.48)',
    divider: 'rgba(255,255,255,0.055)', rowDivider: false,
    ctlBg: 'rgba(255,255,255,0.07)', ctlR: 12, pillR: 999,
    inputBg: 'rgba(0,0,0,0.22)', inputBorder: '1px solid rgba(255,255,255,0.06)',
    barBg: 'rgba(22,25,31,0.94)',
  },
};

// ───────────────────────── Context ─────────────────────────
const AeroCtx = React.createContext(THEMES.soft);
function AeroProvider({ theme, children }) {
  return <AeroCtx.Provider value={theme}>{children}</AeroCtx.Provider>;
}
function useAero() { return React.useContext(AeroCtx); }

// ───────────────────────── Icons (geometric, currentColor) ─────────────────────────
const ic = (path, vb = 12) => (s = 12, st = 1.3) => (
  <svg width={s} height={s} viewBox={`0 0 ${vb} ${vb}`} fill="none" stroke="currentColor" strokeWidth={st} strokeLinecap="round" strokeLinejoin="round">{path}</svg>
);
const I = {
  close:   ic(<path d="M3 3l6 6M9 3l-6 6" />),
  refresh: ic(<path d="M10 6a4 4 0 11-1.2-2.8M9 1v2.5H6.5" />),
  search:  ic(<g><circle cx="5.2" cy="5.2" r="3.4" /><path d="M8 8l2.6 2.6" /></g>),
  prev:    ic(<g><path d="M3 2.5v7" /><path d="M9.5 2.5L4.5 6l5 3.5v-7z" fill="currentColor" stroke="none" /></g>),
  next:    ic(<g><path d="M9 2.5v7" /><path d="M2.5 2.5L7.5 6l-5 3.5v-7z" fill="currentColor" stroke="none" /></g>),
  pause:   ic(<g><path d="M4 2.5v7M8 2.5v7" strokeWidth="1.8" /></g>),
  play:    ic(<path d="M3.5 2.2L9.8 6l-6.3 3.8v-7.6z" fill="currentColor" stroke="none" />),
  wifi:    ic(<g><path d="M2 5a6 6 0 018 0M3.7 7a3.6 3.6 0 014.6 0" /><circle cx="6" cy="9" r="0.9" fill="currentColor" stroke="none" /></g>),
  battery: ic(<g><rect x="1" y="3.5" width="8.5" height="5" rx="1.2" /><path d="M11 5.2v1.6" strokeWidth="1.6" /></g>),
  bolt:    ic(<path d="M6.5 1L3 6.8h2.6L5.2 11 9 5.4H6.2L6.5 1z" fill="currentColor" stroke="none" />),
  clock:   ic(<g><circle cx="6" cy="6" r="4.6" /><path d="M6 3.6V6l1.8 1.2" /></g>),
  cal:     ic(<g><rect x="1.5" y="2.5" width="9" height="8" rx="1.5" /><path d="M1.5 5h9M4 1.2v2M8 1.2v2" /></g>),
  cam:     ic(<g><rect x="1" y="3.5" width="7" height="5.5" rx="1.2" /><path d="M8 5.5l3-1.5v4.5l-3-1.5" /></g>),
  term:    ic(<g><path d="M2 3.5L5 6 2 8.5" /><path d="M6.5 8.5H10" /></g>),
  power:   ic(<g><path d="M6 1.5v4" /><path d="M3.4 3.2a4.4 4.4 0 105.2 0" /></g>),
  lock:    ic(<g><rect x="2.5" y="5" width="7" height="5.5" rx="1.2" /><path d="M4 5V3.8a2 2 0 014 0V5" /></g>),
  moon:    ic(<path d="M9.8 7.2A4.4 4.4 0 015 2.2a4.4 4.4 0 104.8 5z" />),
  restart: ic(<path d="M2 6a4 4 0 111.2 2.8M3 11V8.5h2.5" />),
  map:     ic(<g><path d="M1.5 3l3-1.2L7.5 3l3-1.2v7.4l-3 1.2L4.5 9.2l-3 1.2V3z" /><path d="M4.5 1.8v7.4M7.5 3v7.4" /></g>),
  branch:  ic(<g><circle cx="3" cy="2.8" r="1.4" /><circle cx="9" cy="2.8" r="1.4" /><circle cx="3" cy="9.2" r="1.4" /><path d="M3 4.2v3.6M9 4.2c0 2.4-6 2.2-6 3.6" /></g>),
  extlink: ic(<g><path d="M5 2.5H2.5v7h7V7" /><path d="M6.5 1.5H10.5v4M10.5 1.5L5.8 6.2" /></g>),
  mic:     ic(<g><rect x="4.4" y="1.2" width="3.2" height="5.6" rx="1.6" /><path d="M2.6 5.6a3.4 3.4 0 006.8 0M6 9v1.8" /></g>),
  vol:     ic(<g><path d="M2 4.5h1.8L6.5 2v8L3.8 7.5H2v-3z" fill="currentColor" stroke="none" /><path d="M8 4a2.8 2.8 0 010 4M9.5 2.8a4.8 4.8 0 010 6.4" /></g>),
  bt:      ic(<path d="M3.5 3.5l5 5L6 11V1l2.5 2.5-5 5" />),
  shield:  ic(<path d="M6 1.2l4 1.5v3.2c0 2.6-1.7 4.3-4 5.1-2.3-.8-4-2.5-4-5.1V2.7l4-1.5z" />),
  gear:    ic(<g><circle cx="6" cy="6" r="1.7" /><path d="M6 1.4v1.5M6 9.1v1.5M1.4 6h1.5M9.1 6h1.5M2.7 2.7l1.1 1.1M8.2 8.2l1.1 1.1M9.3 2.7L8.2 3.8M3.8 8.2L2.7 9.3" /></g>),
  music:   ic(<g><path d="M4.5 9.5V2.8l5-1v6.7" /><circle cx="3" cy="9.5" r="1.5" /><circle cx="8" cy="8.5" r="1.5" /></g>),
  chevR:   ic(<path d="M4.5 2.5L8 6l-3.5 3.5" />),
  scissors:ic(<g><circle cx="3" cy="3" r="1.6" /><circle cx="3" cy="9" r="1.6" /><path d="M4.4 4L10.5 9.4M4.4 8L10.5 2.6" /></g>),
  area:    ic(<g><rect x="1.5" y="1.5" width="9" height="9" rx="1" strokeDasharray="2.2 1.6" /><rect x="4" y="4" width="4" height="4" rx="0.5" /></g>),
  chip:    ic(<g><rect x="2.5" y="2.5" width="7" height="7" rx="1.2" /><path d="M4.5 1v1.5M7.5 1v1.5M4.5 9.5V11M7.5 9.5V11M1 4.5h1.5M1 7.5h1.5M9.5 4.5H11M9.5 7.5H11" /></g>),
  phones:  ic(<g><path d="M2 8.5V6.5a4 4 0 018 0v2" /><rect x="1.3" y="7.3" width="2.3" height="3.2" rx="1" /><rect x="8.4" y="7.3" width="2.3" height="3.2" rx="1" /></g>),
  monitor: ic(<g><rect x="1.5" y="2.3" width="9" height="6" rx="1" /><path d="M4.3 10.5h3.4M6 8.3v2.2" /></g>),
  pin:     ic(<g><path d="M6 10.8S2.4 7.6 2.4 5a3.6 3.6 0 117.2 0c0 2.6-3.6 5.8-3.6 5.8z" /><circle cx="6" cy="5" r="1.2" /></g>),
  // weather glyphs
  cloud:   ic(<path d="M3.6 9.2h4.9a2.1 2.1 0 000-4.2 3 3 0 00-5.7.9 1.9 1.9 0 00.8 3.3z" />),
  rain:    ic(<g><path d="M3.6 7.2h4.9a2.1 2.1 0 000-4.2 3 3 0 00-5.7.9 1.9 1.9 0 00.8 3.3z" /><path d="M4 9l-.6 1.4M6.4 9l-.6 1.4M8.8 9l-.6 1.4" /></g>),
  partly:  ic(<g><circle cx="7.8" cy="4" r="1.8" /><path d="M7.8 1v.9M10.8 4h-.9M9.9 1.9l-.6.6" /><path d="M2.6 10h4.2a1.8 1.8 0 000-3.6 2.6 2.6 0 00-4.9.8A1.6 1.6 0 002.6 10z" fill="var(--aero-panel-fill, #15171c)" /></g>),
};

// ───────────────────────── Atoms ─────────────────────────
function Micro({ color, children, style }) {
  const th = useAero();
  return <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, letterSpacing: '0.12em', textTransform: 'uppercase', color: color || th.t3, ...style }}>{children}</div>;
}

function Mono({ size = 12, color, w = 500, children, style }) {
  const th = useAero();
  return <span style={{ fontFamily: 'var(--font-mono)', fontSize: size, fontWeight: w, color: color || th.t1, ...style }}>{children}</span>;
}

// Panel — popup container; header rendering is the per-direction convention.
function Panel({ w, title, icon, hue = ACCENT, actions, children, pad, style }) {
  const th = useAero();
  const padding = pad != null ? pad : th.pad;
  let header = null;
  if (title && th.header === 'titled') {
    header = (
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12 }}>
        {icon && <span style={{ color: hue, display: 'flex' }}>{icon}</span>}
        <span style={{ fontSize: 13, fontWeight: 600, color: th.t1, letterSpacing: '0.01em', whiteSpace: 'nowrap', flexShrink: 0 }}>{title}</span>
        <span style={{ flex: 1 }}></span>
        {actions}
        <span style={{ color: th.t3, display: 'flex', cursor: 'pointer' }}>{I.close(10)}</span>
      </div>
    );
  } else if (title && th.header === 'mono') {
    header = (
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12, paddingBottom: 9, borderBottom: `1px solid ${th.divider}` }}>
        <Micro color={hue} style={{ whiteSpace: 'nowrap', flexShrink: 0 }}>{title}</Micro>
        <span style={{ flex: 1 }}></span>
        {actions}
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: th.t3, cursor: 'pointer' }}>✕</span>
      </div>
    );
  } else if (title && th.header === 'chromeless') {
    header = (
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12 }}>
        <span style={{ fontSize: 13, fontWeight: 600, color: th.t2, whiteSpace: 'nowrap', flexShrink: 0 }}>{title}</span>
        <span style={{ flex: 1 }}></span>
        {actions}
      </div>
    );
  }
  return (
    <div style={{ width: w, background: th.panelBg, border: th.panelBorder, borderRadius: th.panelR, boxShadow: th.shadow, padding: padding, fontFamily: 'var(--font-ui)', color: th.t1, position: 'relative', ...style }}>
      {header}
      {children}
    </div>
  );
}

// Card — inner grouping surface
function Card({ children, style, pad = 12 }) {
  const th = useAero();
  return <div style={{ background: th.cardBg, border: th.cardBorder, borderRadius: th.cardR, padding: pad, ...style }}>{children}</div>;
}

// Pill — unified status pill. Tint = domain hue; pulsing dot = active process.
function Pill({ hue, label, pulse }) {
  const th = useAero();
  return (
    <span style={{ display: 'inline-flex', alignItems: 'center', gap: 7, padding: '4px 12px 4px 10px', borderRadius: th.pillR, border: `1px solid color-mix(in srgb, ${hue} 40%, transparent)`, background: `color-mix(in srgb, ${hue} 9%, transparent)` }}>
      <span style={{ width: 6, height: 6, borderRadius: 999, background: hue, boxShadow: `0 0 6px ${hue}`, animation: pulse ? 'aeroPulse 2.2s ease-in-out infinite' : 'none' }}></span>
      <span style={{ fontSize: 11.5, fontWeight: 600, color: hue, letterSpacing: '0.02em' }}>{label}</span>
    </span>
  );
}

// PBar — progress; color by severity unless hue forced.
function PBar({ pct, hue, h = 4, style }) {
  const th = useAero();
  const c = hue || sev(pct);
  return (
    <div style={{ height: h, borderRadius: h / 2, background: 'rgba(255,255,255,0.07)', overflow: 'hidden', flex: 1, ...style }}>
      <div style={{ width: `${pct}%`, height: '100%', borderRadius: h / 2, background: c }}></div>
    </div>
  );
}

// Stat — value + caption cell
function Stat({ value, label, icon }) {
  const th = useAero();
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 3, flex: 1, padding: '10px 4px' }}>
      {icon && <span style={{ color: th.t3, display: 'flex', marginBottom: 1 }}>{icon}</span>}
      <Mono size={12.5} w={600}>{value}</Mono>
      <Micro style={{ fontSize: 8.5 }}>{label}</Micro>
    </div>
  );
}

// KV — icon + label + right-aligned content row
function KV({ icon, label, children, hue }) {
  const th = useAero();
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, minHeight: 28 }}>
      {icon && <span style={{ color: hue || th.t3, display: 'flex' }}>{icon}</span>}
      <span style={{ fontSize: 12, color: th.t2 }}>{label}</span>
      <span style={{ flex: 1 }}></span>
      {children}
    </div>
  );
}

function GhostBtn({ children, hue, style }) {
  const th = useAero();
  return (
    <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6, padding: '5px 11px', borderRadius: th.ctlR, background: th.ctlBg, fontSize: 11.5, fontWeight: 600, color: hue || th.t2, cursor: 'pointer', fontFamily: 'var(--font-ui)', whiteSpace: 'nowrap', ...style }}>{children}</span>
  );
}

Object.assign(window, { HUE, ACCENT, sev, sevLeft, FONT_PAIRS, THEMES, AeroProvider, useAero, I, Micro, Mono, Panel, Card, Pill, PBar, Stat, KV, GhostBtn });
