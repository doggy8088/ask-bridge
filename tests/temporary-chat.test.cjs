'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');

const {
  activateTemporaryChat,
  detectActive,
  findActivationControl,
  inspectTemporaryChat,
  labelMatches,
} = require('../src/temporary-chat.cjs');

class FakeElement {
  constructor({
    tagName = 'DIV',
    attrs = {},
    textContent = '',
    id = '',
    pressed = false,
    onClick = null,
  } = {}) {
    this.tagName = tagName;
    this.attrs = { ...attrs };
    if (pressed) this.attrs['aria-pressed'] = 'true';
    this.textContent = textContent;
    this.innerText = textContent;
    this.id = id;
    this.clicked = false;
    this.onClick = onClick;
  }

  getAttribute(name) {
    return this.attrs[name] == null ? null : this.attrs[name];
  }

  click() {
    this.clicked = true;
    if (this.onClick) this.onClick(this);
  }
}

class FakeDocument {
  constructor(elements, { title = '', href = 'https://chatgpt.com/' } = {}) {
    this.elements = elements;
    this.title = title;
    this.location = { href };
  }

  querySelectorAll() {
    return this.elements;
  }
}

function button(label, options = {}) {
  return new FakeElement({ tagName: 'BUTTON', textContent: label, ...options });
}

test('labelMatches covers English and Traditional Chinese temporary labels', () => {
  assert.equal(labelMatches('Temporary chat', 'chatgpt'), true);
  assert.equal(labelMatches('開啟臨時聊天', 'gemini'), true);
  assert.equal(labelMatches('Incognito chat', 'claude'), true);
  assert.equal(labelMatches('無痕聊天', 'claude'), true);
  assert.equal(labelMatches('New chat', 'chatgpt'), false);
  // Claude uses incognito wording; the ChatGPT/Gemini wording must not match.
  assert.equal(labelMatches('Temporary chat', 'claude'), false);
  assert.equal(labelMatches('Incognito chat', 'chatgpt'), false);
});

test('findActivationControl finds the ChatGPT temporary toggle by label', () => {
  const toggle = button('Temporary chat');
  const doc = new FakeDocument([button('New chat'), toggle]);
  assert.equal(findActivationControl('chatgpt', doc), toggle);
});

test('findActivationControl finds the Claude incognito (ghost) button by aria-label', () => {
  const ghost = new FakeElement({ tagName: 'BUTTON', attrs: { 'aria-label': 'Incognito chat' } });
  const doc = new FakeDocument([button('New chat'), ghost]);
  assert.equal(findActivationControl('claude', doc), ghost);
});

test('mere presence of the toggle does not count as active (fail-closed)', () => {
  const doc = new FakeDocument([button('Temporary chat')]);
  assert.equal(detectActive('chatgpt', doc), false);
  const state = inspectTemporaryChat('chatgpt', doc);
  assert.equal(state.active, false);
  assert.equal(state.controlFound, true);
  assert.equal(state.controlPressed, false);
});

test('pressed toggle counts as active', () => {
  const doc = new FakeDocument([button('Temporary chat', { pressed: true })]);
  assert.equal(detectActive('chatgpt', doc), true);
});

test('visible non-interactive Temporary chat heading counts as active', () => {
  const heading = new FakeElement({ tagName: 'H2', textContent: 'Temporary chat' });
  const doc = new FakeDocument([button('New chat'), heading]);
  assert.equal(detectActive('gemini', doc), true);
});

test('Claude Incognito chat label counts as active', () => {
  const label = new FakeElement({ tagName: 'SPAN', textContent: 'Incognito chat' });
  const doc = new FakeDocument([label]);
  assert.equal(detectActive('claude', doc), true);
});

test('ChatGPT URL hint alone does not count as active', () => {
  const doc = new FakeDocument([], { href: 'https://chatgpt.com/?temporary-chat=true' });
  const state = inspectTemporaryChat('chatgpt', doc);
  assert.equal(state.urlHint, true);
  assert.equal(state.active, false);
});

test('activateTemporaryChat clicks the control and reports clicked', () => {
  const toggle = button('Temporary chat');
  const doc = new FakeDocument([toggle]);
  const result = activateTemporaryChat('chatgpt', doc);
  assert.equal(toggle.clicked, true);
  assert.equal(result.status, 'clicked');
});

test('activateTemporaryChat reports active when the click flips the pressed state', () => {
  const toggle = button('Temporary chat', {
    onClick: (el) => {
      el.attrs['aria-pressed'] = 'true';
    },
  });
  const doc = new FakeDocument([toggle]);
  const result = activateTemporaryChat('chatgpt', doc);
  assert.equal(result.status, 'active');
  assert.equal(result.active, true);
});

test('activateTemporaryChat reports unavailable when no control exists', () => {
  const doc = new FakeDocument([button('New chat')]);
  const result = activateTemporaryChat('gemini', doc);
  assert.equal(result.status, 'unavailable');
  assert.equal(result.controlFound, false);
});

test('long body text mentioning temporary chat does not count as active', () => {
  const paragraph = new FakeElement({
    tagName: 'P',
    textContent: 'You can start a Temporary chat from the header whenever you like, it is easy.',
  });
  const doc = new FakeDocument([paragraph]);
  assert.equal(detectActive('chatgpt', doc), false);
});
