// aero-bar.jsx — unified top bar with cluster annotations
/* eslint-disable */

// SourceTag — provenance chip for media (Plex / Spotify / YouTube …)
function SourceTag({ name, size = 8 }) {
  const th = useAero();
  return (
    <span style={{ padding: '2px 6px', borderRadius: th.pillR === 999 ? 999 : th.ctlR, border: `1px solid ${th.divider}`, background: 'rgba(255,255,255,0.025)', fontFamily: 'var(--font-mono)', fontSize: size, letterSpacing: '0.14em', textTransform: 'uppercase', color: th.t3, whiteSpace: 'nowrap' }}>{name}</span>
  );
}

function MowerGlyph({ size = 13, color }) {
  const m = "url('icons/mower-glyph.png')";
  return <span style={{ width: size, height: size, display: 'inline-block', flexShrink: 0, background: color || HUE.mower, WebkitMaskImage: m, maskImage: m, WebkitMaskSize: 'contain', maskSize: 'contain', WebkitMaskRepeat: 'no-repeat', maskRepeat: 'no-repeat', WebkitMaskPosition: 'center', maskPosition: 'center' }}></span>;
}

// TrayIcon — system tray icon with explicit states:
// idle = text-3 · active (feature on) = accent · open (popup showing) = text-1 on control fill · alert = red
function TrayIcon({ icon, state = 'idle', size = 11 }) {
  const th = useAero();
  const c = state === 'active' ? 'var(--accent)' : state === 'alert' ? HUE.red : state === 'open' ? th.t1 : th.t3;
  return (
    <span style={{ color: c, display: 'flex', padding: 4, borderRadius: th.ctlR, background: state === 'open' ? th.ctlBg : 'transparent', cursor: 'pointer', position: 'relative' }}>
      {icon(size)}
      {state === 'alert' && <span style={{ position: 'absolute', top: 1, right: 1, width: 4, height: 4, borderRadius: 999, background: HUE.red, boxShadow: `0 0 4px ${HUE.red}`, animation: 'aeroPulse 2.2s ease-in-out infinite' }}></span>}
    </span>
  );
}

function BarChip({ tag, pct }) {
  const th = useAero();
  return (
    <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6, padding: '3px 8px', borderRadius: th.pillR === 999 ? 999 : th.ctlR, border: `1px solid ${th.divider}`, background: 'rgba(255,255,255,0.025)' }}>
      <Mono size={8.5} color={th.t3} w={600}>{tag}</Mono>
      <span style={{ width: 26 }}><PBar pct={pct} hue={sevLeft(pct)} h={3} /></span>
      <Mono size={9} w={600} color={sevLeft(pct)}>{pct}</Mono>
    </span>
  );
}

function BarItem({ icon, hue, text, mono, dim, gap = 6 }) {
  const th = useAero();
  return (
    <span style={{ display: 'inline-flex', alignItems: 'center', gap, padding: '0 4px', cursor: 'pointer' }}>
      {icon && <span style={{ color: hue || th.t3, display: 'flex', opacity: hue ? 0.9 : 1 }}>{icon}</span>}
      {text && <span style={{ fontSize: 11.5, color: dim ? th.t3 : th.t2, whiteSpace: 'nowrap' }}>{text}</span>}
      {mono && <Mono size={10.5} color={th.t1} style={{ whiteSpace: 'nowrap' }}>{mono}</Mono>}
    </span>
  );
}

function BarGroup({ label, children, gap = 8 }) {
  const th = useAero();
  return (
    <span style={{ position: 'relative', display: 'inline-flex', alignItems: 'center', gap, height: '100%' }}>
      {children}
      <span style={{ position: 'absolute', top: 40, left: '50%', transform: 'translateX(-50%)', whiteSpace: 'nowrap', fontFamily: 'var(--font-mono)', fontSize: 8.5, letterSpacing: '0.16em', textTransform: 'uppercase', color: th.t3, opacity: 0.8 }}>{label}</span>
      <span style={{ position: 'absolute', top: 34, left: 2, right: 2, height: 1, background: th.divider }}></span>
    </span>
  );
}

