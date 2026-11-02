declare const IssueIdBrand: unique symbol;

/** A three-digit issue number. Only `parseIssue` may mint one. */
export type IssueId = string & { readonly [IssueIdBrand]: true };

export const STATUSES = ['open', 'active', 'done'] as const;
export type Status = (typeof STATUSES)[number];

export interface Issue {
  id: IssueId;
  title: string;
  /** The directory the file lives in. The tracker moves files to change status. */
  status: Status;
  /** The `**Status:**` line, verbatim. Null when the field is absent. */
  declaredStatus: string | null;
  /** The file claims one status, the directory says another. */
  desync: boolean;
  milestone: string | null;
  dependsOn: IssueId[];
  /** The `**Depends on:**` line verbatim - it carries caveats the ids do not. */
  dependsOnNote: string | null;
  created: string | null;
  closed: string | null;
  body: string;
}

export type Parsed<T> = { kind: 'ok'; value: T } | { kind: 'err'; error: string };

const FILENAME = /^(\d{3})-[a-z0-9-]+\.md$/;
const HEADING = /^#\s+\[(\d{3})\]\s+(.+?)\s*$/m;
const FIELD = /^\*\*([^:*\n]+):\*\*[ \t]*(.*)$/gm;
const REF = /\[(\d{3})\]/g;

function isStatus(value: string): value is Status {
  return (STATUSES as readonly string[]).includes(value);
}

function field(fields: Map<string, string>, name: string): string | null {
  const value = fields.get(name.toLowerCase());
  if (value === undefined) return null;
  const trimmed = value.trim();
  return trimmed === '' || trimmed === '-' ? null : trimmed;
}

function refs(value: string | null): IssueId[] {
  if (value === null) return [];
  const found = [...value.matchAll(REF)].map(([, id]) => id as IssueId);
  return [...new Set(found)];
}

/**
 * Turns one tracker file into an `Issue`, or says why it cannot.
 * `status` comes from the directory: moving the file is how status changes.
 */
export function parseIssue(status: Status, filename: string, source: string): Parsed<Issue> {
  const name = FILENAME.exec(filename);
  if (name === null) {
    return { kind: 'err', error: `${filename}: expected NNN-kebab-title.md` };
  }
  const id = name[1] as IssueId;

  const heading = HEADING.exec(source);
  if (heading === null) {
    return { kind: 'err', error: `${filename}: missing "# [NNN] Title" heading` };
  }
  if (heading[1] !== id) {
    return { kind: 'err', error: `${filename}: heading says [${heading[1]}]` };
  }

  const split = source.search(/^## /m);
  const head = split === -1 ? source : source.slice(0, split);
  const body = split === -1 ? '' : source.slice(split);

  const fields = new Map<string, string>();
  for (const [, key, value] of head.matchAll(FIELD)) {
    fields.set(key.trim().toLowerCase(), value);
  }

  const declaredStatus = field(fields, 'status');
  const declared = declaredStatus?.toLowerCase() ?? '';
  const dependsOnNote = field(fields, 'depends on');

  return {
    kind: 'ok',
    value: {
      id,
      title: heading[2],
      status,
      declaredStatus,
      desync: isStatus(declared) && declared !== status,
      milestone: field(fields, 'milestone'),
      dependsOn: refs(dependsOnNote),
      dependsOnNote,
      created: field(fields, 'created'),
      closed: field(fields, 'closed'),
      body,
    },
  };
}

/**
 * Rewrites bare `[NNN]` references into markdown links, leaving code spans and
 * fenced blocks alone. Unknown ids stay literal.
 */
export function linkifyRefs(markdown: string, known: ReadonlySet<string>): string {
  return markdown
    .split(/(```[\s\S]*?```|`[^`\n]*`)/g)
    .map((chunk, index) =>
      index % 2 === 1
        ? chunk
        : chunk.replace(/\[(\d{3})\](?!\()/g, (whole, id: string) =>
            known.has(id) ? `[[${id}]](/issues/${id}/)` : whole,
          ),
    )
    .join('');
}
