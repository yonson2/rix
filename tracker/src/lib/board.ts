import { getCollection, getEntry } from 'astro:content';
import type { Issue, Status } from './issues';
import type { Milestone } from './tracker';

// The stamp and grouping helpers are pure and live in stamps.ts (node --test
// cannot load this module's astro:content import); re-exported so consumers
// keep one import site.
export {
  displayTitle,
  groupByMilestone,
  milestoneStamp,
  NO_MILESTONE,
  type Group,
} from './stamps';

export interface Board {
  issues: Issue[];
  byId: Map<string, Issue>;
  columns: { status: Status; issues: Issue[] }[];
  /** id -> the issues that name it in their `Depends on` line. */
  blockedBy: Map<string, Issue[]>;
  milestones: Milestone[];
  /** The first milestone the index has not ticked off. */
  current: Milestone | undefined;
  /** Milestone code -> its goal from the index's `## Milestone goals`. */
  goals: Record<string, string>;
}

/**
 * Cards shown per column before the rest folds behind the "more" control.
 * Chosen so the whole board clears a 27" viewport without scrolling - the
 * point of the fold. Raising it defeats that; measure before you do.
 */
export const COLUMN_LIMIT = 5;

const LABEL: Record<Status, string> = {
  open: 'Backlog',
  active: 'In progress',
  done: 'Shipped',
};

export function labelOf(status: Status): string {
  return LABEL[status];
}

/** Inline style that sets a block's `--accent` and `--tint` to its status. */
export function toneOf(status: Status): string {
  return `--accent:var(--${status});--tint:var(--${status}-tint)`;
}

export async function loadBoard(): Promise<Board> {
  const issues = (await getCollection('issues')).map((entry) => entry.data);
  const byId = new Map(issues.map((issue) => [issue.id as string, issue]));

  const blockedBy = new Map<string, Issue[]>();
  for (const issue of issues) {
    for (const dep of issue.dependsOn) {
      blockedBy.set(dep, [...(blockedBy.get(dep) ?? []), issue]);
    }
  }

  const index = await getEntry('index', 'index');
  const priority: string[] = index?.data.openOrder ?? [];
  const milestones: Milestone[] = index?.data.milestones ?? [];
  const goals: Record<string, string> = index?.data.goals ?? {};

  const rank = (id: string) => {
    const at = priority.indexOf(id);
    return at === -1 ? priority.length : at;
  };

  const pick = (status: Status) => issues.filter((issue) => issue.status === status);

  return {
    issues,
    byId,
    blockedBy,
    milestones,
    current: milestones.find((milestone) => !milestone.shipped),
    goals,
    columns: [
      {
        status: 'open',
        issues: pick('open').sort((a, b) => rank(a.id) - rank(b.id) || a.id.localeCompare(b.id)),
      },
      { status: 'active', issues: pick('active').sort((a, b) => a.id.localeCompare(b.id)) },
      {
        status: 'done',
        issues: pick('done').sort(
          (a, b) => (b.closed ?? '').localeCompare(a.closed ?? '') || b.id.localeCompare(a.id),
        ),
      },
    ],
  };
}
