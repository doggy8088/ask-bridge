#!/usr/bin/env node
'use strict';

const { request } = require('node:https');
const { URL } = require('node:url');
const { artifactName, releaseBaseUrl, TARGETS } = require('./postinstall.cjs');

const MAX_REDIRECTS = 5;

function packageVersion() {
  return require('../package.json').version;
}

function expectedReleaseUrls(version = packageVersion()) {
  const base = releaseBaseUrl(version);
  return Object.values(TARGETS).flatMap((target) => {
    const archive = artifactName(target);
    return [`${base}/${archive}`, `${base}/${archive}.sha256`];
  });
}

function checkUrl(url, redirectsRemaining = MAX_REDIRECTS) {
  return new Promise((resolve) => {
    const req = request(url, { method: 'HEAD' }, (res) => {
      const { statusCode, headers } = res;
      res.resume();

      if (statusCode >= 300 && statusCode < 400 && headers.location && redirectsRemaining > 0) {
        const nextUrl = new URL(headers.location, url).toString();
        checkUrl(nextUrl, redirectsRemaining - 1).then((result) => resolve({ ...result, url }));
        return;
      }

      resolve({
        url,
        ok: statusCode >= 200 && statusCode < 300,
        statusCode,
      });
    });

    req.on('error', (error) => {
      resolve({ url, ok: false, errorMessage: error.message });
    });
    req.end();
  });
}

function retryCountFromEnv() {
  return Number.parseInt(process.env.ASK_CHATGPT_RELEASE_ASSET_RETRIES ?? '1', 10);
}

function retryDelayMsFromEnv() {
  return Number.parseInt(process.env.ASK_CHATGPT_RELEASE_ASSET_RETRY_DELAY_MS ?? '1000', 10);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function formatFailure(result) {
  const reason = result.statusCode ? `HTTP ${result.statusCode}` : result.errorMessage;
  return `- ${result.url} (${reason})`;
}

async function verifyReleaseAssets({
  version = packageVersion(),
  check = checkUrl,
  retries = retryCountFromEnv(),
  retryDelayMs = retryDelayMsFromEnv(),
} = {}) {
  const urls = expectedReleaseUrls(version);
  let failures = [];

  for (let attempt = 1; attempt <= retries; attempt += 1) {
    const results = await Promise.all(
      urls.map(async (url) => ({
        url,
        ...(await check(url)),
      })),
    );
    failures = results.filter((result) => !result.ok);
    if (failures.length === 0) return urls;
    if (attempt < retries) await sleep(retryDelayMs);
  }

  throw new Error(
    [
      `Missing or unavailable release assets for v${version}:`,
      ...failures.map(formatFailure),
      'Create and host the GitHub release assets before publishing npm.',
    ].join('\n'),
  );
}

async function main() {
  const version = packageVersion();
  const urls = await verifyReleaseAssets({ version });
  console.log(`Verified ${urls.length} release assets for v${version}.`);
}

if (require.main === module) {
  main().catch((error) => {
    console.error(error.message);
    process.exit(1);
  });
}

module.exports = {
  checkUrl,
  expectedReleaseUrls,
  formatFailure,
  retryCountFromEnv,
  retryDelayMsFromEnv,
  verifyReleaseAssets,
};
