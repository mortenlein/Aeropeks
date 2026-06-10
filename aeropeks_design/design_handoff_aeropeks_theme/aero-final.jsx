// aero-final.jsx — Final spec page: Direction B (Terminal Precision) only
/* eslint-disable */

const FINAL_DEFAULTS = /*EDITMODE-BEGIN*/{
  "accent": "#22C55E",
  "fonts": "grotesk"
}/*EDITMODE-END*/;

function FSection({ num, title, note, children }) {
  const th = useAero();
  return (
    <section style={{ marginBottom: 72 }}>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: 14, marginBottom: 6 }}>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--accent)', fontWeight: 600 }}>{num}</span>
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 12, letterSpacing: '0.18em', textTransform: 'uppercase', color: th.t1, fontWeight: 600 }}>{title}</span>
        <span style={{ flex: 1, height: 1, background: th.divider, alignSelf: 'center' }}></span>
      </div>
      {note && <div style={{ fontSize: 12, color: th.t3, marginBottom: 26, maxWidth: 720, lineHeight: 1.6 }}>{note}</div>}
      {!note && <div style={{ height: 26 }}></div>}
      {children}
    </section>
  );
}

function FinalApp() {
  const [t, setTweak] = useTweaks(FINAL_DEFAULTS);
  const pair = FONT_PAIRS[t.fonts] || FONT_PAIRS.grotesk;
  const th = THEMES.term;
  return (
    <div style={{ '--accent': t.accent, '--font-ui': pair.ui, '--font-mono': pair.mono, minHeight: '100vh', background: '#0b0c10', fontFamily: 'var(--font-ui)' }}>
      <AeroProvider theme={th}>
        <div style={{ maxWidth: 1660, margin: '0 auto', padding: '64px 48px 120px' }}>

          {/* masthead */}
          <header style={{ marginBottom: 72 }}>
            <Micro color="var(--accent)" style={{ marginBottom: 12 }}>Aeropeks · Design System v1</Micro>
            <h1 style={{ fontSize: 34, fontWeight: 700, letterSpacing: '-0.02em', color: th.t1, margin: 0 }}>Terminal Precision</h1>
            <p style={{ fontSize: 13, color: th.t2, maxWidth: 640, lineHeight: 1.65, marginTop: 12 }}>
              One theme for the bar and every popup. 8px panel corners, hairline dividers, mono uppercase
              headers in the popup's hue. Accent and font pairing stay user-settable — they flow through
              everything on this page via Tweaks.
            </p>
          </header>

          <FSection num="01" title="Tokens"
            note="Shared rules. Domain hues sit at one lightness + chroma; dev tools (AI, projects, terminal) ride the user accent; usage bars color by severity, not domain; mono carries data, sans carries labels.">
            <div style={{ display: 'flex', gap: 24, alignItems: 'flex-start', flexWrap: 'wrap' }}>
              <SpecPalette />
              <SpecType />
              <SpecAnatomy />
            </div>
          </FSection>

          <FSection num="02" title="Top bar"
            note="32px strip, items grouped into clusters with a hairline divider between each. The small labels underneath are documentation only — they name the clusters and never render in the app. System tray icons carry state by color alone — the bar below shows bluetooth active, mic alerting, terminal open.">
            <div style={{ display: 'flex', flexDirection: 'column', gap: 28 }}>
              <TopBarAero />
              <SpecIconStates />
            </div>
          </FSection>

          <FSection num="03" title="Popups"
            note="Every popup: panel chrome from tokens, mono uppercase header in the popup's hue, ✕ to dismiss, rows separated by hairlines. One domain hue per popup — everything else neutral.">
            <div style={{ display: 'flex', flexDirection: 'column', gap: 28 }}>
              <div style={{ display: 'flex', gap: 28, alignItems: 'flex-start', flexWrap: 'wrap' }}>
                <AiUsage />
                <MediaPlayer />
                <TerminalPanel />
              </div>
              <div style={{ display: 'flex', gap: 28, alignItems: 'flex-start', flexWrap: 'wrap' }}>
                <Weather />
                <Projects />
                <CalendarPanel />
                <CameraPanel />
              </div>
              <div style={{ display: 'flex', gap: 28, alignItems: 'flex-start', flexWrap: 'wrap' }}>
                <Mower />
                <Vacuum />
                <Phone />
                <QuickControls />
              </div>
            </div>
          </FSection>

          <FSection num="04" title="Settings window"
            note="Same tokens at window scale: section cards with mono accent labels, calm neutral helper text, accent reserved for the primary action and active states.">
            <SettingsWindow accent={t.accent} fontKey={t.fonts} />
          </FSection>

        </div>
      </AeroProvider>

      <TweaksPanel>
        <TweakSection label="Theme" />
        <TweakColor label="Accent" value={t.accent}
          options={['#22C55E', '#38BDF8', '#A78BFA', '#F4845F']}
          onChange={(v) => setTweak('accent', v)} />
        <TweakSection label="Typography" />
        <TweakSelect label="Font pairing" value={t.fonts}
          options={Object.keys(FONT_PAIRS).map(k => ({ value: k, label: FONT_PAIRS[k].label }))}
          onChange={(v) => setTweak('fonts', v)} />
      </TweaksPanel>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<FinalApp />);
