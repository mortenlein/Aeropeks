import { type CSSProperties, type ReactNode, type MouseEvent } from 'react';
import { T, HUE, sevLeft } from './tokens';
import mowerGlyphUrl from './assets/mower-glyph.png';

// ─── Typography ────────────────────────────────────────────────────────────────

export function Micro({ color, children, style }: { color?: string; children: ReactNode; style?: CSSProperties }) {
  return (
    <div style={{ fontFamily: 'var(--font-mono)', fontSize: 9.5, letterSpacing: '0.12em', textTransform: 'uppercase', color: color ?? T.t3, margin: 0, ...style }}>
      {children}
    </div>
  );
}

export function Mono({ size = 12, color, w = 500, children, style }: { size?: number; color?: string; w?: number; children: ReactNode; style?: CSSProperties }) {
  return (
    <span style={{ fontFamily: 'var(--font-mono)', fontSize: size, fontWeight: w, color: color ?? T.t1, ...style }}>
      {children}
    </span>
  );
}

// ─── Panel / Card ──────────────────────────────────────────────────────────────

export function Panel({ w, title, icon, hue = 'var(--accent)', actions, children, pad, style, onClose, onClick }: {
  w?: number | string;
  title?: string;
  icon?: ReactNode;
  hue?: string;
  actions?: ReactNode;
  children: ReactNode;
  pad?: number;
  style?: CSSProperties;
  onClose?: () => void;
  onClick?: (e: MouseEvent<HTMLDivElement>) => void;
}) {
  const p = pad ?? T.pad;
  return (
    <div onClick={onClick} style={{ width: w, background: T.panelBg, border: T.panelBorder, borderRadius: T.panelR, boxShadow: T.shadow, padding: p, fontFamily: 'var(--font-ui)', color: T.t1, position: 'absolute', top: 36, zIndex: 100, ...style }}>
      {title && (
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12, paddingBottom: 9, borderBottom: `1px solid ${T.divider}` }}>
          {icon && <span style={{ color: hue, display: 'flex', flexShrink: 0 }}>{icon}</span>}
          <Micro color={hue} style={{ margin: 0, whiteSpace: 'nowrap', flexShrink: 0 }}>{title}</Micro>
          <span style={{ flex: 1 }} />
          {actions}
          {onClose && (
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: T.t3, cursor: 'pointer' }} onClick={onClose}>✕</span>
          )}
        </div>
      )}
      {children}
    </div>
  );
}

export function Card({ children, style, pad = 12 }: { children: ReactNode; style?: CSSProperties; pad?: number }) {
  return (
    <div style={{ background: T.cardBg, border: T.cardBorder, borderRadius: T.cardR, padding: pad, ...style }}>
      {children}
    </div>
  );
}

// ─── Status Atoms ──────────────────────────────────────────────────────────────

export function Pill({ hue, label, pulse }: { hue: string; label: string; pulse?: boolean }) {
  return (
    <span style={{ display: 'inline-flex', alignItems: 'center', gap: 7, padding: '4px 12px 4px 10px', borderRadius: T.pillR, border: `1px solid color-mix(in srgb, ${hue} 40%, transparent)`, background: `color-mix(in srgb, ${hue} 9%, ${T.panelBg})` }}>
      <span style={{ width: 6, height: 6, borderRadius: 999, background: hue, boxShadow: `0 0 6px ${hue}`, animation: pulse ? 'aeroPulse 2.2s ease-in-out infinite' : 'none', flexShrink: 0 }} />
      <span style={{ fontSize: 11.5, fontWeight: 600, color: hue, letterSpacing: '0.02em' }}>{label}</span>
    </span>
  );
}

export function PBar({ pct, hue, h = 4, style }: { pct: number; hue: string; h?: number; style?: CSSProperties }) {
  return (
    <div style={{ height: h, borderRadius: h / 2, background: 'rgba(255,255,255,0.07)', overflow: 'hidden', flex: 1, ...style }}>
      <div style={{ width: `${Math.min(100, Math.max(0, pct))}%`, height: '100%', borderRadius: h / 2, background: hue }} />
    </div>
  );
}

export function Stat({ value, label }: { value: string; label: string }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 3, flex: 1, padding: '10px 4px' }}>
      <Mono size={12.5} w={600}>{value}</Mono>
      <Micro style={{ margin: 0, fontSize: 8.5 }}>{label}</Micro>
    </div>
  );
}

export function KV({ icon, label, children, hue }: { icon?: ReactNode; label: string; children?: ReactNode; hue?: string }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, minHeight: 28 }}>
      {icon && <span style={{ color: hue ?? T.t3, display: 'flex', flexShrink: 0 }}>{icon}</span>}
      <span style={{ fontSize: 12, color: T.t2 }}>{label}</span>
      <span style={{ flex: 1 }} />
      {children}
    </div>
  );
}

export function GhostBtn({ children, hue, style, onClick }: { children: ReactNode; hue?: string; style?: CSSProperties; onClick?: () => void }) {
  return (
    <span onClick={onClick} style={{ display: 'inline-flex', alignItems: 'center', gap: 6, padding: '5px 11px', borderRadius: T.ctlR, background: T.ctlBg, fontSize: 11.5, fontWeight: 600, color: hue ?? T.t2, cursor: 'pointer', fontFamily: 'var(--font-ui)', whiteSpace: 'nowrap', userSelect: 'none', ...style }}>
      {children}
    </span>
  );
}