function TopBarAero() {
  const th = useAero();
  const Div = () => <span style={{ width: 1, height: 12, background: th.divider, margin: '0 7px', flexShrink: 0 }}></span>;
  return (
    <div style={{ width: 1520, fontFamily: 'var(--font-ui)' }}>
      <div style={{ display: 'flex', alignItems: 'center', height: 32, background: th.barBg, borderBottom: `1px solid ${th.divider}`, borderRadius: th.key === 'C' ? 10 : th.key === 'A' ? 6 : 0, padding: '0 12px', boxShadow: th.shadow }}>
        {/* left — dev */}
        <BarGroup label="dev tools" gap={7}>
          <BarChip tag="CDX" pct={82} />
          <BarChip tag="CLD" pct={94} />
          <Div />
          <BarItem icon={I.branch(12)} hue="var(--accent)" mono="68" />
          <span style={{ padding: '1.5px 6px', borderRadius: th.pillR, background: `color-mix(in srgb, ${HUE.red} 18%, transparent)`, border: `1px solid color-mix(in srgb, ${HUE.red} 35%, transparent)` }}>
            <Mono size={9} w={600} color={HUE.red}>44</Mono>
          </span>
        </BarGroup>

        <span style={{ flex: 1 }}></span>

        {/* center — media */}
        <BarGroup label="now playing" gap={4}>
          <span style={{ display: 'inline-flex', gap: 2, marginRight: 8 }}>
            <span style={{ color: th.t3, display: 'flex', padding: 3 }}>{I.prev(10)}</span>
            <span style={{ color: th.t2, display: 'flex', padding: 3 }}>{I.pause(10, 1.8)}</span>
            <span style={{ color: th.t3, display: 'flex', padding: 3 }}>{I.next(10)}</span>
          </span>
          <BarItem icon={I.music(11)} hue={HUE.media} text="Kevin Atwater" />
          <span style={{ fontSize: 11.5, fontWeight: 600, color: th.t1, whiteSpace: 'nowrap' }}>I'm not where you're at</span>
          <span style={{ marginLeft: 8, display: 'inline-flex' }}><SourceTag name="Plex" /></span>
        </BarGroup>

        <span style={{ flex: 1 }}></span>

        {/* right clusters */}
        <BarGroup label="environment">
          <BarItem icon={I.cloud(12)} hue={HUE.weather} mono="17°" />
        </BarGroup>
        <Div />
        <BarGroup label="home" gap={10}>
          <BarItem icon={I.cam(12)} hue={th.t3} text="Garage" />
          <BarItem icon={<MowerGlyph size={13} />} hue={HUE.mower} text="Docked" />
          <BarItem icon={I.bolt(11)} hue={HUE.vacuum} text="Charging" />
        </BarGroup>
        <Div />
        <BarGroup label="personal" gap={10}>
          <BarItem icon={I.battery(12)} hue={HUE.phone} mono="76%" />
          <BarItem icon={I.cal(11)} hue={HUE.cal} text="Gruben J9 2 – Selfors J9 1" />
        </BarGroup>
        <Div />
        <BarGroup label="system" gap={2}>
          <TrayIcon icon={I.shield} state="idle" />
          <TrayIcon icon={I.bt} state="active" />
          <TrayIcon icon={I.mic} state="alert" />
          <TrayIcon icon={I.vol} state="idle" />
          <TrayIcon icon={I.term} state="open" />
          <TrayIcon icon={I.power} />
          <TrayIcon icon={I.gear} />
        </BarGroup>
        <Div />
        <BarGroup label="clock">
          <Mono size={11} w={600}>17:41</Mono>
        </BarGroup>
      </div>
      <div style={{ height: 36 }}></div>
    </div>
  );
}

Object.assign(window, { TopBarAero, BarChip, BarItem, SourceTag, MowerGlyph, TrayIcon });
