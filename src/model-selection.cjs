'use strict';

(function initializeModelSelection(root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) {
    module.exports = api;
  }
  root.AskBridgeModelSelection = api;
})(typeof globalThis === 'object' ? globalThis : this, function createModelSelection() {
  function normalizeLabel(value) {
    return String(value || '')
      .normalize('NFKC')
      .toLowerCase()
      .replace(/[^\p{Letter}\p{Number}]+/gu, '');
  }

  function parseMenuEntry(input) {
    const source = typeof input === 'string' ? { text: input } : (input || {});
    const lines = String(source.text || '')
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean);
    const selectedPrefix = /^(?:selected|已選取|已选中)\s*[:：]?\s*/i;
    const textMarksSelected = Boolean(lines[0] && selectedPrefix.test(lines[0]));
    if (textMarksSelected) {
      lines[0] = lines[0].replace(selectedPrefix, '').trim();
      if (!lines[0]) lines.shift();
    }
    const badgeText = String(source.badgeText || '').trim();
    const secondaryLines = lines.slice(1).filter((line) => line !== badgeText);
    const selectedValues = [
      source.ariaChecked,
      source.ariaSelected,
      source.dataSelected,
      source.dataState,
    ].map((value) => String(value || '').toLowerCase());

    return {
      primaryLabel: lines[0] || '',
      secondaryDescription: secondaryLines.join(' '),
      badgeText,
      selected: textMarksSelected
        || selectedValues.some((value) => ['true', 'checked', 'selected'].includes(value)),
      element: source.element,
    };
  }

  function labelsMatch(label, aliases) {
    const normalized = normalizeLabel(label);
    return Boolean(normalized) && aliases.some((alias) => normalizeLabel(alias) === normalized);
  }

  function findMatchingEntry(entries, aliases) {
    return entries.find((entry) => labelsMatch(entry.primaryLabel, aliases));
  }

  function availablePrimaryLabels(entries) {
    return [...new Set(entries.map((entry) => entry.primaryLabel).filter(Boolean))];
  }

  function selectionVerified(entry, pickerText, verificationAliases) {
    return Boolean(
      (entry && entry.selected)
      || labelsMatch(parseMenuEntry(pickerText).primaryLabel, verificationAliases),
    );
  }

  function textOf(element) {
    return element
      ? (element.innerText || element.textContent || element.getAttribute('aria-label') || '')
      : '';
  }

  function entryFromElement(element) {
    const badge = element.querySelector
      ? element.querySelector('[data-testid*="badge"], [class*="badge"]')
      : null;
    const selectedDescendant = element.querySelector
      ? element.querySelector(
        '[aria-checked="true"], [aria-selected="true"], [data-selected="true"], [data-state="checked"]',
      )
      : null;
    return parseMenuEntry({
      text: textOf(element),
      ariaChecked: element.getAttribute('aria-checked')
        || (selectedDescendant && selectedDescendant.getAttribute('aria-checked')),
      ariaSelected: element.getAttribute('aria-selected')
        || (selectedDescendant && selectedDescendant.getAttribute('aria-selected')),
      dataSelected: element.getAttribute('data-selected')
        || (selectedDescendant && selectedDescendant.getAttribute('data-selected')),
      dataState: element.getAttribute('data-state')
        || (selectedDescendant && selectedDescendant.getAttribute('data-state')),
      badgeText: textOf(badge),
      element,
    });
  }

  function visibleElements(selector) {
    return Array.from(document.querySelectorAll(selector)).filter((element) => {
      if (typeof element.getClientRects !== 'function') return true;
      return element.getClientRects().length > 0;
    });
  }

  function dispatchClick(element) {
    element.dispatchEvent(new MouseEvent('pointerdown', { bubbles: true }));
    element.dispatchEvent(new MouseEvent('pointerup', { bubbles: true }));
    element.click();
  }

  function findPicker(provider) {
    if (provider === 'chatgpt') {
      return document.querySelector('button.__composer-pill');
    }
    if (provider === 'gemini') {
      return visibleElements('button').find((button) => (
        /模式挑選器|model picker|mode picker/i.test([
          button.getAttribute('aria-label'),
          button.textContent,
        ].filter(Boolean).join(' '))
      ));
    }
    return undefined;
  }

  async function selectProviderOption(config) {
    const sleep = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));
    const menuSelector = '[role="menuitem"], [role="menuitemradio"], [role="option"]';
    const available = new Set();
    const visited = new Set();

    document.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'Escape',
      keyCode: 27,
      bubbles: true,
    }));
    await sleep(250);

    let picker;
    if (config.provider === 'chatgpt') {
      for (let attempt = 0; attempt < 20; attempt += 1) {
        picker = findPicker(config.provider);
        if (picker) break;
        await sleep(250);
      }
    } else if (config.provider === 'gemini') {
      picker = findPicker(config.provider);
    }

    if (!picker) {
      return { ok: false, error: `${config.provider} picker not found`, available: [] };
    }

    dispatchClick(picker);
    await sleep(800);

    let chosen;
    const maxDepth = config.provider === 'chatgpt' ? 6 : 1;
    for (let depth = 0; depth < maxDepth && !chosen; depth += 1) {
      const elements = visibleElements(menuSelector);
      const entries = elements.map(entryFromElement);
      entries.forEach((entry) => available.add(entry.primaryLabel));
      const leaves = entries.filter((entry) => entry.element.getAttribute('aria-haspopup') !== 'menu');
      chosen = findMatchingEntry(leaves, config.targetAliases);
      if (chosen || config.provider !== 'chatgpt') break;

      const triggers = entries.filter((entry) => entry.element.getAttribute('aria-haspopup') === 'menu');
      const trigger = triggers.find((entry) => {
        const key = `${normalizeLabel(entry.primaryLabel)}|${entry.element.getAttribute('aria-label') || ''}`;
        return !visited.has(key);
      });
      if (!trigger) break;

      const key = `${normalizeLabel(trigger.primaryLabel)}|${trigger.element.getAttribute('aria-label') || ''}`;
      visited.add(key);
      trigger.element.dispatchEvent(new MouseEvent('pointerenter', { bubbles: true }));
      trigger.element.dispatchEvent(new MouseEvent('pointermove', { bubbles: true }));
      trigger.element.dispatchEvent(new MouseEvent('mouseover', { bubbles: true }));
      trigger.element.click();
      await sleep(750);
    }

    if (!chosen) {
      document.dispatchEvent(new KeyboardEvent('keydown', {
        key: 'Escape',
        keyCode: 27,
        bubbles: true,
      }));
      return {
        ok: false,
        error: 'option not found',
        available: [...available].filter(Boolean),
      };
    }

    chosen.element.click();
    await sleep(600);

    picker = findPicker(config.provider) || picker;
    let currentEntry = entryFromElement(chosen.element);
    let verified = selectionVerified(
      currentEntry,
      textOf(picker),
      config.verificationAliases,
    );

    if (!verified) {
      picker = findPicker(config.provider) || picker;
      dispatchClick(picker);
      await sleep(500);
      const refreshedEntries = visibleElements(menuSelector).map(entryFromElement);
      currentEntry = findMatchingEntry(refreshedEntries, config.targetAliases);
      verified = selectionVerified(
        currentEntry,
        textOf(picker),
        config.verificationAliases,
      );
    }

    document.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'Escape',
      keyCode: 27,
      bubbles: true,
    }));

    if (!verified) {
      return {
        ok: false,
        error: `selection could not be verified for ${chosen.primaryLabel}`,
        available: [...available].filter(Boolean),
      };
    }

    return {
      ok: true,
      selected: chosen.primaryLabel,
      available: [...available].filter(Boolean),
    };
  }

  return {
    availablePrimaryLabels,
    findMatchingEntry,
    normalizeLabel,
    parseMenuEntry,
    selectProviderOption,
    selectionVerified,
  };
});
