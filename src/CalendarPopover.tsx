import { CalendarDays } from "lucide-react";
import type { CalendarEvent } from "./contracts";

interface Props {
  events: CalendarEvent[];
}

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

function isOngoing(event: CalendarEvent): boolean {
  if (event.all_day) return false;
  const now = Date.now();
  return new Date(event.start).getTime() <= now && now < new Date(event.end).getTime();
}

export function CalendarPopover({ events }: Props) {
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
    <div className="calendar-popover">
      <div className="calendar-popover-header">
        <CalendarDays size={13} />
        <span>Upcoming Events</span>
      </div>

      {sortedGroups.length === 0 ? (
        <div className="calendar-empty">No upcoming events</div>
      ) : (
        sortedGroups.map((group) => (
          <div key={dayKey(group.date)} className="calendar-day-group">
            <div className="calendar-day-header">{formatDayHeader(group.date)}</div>
            {group.events.map((event, i) => (
              <div
                key={i}
                className={`calendar-event-row${isOngoing(event) ? " calendar-event-ongoing" : ""}`}
              >
                <span className="calendar-event-time">{formatTimeRange(event)}</span>
                <div className="calendar-event-info">
                  <span className="calendar-event-title">{event.summary}</span>
                  {event.location && (
                    <span className="calendar-event-location">{event.location}</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        ))
      )}
    </div>
  );
}
