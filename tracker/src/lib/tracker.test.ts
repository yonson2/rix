import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { describe, test } from 'node:test';
import { parseIssue, type Issue } from './issues.ts';
import { listIssueFiles, parseMilestoneGoals, parseMilestones, parseOpenOrder } from './tracker.ts';
import { groupByMilestone, milestoneStamp, NO_MILESTONE } from './stamps.ts';

const README = [
  '# Rix Issues',
  '',
  '## Active',
  '',
  '- [099] M0: Already started',
  '',
  '## Open (priority order)',
  '',
  '*Order set 2026-11-02.*',
  '',
  '- [006] M0: Dev harness - fixed seed, scripted line, screenshot and exit',
  '- [013] M1: Replace bevy_prototype_lyon (D2)',
  '- [010] Gate: Product decisions for v1; **gate**',
  '',
  '## Milestones',
  '- [x] M0 Foundations',
  '- [ ] M1 Engine upgrade',
  '- [ ] M6 Desktop and web release',
  '',
  '## Milestone goals',
  '',
  '- **M0 Foundations:** an agent can pick up Rix cold: `AGENTS.md`,',
  '  `ARCHITECTURE.md`, the board.',
  '- **M1 Engine upgrade:** Rix on Bevy 0.20.1 (D3).',
  '- [ ] M9 Not a milestone, only a line under the goals heading',
].join('\n');

describe('parseMilestones', () => {
  test('reads the checkbox lines under ## Milestones, in order', () => {
    assert.deepEqual(parseMilestones(README), [
      { code: 'M0', name: 'Foundations', shipped: true },
      { code: 'M1', name: 'Engine upgrade', shipped: false },
      { code: 'M6', name: 'Desktop and web release', shipped: false },
    ]);
  });

  test('reads no line from ## Milestone goals', () => {
    assert.ok(!parseMilestones(README).some((milestone) => milestone.code === 'M9'));
  });

  test('has no point milestones: M9.5 is not a milestone line', () => {
    assert.deepEqual(parseMilestones('## Milestones\n- [ ] M9.5 UX polish batch\n'), []);
  });
});

describe('parseMilestoneGoals', () => {
  test('joins a wrapped goal into one line', () => {
    assert.deepEqual(parseMilestoneGoals(README), {
      M0: 'an agent can pick up Rix cold: `AGENTS.md`, `ARCHITECTURE.md`, the board.',
      M1: 'Rix on Bevy 0.20.1 (D3).',
    });
  });

  test('is empty when the section is absent', () => {
    assert.deepEqual(parseMilestoneGoals('## Milestones\n- [ ] M0 Foundations\n'), {});
  });
});

describe('parseOpenOrder', () => {
  test('keeps the index order and reads the open section only', () => {
    assert.deepEqual(parseOpenOrder(README), ['006', '013', '010']);
  });
});

describe('milestoneStamp', () => {
  test('takes the code at the start of the field', () => {
    assert.equal(milestoneStamp('M0'), 'M0');
    assert.equal(milestoneStamp('M12 (after the gate)'), 'M12');
  });

  test('anything else is no milestone', () => {
    assert.equal(milestoneStamp(null), NO_MILESTONE);
    assert.equal(milestoneStamp('Infrastructure and tooling'), NO_MILESTONE);
  });
});

describe('groupByMilestone', () => {
  const issue = (id: string, milestone: string): Issue => {
    const source = `# [${id}] Issue ${id}\n\n**Status:** open\n**Milestone:** ${milestone}\n`;
    const parsed = parseIssue('open', `${id}-thing.md`, source);
    assert.equal(parsed.kind, 'ok');
    return (parsed as { kind: 'ok'; value: Issue }).value;
  };
  const issues = [issue('002', 'M1'), issue('001', 'M1'), issue('003', 'M7'), issue('004', '-')];
  const { spine, rest } = groupByMilestone(issues, parseMilestones(README));

  test('one group per index milestone, issues sorted by id', () => {
    assert.deepEqual(
      spine.map((group) => [group.code, group.issues.map((i) => i.id)]),
      [
        ['M0', []],
        ['M1', ['001', '002']],
        ['M6', []],
      ],
    );
  });

  test('an issue on a milestone the index does not list still shows', () => {
    assert.deepEqual(
      rest.map((group) => [group.code, group.issues.map((i) => i.id)]),
      [[NO_MILESTONE, ['003', '004']]],
    );
  });
});

describe('listIssueFiles', () => {
  test('reads a missing status directory as empty', async () => {
    const root = await mkdtemp(path.join(tmpdir(), 'rix-issues-'));
    assert.deepEqual(await listIssueFiles(path.join(root, 'active')), []);
  });

  test('lists the markdown files only, sorted', async () => {
    const dir = path.join(await mkdtemp(path.join(tmpdir(), 'rix-issues-')), 'open');
    await mkdir(dir);
    for (const file of ['101-b.md', '100-a.md', '.gitkeep']) {
      await writeFile(path.join(dir, file), '');
    }
    assert.deepEqual(await listIssueFiles(dir), ['100-a.md', '101-b.md']);
  });
});
