import { describe, expect, it } from "vitest";
import { normalizeWindowType } from "./window-routing";

describe("normalizeWindowType", () => {
  it("maps the legacy app label to the main window", () => {
    expect(normalizeWindowType("aeropeks")).toBe("main");
  });

  it("rejects unknown window labels", () => {
    expect(normalizeWindowType("main-dev")).toBeNull();
  });
});
