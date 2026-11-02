import type { Issue } from './issues';

/**
 * Derived numbers over the issue set. Depends on `issues.ts` only - no content
 * layer, no Astro - so it can be exercised by `node --test`.
 */

const MS_PER_DAY = 86_400_000;
const DATE = /^(\d{4})-(\d{2})-(\d{2})/;

/** Whole days since the epoch, or null when the field is not a date. */
export function toDay(value: string | null): number | null {
  if (value === null) return null;
  const parts = DATE.exec(value);
  if (parts === null) return null;
  return Date.UTC(Number(parts[1]), Number(parts[2]) - 1, Number(parts[3])) / MS_PER_DAY;
}

export function fromDay(day: number): string {
  return new Date(day * MS_PER_DAY).toISOString().slice(0, 10);
}

export interface Day {
  date: string;
  filed: Issue[];
  closed: Issue[];
}

export interface Timeline {
  /** Every day from the first stamp to today, gaps included: a pause is data. */
  days: Day[];
  peakFiled: number;
  peakClosed: number;
  closedTotal: number;
}

export function buildTimeline(issues: Issue[], today: string): Timeline {
  const now = toDay(today) ?? 0;
  const stamps = issues
    .flatMap((issue) => [toDay(issue.created), toDay(issue.closed)])
    .filter((day): day is number => day !== null);

  const first = Math.min(now, ...stamps);
  const last = Math.max(now, ...stamps);

  const days: Day[] = [];
  for (let day = first; day <= last; day += 1) {
    days.push({ date: fromDay(day), filed: [], closed: [] });
  }

  for (const issue of issues) {
    const filed = toDay(issue.created);
    if (filed !== null) days[filed - first].filed.push(issue);
    const closed = toDay(issue.closed);
    if (closed !== null) days[closed - first].closed.push(issue);
  }

  return {
    days,
    peakFiled: Math.max(0, ...days.map((day) => day.filed.length)),
    peakClosed: Math.max(0, ...days.map((day) => day.closed.length)),
    closedTotal: days.reduce((sum, day) => sum + day.closed.length, 0),
  };
}

export interface Span {
  issue: Issue;
  /** Indices into `Timeline.days`, both inclusive. */
  from: number;
  to: number;
  /** Days from filed to closed. Null while the issue is still running. */
  days: number | null;
}

/** One bar per issue. Issues with no Created date have nothing to draw. */
export function buildSpans(issues: Issue[], timeline: Timeline): Span[] {
  const origin = toDay(timeline.days[0].date)!;
  const end = timeline.days.length - 1;

  return issues
    .flatMap((issue) => {
      const filed = toDay(issue.created);
      if (filed === null) return [];
      const closed = toDay(issue.closed);
      const from = filed - origin;
      return [
        {
          issue,
          from,
          to: closed === null ? end : closed - origin,
          days: closed === null ? null : closed - filed,
        },
      ];
    })
    .sort((a, b) => a.from - b.from || a.issue.id.localeCompare(b.issue.id));
}

export interface Turnaround {
  /** Days from filed to closed, middle value. Null when nothing has closed. */
  median: number | null;
  counted: number;
  sameDay: number;
  longest: Span | null;
}

export function turnaround(spans: Span[]): Turnaround {
  const done = spans.filter((span) => span.days !== null);
  const lengths = done.map((span) => span.days!).sort((a, b) => a - b);
  const middle = lengths.length >> 1;

  return {
    median:
      lengths.length === 0
        ? null
        : lengths.length % 2 === 1
          ? lengths[middle]
          : (lengths[middle - 1] + lengths[middle]) / 2,
    counted: lengths.length,
    sameDay: lengths.filter((length) => length === 0).length,
    longest: done.reduce<Span | null>(
      (worst, span) => (worst === null || span.days! > worst.days! ? span : worst),
      null,
    ),
  };
}

export interface Waiting {
  issue: Issue;
  /** The dependencies that have not closed yet. */
  on: Issue[];
}

/**
 * Unfinished issues with an unfinished dependency. The complement - everything
 * with nothing in the way - is the pile you can actually pull from.
 */
export function waiting(issues: Issue[], byId: ReadonlyMap<string, Issue>): Waiting[] {
  return issues
    .filter((issue) => issue.status !== 'done')
    .flatMap((issue) => {
      const on = issue.dependsOn
        .map((id) => byId.get(id))
        .filter((dep): dep is Issue => dep !== undefined && dep.status !== 'done');
      return on.length === 0 ? [] : [{ issue, on }];
    })
    .sort((a, b) => b.on.length - a.on.length || a.issue.id.localeCompare(b.issue.id));
}

export interface Bearing {
  issue: Issue;
  /** Unfinished issues that name this one. */
  holds: Issue[];
  /** Every issue that names it, closed ones included. */
  total: number;
}

/** Closing these frees the most work. */
export function loadBearing(
  issues: Issue[],
  blockedBy: ReadonlyMap<string, Issue[]>,
): Bearing[] {
  return issues
    .flatMap((issue) => {
      const dependents = blockedBy.get(issue.id) ?? [];
      const holds = dependents.filter((dependent) => dependent.status !== 'done');
      return holds.length === 0 ? [] : [{ issue, holds, total: dependents.length }];
    })
    .sort(
      (a, b) =>
        b.holds.length - a.holds.length ||
        b.total - a.total ||
        a.issue.id.localeCompare(b.issue.id),
    );
}
