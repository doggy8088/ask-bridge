'use strict';

(function initializeImageProgress(root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) {
    module.exports = api;
  }
  root.AskBridgeImageProgress = api;
})(typeof globalThis === 'object' ? globalThis : this, function createImageProgress() {
  const DEFAULT_PROGRESS_SELECTOR = '[data-testid="image-gen-loading-progress"]';
  const DEFAULT_IMAGE_SELECTOR = 'img';
  const GENERATED_IMAGE_HINT_SELECTOR = [
    '[data-testid="image-gen-loading-state-frame"]',
    '[data-testid="image-gen-loading-state"]',
    '[data-testid="image-gen-loading-game-board"]',
    '[data-testid="image-gen-overlay-actions"]',
    '[class*="group/imagegen-image"]',
  ].join(', ');

  function asArray(value) {
    return Array.from(value || []);
  }

  function queryAll(root, selector) {
    if (!root || typeof root.querySelectorAll !== 'function' || !selector) return [];
    try {
      return asArray(root.querySelectorAll(selector));
    } catch (error) {
      return [];
    }
  }

  function attribute(element, name) {
    if (!element || typeof element.getAttribute !== 'function') return '';
    return element.getAttribute(name) || '';
  }

  function textOf(element) {
    if (!element) return '';
    return String(element.textContent || element.innerText || '');
  }

  function parseProgressValue(value, allowPercentText = false) {
    const source = String(value == null ? '' : value).trim();
    if (!source) return null;
    const pattern = allowPercentText
      ? /^(\d{1,3}(?:\.\d+)?)\s*%$/
      : /^(\d{1,3}(?:\.\d+)?)\s*%?$/;
    const match = source.match(pattern);
    if (!match) return null;
    const parsed = Number(match[1]);
    if (!Number.isFinite(parsed) || parsed < 0 || parsed > 100) return null;
    return parsed;
  }

  function readProgressMarker(marker) {
    if (!marker) {
      return { value: null, source: null, valid: false };
    }

    const ariaValue = parseProgressValue(attribute(marker, 'aria-valuenow'));
    const textValue = parseProgressValue(textOf(marker), true);
    const value = ariaValue == null ? textValue : ariaValue;
    const min = parseProgressValue(attribute(marker, 'aria-valuemin'));
    const max = parseProgressValue(attribute(marker, 'aria-valuemax'));
    const validRange = (min == null || min === 0) && (max == null || max === 100);

    return {
      value: validRange ? value : null,
      source: ariaValue == null ? (textValue == null ? null : 'text') : 'aria-valuenow',
      valid: validRange && value != null,
    };
  }

  function getDocument(explicitDocument) {
    if (explicitDocument) return explicitDocument;
    return typeof document === 'undefined' ? null : document;
  }

  function numericOption(value) {
    if (value == null || value === '') return null;
    const parsed = Number(value);
    return Number.isFinite(parsed) && parsed >= 0 ? Math.floor(parsed) : null;
  }

  function turnInfo(options = {}) {
    const activeDocument = getDocument(options.document);
    const assistantSelector = options.assistantSelector || '[data-message-author-role="assistant"], .agent-turn';
    const latestSelector = options.latestSelector || assistantSelector;
    const turns = queryAll(activeDocument, assistantSelector);
    const initialCount = numericOption(options.initialAssistantCount);
    const hasBaseline = initialCount != null;
    const isNew = hasBaseline ? turns.length > initialCount : turns.length > 0;

    if (hasBaseline && !isNew) {
      return { turn: null, count: turns.length, isNew: false };
    }

    if (hasBaseline) {
      if (turns.length > 0) {
        return { turn: normalizeTurn(turns[turns.length - 1]), count: turns.length, isNew };
      }
      return { turn: null, count: 0, isNew: false };
    }

    const responseSelector = assistantSelector === latestSelector
      ? assistantSelector
      : `${assistantSelector}, ${latestSelector}`;
    const latestTurns = queryAll(activeDocument, responseSelector);
    return {
      turn: latestTurns.length > 0 ? normalizeTurn(latestTurns[latestTurns.length - 1]) : null,
      count: latestTurns.length,
      isNew: latestTurns.length > 0,
    };
  }

  function normalizeTurn(element) {
    if (!element) return null;
    let normalized = element;
    let current = element;
    while (current) {
      if (typeof current.matches === 'function' && current.matches('.agent-turn')) {
        normalized = current;
      }
      current = current.parentElement || current.parentNode;
    }
    return normalized;
  }

  function findActiveTurn(options = {}) {
    return turnInfo(options).turn;
  }

  function progressNodes(turn, selector = DEFAULT_PROGRESS_SELECTOR) {
    return queryAll(turn, selector);
  }

  function readTurnProgress(turn, selector = DEFAULT_PROGRESS_SELECTOR) {
    const markers = progressNodes(turn, selector);
    if (markers.length === 0) {
      return {
        markerPresent: false,
        value: null,
        source: null,
        valid: false,
        state: 'none',
      };
    }

    // React can briefly leave an old marker next to a new one during a
    // transition. Treat the lowest valid value as the turn's progress so an
    // old low marker cannot be mistaken for completion. Never retain a
    // previous value after every marker disappears.
    const readings = [];
    for (const marker of markers) {
      const candidate = readProgressMarker(marker);
      if (candidate.valid) readings.push(candidate);
    }
    const reading = readings.reduce(
      (lowest, candidate) => (lowest.value == null || candidate.value < lowest.value ? candidate : lowest),
      { value: null, source: null, valid: false },
    );

    return {
      markerPresent: true,
      value: reading.value,
      source: reading.source,
      valid: reading.valid,
      state: reading.value == null ? 'pending' : (reading.value < 100 ? 'pending' : 'complete'),
    };
  }

  function imageSource(image) {
    if (!image) return '';
    const declaredSource = attribute(image, 'src');
    return String(image.currentSrc || (declaredSource ? image.src : '') || declaredSource || '');
  }

  function imageCandidate(image) {
    const source = imageSource(image);
    if (source.includes('avatar') || source.includes('profile')) return false;
    if (!source.startsWith('http')
      && !source.startsWith('blob:')
      && !source.startsWith('data:image/')) return false;
    const width = Number(image.naturalWidth || image.width || 0);
    const height = Number(image.naturalHeight || image.height || 0);
    if (width > 0 && width < 100) return false;
    if (height > 0 && height < 100) return false;
    return true;
  }

  function collectImageCandidates(turn, selector = DEFAULT_IMAGE_SELECTOR) {
    const seenSources = new Set();
    return queryAll(turn, selector).filter((image) => {
      if (!imageCandidate(image)) return false;
      const source = imageSource(image);
      if (seenSources.has(source)) return false;
      seenSources.add(source);
      return true;
    });
  }

  function isImageLoadReady(image) {
    return Boolean(image && image.complete === true && Number(image.naturalWidth || 0) > 0);
  }

  function inspectImageState(turn, options = {}) {
    const candidates = collectImageCandidates(turn, options.imageSelector || DEFAULT_IMAGE_SELECTOR);
    const readyCount = candidates.filter(isImageLoadReady).length;
    const hintCount = queryAll(
      turn,
      options.imageHintSelector || GENERATED_IMAGE_HINT_SELECTOR,
    ).length;
    return {
      candidateCount: candidates.length,
      readyCount,
      allReady: candidates.length > 0 && readyCount === candidates.length,
      hasImageHint: candidates.length > 0 || hintCount > 0,
    };
  }

  function progressBlocksCompletion(progress) {
    if (!progress || !progress.markerPresent) return false;
    if (progress.progress == null || progress.progress < 100) return true;
    return progress.progress === 100 && !progress.allReady;
  }

  function imageScanDecision({
    generationActive = false,
    progress = null,
    candidateCount = 0,
    imageCount = 0,
    allImagesReady = false,
    hasImageHint = false,
    chatgptProgress = true,
  } = {}) {
    const progressPending = chatgptProgress && progress && progress.markerPresent &&
      (progress.progress == null || progress.progress < 100);
    const progressAt100 = chatgptProgress && progress && progress.markerPresent
      && progress.progress === 100;
    const canReturnImages = !generationActive
      && !progressPending
      && (!progressAt100 || allImagesReady)
      && (chatgptProgress ? candidateCount > 0 && imageCount === candidateCount : imageCount > 0);
    return {
      canReturnImages,
      shouldRetry: Boolean(
        generationActive
        || progressPending
        || (hasImageHint && !canReturnImages),
      ),
    };
  }

  function inspectActiveImageProgress(options = {}) {
    const info = turnInfo(options);
    if (!info.turn) {
      return {
        hasTurn: false,
        isNew: false,
        markerPresent: false,
        progress: null,
        progressSource: null,
        progressState: 'none',
        candidateCount: 0,
        readyCount: 0,
        allReady: false,
        hasImageHint: false,
      };
    }

    const progress = readTurnProgress(info.turn, options.progressSelector || DEFAULT_PROGRESS_SELECTOR);
    const images = inspectImageState(info.turn, options);
    return {
      hasTurn: true,
      isNew: info.isNew,
      markerPresent: progress.markerPresent,
      progress: progress.value,
      progressSource: progress.source,
      progressState: progress.state,
      candidateCount: images.candidateCount,
      readyCount: images.readyCount,
      allReady: images.allReady,
      hasImageHint: images.hasImageHint,
    };
  }

  return {
    DEFAULT_PROGRESS_SELECTOR,
    DEFAULT_IMAGE_SELECTOR,
    GENERATED_IMAGE_HINT_SELECTOR,
    collectImageCandidates,
    findActiveTurn,
    imageScanDecision,
    imageSource,
    inspectActiveImageProgress,
    inspectImageState,
    isImageLoadReady,
    parseProgressValue,
    progressBlocksCompletion,
    readProgressMarker,
    readTurnProgress,
  };
});
