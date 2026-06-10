// aero-popups-a.jsx — AI usage, Projects, Media, Weather, Camera
/* eslint-disable */

// ───────────────────────── AI Usage ─────────────────────────
function AiUsage() {
  const th = useAero();
  const row = (label, pct, reset) => (
    <div style={{ display: 'flex', alignItems: 'center', gap: 10, minHeight: 24 }}>
      <Mono size={10} color={th.t3} style={{ width: 18 }}>{label}</Mono>
      <PBar pct={pct} hue={sevLeft(pct)} />
      <Mono size={11.5} w={600} color={sevLeft(pct)} style={{ width: 34, textAlign: 'right' }}>{pct}%</Mono>
      <Mono size={9.5} color={th.t3} style={{ width: 52, textAlign: 'right' }}>{reset}</Mono>
    </div>
  );
  const model = (name, badge, rows) => (
    <Card style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
        <span style={{ fontSize: 12.5, fontWeight: 600, color: 'var(--accent)' }}>{name}</span>
        {badge && <Micro style={{ fontSize: 8, padding: '2px 6px', borderRadius: th.pillR, border: `1px solid ${th.divider}`, color: th.t3 }}>{badge}</Micro>}
      </div>
      {rows}
    </Card>
  );
  return (
    <Panel w={320} title="AI Usage Limits" icon={I.chip(13)}>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
        {model('Codex', 'Free', <React.Fragment>{row('5h', 82, '26d 19h')}{row('7d', 0, '')}</React.Fragment>)}
        {model('Claude', null, <React.Fragment>{row('5h', 94, '3h 48m')}{row('7d', 93, '3d 22h')}</React.Fragment>)}
      </div>
    </Panel>
  );
}

// ───────────────────────── Projects ─────────────────────────
function Projects() {
  const th = useAero();
  const repos = [
    { n: 'ai-vault', count: 45, ago: '18d', priv: true,  iss: 31, pr: 9,  rel: 5 },
    { n: 'dot-gemini', count: 45, ago: '1mo', priv: true, iss: 28, pr: 12, rel: 5 },
    { n: 'cs2binds', count: 35, ago: '2y', priv: true,  iss: 22, pr: 8,  rel: 5 },
    { n: 'song-splitter', count: 45, ago: '2mo', priv: false, iss: 30, pr: 10, rel: 5 },
  ];
  return (
    <Panel w={360} title="Projects" icon={I.branch(13)}
      actions={<span style={{ color: th.t3, display: 'flex', cursor: 'pointer' }}>{I.refresh(11)}</span>}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12, padding: '7px 10px', background: th.inputBg, border: th.inputBorder, borderRadius: th.ctlR }}>
        <span style={{ color: th.t3, display: 'flex' }}>{I.search(11)}</span>
        <span style={{ fontSize: 12, color: th.t3 }}>Search repositories</span>
        <span style={{ flex: 1 }}></span>
        <Mono size={9.5} color={th.t3}>44 flagged</Mono>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column' }}>
        {repos.map((r, i) => (
          <div key={r.n} style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '10px 2px', borderTop: i > 0 && th.rowDivider ? `1px solid ${th.divider}` : 'none' }}>
            <Mono size={13} w={700} color={r.count >= 40 ? HUE.red : HUE.amber} style={{ width: 26, textAlign: 'right' }}>{r.count}</Mono>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 7 }}>
                <span style={{ fontSize: 12.5, fontWeight: 600, color: th.t1 }}>{r.n}</span>
                {r.priv && <Micro style={{ fontSize: 7.5, padding: '1.5px 5px', borderRadius: th.pillR, border: `1px solid ${th.divider}` }}>private</Micro>}
              </div>
              <div style={{ display: 'flex', gap: 10, marginTop: 4 }}>
                <Mono size={9.5} color={th.t3}>{r.iss} issues</Mono>
                <Mono size={9.5} color={th.t3}>{r.pr} pulls</Mono>
                <Mono size={9.5} color={th.t3}>{r.rel} releases</Mono>
                <Mono size={9.5} color={th.t3} style={{ marginLeft: 'auto' }}>{r.ago}</Mono>
              </div>
            </div>
            <span style={{ color: th.t3, display: 'flex', cursor: 'pointer' }}>{I.extlink(11)}</span>
          </div>
        ))}
      </div>
    </Panel>
  );
}

// ───────────────────────── Media player ─────────────────────────
function MediaPlayer() {
  const th = useAero();
  const tBtn = (icon) => <span style={{ color: th.t2, display: 'flex', cursor: 'pointer', padding: 6 }}>{icon}</span>;
  return (
    <Panel w={560} title="Now Playing" icon={I.music(13)} hue={HUE.media}
      actions={<span style={{ display: 'inline-flex', marginRight: 6 }}><SourceTag name="Plex" size={8.5} /></span>}>
      <div style={{ display: 'flex', gap: 16, alignItems: 'center' }}>
        <image-slot id={`aero-album-${th.key}`} shape="rounded" radius={th.cardR} placeholder="album art" style={{ width: 92, height: 92, flexShrink: 0 }}></image-slot>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ fontSize: 16, fontWeight: 700, color: th.t1, letterSpacing: '-0.01em' }}>I'm not where you're at</div>
          <div style={{ fontSize: 12.5, color: th.t2, marginTop: 3 }}>Kevin Atwater</div>
          <Micro style={{ marginTop: 5 }}>I'm Not Where You're At — Single</Micro>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginTop: 14 }}>
            <Mono size={9.5} color={th.t3}>0:08</Mono>
            <PBar pct={4} hue={HUE.media} h={3} />
            <Mono size={9.5} color={th.t3}>3:57</Mono>
          </div>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexShrink: 0 }}>
          {tBtn(I.prev(13))}
          <span style={{ width: 40, height: 40, borderRadius: th.pillR === 999 ? 999 : th.ctlR, background: HUE.media, color: '#15171c', display: 'flex', alignItems: 'center', justifyContent: 'center', cursor: 'pointer', boxShadow: `0 4px 14px color-mix(in srgb, ${HUE.media} 40%, transparent)` }}>{I.pause(13, 2)}</span>
          {tBtn(I.next(13))}
        </div>
      </div>
    </Panel>
  );
}

