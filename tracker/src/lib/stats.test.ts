import assert from 'node:assert/strict';
import { describe, test } from 'node:test';
import { parseIssue, type Issue, type Status } from './issues.ts';
import {
  buildSpans,
  buildTimeline,
  fromDay,
  loadBearing,
  toDay,
  turnaround,
  waiting,
} from './stats.ts';

/** Builds an issue the way the loader does, so the tests see real parsed shapes. */
function issue(
  id: string,
  status: Status,
  fields: { created?: string; closed?: string; deps?: string } = {},
): Issue {
  const source = [
    `# [${id}] Issue ${id}`,
    '',
    `**Status:** ${status}`,
    '**Milestone:** M1',
    `**Depends on:** ${fields.deps ?? 'none'}`,
    `**Created:** ${fields.created ?? '-'}`,
    `**Closed:** ${fields.closed ?? '-'}`,
    '',
    '## Problem',
  ].join('\n');
  const parsed = parseIssue(status, `${id}-thing.md`, source);
  assert.equal(parsed.kind, 'ok');
  return (parsed as { kind: 'ok'; value: Issue }).value;
}

const byId = (issues: Issue[]) => new Map(issues.map((i) => [i.id as string, i]));

const blockedBy = (issues: Issue[]) => {
  const map = new Map<string, Issue[]>();
  for (const i of issues) {
    for (const dep of i.dependsOn) map.set(dep, [...(map.get(dep) ?? []), i]);
  }
  return map;
};

describe('dates', () => {
  test('round trips through day numbers', () => {
    assert.equal(fromDay(toDay('2026-09-16')!), '2026-09-16');
  });

  test('reads a date with a trailing note, and rejects prose', () => {
    assert.equal(fromDay(toDay('2026-09-16 (estimated)')!), '2026-09-16');
    assert.equal(toDay('none'), null);
    assert.equal(toDay(null), null);
  });
});

describe('timeline', () => {
  test('covers every day between the first stamp and today, gaps included', () => {
    const timeline = buildTimeline(
      [issue('001', 'done', { created: '2026-09-16', closed: '2026-09-16' })],
      '2026-09-20',
    );
    assert.equal(timeline.days.length, 5);
    assert.equal(timeline.days[0].date, '2026-09-16');
    assert.equal(timeline.days[4].date, '2026-09-20');
    assert.equal(timeline.days[1].filed.length, 0);
  });

  test('counts a filing and its close on the right days', () => {
    const timeline = buildTimeline(
      [issue('001', 'done', { created: '2026-09-16', closed: '2026-09-18' })],
      '2026-09-18',
    );
    assert.equal(timeline.days[0].filed.length, 1);
    assert.equal(timeline.days[0].closed.length, 0);
    assert.equal(timeline.days[2].closed.length, 1);
    assert.equal(timeline.closedTotal, 1);
    assert.equal(timeline.peakFiled, 1);
  });

  test('an issue with no Created date files on no day', () => {
    const timeline = buildTimeline([issue('001', 'open')], '2026-09-18');
    assert.equal(timeline.days.length, 1);
    assert.equal(timeline.days[0].filed.length, 0);
  });
});

describe('spans', () => {
  const issues = [
    issue('001', 'done', { created: '2026-09-16', closed: '2026-09-17' }),
    issue('002', 'open', { created: '2026-09-17' }),
    issue('003', 'open'),
  ];
  const timeline = buildTimeline(issues, '2026-09-18');
  const spans = buildSpans(issues, timeline);

  test('drops issues that have no start', () => {
    assert.deepEqual(
      spans.map((span) => span.issue.id),
      ['001', '002'],
    );
  });

  test('an open issue runs to the last day and has no length yet', () => {
    const open = spans.find((span) => span.issue.id === '002')!;
    assert.equal(open.from, 1);
    assert.equal(open.to, 2);
    assert.equal(open.days, null);
  });

  test('a closed issue measures filed to closed', () => {
    assert.equal(spans[0].days, 1);
  });
});

describe('turnaround', () => {
  const measure = (lengths: [string, string][]) => {
    const issues = lengths.map(([created, closed], index) =>
      issue(String(index + 101), 'done', { created, closed }),
    );
    return turnaround(buildSpans(issues, buildTimeline(issues, '2026-09-30')));
  };

  test('takes the middle value of an odd set', () => {
    const result = measure([
      ['2026-09-01', '2026-09-01'],
      ['2026-09-01', '2026-09-03'],
      ['2026-09-01', '2026-09-09'],
    ]);
    assert.equal(result.median, 2);
    assert.equal(result.sameDay, 1);
    assert.equal(result.longest?.days, 8);
  });

  test('averages the two middle values of an even set', () => {
    const result = measure([
      ['2026-09-01', '2026-09-01'],
      ['2026-09-01', '2026-09-02'],
      ['2026-09-01', '2026-09-03'],
      ['2026-09-01', '2026-09-04'],
    ]);
    assert.equal(result.median, 1.5);
    assert.equal(result.counted, 4);
  });

  test('says nothing when nothing has closed', () => {
    const issues = [issue('001', 'open', { created: '2026-09-01' })];
    const result = turnaround(buildSpans(issues, buildTimeline(issues, '2026-09-02')));
    assert.equal(result.median, null);
    assert.equal(result.longest, null);
  });
});

describe('waiting', () => {
  const issues = [
    issue('001', 'done', { created: '2026-09-01', closed: '2026-09-01' }),
    issue('002', 'open', { created: '2026-09-01', deps: '[001]' }),
    issue('003', 'open', { created: '2026-09-01', deps: '[002], [001]' }),
    issue('004', 'open', { created: '2026-09-01', deps: '[999]' }),
  ];

  test('only lists issues whose dependencies are still open', () => {
    const blocked = waiting(issues, byId(issues));
    assert.deepEqual(
      blocked.map((entry) => entry.issue.id),
      ['003'],
    );
    assert.deepEqual(
      blocked[0].on.map((dep) => dep.id),
      ['002'],
    );
  });

  test('a reference to an issue that does not exist blocks nothing', () => {
    assert.equal(
      waiting(issues, byId(issues)).some((entry) => entry.issue.id === '004'),
      false,
    );
  });
});

describe('loadBearing', () => {
  const issues = [
    issue('001', 'done', { created: '2026-09-01', closed: '2026-09-01' }),
    issue('002', 'open', { created: '2026-09-01', deps: '[001]' }),
    issue('003', 'open', { created: '2026-09-01', deps: '[001], [002]' }),
    issue('004', 'done', { created: '2026-09-01', closed: '2026-09-02', deps: '[002]' }),
  ];
  const ranked = loadBearing(issues, blockedBy(issues));

  test('ranks by how much unfinished work each one holds', () => {
    assert.deepEqual(
      ranked.map((entry) => entry.issue.id),
      ['001', '002'],
    );
    assert.equal(ranked[0].holds.length, 2);
  });

  test('counts closed dependents in the total but not in the hold', () => {
    const second = ranked[1];
    assert.equal(second.holds.length, 1);
    assert.equal(second.total, 2);
  });
});
