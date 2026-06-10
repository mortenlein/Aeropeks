// aero-settings.jsx — Aeropeks Settings window, themed
/* eslint-disable */

function SetInput({ value, placeholder, mono, flex = 1, w }) {
  const th = useAero();
  return (
    <span style={{ display: 'inline-flex', alignItems: 'center', flex: w ? 'none' : flex, width: w, padding: '8px 11px', background: th.inputBg, border: th.inputBorder, borderRadius: th.ctlR, fontSize: 12, fontFamily: mono ? 'var(--font-mono)' : 'var(--font-ui)', color: value ? th.t1 : th.t3, whiteSpace: 'nowrap', overflow: 'hidden' }}>
      {value || placeholder}
    </span>
  );
}

function SetField({ label, help, children }) {
  const th = useAero();
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
      <span style={{ fontSize: 12, fontWeight: 600, color: th.t1 }}>{label}</span>
      <div style={{ display: 'flex', gap: 8 }}>{children}</div>
      {help && <span style={{ fontSize: 10.5, color: th.t3, lineHeight: 1.5 }}>{help}</span>}
    </div>
  );
}

function SetSection({ title, action, children }) {
  const th = useAero();
  return (
    <Card pad={18} style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <div style={{ display: 'flex', alignItems: 'center' }}>
        <Micro color="var(--accent)">{title}</Micro>
        <span style={{ flex: 1 }}></span>
        {action}
      </div>
      {children}
    </Card>
  );
}

