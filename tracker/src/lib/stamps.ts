import type { Issue } from './issues.ts';
import type { Milestone } from './tracker.ts';

/** Stamp for an issue whose Milestone field names no point on the spine. */
export const NO_MILESTONE = 'none';

/** The Milestone field is free text. Squeeze it into something stampable. */
export function milestoneStamp(milestone: string | null): string {
  if (milestone === null) return NO_MILESTONE;
  return /^M\d+\b/.exec(milestone)?.[0] ?? NO_MILESTONE;
}

/** Most titles repeat the milestone they are already stamped with; strip it. */
export function displayTitle(issue: Issue): string {
  return issue.title.replace(new RegExp(`^${milestoneStamp(issue.milestone)}:\\s*`), '');
}

export interface Group {
  code: string;
  name: string;
  issues: Issue[];
}

/**
 * The board groups by status. The index groups by milestone, which is the
 * question the board cannot answer: what is actually inside M5? An issue whose
 * milestone is not in the index lands in `rest`, so no issue drops off the page.
 */
export function groupByMilestone(
  issues: Issue[],
  milestones: Milestone[],
): { spine: Group[]; rest: Group[] } {
  const codes = new Set(milestones.map((milestone) => milestone.code));
  const byStamp = new Map<string, Issue[]>();
  for (const issue of issues) {
    const stamp = milestoneStamp(issue.milestone);
    const key = codes.has(stamp) ? stamp : NO_MILESTONE;
    byStamp.set(key, [...(byStamp.get(key) ?? []), issue]);
  }
  const take = (code: string, name: string): Group => ({
    code,
    name,
    issues: (byStamp.get(code) ?? []).sort((a, b) => a.id.localeCompare(b.id)),
  });

  return {
    spine: milestones.map((milestone) => take(milestone.code, milestone.name)),
    rest: [take(NO_MILESTONE, 'No milestone')].filter((group) => group.issues.length > 0),
  };
}
