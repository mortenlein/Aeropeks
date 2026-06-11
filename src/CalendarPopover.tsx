import type { CalendarEvent } from "./contracts";
import { Panel, Micro, Mono } from "./atoms";
import { HUE, T } from "./tokens";

function getLocalDate(isoStr: string, allDay: boolean): Date {
  if (allDay) {
    const [y, m, d] = isoStr.split("-").map(Number);
    return new Date(y, m - 1, d);
  }
  return new Date(isoStr);
}

function dayKey(date: Date): string {
  return `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`;
}

function formatDayHeader(date: Date): string {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const tomorrow = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1);
  const suffix = date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  if (date.toDateString() === today.toDateString()) return `Today · ${suffix}`;
  if (date.toDateString() === tomorrow.toDateString()) return `Tomorrow · ${suffix}`;
  return date.toLocaleDateString(undefined, { weekday: "long", month: "short", day: "numeric" });
}

function formatTime(isoStr: string): string {
  return new Date(isoStr).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
}

function formatTimeRange(event: CalendarEvent): string {
  if (event.all_day) return "All day";
  return `${formatTime(event.start)}–${formatTime(event.end)}`;
}

export function CalendarPopover({ events }: { events: CalendarEvent[] }) {
  const now = Date.now();
  const liveEvent = events
    .filter(e => !e.all_day)
    .slice()
    .sort((a, b) => new Date(a.start).getTime() - new Date(b.start).getTime())
    .find(e => new Date(e.end).getTime() > now) ?? null;

  const groups = new Map<string, { date: Date; events: CalendarEvent[] }>();
  for (const event of events) {
    const date = getLocalDate(event.start, event.all_day);
    const key = dayKey(date);
    if (!groups.has(key)) groups.set(key, { date, events: [] });
    groups.get(key)!.events.push(event);
  }
  const sortedGroups = Array.from(groups.values()).sort(
    (a, b) => a.date.getTime() - b.date.getTime(),
  );

  return (
    <Panel w={340} title="Upcoming Events" hue={HUE.cal} style={{ right: 0 }}>
      {sortedGroups.length === 0 ? (
        <div style={{ textAlign: "center", padding: "20px 0", fontSize: 12, color: T.t3 }}>
          No upcoming events
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 14, maxHeight: 400, overflowY: "auto" }}>
          {sortedGroups.map((group) => (
            <div key={dayKey(group.date)}>
              <Micro style={{ marginBottom: 6 }}>{formatDayHeader(group.date)}</Micro>
              <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
                {group.events.map((event, i) => {
                  const live = event === liveEvent;
                  return (
                    <div
                      key={i}
                      style={{
                        display: "flex",
                        gap: 10,
                        padding: "7px 9px",
                        background: live ? "color-mix(in srgb, var(--accent) 8%, transparent)" : "transparent",
                        borderLeft: live ? "2px solid var(--accent)" : "2px solid transparent",
                      }}
                    >
                      <Mono
                        size={10.5}
                        color={live ? "var(--accent)" : T.t3}
                        style={{ width: 76, flexShrink: 0, paddingTop: 1 }}
                      >
                        {formatTimeRange(event)}
                      </Mono>
                      <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                        <span style={{ fontSize: 12, fontWeight: 500, color: T.t1 }}>
                          {event.summary}
                        </span>
                        {event.location && (
                          <span style={{ fontSize: 10.5, color: T.t3 }}>{event.location}</span>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      )}
    </Panel>
  );
}
