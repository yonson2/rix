import assert from 'node:assert/strict';
import { after, before, describe, test } from 'node:test';
import middleware, { config } from './middleware.ts';

const USER = 'peter';
const PASSWORD = 'pa:ss:word';

const call = (headers: Record<string, string> = {}) =>
  middleware(new Request('https://example.test/', { headers }));

const basic = (user: string, password: string) => ({
  authorization: `Basic ${Buffer.from(`${user}:${password}`).toString('base64')}`,
});

/** The matcher decides which paths the edge runs this on at all. */
const matches = (path: string) =>
  new RegExp(`^${config.matcher.replace(/^\//, '\\/')}$`).test(path);

describe('basic auth middleware', () => {
  before(() => {
    process.env.BASIC_AUTH_USER = USER;
    process.env.BASIC_AUTH_PASSWORD = PASSWORD;
  });

  after(() => {
    delete process.env.BASIC_AUTH_USER;
    delete process.env.BASIC_AUTH_PASSWORD;
  });

  test('a deployment with no credentials serves nothing', () => {
    delete process.env.BASIC_AUTH_USER;
    delete process.env.BASIC_AUTH_PASSWORD;
    assert.equal(call().status, 503);
    process.env.BASIC_AUTH_USER = USER;
    process.env.BASIC_AUTH_PASSWORD = PASSWORD;
  });

  test('an anonymous request is challenged', () => {
    const response = call();
    assert.equal(response.status, 401);
    assert.match(response.headers.get('www-authenticate') ?? '', /^Basic realm="Rix issue board"/);
  });

  test('the right credentials pass', () => {
    assert.ok(![401, 503].includes(call(basic(USER, PASSWORD)).status));
  });

  test('a password containing colons survives the round trip', () => {
    assert.ok(PASSWORD.includes(':'));
    assert.ok(![401, 503].includes(call(basic(USER, PASSWORD)).status));
  });

  test('wrong credentials are rejected', () => {
    assert.equal(call(basic(USER, 'wrong')).status, 401);
    assert.equal(call(basic('wrong', PASSWORD)).status, 401);
    assert.equal(call(basic(`${USER}:${PASSWORD}`, '')).status, 401);
  });

  test('a malformed header is rejected, not thrown on', () => {
    assert.equal(call({ authorization: 'Basic !!!!' }).status, 401);
    assert.equal(call({ authorization: 'Basic' }).status, 401);
    assert.equal(call({ authorization: 'Bearer token' }).status, 401);
    assert.equal(call({ authorization: '' }).status, 401);
  });

  test('every page is behind the matcher', () => {
    assert.ok(matches('/'));
    assert.ok(matches('/issues/'));
    assert.ok(matches('/issues/035/'));
  });

  test('the crawler policy files stay readable', () => {
    assert.ok(!matches('/robots.txt'));
    assert.ok(!matches('/llms.txt'));
  });
});