// ───────────────────────── Weather ─────────────────────────
function Weather() {
  const th = useAero();
  const hours = [['Now', 17, I.rain], ['18:00', 17, I.rain], ['19:00', 16, I.rain], ['20:00', 15, I.rain], ['21:00', 12, I.cloud], ['22:00', 12, I.cloud]];
  const days = [
    ['Today', I.rain, 17, 12, 92], ['tor. 11', I.rain, 18, 11, 91], ['fre. 12', I.partly, 20, 11, 86],
    ['lør. 13', I.cloud, 21, 13, 85], ['søn. 14', I.partly, 18, 12, 89], ['man. 15', I.cloud, 17, 11, 90], ['tir. 16', I.cloud, 16, 12, 88],
  ];
  return (
    <Panel w={380} title="Weather" icon={I.cloud(13)} hue={HUE.weather}>
      <div style={{ display: 'flex', alignItems: 'flex-start', gap: 14, marginBottom: 18 }}>
        <span style={{ fontSize: 46, fontWeight: 600, lineHeight: 1, letterSpacing: '-0.03em', color: th.t1 }}>17°</span>
        <div style={{ paddingTop: 4 }}>
          <div style={{ fontSize: 14.5, fontWeight: 600, color: th.t1 }}>Mo i Rana</div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginTop: 4, color: HUE.weather }}>
            {I.rain(12)}
            <span style={{ fontSize: 11.5, color: th.t2 }}>Heavy rain</span>
          </div>
        </div>
      </div>
      <Micro color={HUE.weather} style={{ marginBottom: 10 }}>Hourly</Micro>
      <Card pad={10} style={{ display: 'flex', marginBottom: 16 }}>
        {hours.map(([h, t, icon]) => (
          <div key={h} style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 7 }}>
            <Mono size={9} color={th.t3}>{h}</Mono>
            <span style={{ color: HUE.weather, display: 'flex' }}>{icon(14)}</span>
            <Mono size={11.5} w={600}>{t}°</Mono>
          </div>
        ))}
      </Card>
      <Micro color={HUE.weather} style={{ marginBottom: 4 }}>Next 7 days</Micro>
      <div style={{ display: 'flex', flexDirection: 'column' }}>
        {days.map(([d, icon, hi, lo, pp], i) => (
          <div key={d} style={{ display: 'flex', alignItems: 'center', gap: 10, minHeight: 34, borderTop: i > 0 && th.rowDivider ? `1px solid ${th.divider}` : 'none' }}>
            <span style={{ fontSize: 12, color: i === 0 ? th.t1 : th.t2, width: 64, fontWeight: i === 0 ? 600 : 400 }}>{d}</span>
            <span style={{ color: th.t3, display: 'flex' }}>{icon(13)}</span>
            <span style={{ flex: 1 }}></span>
            <Mono size={11.5} w={600} style={{ width: 28, textAlign: 'right' }}>{hi}°</Mono>
            <Mono size={11.5} color={th.t3} style={{ width: 28, textAlign: 'right' }}>{lo}°</Mono>
            <Mono size={10} color={th.t3} style={{ width: 36, textAlign: 'right' }}>{pp}%</Mono>
          </div>
        ))}
      </div>
    </Panel>
  );
}

// ───────────────────────── Camera ─────────────────────────
function CameraPanel() {
  const th = useAero();
  return (
    <Panel w={460} title="Garage" icon={I.cam(13)}
      actions={
        <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6, marginRight: 4 }}>
          <span style={{ width: 6, height: 6, borderRadius: 999, background: HUE.red, boxShadow: `0 0 6px ${HUE.red}`, animation: 'aeroPulse 2.2s ease-in-out infinite' }}></span>
          <Micro color={HUE.red} style={{ fontSize: 9 }}>Live</Micro>
        </span>
      }>
      <div style={{ position: 'relative' }}>
        <image-slot id={`aero-cam-${th.key}`} shape="rounded" radius={th.cardR} placeholder="camera feed" style={{ width: '100%', height: 240, display: 'block' }}></image-slot>
        <span style={{ position: 'absolute', left: 10, top: 10, padding: '3px 8px', borderRadius: th.ctlR, background: 'rgba(0,0,0,0.55)', backdropFilter: 'blur(4px)' }}>
          <Mono size={9} color="rgba(255,255,255,0.85)">2026-06-10 17:43:50</Mono>
        </span>
      </div>
    </Panel>
  );
}

Object.assign(window, { AiUsage, Projects, MediaPlayer, Weather, CameraPanel });
