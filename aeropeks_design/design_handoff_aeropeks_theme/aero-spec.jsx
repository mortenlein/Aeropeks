// aero-spec.jsx — design-system spec artboards
/* eslint-disable */

function SpecCard({ w, title, children }) {
  const th = useAero();
  return (
    <div style={{ width: w, background: th.panelBg, border: th.panelBorder, borderRadius: 14, padding: 20, fontFamily: 'var(--font-ui)', color: th.t1 }}>
      <Micro color="var(--accent)" style={{ marginBottom: 16 }}>{title}</Micro>
      {children}
    </div>
  );
}

function SpecPalette() {
  const th = useAero();
  const domains = [
    ['media', HUE.media], ['charge', HUE.amber], ['ok', HUE.ok], ['phone', HUE.phone],
    ['weather', HUE.weather], ['vacuum', HUE.vacuum], ['calendar', HUE.cal], ['mower', HUE.mower], ['alert', HUE.red],
  ];
  const sw = (c, n, sub) => (
    <div key={n} style={{ display: 'flex', flexDirection: 'column', gap: 6, alignItems: 'center', flex: 1 }}>
      <span style={{ width: '100%', height: 34, borderRadius: 7, background: c, border: '1px solid rgba(255,255,255,0.08)' }}></span>
      <Mono size={9} color={th.t2}>{n}</Mono>
      {sub && <Mono size={8} color={th.t3} style={{ marginTop: -4 }}>{sub}</Mono>}
    </div>
  );
  return (
    <SpecCard w={520} title="Color — one hue per popup">
      <div style={{ fontSize: 11.5, color: th.t2, lineHeight: 1.6, marginBottom: 14 }}>
        Domain hues share one lightness + chroma — <Mono size={10.5} color={th.t1}>oklch(0.74 0.13 H)</Mono> — only hue varies.
        Everything else stays neutral. Dev tools (AI, projects, terminal) ride the user accent.
      </div>
      <div style={{ display: 'flex', gap: 7, marginBottom: 18 }}>{domains.map(([n, c]) => sw(c, n))}</div>
      <Micro style={{ marginBottom: 10 }}>Accent · user-set, follows Tweaks</Micro>
      <div style={{ display: 'flex', gap: 7, marginBottom: 18, maxWidth: 120 }}>{sw('var(--accent)', 'accent', 'live')}</div>
      <Micro style={{ marginBottom: 10 }}>Neutrals</Micro>
      <div style={{ display: 'flex', gap: 7 }}>
        {sw('#0b0c10', 'desktop')}{sw(th.panelBg, 'panel')}{sw('rgba(255,255,255,0.04)', 'card')}{sw('rgba(255,255,255,0.07)', 'line')}
        {sw('rgba(236,239,243,0.92)', 'text 1')}{sw('rgba(170,178,192,0.72)', 'text 2')}{sw('rgba(126,134,152,0.48)', 'text 3')}
      </div>
    </SpecCard>
  );
}

function SpecType() {
  const th = useAero();
  const row = (sample, name, spec, style) => (
    <div style={{ display: 'flex', alignItems: 'baseline', gap: 14, padding: '11px 0', borderBottom: `1px solid ${th.divider}` }}>
      <span style={{ flex: 1, ...style }}>{sample}</span>
      <span style={{ fontSize: 10.5, color: th.t2, width: 56 }}>{name}</span>
      <Mono size={9} color={th.t3} style={{ width: 150, textAlign: 'right' }}>{spec}</Mono>
    </div>
  );
  return (
    <SpecCard w={520} title="Type — sans labels, mono data">
      {row('17°', 'Display', 'sans · 46 / 600 / -3%', { fontSize: 46, fontWeight: 600, letterSpacing: '-0.03em', lineHeight: 1 })}
      {row('Upcoming Events', 'Title', 'sans · 13 / 600', { fontSize: 13, fontWeight: 600 })}
      {row('Gruben J9 2 – Selfors J9 1', 'Label', 'sans · 12.5 / 600', { fontSize: 12.5, fontWeight: 600 })}
      {row('Companion mode keeps Explorer alive.', 'Body', 'sans · 12 / 400 · text-2', { fontSize: 12, color: th.t2 })}
      {row('608h 19m · 82% · 17:41', 'Data', 'mono · 11–12 / 500–600', { fontFamily: 'var(--font-mono)', fontSize: 12, fontWeight: 600 })}
      {row('HOURLY FORECAST', 'Micro', 'mono · 9.5 / upper / +12%', { fontFamily: 'var(--font-mono)', fontSize: 9.5, letterSpacing: '0.12em', color: th.t3 })}
      <div style={{ fontSize: 11, color: th.t3, lineHeight: 1.6, marginTop: 12 }}>
        Numbers, times, counts, IDs and units are always mono. Names and sentences are always sans. No exceptions — this is what makes the bar scannable.
      </div>
    </SpecCard>
  );
}

