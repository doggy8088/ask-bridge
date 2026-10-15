'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const {
  DEFAULT_PROGRESS_SELECTOR,
  collectImageCandidates,
  findActiveTurn,
  imageScanDecision,
  inspectActiveImageProgress,
  isImageLoadReady,
  parseProgressValue,
  progressBlocksCompletion,
  readTurnProgress,
} = require('../src/image-progress.cjs');

const ASSISTANT_SELECTOR = '[data-message-author-role="assistant"], .agent-turn';

class FakeElement {
  constructor({
    attrs = {},
    textContent = '',
    images = [],
    markers = [],
    hints = [],
    className = '',
    parentElement = null,
  } = {}) {
    this.attrs = { ...attrs };
    this.textContent = textContent;
    this.innerText = textContent;
    this.images = images;
    this.markers = markers;
    this.hints = hints;
    this.className = className;
    this.parentElement = parentElement;
    this.parentNode = parentElement;
  }

  getAttribute(name) {
    return this.attrs[name] == null ? null : this.attrs[name];
  }

  matches(selector) {
    return selector === '.agent-turn' && this.className.split(/\s+/).includes('agent-turn');
  }

  closest(selector) {
    let current = this;
    while (current) {
      if (current.matches && current.matches(selector)) return current;
      current = current.parentElement;
    }
    return null;
  }

  querySelectorAll(selector) {
    if (selector === 'img') return this.images;
    if (selector === DEFAULT_PROGRESS_SELECTOR) return this.markers;
    if (selector.includes('[data-testid="image-gen-loading-state-frame"]')) return this.hints;
    return [];
  }

  querySelector(selector) {
    return this.querySelectorAll(selector)[0] || null;
  }
}

class FakeDocument {
  constructor(turns, selector = ASSISTANT_SELECTOR) {
    this.turns = turns;
    this.selector = selector;
    this.stopButton = null;
  }

  querySelectorAll(selector) {
    if (selector === this.selector) return this.turns;
    return [];
  }

  querySelector(selector) {
    return selector === '[data-testid="stop-button"]' ? this.stopButton : null;
  }
}

function marker(value, { ariaValue = value, text = `${value}%` } = {}) {
  const attrs = {
    'data-testid': 'image-gen-loading-progress',
    role: 'progressbar',
    'aria-valuemin': '0',
    'aria-valuemax': '100',
  };
  if (ariaValue != null) attrs['aria-valuenow'] = String(ariaValue);
  return new FakeElement({ attrs, textContent: text });
}

function image({ src = 'https://chatgpt.com/backend-api/estuary/content?id=1', complete = true, naturalWidth = 1024 } = {}) {
  return {
    src,
    alt: 'generated',
    complete,
    naturalWidth,
    naturalHeight: naturalWidth,
    addEventListener() {},
    removeEventListener() {},
    getAttribute(name) {
      return name === 'src' ? src : null;
    },
  };
}

function turn({ markers = [], images = [], hints = [], className = 'agent-turn', textContent = '' } = {}) {
  return new FakeElement({ markers, images, hints, className, textContent });
}

function inspect(turns, initialAssistantCount = 0) {
  return inspectActiveImageProgress({
    document: new FakeDocument(turns),
    assistantSelector: ASSISTANT_SELECTOR,
    initialAssistantCount,
  });
}

test('reads the exact ChatGPT marker and keeps 0..99 pending', () => {
  const result = inspect([turn({ markers: [marker(27)] })]);

  assert.equal(result.isNew, true);
  assert.equal(result.markerPresent, true);
  assert.equal(result.progress, 27);
  assert.equal(result.progressState, 'pending');
  assert.equal(progressBlocksCompletion(result), true);
});

test('uses marker text only as a percent fallback', () => {
  const result = inspect([turn({ markers: [marker(null, { ariaValue: null, text: '42%' })] })]);

  assert.equal(result.progress, 42);
  assert.equal(result.progressSource, 'text');
});

