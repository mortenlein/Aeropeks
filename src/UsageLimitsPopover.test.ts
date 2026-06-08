import { describe, expect, it } from "vitest";
import {
  lowestRemaining,
  resetIn,
  usageLimitsSummary,
} from "./UsageLimitsPopover";
import type { LimitsSnapshot } from "./contracts";

const snapshot: LimitsSnapshot = {
  providers: {
    codex: {
      enabled: true,
      ok: true,
      planType: "plus",
      shortWindow: { label: "5H", usedPercent: 28, remainingPercent: 72, resetsAt: null },
      longWindow: { label: "7D", usedPercent: 61, remainingPercent: 39, resetsAt: null },
      rateLimitReachedType: null,
      error: null,
    },
    claude: {
      enabled: true,
      ok: true,
      planType: null,
      shortWindow: { label: "5H", usedPercent: 28, remainingPercent: 72, resetsAt: null },
      longWindow: { label: "7D", usedPercent: 100, remainingPercent: 0, resetsAt: null },
      rateLimitReachedType: null,
      error: null,
    },
  },
};

describe("usage limits", () => {
  it("shows the lowest available remaining window", () => {
    expect(lowestRemaining(snapshot)).toBe(0);
  });

  it("formats compact provider windows", () => {
    expect(usageLimitsSummary(snapshot)).toBe("cdx 72% 39% / cld 72% 0%");
  });

  it("formats reset durations", () => {
    expect(resetIn(10_000, 9_100_000)).toBe("15m");
  });
});