function SpecAnatomy() {
  const th = useAero();
  const note = (txt) => <div style={{ fontSize: 10.5, color: th.t3, lineHeight: 1.55, marginTop: 8 }}>{txt}</div>;
  return (
    <SpecCard w={520} title="Components">
      <Micro style={{ marginBottom: 10 }}>Status pill — tint = domain hue, pulse = active</Micro>
      <div style={{ display: 'flex', gap: 10, alignItems: 'center', marginBottom: 4 }}>
        <Pill hue={HUE.mower} label="Docked" />
        <Pill hue={HUE.vacuum} label="Charging" pulse />
        <Pill hue={HUE.phone} label="Home" />
        <Pill hue={HUE.red} label="Offline" />
      </div>
      {note('6px dot + glow · 11.5/600 label · 1px border at 40% · fill at 9%. The dot pulses only while a process is running (charging, mowing, recording).')}

      <Micro style={{ margin: '18px 0 10px' }}>Quota — colored by what's left</Micro>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {[[82, 'plenty left'], [20, 'running low · ≤25%'], [8, 'nearly out · ≤10%']].map(([p, l]) => (
          <div key={l} style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <PBar pct={p} hue={sevLeft(p)} style={{ maxWidth: 180 }} />
            <Mono size={10} color={sevLeft(p)} w={600} style={{ width: 30 }}>{p}%</Mono>
            <Mono size={9} color={th.t3}>{l}</Mono>
          </div>
        ))}
      </div>
      {note('AI limit bars show remaining allowance — full and green is good. Battery / device bars keep the device hue instead.')}

      <Micro style={{ margin: '18px 0 10px' }}>Spacing</Micro>
      <div style={{ display: 'flex', gap: 16 }}>
        {[['4', 'grid base'], ['16–18', 'panel pad'], ['10–12', 'card pad'], ['12', 'header gap'], ['28–34', 'row height']].map(([v, l]) => (
          <div key={l} style={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
            <Mono size={13} w={600}>{v}</Mono>
            <Micro style={{ fontSize: 8 }}>{l}</Micro>
          </div>
        ))}
      </div>
    </SpecCard>
  );
}

function SpecIconStates() {
  const th = useAero();
  const cell = (state, label, desc) => (
    <div key={label} style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 9, flex: 1, padding: '6px 4px' }}>
      <TrayIcon icon={I.bt} state={state} size={14} />
      <Micro style={{ fontSize: 8.5 }} color={th.t2}>{label}</Micro>
      <span style={{ fontSize: 10, color: th.t3, textAlign: 'center', lineHeight: 1.5 }}>{desc}</span>
    </div>
  );
  return (
    <SpecCard w={520} title="Tray icon states">
      <div style={{ display: 'flex', gap: 8, marginBottom: 12 }}>
        {cell('idle', 'Idle', 'feature off or quiet · text-3')}
        {cell('active', 'Active', 'feature on (connected, enabled) · accent')}
        {cell('open', 'Open', 'popup showing · text-1 on control fill')}
        {cell('alert', 'Alert', 'needs attention (hot mic, error) · red + dot')}
      </div>
      <div style={{ fontSize: 10.5, color: th.t3, lineHeight: 1.55 }}>
        One icon, four states — never a different glyph per state. Hover adds the control fill without
        changing color. “Active” rides the user accent like the dev tools do; red is reserved for
        things that genuinely need you.
      </div>
    </SpecCard>
  );
}

Object.assign(window, { SpecPalette, SpecType, SpecAnatomy, SpecIconStates });
