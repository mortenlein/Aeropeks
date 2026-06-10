// aero-popups-b.jsx — Mower, Vacuum, Phone, Calendar, Quick controls, Terminal
/* eslint-disable */

// ───────────────────────── Device card (mower / vacuum / phone) ─────────────────────────
function DeviceCard({ w = 264, slotId, src, imgH = 140, title, pill, pulse, hue, children, footer }) {
  const th = useAero();
  return (
    <Panel w={w} title={title} hue={hue} icon={null}>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
        <image-slot id={`${slotId}-${th.key}`} src={src} fit="contain" shape="rounded" radius={th.cardR} placeholder="device photo" style={{ width: '100%', height: imgH, display: 'block' }}></image-slot>
        <div style={{ marginTop: -14 }}>
          <Pill hue={hue} label={pill} pulse={pulse} />
        </div>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', marginTop: 14, gap: 2 }}>
        {children}
      </div>
      {footer && <div style={{ marginTop: 10, paddingTop: 9, borderTop: `1px solid ${th.divider}` }}>{footer}</div>}
    </Panel>
  );
}

function Mower() {
  const th = useAero();
  return (
    <DeviceCard slotId="aero-mower" src="uploads/049be956-8d9b-485e-a8b2-c5c29d4379b3.webp" title="Mower" pill="Docked" hue={HUE.mower}
      footer={<KV icon={I.chip(11)} label="Firmware"><Mono size={10.5} color={th.t3}>fw 600</Mono></KV>}>
      <KV icon={I.wifi(12)} label="Online" hue={HUE.ok}><span></span></KV>
      <Card pad={0} style={{ display: 'flex', marginTop: 8 }}>
        <Stat value="84" label="sessions" />
        <Stat value="1.28 ha" label="total area" />
        <Stat value="608h" label="runtime" />
      </Card>
    </DeviceCard>
  );
}

function Vacuum() {
  const th = useAero();
  return (
    <DeviceCard slotId="aero-vacuum" src="uploads/roborocks7maxv-a0c3760d.webp" title="Vacuum" pill="Charging" pulse hue={HUE.vacuum}>
      <KV icon={I.bolt(11)} label="Battery">
        <PBar pct={100} hue={HUE.vacuum} style={{ maxWidth: 70 }} />
        <Mono size={11} w={600} style={{ marginLeft: 8 }}>100%</Mono>
      </KV>
      <KV icon={I.map(12)} label="Map"><Mono size={11} color={th.t2}>Map 3</Mono></KV>
    </DeviceCard>
  );
}

function Phone() {
  const th = useAero();
  return (
    <DeviceCard slotId="aero-phone" src="uploads/pixel9.png" w={248} imgH={150} title="Phone" pill="Home" hue={HUE.phone}>
      <KV icon={I.battery(12)} label="Battery">
        <PBar pct={76} hue={HUE.phone} style={{ maxWidth: 64 }} />
        <Mono size={11} w={600} style={{ marginLeft: 8 }}>76%</Mono>
      </KV>
    </DeviceCard>
  );
}

