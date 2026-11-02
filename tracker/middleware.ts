import { next } from '@vercel/edge';

/**
 * Basic auth over the whole board. The tracker is private working material and
 * the robots directives that ship with it are advisory - this is the part that
 * actually keeps strangers out.
 *
 * Credentials come from the deployment's environment (`BASIC_AUTH_USER`,
 * `BASIC_AUTH_PASSWORD`), never from source: this file is public in the repo.
 * They must be set as runtime environment variables on the host, not injected
 * at build time, or the password ends up inside the deployed bundle.
 *
 * Runs on the edge only. Local `npm run dev` and `npm run preview` are
 * unaffected.
 */
export const config = {
  // Everything except the crawler policy files, which have to stay readable
  // for the robots rules to mean anything.
  matcher: '/((?!robots\\.txt|llms\\.txt).*)',
};

const REALM = 'Basic realm="Rix issue board", charset="UTF-8"';

/** Constant time, so a wrong guess leaks nothing through response timing. */
function sameCredential(given: string, expected: string): boolean {
  if (given.length !== expected.length) return false;
  let diff = 0;
  for (let i = 0; i < given.length; i += 1) {
    diff |= given.charCodeAt(i) ^ expected.charCodeAt(i);
  }
  return diff === 0;
}

function decode(header: string): string {
  const [scheme, encoded] = header.split(' ');
  if (scheme?.toLowerCase() !== 'basic' || !encoded) return '';
  try {
    return atob(encoded);
  } catch {
    return '';
  }
}

export default function middleware(request: Request): Response {
  const user = process.env.BASIC_AUTH_USER;
  const password = process.env.BASIC_AUTH_PASSWORD;

  // Fail closed. A deployment with no credentials set serves nothing rather
  // than quietly publishing the tracker.
  if (!user || !password) {
    return new Response('This deployment has no credentials configured.\n', {
      status: 503,
      headers: { 'Cache-Control': 'no-store' },
    });
  }

  const given = decode(request.headers.get('authorization') ?? '');
  if (given && sameCredential(given, `${user}:${password}`)) {
    return next();
  }

  return new Response('Authentication required.\n', {
    status: 401,
    headers: { 'WWW-Authenticate': REALM, 'Cache-Control': 'no-store' },
  });
}
