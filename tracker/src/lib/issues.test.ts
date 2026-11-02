import assert from 'node:assert/strict';
import { describe, test } from 'node:test';
import { linkifyRefs, parseIssue, type Issue } from './issues.ts';

const SOURCE = [
  "# [004] Issue board - port mud's Astro board to `tracker/`",
  '',
  '**Status:** open',
  '**Milestone:** M0',
  '**Depends on:** [003]',
  '**Created:** 2026-11-02',
  '**Closed:** -',
  '',
  '## Problem',
  'See [003].',
].join('\n');

describe('parseIssue', () => {
  test('reads the heading and every field', () => {
    const parsed = parseIssue('open', '004-issue-board.md', SOURCE);
    assert.equal(parsed.kind, 'ok');
    const issue = (parsed as { kind: 'ok'; value: Issue }).value;
    assert.equal(issue.id, '004');
    assert.equal(issue.title, "Issue board - port mud's Astro board to `tracker/`");
    assert.equal(issue.milestone, 'M0');
    assert.deepEqual(issue.dependsOn, ['003']);
    assert.equal(issue.created, '2026-11-02');
    assert.equal(issue.closed, null);
    assert.equal(issue.desync, false);
    assert.match(issue.body, /^## Problem/);
  });

  test('flags a file whose Status line disagrees with its directory', () => {
    const parsed = parseIssue('done', '004-issue-board.md', SOURCE);
    assert.equal(parsed.kind === 'ok' && parsed.value.desync, true);
  });

  test('rejects a bad filename, a missing heading and a wrong id', () => {
    assert.equal(parseIssue('open', '4-issue-board.md', SOURCE).kind, 'err');
    assert.equal(parseIssue('open', '004-Issue_Board.md', SOURCE).kind, 'err');
    assert.equal(parseIssue('open', '004-x.md', SOURCE.replace(/^# /, '')).kind, 'err');
    assert.equal(parseIssue('open', '005-issue-board.md', SOURCE).kind, 'err');
  });
});

describe('linkifyRefs', () => {
  test('links known ids outside code and leaves the rest', () => {
    assert.equal(
      linkifyRefs('[003] and [999], not `[003]`', new Set(['003'])),
      '[[003]](/issues/003/) and [999], not `[003]`',
    );
  });
});
