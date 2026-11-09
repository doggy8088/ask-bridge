'use strict';

(function initializeTemporaryChat(root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) {
    module.exports = api;
  }
  root.AskBridgeTemporaryChat = api;
})(typeof globalThis === 'object' ? globalThis : this, function createTemporaryChat() {
  // Labels are matched against each provider's own UI wording for its
  // temporary mode:
  // - ChatGPT: "Temporary Chat" (臨時聊天)
  // - Gemini: "Temporary chat" (臨時聊天)
  // - Claude: "Incognito chat" (無痕聊天)
  const TEMPORARY_LABEL_PATTERNS = [
    /temporary\s*chat/i,
    /臨時聊天/,
    /临时聊天/,
    /臨時對話/,
    /临时对话/,
  ];
  const INCOGNITO_LABEL_PATTERNS = [
    /incognito/i,
    /無痕/,
    /无痕/,
  ];

  const PROVIDER_KEYS = ['chatgpt', 'gemini', 'claude'];

  function labelPatterns(provider) {
    return provider === 'claude' ? INCOGNITO_LABEL_PATTERNS : TEMPORARY_LABEL_PATTERNS;
  }

  function markerToken(provider) {
    return provider === 'claude' ? 'incognito' : 'temporary';
  }

  function assertProvider(provider) {
    if (!PROVIDER_KEYS.includes(provider)) {
      throw new Error(`Unsupported temporary chat provider: ${provider}`);
    }
  }

  function normalizeText(value) {
    return String(value == null ? '' : value).replace(/\s+/g, ' ').trim();
  }

  function textOf(element) {
    if (!element) return '';
    return String(element.textContent || element.innerText || '');
  }

  function attribute(element, name) {
    if (!element || typeof element.getAttribute !== 'function') return '';
    return element.getAttribute(name) || '';
  }

  function elementLabel(element) {
    return normalizeText(
      attribute(element, 'aria-label') || attribute(element, 'title') || textOf(element),
    );
  }

  function labelMatches(label, provider) {
    const normalized = normalizeText(label);
    if (!normalized) return false;
    return labelPatterns(provider).some((pattern) => pattern.test(normalized));
  }

  function isVisible(element) {
    if (!element) return false;
    if (element.hidden) return false;
    const view = (element.ownerDocument && element.ownerDocument.defaultView)
      || (typeof globalThis === 'object' ? globalThis : null);
    if (view && typeof view.getComputedStyle === 'function') {
      try {
        const style = view.getComputedStyle(element);
        if (style && (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0')) {
          return false;
        }
      } catch (error) {
        // Ignore style probing failures and fall back to geometry checks.
      }
    }
    if (typeof element.getClientRects === 'function') {
      return element.getClientRects().length > 0;
    }
    if (typeof element.getBoundingClientRect === 'function') {
      const rect = element.getBoundingClientRect();
      return rect.width > 0 && rect.height > 0;
    }
    return true;
  }

  function isInteractive(element) {
    const tag = String(element.tagName || '').toUpperCase();
    if (tag === 'BUTTON' || tag === 'A' || tag === 'INPUT') return true;
    const role = attribute(element, 'role');
    return role === 'button' || role === 'menuitem' || role === 'option' || role === 'switch';
  }

  function isPressed(element) {
    if (attribute(element, 'aria-pressed') === 'true') return true;
    if (attribute(element, 'aria-checked') === 'true') return true;
    if (attribute(element, 'aria-selected') === 'true') return true;
    const state = attribute(element, 'data-state').toLowerCase();
    if (state === 'checked' || state === 'active' || state === 'on' || state === 'open') return true;
    return attribute(element, 'data-active') === 'true';
  }

  function collectElements(doc) {
    if (!doc || typeof doc.querySelectorAll !== 'function') return [];
    let elements = [];
    try {
      elements = Array.from(doc.querySelectorAll('*') || []);
    } catch (error) {
      elements = [];
    }
    if (elements.length === 0) {
      try {
        elements = Array.from(
          doc.querySelectorAll('button, a, [role="button"], h1, h2, h3, [data-testid], [aria-label]') || [],
        );
      } catch (error) {
        elements = [];
      }
    }
    return elements.slice(0, 8000);
  }

  function hasMarkerAttribute(element, provider) {
    const token = markerToken(provider);
    const testid = attribute(element, 'data-testid').toLowerCase();
    if (testid.includes(token)) return true;
    const id = String(element.id || '').toLowerCase();
    return id.includes(token);
  }

  function findActivationControl(provider, doc) {
    assertProvider(provider);
    const elements = collectElements(doc);
    for (const element of elements) {
      if (!isVisible(element) || !isInteractive(element)) continue;
      if (labelMatches(elementLabel(element), provider) || hasMarkerAttribute(element, provider)) {
        return element;
      }
    }
    return null;
  }

  function detectActive(provider, doc) {
    assertProvider(provider);
    const control = findActivationControl(provider, doc);
    if (control && isPressed(control)) return true;

    // A marker attribute on any visible element (e.g. a temporary-mode
    // banner) is a positive signal; note the plain activation control is
    // interactive, so it is excluded from the text scan below to avoid
    // treating the mere presence of the toggle as "already active".
    const elements = collectElements(doc);
    for (const element of elements) {
      if (!isVisible(element)) continue;
      if (hasMarkerAttribute(element, provider) && !isInteractive(element)) return true;
    }
    for (const element of elements) {
      if (!isVisible(element) || isInteractive(element)) continue;
      const text = normalizeText(textOf(element));
      if (text.length > 0 && text.length <= 60 && labelMatches(text, provider)) return true;
    }

    if (provider === 'chatgpt' && doc && labelMatches(doc.title || '', provider)) {
      return true;
    }
    return false;
  }

  function urlHint(provider, doc) {
    if (provider !== 'chatgpt' || !doc) return false;
    const url = String((doc.location && doc.location.href) || '');
    return /[?&]temporary-chat=true(?:&|$)/.test(url);
  }

  function inspectTemporaryChat(provider, doc) {
    assertProvider(provider);
    const target = doc || (typeof document === 'undefined' ? null : document);
    const control = findActivationControl(provider, target);
    return {
      provider,
      active: detectActive(provider, target),
      controlFound: Boolean(control),
      controlLabel: control ? elementLabel(control) : null,
      controlPressed: control ? isPressed(control) : false,
      urlHint: urlHint(provider, target),
    };
  }

  function activateTemporaryChat(provider, doc) {
    assertProvider(provider);
    const target = doc || (typeof document === 'undefined' ? null : document);
    const before = inspectTemporaryChat(provider, target);
    if (before.active) {
      return { ...before, status: 'active' };
    }
    const control = findActivationControl(provider, target);
    if (!control || typeof control.click !== 'function') {
      return { ...before, status: 'unavailable' };
    }
    control.click();
    const after = inspectTemporaryChat(provider, target);
    return { ...after, status: after.active ? 'active' : 'clicked' };
  }

  return {
    INCOGNITO_LABEL_PATTERNS,
    TEMPORARY_LABEL_PATTERNS,
    activateTemporaryChat,
    detectActive,
    findActivationControl,
    inspectTemporaryChat,
    labelMatches,
  };
});
