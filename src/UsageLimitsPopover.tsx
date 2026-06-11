import type { LimitProvider, LimitsSnapshot, RateLimitWindow } from "./contracts";
import { Panel, Card, PBar, Mono, Micro } from "./atoms";
import { HUE, T, sevLeft } from "./tokens";

const providerLabel = (id: string) =>
  id.charAt(0).toUpperCase() + id.slice(1);

export const resetIn = (timestamp: number | null, now = Date.now()) => {
  if (!timestamp) return "";
  const seconds = timestamp - Math.floor(now / 1000);
  if (seconds <= 0) return "now";
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours >= 24) return `${Math.floor(hours / 24)}d ${hours % 24}h`;
  return hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`;
};

export const usableProviders = (snapshot: LimitsSnapshot) =>
  Object.entries(snapshot.providers).filter(([, provider]) => {
    const windows = [provider.shortWindow, provider.longWindow];
    return (
      provider.enabled &&
      provider.ok &&
      !provider.error &&
      windows.some(
        (w) =>
          w.remainingPercent !== null ||
          w.usedPercent !== null ||
          w.resetsAt !== null,
      )
    );
  });

export const usageLimitsSummary = (snapshot: LimitsSnapshot) => {
  const labels: Record<string, string> = { codex: "cdx", claude: "cld" };
  return usableProviders(snapshot)
    .filter(([id]) => id in labels)
    .map(([id, provider]) => {
      const pct = (w: { remainingPercent: number | null }) =>
        w.remainingPercent === null ? "-" : `${Math.round(w.remainingPercent)}%`;
      return `${labels[id]} ${pct(provider.shortWindow)} ${pct(provider.longWindow)}`;
    })
    .join(" / ");
};

export const lowestRemaining = (snapshot: LimitsSnapshot) => {
  const values = usableProviders(snapshot).flatMap(([, provider]) =>
    [provider.shortWindow, provider.longWindow]
      .map((w) => w.remainingPercent)
      .filter((v): v is number => v !== null),
  );
  return values.length > 0 ? Math.min(...values) : null;
};

function WindowRow({ w }: { w: RateLimitWindow }) {
  const pct = w.remainingPercent ?? 0;
  const hue = sevLeft(pct);
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 10, minHeight: 24 }}>
      <Mono size={10} color={T.t3} style={{ width: 18 }}>{w.label}</Mono>
      <PBar pct={pct} hue={hue} />
      <Mono size={11.5} w={600} color={hue} style={{ width: 34, textAlign: "right" }}>
        {Math.round(pct)}%
      </Mono>
      <Mono size={9.5} color={T.t3} style={{ width: 52, textAlign: "right" }}>
        {resetIn(w.resetsAt)}
      </Mono>
    </div>
  );
}

function ProviderCard({ id, provider }: { id: string; provider: LimitProvider }) {
  return (
    <Card style={{ display: "flex", flexDirection: "column", gap: 4 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 4 }}>
        <span style={{ fontSize: 12.5, fontWeight: 600, color: "var(--accent)" }}>{providerLabel(id)}</span>
        {provider.planType && (
          <Micro style={{ fontSize: 8, padding: "2px 6px", borderRadius: T.pillR, border: `1px solid ${T.divider}`, display: "inline-flex" }}>
            {provider.planType}
          </Micro>
        )}
        <span style={{ flex: 1 }} />
        {provider.rateLimitReachedType && (
          <Mono size={9} color={HUE.red} w={600}>LIMITED</Mono>
        )}
      </div>
      <WindowRow w={provider.shortWindow} />
      <WindowRow w={provider.longWindow} />
    </Card>
  );
}

export function UsageLimitsPopover({ snapshot }: { snapshot: LimitsSnapshot }) {
  const providers = usableProviders(snapshot);
  return (
    <Panel w={320} title="AI Usage Limits" style={{ left: 0 }}>
      <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
        {providers.map(([id, provider]) => (
          <ProviderCard key={id} id={id} provider={provider} />
        ))}
        {providers.length === 0 && (
          <div style={{ textAlign: "center", padding: "16px 0", fontSize: 12, color: T.t3 }}>
            No active providers
          </div>
        )}
      </div>
    </Panel>
  );
}
