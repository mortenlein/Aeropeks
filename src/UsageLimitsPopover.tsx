import type { LimitProvider, LimitsSnapshot, RateLimitWindow } from "./contracts";

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
        (window) =>
          window.remainingPercent !== null ||
          window.usedPercent !== null ||
          window.resetsAt !== null,
      )
    );
  });

export const lowestRemaining = (snapshot: LimitsSnapshot) => {
  const values = usableProviders(snapshot).flatMap(([, provider]) =>
    [provider.shortWindow, provider.longWindow]
      .map((window) => window.remainingPercent)
      .filter((value): value is number => value !== null),
  );
  return values.length > 0 ? Math.min(...values) : null;
};

export const usageLimitsSummary = (snapshot: LimitsSnapshot) => {
  const labels: Record<string, string> = {
    codex: "cdx",
    claude: "cld",
  };

  return usableProviders(snapshot)
    .filter(([id]) => id in labels)
    .map(([id, provider]) => {
      const percentage = (window: RateLimitWindow) =>
        window.remainingPercent === null
          ? "-"
          : `${Math.round(window.remainingPercent)}%`;
      return `${labels[id]} ${percentage(provider.shortWindow)} ${percentage(provider.longWindow)}`;
    })
    .join(" / ");
};

const compactProviderLabels: Record<string, string> = {
  codex: "cdx",
  claude: "cld",
};

function CompactPercentage({ value }: { value: number | null }) {
  return (
    <span
      className={`usage-compact-value ${
        value === null
          ? "usage-value-missing"
          : value <= 20
            ? "usage-value-critical"
            : "usage-value-ok"
      }`}
    >
      {value === null ? "--" : Math.round(value)}
    </span>
  );
}

export function UsageLimitsSummary({ snapshot }: { snapshot: LimitsSnapshot }) {
  const providers = usableProviders(snapshot).filter(
    ([id]) => id in compactProviderLabels,
  );

  return (
    <span className="usage-limits-summary">
      {providers.map(([id, provider], index) => (
        <span className="usage-limits-provider-summary" key={id}>
          {index > 0 && <span className="usage-limits-separator"> / </span>}
          <span className="usage-provider-code">{compactProviderLabels[id]}</span>
          <span className="usage-window-compact">
            <small>5h</small>
            <CompactPercentage value={provider.shortWindow.remainingPercent} />
          </span>
          <span className="usage-window-compact">
            <small>7d</small>
            <CompactPercentage value={provider.longWindow.remainingPercent} />
          </span>
        </span>
      ))}
    </span>
  );
}

function WindowRow({ window }: { window: RateLimitWindow }) {
  const remaining = window.remainingPercent ?? 0;
  return (
    <div className="usage-window-row">
      <span>{window.label}</span>
      <div className="usage-track">
        <div
          className={`usage-fill ${remaining <= 20 ? "critical" : remaining <= 40 ? "warning" : ""}`}
          style={{ width: `${Math.max(0, Math.min(remaining, 100))}%` }}
        />
      </div>
      <strong>{Math.round(remaining)}%</strong>
      <small>{resetIn(window.resetsAt)}</small>
    </div>
  );
}

function ProviderCard({ id, provider }: { id: string; provider: LimitProvider }) {
  return (
    <div className="usage-provider-card">
      <div className="usage-provider-header">
        <strong>{providerLabel(id)}</strong>
        {provider.planType && <span>{provider.planType}</span>}
        {provider.rateLimitReachedType && <em>limited</em>}
      </div>
      <WindowRow window={provider.shortWindow} />
      <WindowRow window={provider.longWindow} />
    </div>
  );
}

export function UsageLimitsPopover({ snapshot }: { snapshot: LimitsSnapshot }) {
  return (
    <div className="usage-limits-popover dropdown">
      <div className="ctx-header">AI Usage Limits</div>
      {usableProviders(snapshot).map(([id, provider]) => (
        <ProviderCard key={id} id={id} provider={provider} />
      ))}
    </div>
  );
}