test('accepts only bounded, complete percent values', () => {
  assert.equal(parseProgressValue('0%', true), 0);
  assert.equal(parseProgressValue('100%', true), 100);
  assert.equal(parseProgressValue('-10%', true), null);
  assert.equal(parseProgressValue('101%', true), null);
  assert.equal(parseProgressValue('1000%', true), null);
  assert.equal(parseProgressValue('NaN%', true), null);
});

test('100% stays blocked until every generated image is load ready', () => {
  const pendingImage = image({ complete: false, naturalWidth: 0 });
  const currentTurn = turn({ markers: [marker(100)], images: [pendingImage] });
  const beforeLoad = inspect([currentTurn]);

  assert.equal(beforeLoad.progress, 100);
  assert.equal(beforeLoad.readyCount, 0);
  assert.equal(beforeLoad.allReady, false);
  assert.equal(progressBlocksCompletion(beforeLoad), true);
  assert.equal(imageScanDecision({
    progress: beforeLoad,
    candidateCount: 1,
    imageCount: 0,
    allImagesReady: false,
    hasImageHint: true,
  }).shouldRetry, true);

  pendingImage.complete = true;
  pendingImage.naturalWidth = 1024;
  pendingImage.naturalHeight = 1024;
  const afterLoad = inspect([currentTurn]);

  assert.equal(afterLoad.allReady, true);
  assert.equal(progressBlocksCompletion(afterLoad), false);
  assert.equal(imageScanDecision({
    progress: afterLoad,
    candidateCount: 1,
    imageCount: 1,
    allImagesReady: true,
    hasImageHint: true,
  }).canReturnImages, true);
});

test('treats mixed progress markers conservatively while a lower value remains', () => {
  const result = inspect([turn({ markers: [marker(20), marker(100)] })]);

  assert.equal(result.progress, 20);
  assert.equal(result.progressState, 'pending');
  assert.equal(progressBlocksCompletion(result), true);
});

test('does not latch progress when the marker disappears during preview', () => {
  const generatedImage = image();
  const currentTurn = turn({ markers: [marker(70)], images: [generatedImage] });
  assert.equal(inspect([currentTurn]).progress, 70);

  currentTurn.markers = [];
  const disappeared = inspect([currentTurn]);
  assert.equal(disappeared.markerPresent, false);
  assert.equal(disappeared.progress, null);
  assert.equal(disappeared.progressState, 'none');

  const previewDecision = imageScanDecision({
    generationActive: true,
    progress: disappeared,
    candidateCount: 1,
    imageCount: 1,
    allImagesReady: true,
    hasImageHint: true,
  });
  assert.equal(previewDecision.canReturnImages, false);
  assert.equal(previewDecision.shouldRetry, true);

  const finalDecision = imageScanDecision({
    generationActive: false,
    progress: disappeared,
    candidateCount: 1,
    imageCount: 1,
    allImagesReady: true,
    hasImageHint: true,
  });
  assert.equal(finalDecision.canReturnImages, true);
});

test('reports a retreat or restart in the current marker value', () => {
  const currentTurn = turn({ markers: [marker(70)] });
  assert.equal(inspect([currentTurn]).progress, 70);

  currentTurn.markers = [marker(35)];
  assert.equal(inspect([currentTurn]).progress, 35);
});

test('waits for all distinct images in a multi-image response', () => {
  const images = [
    image({ src: 'https://chatgpt.com/backend-api/estuary/content?id=1' }),
    image({ src: 'https://chatgpt.com/backend-api/estuary/content?id=2' }),
  ];
  const currentTurn = turn({ markers: [marker(100)], images });
  const result = inspect([currentTurn]);

  assert.equal(result.candidateCount, 2);
  assert.equal(result.readyCount, 2);
  assert.equal(collectImageCandidates(currentTurn).length, 2);
  assert.equal(imageScanDecision({
    progress: result,
    candidateCount: 2,
    imageCount: 1,
    allImagesReady: true,
    hasImageHint: true,
  }).canReturnImages, false);
});

test('ignores an ordinary article percentage without the exact marker', () => {
  const result = inspect([turn({ textContent: 'The article reports 73% completion.' })]);

  assert.equal(result.markerPresent, false);
  assert.equal(result.progress, null);
  assert.equal(result.hasImageHint, false);
});

