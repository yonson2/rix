import { existsSync } from 'node:fs';
import { readdir } from 'node:fs/promises';
import path from 'node:path';

export interface Milestone {
  code: string;
  name: string;
  shipped: boolean;
}

const MILESTONE_LINE = /^- \[( |x)\] (M\d+) (.+?)\s*$/gm;
const GOAL_LINE = /^- \*\*(M\d+)\b[^*\n]*:\*\*\s*(.*)$/;
const LISTED_ID = /^- \[(\d{3})\]/gm;

/**
 * The tracker lives outside this package, and the build's working directory is
 * either `tracker/` (local) or the repo root (host), so walk up and find it.
 * Only the content layer calls this: bundled page code has no source path.
 */
export function findIssuesDir(): string {
  let dir = process.cwd();
  for (let up = 0; up < 5; up += 1) {
    const candidate = path.join(dir, 'issues');
    if (existsSync(path.join(candidate, 'README.md'))) return candidate;
    dir = path.dirname(dir);
  }
  throw new Error(`no issues/README.md at or above ${process.cwd()}`);
}

/**
 * The `.md` files of one status directory, sorted. Git keeps no empty
 * directories, so a status with no issues (often `active/`) is absent from a
 * fresh checkout: that reads as empty.
 */
export async function listIssueFiles(statusDir: string): Promise<string[]> {
  try {
    return (await readdir(statusDir)).filter((file) => file.endsWith('.md')).sort();
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') return [];
    throw error;
  }
}

function section(readme: string, heading: string): string {
  const start = readme.search(new RegExp(`^## ${heading}`, 'm'));
  if (start === -1) return '';
  const rest = readme.slice(start);
  const end = rest.slice(1).search(/^## /m);
  return end === -1 ? rest : rest.slice(0, end + 1);
}

export function parseMilestones(readme: string): Milestone[] {
  return [...section(readme, 'Milestones').matchAll(MILESTONE_LINE)].map(
    ([, mark, code, name]) => ({ code, name, shipped: mark === 'x' }),
  );
}

/**
 * The index keeps the open pile in priority order and "continue" means take the
 * top of it, so the board has to show that order rather than sort by id.
 */
export function parseOpenOrder(readme: string): string[] {
  return [...section(readme, 'Open').matchAll(LISTED_ID)].map(([, id]) => id);
}

/**
 * `## Milestone goals` holds one `- **M0 Name:** text` item per milestone; the
 * text may wrap onto indented lines. Kept apart from `parseMilestones`, which
 * reads `## Milestones` only.
 */
export function parseMilestoneGoals(readme: string): Record<string, string> {
  const goals: Record<string, string> = {};
  let code: string | null = null;
  for (const line of section(readme, 'Milestone goals').split('\n')) {
    const item = GOAL_LINE.exec(line);
    if (item !== null) {
      code = item[1];
      goals[code] = item[2].trim();
    } else if (code !== null && /^\s+\S/.test(line)) {
      goals[code] = `${goals[code]} ${line.trim()}`.trim();
    } else {
      code = null;
    }
  }
  return goals;
}