// ─── Bar Atoms ─────────────────────────────────────────────────────────────────

export function TrayIcon({ icon, state = 'idle', onClick }: {
  icon: ReactNode;
  state?: 'idle' | 'active' | 'open' | 'alert';
  onClick?: () => void;
}) {
  const c = state === 'active' ? 'var(--accent)' : state === 'alert' ? HUE.red : state === 'open' ? T.t1 : T.t3;
  return (
    <span onClick={onClick} style={{ color: c, display: 'flex', padding: 4, borderRadius: T.ctlR, background: state === 'open' ? T.ctlBg : 'transparent', cursor: 'pointer', position: 'relative', flexShrink: 0 }}>
      {icon}
      {state === 'alert' && (
        <span style={{ position: 'absolute', top: 1, right: 1, width: 4, height: 4, borderRadius: 999, background: HUE.red, boxShadow: `0 0 4px ${HUE.red}`, animation: 'aeroPulse 2.2s ease-in-out infinite' }} />
      )}
    </span>
  );
}

export function BarChip({ tag, pct, onClick }: { tag: string; pct: number; onClick?: () => void }) {
  return (
    <span onClick={onClick} style={{ display: 'inline-flex', alignItems: 'center', gap: 6, padding: '3px 8px', borderRadius: T.ctlR, border: `1px solid ${T.divider}`, background: 'rgba(255,255,255,0.025)', cursor: 'pointer', flexShrink: 0 }}>
      <Mono size={8.5} color={T.t3} w={600}>{tag}</Mono>
      <span style={{ width: 26, display: 'inline-flex' }}><PBar pct={pct} hue={sevLeft(pct)} h={3} /></span>
      <Mono size={9} w={600} color={sevLeft(pct)}>{String(pct)}</Mono>
    </span>
  );
}

export function BarItem({ icon, hue, text, mono, dim, gap = 6, onClick, style }: {
  icon?: ReactNode;
  hue?: string;
  text?: string;
  mono?: string;
  dim?: boolean;
  gap?: number;
  onClick?: () => void;
  style?: CSSProperties;
}) {
  return (
    <span onClick={onClick} style={{ display: 'inline-flex', alignItems: 'center', gap, padding: '0 4px', cursor: 'pointer', ...style }}>
      {icon && <span style={{ color: hue ?? T.t3, display: 'flex', opacity: hue ? 0.9 : 1, flexShrink: 0 }}>{icon}</span>}
      {text && <span style={{ fontSize: 11.5, color: dim ? T.t3 : T.t2, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{text}</span>}
      {mono && <Mono size={10.5} color={T.t1} style={{ whiteSpace: 'nowrap' }}>{mono}</Mono>}
    </span>
  );
}

export function BarGroup({ children, gap = 8, style }: { children: ReactNode; gap?: number; style?: CSSProperties }) {
  return (
    <div style={{ display: 'inline-flex', alignItems: 'center', gap, height: '100%', ...style }}>
      {children}
    </div>
  );
}

export function BarDivider() {
  return <span style={{ width: 1, height: 12, background: T.divider, margin: '0 7px', flexShrink: 0 }} />;
}

export function SourceTag({ name, size = 8 }: { name: string; size?: number }) {
  return (
    <span style={{ padding: '2px 6px', borderRadius: T.ctlR, border: `1px solid ${T.divider}`, background: 'rgba(255,255,255,0.025)', fontFamily: 'var(--font-mono)', fontSize: size, letterSpacing: '0.14em', textTransform: 'uppercase', color: T.t3, whiteSpace: 'nowrap', flexShrink: 0 }}>
      {name}
    </span>
  );
}

export function MowerGlyph({ size = 13, color }: { size?: number; color?: string }) {
  const url = `url('${mowerGlyphUrl}')`;
  return (
    <span style={{ width: size, height: size, display: 'inline-block', flexShrink: 0, background: color ?? HUE.mower, WebkitMaskImage: url, maskImage: url, WebkitMaskSize: 'contain', maskSize: 'contain', WebkitMaskRepeat: 'no-repeat', maskRepeat: 'no-repeat', WebkitMaskPosition: 'center', maskPosition: 'center' }} />
  );
}

// ─── DeviceCard ────────────────────────────────────────────────────────────────

export function DeviceCard({ w = 264, imgSrc, imgH = 140, title, hue, pill, pillPulse, children, footer, style }: {
  w?: number;
  imgSrc: string;
  imgH?: number;
  title: string;
  hue: string;
  pill: string;
  pillPulse?: boolean;
  children?: ReactNode;
  footer?: ReactNode;
  style?: CSSProperties;
}) {
  return (
    <Panel w={w} title={title} hue={hue} style={{ right: 0, ...style }}>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
        <div style={{ width: '100%', height: imgH, borderRadius: T.cardR, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <img src={imgSrc} alt={title} style={{ width: '100%', height: imgH, objectFit: 'contain', filter: 'drop-shadow(0 8px 24px rgba(0,0,0,0.55))' }} />
        </div>
        <div style={{ marginTop: 10 }}>
          <Pill hue={hue} label={pill} pulse={pillPulse} />
        </div>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', marginTop: 12, gap: 2 }}>
        {children}
      </div>
      {footer && (
        <div style={{ marginTop: 10, paddingTop: 9, borderTop: `1px solid ${T.divider}` }}>
          {footer}
        </div>
      )}
    </Panel>
  );
}
