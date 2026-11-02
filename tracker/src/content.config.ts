import fs from 'node:fs/promises';
import path from 'node:path';
import { defineCollection } from 'astro:content';
import type { Loader } from 'astro/loaders';
import {
  findIssuesDir,
  listIssueFiles,
  parseMilestoneGoals,
  parseMilestones,
  parseOpenOrder,
} from './lib/tracker';
import { STATUSES, linkifyRefs, parseIssue, type Issue } from './lib/issues';

/**
 * Reads `issues/{open,active,done}/*.md`. A file that does not parse fails the
 * build: a status board that quietly drops an issue is worse than no board.
 */
const issueLoader: Loader = {
  name: 'rix-issues',
  async load({ store, watcher, renderMarkdown, logger }) {
    const dir = findIssuesDir();
    const issues: Issue[] = [];

    for (const status of STATUSES) {
      const files = await listIssueFiles(path.join(dir, status));
      for (const file of files) {
        const source = await fs.readFile(path.join(dir, status, file), 'utf8');
        const parsed = parseIssue(status, file, source);
        if (parsed.kind === 'err') throw new Error(parsed.error);
        issues.push(parsed.value);
      }
    }

    const known = new Set(issues.map((issue) => issue.id));
    store.clear();
    for (const issue of issues) {
      store.set({
        id: issue.id,
        // The store is keyed loosely; `Issue` is the type readers get back.
        data: issue as unknown as Record<string, unknown>,
        rendered: await renderMarkdown(linkifyRefs(issue.body, known)),
      });
    }

    logger.info(`loaded ${issues.length} issues`);
    watcher?.add(dir);
  },
};

/** The index carries what the files cannot: milestone state, goals and the open pile's order. */
const indexLoader: Loader = {
  name: 'rix-index',
  async load({ store, watcher }) {
    const dir = findIssuesDir();
    const readme = await fs.readFile(path.join(dir, 'README.md'), 'utf8');
    store.clear();
    store.set({
      id: 'index',
      data: {
        milestones: parseMilestones(readme),
        goals: parseMilestoneGoals(readme),
        openOrder: parseOpenOrder(readme),
      },
    });
    watcher?.add(dir);
  },
};

export const collections = {
  issues: defineCollection({ loader: issueLoader }),
  index: defineCollection({ loader: indexLoader }),
};
