'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');

const {
  availablePrimaryLabels,
  findMatchingEntry,
  parseMenuEntry,
  selectionVerified,
} = require('../src/model-selection.cjs');

function fixture(text, attributes = {}) {
  return parseMenuEntry({
    text,
    ariaChecked: attributes.ariaChecked,
    dataState: attributes.dataState,
    badgeText: attributes.badgeText,
  });
}

test('matches ChatGPT reasoning by primary label and ignores secondary version text', () => {
  const entries = [
    fixture('即時\n5.5'),
    fixture('中'),
    fixture('高'),
  ];

  assert.equal(findMatchingEntry(entries, ['instant', '即時']).primaryLabel, '即時');
  assert.equal(findMatchingEntry(entries, ['medium', '中', '中等']).primaryLabel, '中');
  assert.equal(findMatchingEntry(entries, ['high', '高']).primaryLabel, '高');
  assert.equal(findMatchingEntry(entries, ['5.5']), undefined);
});

test('matches English ChatGPT reasoning labels', () => {
  const entries = [
    fixture('Instant\n5.5'),
    fixture('Medium'),
    fixture('High'),
  ];

  assert.equal(findMatchingEntry(entries, ['instant', '即時']).primaryLabel, 'Instant');
  assert.equal(findMatchingEntry(entries, ['medium', '中', '中等']).primaryLabel, 'Medium');
  assert.equal(findMatchingEntry(entries, ['high', '高']).primaryLabel, 'High');
});

test('matches Gemini modes exactly without subtitles or badges', () => {
  const entries = [
    fixture('3.5 Flash-Lite\n回覆最快\n新模型', { badgeText: '新模型' }),
    fixture('3.6 Flash\n全方位協助\n新模型', { badgeText: '新模型' }),
    fixture('3.1 Pro\n進階數學與程式設計'),
    fixture('延伸思考\n解決複雜問題'),
  ];

  assert.equal(findMatchingEntry(entries, ['3.5 Flash-Lite']).primaryLabel, '3.5 Flash-Lite');
  assert.equal(findMatchingEntry(entries, ['3.6 Flash']).primaryLabel, '3.6 Flash');
  assert.equal(findMatchingEntry(entries, ['3.1 Pro']).primaryLabel, '3.1 Pro');
  assert.equal(findMatchingEntry(entries, ['extended thinking', '延伸思考']).primaryLabel, '延伸思考');
  assert.equal(findMatchingEntry(entries, ['3.5 Flash']), undefined);
  assert.deepEqual(availablePrimaryLabels(entries), [
    '3.5 Flash-Lite',
    '3.6 Flash',
    '3.1 Pro',
    '延伸思考',
  ]);
});

test('separates selected-state text from the Gemini primary label', () => {
  const selected = fixture('已選取\n3.1 Pro\n進階數學與程式設計');
  const selectedEnglish = fixture('Selected: Extended Thinking\nSolve complex problems');

  assert.equal(selected.primaryLabel, '3.1 Pro');
  assert.equal(selected.selected, true);
  assert.equal(selectedEnglish.primaryLabel, 'Extended Thinking');
  assert.equal(selectedEnglish.selected, true);
});

test('matches English Gemini Extended Thinking label', () => {
  const entries = [
    fixture('3.1 Pro\nAdvanced math and coding'),
    fixture('Extended Thinking\nSolve complex problems'),
  ];

  assert.equal(
    findMatchingEntry(entries, ['extended thinking', '延伸思考']).primaryLabel,
    'Extended Thinking',
  );
});

test('verifies selection through checked state or the picker primary label', () => {
  const checked = fixture('高', { ariaChecked: 'true' });
  const unchecked = fixture('高', { ariaChecked: 'false' });

  assert.equal(selectionVerified(checked, '', ['high', '高']), true);
  assert.equal(selectionVerified(unchecked, '高', ['high', '高']), true);
  assert.equal(selectionVerified(unchecked, '中', ['high', '高']), false);
  assert.equal(selectionVerified(undefined, 'Pro 延伸', ['Pro Extended', 'Pro 延伸']), true);
});