test('does not inspect an old turn when no new assistant turn exists', () => {
  const oldTurn = turn({ markers: [marker(99)] });
  const result = inspect([oldTurn], 1);

  assert.equal(result.hasTurn, false);
  assert.equal(result.isNew, false);
  assert.equal(result.progress, null);
});

test('normalizes a nested assistant marker to its outer agent turn', () => {
  const outerTurn = turn({ markers: [marker(27)] });
  const nestedAssistant = new FakeElement({
    attrs: { 'data-message-author-role': 'assistant' },
    parentElement: outerTurn,
  });
  const turns = [outerTurn, nestedAssistant];
  const document = new FakeDocument(turns);

  assert.equal(findActiveTurn({
    document,
    assistantSelector: ASSISTANT_SELECTOR,
    initialAssistantCount: 0,
  }), outerTurn);
  assert.equal(inspectActiveImageProgress({
    document,
    assistantSelector: ASSISTANT_SELECTOR,
    initialAssistantCount: 0,
  }).progress, 27);
});

test('normalizes nested agent turns to the outermost agent turn', () => {
  const outerTurn = turn({ markers: [marker(27)], images: [image()] });
  const nestedTurn = new FakeElement({
    className: 'agent-turn',
    parentElement: outerTurn,
  });
  const document = new FakeDocument([nestedTurn]);

  const result = inspectActiveImageProgress({
    document,
    assistantSelector: ASSISTANT_SELECTOR,
    initialAssistantCount: 0,
  });

  assert.equal(findActiveTurn({
    document,
    assistantSelector: ASSISTANT_SELECTOR,
    initialAssistantCount: 0,
  }), outerTurn);
  assert.equal(result.progress, 27);
  assert.equal(result.candidateCount, 1);
});

test('chooses the latest response across selectors without a baseline', () => {
  const latestSelector = '[data-testid="fallback-response"]';
  const oldTurn = turn({ markers: [marker(20)] });
  const latestFallback = new FakeElement({
    attrs: { 'data-testid': 'fallback-response' },
    markers: [marker(80)],
  });
  const document = {
    querySelectorAll(selector) {
      if (selector === ASSISTANT_SELECTOR) return [oldTurn];
      if (selector === latestSelector) return [latestFallback];
      if (selector === `${ASSISTANT_SELECTOR}, ${latestSelector}`) {
        return [oldTurn, latestFallback];
      }
      return [];
    },
  };

  const result = inspectActiveImageProgress({
    document,
    assistantSelector: ASSISTANT_SELECTOR,
    latestSelector,
  });

  assert.equal(result.progress, 80);
});

test('keeps no-progress responses on the existing image readiness fallback', () => {
  const currentTurn = turn({ images: [image()] });
  const result = inspect([currentTurn]);

  assert.equal(result.markerPresent, false);
  assert.equal(result.allReady, true);
  assert.equal(imageScanDecision({
    chatgptProgress: false,
    progress: result,
    candidateCount: 1,
    imageCount: 1,
    allImagesReady: true,
    hasImageHint: true,
  }).canReturnImages, true);
});

test('keeps pending scans retryable for the Rust wall-clock timeout', () => {
  const decision = imageScanDecision({
    progress: { markerPresent: true, progress: 12 },
    candidateCount: 0,
    imageCount: 0,
    allImagesReady: false,
    hasImageHint: true,
  });

  assert.equal(decision.canReturnImages, false);
  assert.equal(decision.shouldRetry, true);
  assert.equal(isImageLoadReady(image({ complete: false, naturalWidth: 0 })), false);
});


const rust = fs.readFileSync(path.join(__dirname, '../src/main.rs'), 'utf8');
const helper = fs.readFileSync(path.join(__dirname, '../src/image-progress.cjs'), 'utf8');