function SetCheck({ on, label, help }) {
  const th = useAero();
  return (
    <div style={{ display: 'flex', gap: 10 }}>
      <span style={{ width: 16, height: 16, borderRadius: Math.min(th.ctlR, 5), flexShrink: 0, marginTop: 1, background: on ? 'var(--accent)' : th.inputBg, border: on ? '1px solid transparent' : th.inputBorder, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#10131a' }}>
        {on && <svg width="9" height="9" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><path d="M1.5 5.5l2.5 2.5 4.5-5" /></svg>}
      </span>
      <div>
        <div style={{ fontSize: 12.5, fontWeight: 600, color: th.t1 }}>{label}</div>
        {help && <div style={{ fontSize: 10.5, color: th.t3, marginTop: 3, lineHeight: 1.5 }}>{help}</div>}
      </div>
    </div>
  );
}

function SettingsWindow({ accent, fontKey }) {
  const th = useAero();
  const shortcut = (name, cmd) => (
    <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
      <SetInput value={name} />
      <SetInput value={cmd} mono />
      <SetInput placeholder="Hotkey" w={90} />
      <span style={{ color: HUE.red, display: 'flex', cursor: 'pointer', opacity: 0.8 }}>{I.close(10)}</span>
    </div>
  );
  const seg = (opts, sel) => (
    <span style={{ display: 'inline-flex', background: th.inputBg, border: th.inputBorder, borderRadius: th.ctlR, padding: 2 }}>
      {opts.map(o => (
        <span key={o} style={{ padding: '5px 14px', borderRadius: Math.max(th.ctlR - 2, 2), fontSize: 11.5, fontWeight: 600, background: o === sel ? 'var(--accent)' : 'transparent', color: o === sel ? '#10131a' : th.t2, cursor: 'pointer' }}>{o}</span>
      ))}
    </span>
  );
  const ACCENTS = ['#22C55E', '#38BDF8', '#A78BFA', '#F4845F'];
  return (
    <div style={{ width: 880, background: th.panelBg, border: th.panelBorder, borderRadius: th.key === 'B' ? th.panelR : 12, boxShadow: th.shadow, fontFamily: 'var(--font-ui)', color: th.t1, overflow: 'hidden' }}>
      {/* titlebar */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, height: 36, padding: '0 14px', borderBottom: `1px solid ${th.divider}` }}>
        <span style={{ color: 'var(--accent)', display: 'flex' }}>{I.chip(11)}</span>
        <span style={{ fontSize: 11.5, color: th.t2 }}>Aeropeks Settings</span>
        <span style={{ flex: 1 }}></span>
        <span style={{ color: th.t3, display: 'flex', gap: 14 }}>
          <span style={{ width: 9, height: 1.4, background: 'currentColor', alignSelf: 'center' }}></span>
          <span style={{ width: 8, height: 8, border: '1.3px solid currentColor', borderRadius: 1.5 }}></span>
          {I.close(10)}
        </span>
      </div>
      <div style={{ padding: 24, display: 'flex', flexDirection: 'column', gap: 16 }}>
        <div style={{ marginBottom: 4 }}>
          <div style={{ fontSize: 19, fontWeight: 700, letterSpacing: '-0.01em' }}>Settings</div>
          <div style={{ fontSize: 11.5, color: th.t3, marginTop: 4 }}>Configure your personalized desktop menu bar.</div>
        </div>

        <SetSection title="Media Integration">
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
            <SetField label="Plex Server URL" help="Address of your Plex Media Server."><SetInput value="http://158.248.97.96:32400" mono /></SetField>
            <SetField label="Plex Token" help="Used for authentication and playback control."><SetInput value="••••••••••••••••" mono /></SetField>
          </div>
        </SetSection>

        <SetSection title="System Shortcuts" action={<span style={{ fontSize: 11.5, fontWeight: 600, color: 'var(--accent)', cursor: 'pointer' }}>+ Add Action</span>}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {shortcut('SSH: Home Lab (pi@homeserver)', 'ssh pi@homeserver.local')}
            {shortcut('Git Status', 'git status')}
            {shortcut('Git Fetch All', 'git fetch --all')}
          </div>
        </SetSection>

        <SetSection title="External Services">
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
            <SetField label="Weather Forecast Location" help="Current location: Mo i Rana (66.31, 14.14)"><SetInput value="Mo i Rana" w={200} /></SetField>
            <SetField label="Usage Limits Service URL" help="Local AI usage tracking endpoint. Leave blank to disable."><SetInput value="http://192.168.10.118:8765/api/v1/snapshot" mono /></SetField>
            <SetField label="GitHub Personal Access Token" help="Powers the Projects view. Needs repository read access."><SetInput value="••••••••••••••••••••••••••••" mono /></SetField>
            <SetField label="Home Assistant" help="Shows the Garage camera in the menu bar.">
              <SetInput value="https://ha.mortenlab.xyz/" mono />
              <SetInput value="••••••••••" mono w={120} />
            </SetField>
          </div>
        </SetSection>

        <SetSection title="Shell Companion">
          <SetCheck on label="Reserve screen space for Aeropeks bars" help="Companion mode keeps Explorer alive. Turn this off if work-area reservation gets weird." />
          <SetCheck label="Hide the native Windows taskbar" help="Advanced replacement mode. Leave off unless Aeropeks should take over more shell surface." />
          <div style={{ display: 'flex', gap: 8 }}>
            <GhostBtn>Restore Windows Shell</GhostBtn>
            <GhostBtn>Clear Icon Cache</GhostBtn>
            <GhostBtn>Screenshot Mode</GhostBtn>
          </div>
        </SetSection>

        <SetSection title="Personalization">
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
            <SetField label="Accent Color" help="Used for highlights and active indicators across the app.">
              <span style={{ display: 'inline-flex', gap: 8, alignItems: 'center' }}>
                {ACCENTS.map(c => (
                  <span key={c} style={{ width: 22, height: 22, borderRadius: th.pillR === 999 ? 999 : 5, background: c, cursor: 'pointer', outline: c === accent ? `2px solid ${th.t1}` : 'none', outlineOffset: 2 }}></span>
                ))}
                <Mono size={10.5} color={th.t3} style={{ marginLeft: 6 }}>{accent}</Mono>
              </span>
            </SetField>
            <SetField label="Clock Format" help="HH:MM (24h) or HH:MM AM/PM (12h).">{seg(['24-Hour', '12-Hour'], '24-Hour')}</SetField>
            <SetField label="Font Pairing" help="Sans for labels, mono for data — pairs are curated.">
              <SetInput value={FONT_PAIRS[fontKey].label} w={280} />
            </SetField>
          </div>
        </SetSection>

        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 10, marginTop: 4 }}>
          <GhostBtn style={{ padding: '9px 16px', fontSize: 12 }}>Close Without Saving</GhostBtn>
          <span style={{ padding: '9px 18px', borderRadius: th.ctlR, background: 'var(--accent)', color: '#10131a', fontSize: 12, fontWeight: 700, cursor: 'pointer' }}>Save All Changes</span>
        </div>
      </div>
    </div>
  );
}

Object.assign(window, { SettingsWindow });