// ───────────────────────── Calendar ─────────────────────────
function CalendarPanel() {
  const th = useAero();
  const group = (label, events) => (
    <div style={{ marginBottom: 14 }}>
      <Micro style={{ marginBottom: 6 }}>{label}</Micro>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        {events.map((e, i) => (
          <div key={i} style={{ display: 'flex', gap: 10, padding: '7px 9px', borderRadius: th.cardR, background: e.live ? 'color-mix(in srgb, var(--accent) 8%, transparent)' : 'transparent', borderLeft: e.live ? '2px solid var(--accent)' : '2px solid transparent' }}>
            <Mono size={10.5} color={e.live ? 'var(--accent)' : th.t3} style={{ width: 76, flexShrink: 0, paddingTop: 1 }}>{e.t}</Mono>
            <div style={{ minWidth: 0 }}>
              <div style={{ fontSize: 12.5, fontWeight: 600, color: th.t1 }}>{e.n}</div>
              {e.loc && <div style={{ fontSize: 10.5, color: th.t3, marginTop: 2 }}>{e.loc}</div>}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
  return (
    <Panel w={340} title="Upcoming Events" icon={I.cal(13)} hue={HUE.cal}>
      {group('Today · Jun 10', [
        { t: '17:30–18:54', n: 'Gruben J9 2 – Selfors J9 1', loc: 'Gruben kgb 5C', live: true },
        { t: '19:00–21:30', n: 'Trening', loc: 'Fossetangen forsamlingshus' },
      ])}
      {group('Tomorrow · Jun 11', [
        { t: '19:00–21:30', n: 'Trening', loc: 'Fossetangen forsamlingshus' },
      ])}
      {group('Monday · Jun 15', [
        { t: '17:30–18:54', n: 'Gruben J9 1 – Åga J9 2', loc: 'Gruben kgb 5A' },
        { t: '18:00–19:30', n: 'Trening' },
      ])}
    </Panel>
  );
}

// ───────────────────────── Quick controls: volume + power ─────────────────────────
function VolumePanel() {
  const th = useAero();
  const dev = (icon, name, sub, active) => (
    <div key={name} style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '8px 10px', borderRadius: th.ctlR, background: active ? th.ctlBg : 'transparent', cursor: 'pointer' }}>
      <span style={{ color: active ? 'var(--accent)' : th.t3, display: 'flex', flexShrink: 0 }}>{icon(12)}</span>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontSize: 12, fontWeight: active ? 600 : 400, color: active ? th.t1 : th.t2, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{name}</div>
        <Mono size={9} color={th.t3}>{sub}</Mono>
      </div>
      {active && <span style={{ width: 6, height: 6, borderRadius: 999, background: 'var(--accent)', boxShadow: '0 0 6px var(--accent)', flexShrink: 0 }}></span>}
    </div>
  );
  return (
    <Panel w={300} title="Volume">
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, padding: '4px 2px 2px' }}>
        <span style={{ color: th.t2, display: 'flex', cursor: 'pointer' }}>{I.vol(14)}</span>
        <div style={{ flex: 1, position: 'relative', height: 16, display: 'flex', alignItems: 'center' }}>
          <PBar pct={62} hue="var(--accent)" />
          <span style={{ position: 'absolute', left: '62%', transform: 'translateX(-50%)', width: 13, height: 13, borderRadius: 999, background: th.t1, boxShadow: '0 2px 6px rgba(0,0,0,0.5)', cursor: 'grab' }}></span>
        </div>
        <Mono size={11.5} w={600} style={{ width: 24, textAlign: 'right' }}>62</Mono>
      </div>
      <Micro style={{ margin: '16px 0 6px' }}>Output</Micro>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        {dev(I.vol, 'Speakers', 'Realtek HD Audio', true)}
        {dev(I.phones, 'WH-1000XM5', 'Bluetooth', false)}
        {dev(I.monitor, 'LG UltraGear', 'HDMI', false)}
      </div>
    </Panel>
  );
}

function QuickControls() {
  const th = useAero();
  const items = [['Lock', I.lock], ['Sleep', I.moon], ['Restart', I.restart], ['Shut Down', I.power]];
  return (
    <div style={{ display: 'flex', gap: 18, alignItems: 'flex-start' }}>
      <VolumePanel />
      <Panel w={172} pad={8} title={th.header === 'mono' ? 'System' : null}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
          {items.map(([n, icon], i) => (
            <div key={n} style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '8px 10px', borderRadius: th.ctlR, background: i === 0 ? th.ctlBg : 'transparent', cursor: 'pointer' }}>
              <span style={{ color: i === 3 ? HUE.red : th.t3, display: 'flex' }}>{icon(12)}</span>
              <span style={{ fontSize: 12.5, color: i === 3 ? HUE.red : th.t1 }}>{n}</span>
            </div>
          ))}
        </div>
      </Panel>
    </div>
  );
}

// ───────────────────────── Terminal ─────────────────────────
function TerminalPanel() {
  const th = useAero();
  return (
    <Panel w={560} title="Terminal" icon={I.term(13)}
      actions={
        <span style={{ display: 'flex', gap: 6, marginRight: th.header === 'titled' ? 4 : 0 }}>
          <GhostBtn style={{ padding: '3px 9px', fontSize: 10 }}>Reset</GhostBtn>
          <GhostBtn hue={HUE.red} style={{ padding: '3px 9px', fontSize: 10 }}>Kill</GhostBtn>
        </span>
      }>
      <div style={{ background: 'rgba(0,0,0,0.35)', border: `1px solid ${th.divider}`, borderRadius: th.cardR, padding: 14, minHeight: 200, fontFamily: 'var(--font-mono)', fontSize: 11.5, lineHeight: 1.75 }}>
        <div>
          <span style={{ color: 'var(--accent)', fontWeight: 600 }}>morten</span>
          <span style={{ color: th.t3 }}> in </span>
          <span style={{ color: HUE.vacuum }}>~/dev/aeropeks</span>
          <span style={{ color: th.t3 }}> on </span>
          <span style={{ color: HUE.mower }}> main</span>
          <span style={{ color: th.t3 }}> via pwsh</span>
        </div>
        <div><span style={{ color: th.t2 }}>❯ git status</span></div>
        <div style={{ color: th.t3 }}>On branch main · nothing to commit, working tree clean</div>
        <div style={{ marginTop: 6 }}>
          <span style={{ color: th.t2 }}>❯ </span>
          <span style={{ display: 'inline-block', width: 7, height: 13, background: 'var(--accent)', verticalAlign: 'text-bottom', animation: 'aeroBlink 1.2s step-end infinite' }}></span>
        </div>
      </div>
    </Panel>
  );
}

Object.assign(window, { DeviceCard, Mower, Vacuum, Phone, CalendarPanel, QuickControls, VolumePanel, TerminalPanel });