function embeddedScript(kind, {
  chatgpt = true,
  selector = ASSISTANT_SELECTOR,
  latestSelector = selector,
  baseline = 0,
  timeout = 0,
} = {}) {
  const template = kind === 'response'
    ? rust.match(/let response_check_js = r###"([\s\S]*?)"###/)[1]
    : rust.slice(rust.indexOf('fn build_image_scan_script(')).match(/r###"([\s\S]*?)"###/)[1];
  const replacements = {
    __IMAGE_PROGRESS_HELPER__: helper,
    __CHATGPT_PROGRESS__: String(chatgpt),
    __LATEST_SELECTOR__: JSON.stringify(latestSelector),
    __ASSISTANT_SELECTOR__: JSON.stringify(selector),
    __INITIAL_ASSISTANT_COUNT__: JSON.stringify(baseline),
    __INITIAL_COUNT__: JSON.stringify(baseline),
    __STOP_SELECTORS__: JSON.stringify(['[data-testid="stop-button"]']),
    __IMAGE_WAIT_TIMEOUT_MS__: String(timeout),
  };
  let script = template;
  for (const [placeholder, value] of Object.entries(replacements)) {
    script = script.split(placeholder).join(value);
  }
  assert.doesNotMatch(script, /__[A-Z_]+__/);
  return script;
}

function scriptContext(document, overrides = {}) {
  const context = {
    document,
    performance,
    setTimeout,
    clearTimeout,
    getComputedStyle: () => ({ display: 'block', visibility: 'visible', opacity: '1' }),
    ...overrides,
  };
  context.window = context;
  return context;
}

function responseCheck(document, options) {
  return vm.runInNewContext(`(${embeddedScript('response', options)})()`, scriptContext(document));
}

async function imageScan(document, options, overrides) {
  const context = scriptContext(document, overrides);
  vm.runInNewContext(`(${embeddedScript('scan', options)})()`, context);
  const deadline = Date.now() + 2000;
  while (context.__downloaded_images_status === 'pending' && Date.now() < deadline) {
    await new Promise(resolve => setTimeout(resolve, 5));
  }
  assert.equal(context.__downloaded_images_status, 'success');
  return JSON.parse(JSON.stringify(context.__downloaded_images));
}

const dataImage = index => image({ src: `data:image/png;base64,${Buffer.from(String(index)).toString('base64')}` });

test('text-only ChatGPT toolbars with mask-image CSS do not trigger image retries at zero budget', async () => {
  const latest = turn({ textContent: 'ABIMG-TEXT-OK' });
  const toolbar = new FakeElement({ className: '[mask-image:linear-gradient(to_right,black,transparent)]' });
  latest.querySelector = (selector) => selector.includes('[class*="image"]') ? toolbar : null;
  const result = await imageScan(new FakeDocument([latest]));

  assert.equal(result.images.length, 0);
  assert.equal(result.shouldRetry, false);
});

test('embedded response script waits for a new turn and never downloads old images', async () => {
  const document = new FakeDocument([turn({ markers: [marker(99)], images: [dataImage(1)] })]);
  const result = responseCheck(document, { baseline: 1 });

  assert.equal(result.status, 'waiting');
  assert.equal(result.isNew, false);
  assert.equal(result.imageProgressMarkerPresent, false);
  const scan = await imageScan(document, { baseline: 1 });
  assert.equal(scan.images.length, 0);
  assert.equal(scan.shouldRetry, false);
});

test('embedded scripts keep every unfinished percentage pending without fetching a preview', async () => {
  let fetches = 0;
  for (const value of [0, 27, 99]) {
    const document = new FakeDocument([turn({ markers: [marker(value)], images: [image()] })]);
    assert.equal(responseCheck(document).status, 'generating');
    const scan = await imageScan(document, {}, { fetch: () => { fetches += 1; throw new Error('must not fetch'); } });
    assert.equal(scan.images.length, 0);
    assert.equal(scan.shouldRetry, true);
  }
  assert.equal(fetches, 0);
});

test('embedded scripts wait at 100% until the image appears and finishes loading', async () => {
  const latest = turn({ markers: [marker(100)] });
  const document = new FakeDocument([latest]);
  assert.equal(responseCheck(document).status, 'generating');
  assert.equal((await imageScan(document)).shouldRetry, true);

  const pending = image({ src: dataImage(1).src, complete: false, naturalWidth: 0 });
  latest.images = [pending];
  assert.equal(responseCheck(document).status, 'generating');
  assert.equal((await imageScan(document)).shouldRetry, true);

  pending.complete = true;
  pending.naturalWidth = pending.naturalHeight = 1024;
  assert.equal(responseCheck(document).status, 'done');
  const scan = await imageScan(document);
  assert.equal(scan.images.length, 1);
  assert.equal(scan.shouldRetry, false);
});

test('embedded scripts reject a permanently broken image even at zero budget', async () => {
  const latest = turn({ markers: [marker(100)], images: [image({ complete: true, naturalWidth: 0 })] });
  const document = new FakeDocument([latest]);
  assert.equal(responseCheck(document).status, 'generating');
  const scan = await imageScan(document);
  assert.equal(scan.candidateCount, 1);
  assert.equal(scan.readyCount, 0);
  assert.equal(scan.images.length, 0);
  assert.equal(scan.shouldRetry, true);
});

test('embedded scripts do not finish a preview when the marker disappears before generation stops', async () => {
  const latest = turn({ markers: [marker(70)], images: [dataImage(1)] });
  const document = new FakeDocument([latest]);
  document.stopButton = {
    getAttribute: () => null,
    getBoundingClientRect: () => ({ width: 20, height: 20 }),
  };
  latest.markers = [];
  assert.equal(responseCheck(document).status, 'generating');
  assert.equal((await imageScan(document)).shouldRetry, true);
  document.stopButton = null;
  assert.equal(responseCheck(document).status, 'done');
  assert.equal((await imageScan(document)).images.length, 1);
});

test('embedded scan retries partial multi-image failures instead of claiming complete success', async () => {
  const second = image({ src: 'https://chatgpt.com/unavailable.png' });
  const document = new FakeDocument([turn({ images: [dataImage(1), second] })]);
  document.createElement = () => ({ getContext: () => null });
  const failed = await imageScan(document, {}, { fetch: async () => { throw new Error('offline'); } });
  assert.equal(failed.candidateCount, 2);
  assert.equal(failed.images.length, 0);
  assert.equal(failed.shouldRetry, true);

  second.src = dataImage(2).src;
  second.currentSrc = second.src;
  const recovered = await imageScan(document);
  assert.equal(recovered.images.length, 2);
  assert.equal(recovered.shouldRetry, false);
});

test('embedded scan uses latest fallback responses for Open/Get without a baseline', async () => {
  const latestSelector = '[data-testid="fallback-response"]';
  const old = turn({ markers: [marker(20)], images: [dataImage(1)] });
  const latest = turn({ images: [dataImage(2)] });
  const document = new FakeDocument([old]);
  document.querySelectorAll = selector => {
    if (selector === ASSISTANT_SELECTOR) return [old];
    if (selector === `${ASSISTANT_SELECTOR}, ${latestSelector}`) return [old, latest];
    return [];
  };
  const scan = await imageScan(document, { baseline: null, latestSelector });
  assert.equal(scan.images.length, 1);
  assert.equal(scan.images[0].src, latest.images[0].src);
  assert.equal(scan.shouldRetry, false);
});

for (const [provider, selector] of [['Gemini', 'model-response'], ['Claude', '.font-claude-response']]) {
  test(`embedded scripts preserve ${provider} text and image behavior`, async () => {
    const latest = turn({ textContent: 'TEXT-OK', markers: [marker(40)] });
    const document = new FakeDocument([latest], selector);
    const options = { chatgpt: false, selector };
    assert.equal(responseCheck(document, options).status, 'done');
    assert.equal(responseCheck(document, { ...options, baseline: 1 }).status, 'waiting');
    assert.equal((await imageScan(document, options)).shouldRetry, false);
    latest.images = [dataImage(1), dataImage(2)];
    const scan = await imageScan(document, options);
    assert.equal(scan.images.length, 2);
    assert.equal(scan.shouldRetry, false);
  });
}
