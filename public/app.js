/* ask-bridge — interactions: theme, nav, copy, tabs, terminal, reveals */
(function () {
  "use strict";

  /* ---- theme ---- */
  const THEME_STORE = "ab-theme";
  const root = document.documentElement;

  function setTheme(t) {
    root.setAttribute("data-theme", t);
    const btn = document.getElementById("themeBtn");
    if (btn) {
      btn.setAttribute("aria-pressed", t === "light" ? "true" : "false");
      if (window.i18n && window.i18n.t) {
        const label = window.i18n.t(t === "light" ? "nav.toDark" : "nav.toLight");
        btn.setAttribute("aria-label", label);
        btn.setAttribute("title", label);
      }
    }
    try { localStorage.setItem(THEME_STORE, t); } catch (e) {}
  }
  function fromUrl() {
    try {
      const p = new URLSearchParams(window.location.search);
      const t = p.get("theme");
      if (t === "dark" || t === "light") return t;
    } catch (e) {}
    return null;
  }
  (function initTheme() {
    let t = fromUrl();
    if (t !== "dark" && t !== "light") {
      try { t = localStorage.getItem(THEME_STORE); } catch (e) {}
    }
    if (t !== "dark" && t !== "light") {
      t = window.matchMedia && window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
    }
    setTheme(t);
    const btn = document.getElementById("themeBtn");
    if (btn) btn.addEventListener("click", () => setTheme(root.getAttribute("data-theme") === "dark" ? "light" : "dark"));
  })();

  /* ---- i18n ---- */
  if (window.i18n) window.i18n.init();

  /* ---- nav ---- */
  (function nav() {
    const nav = document.getElementById("nav");
    const burger = document.getElementById("burger");
    const links = document.querySelector(".nav__links");
    const onScroll = () => nav.classList.toggle("scrolled", window.scrollY > 8);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    if (burger && links) {
      burger.addEventListener("click", () => {
        const open = links.classList.toggle("open");
        burger.setAttribute("aria-expanded", open ? "true" : "false");
      });
      links.addEventListener("click", (e) => {
        if (e.target.tagName === "A") {
          links.classList.remove("open");
          burger.setAttribute("aria-expanded", "false");
        }
      });
    }
  })();

  /* ---- copy ---- */
  (function copy() {
    document.addEventListener("click", (e) => {
      const btn = e.target.closest("[data-copy-target]");
      if (!btn) return;
      const target = document.getElementById(btn.getAttribute("data-copy-target"));
      if (!target) return;
      const text = target.innerText;
      const done = () => {
        btn.classList.add("copied");
        const prev = btn.getAttribute("aria-label");
        if (window.i18n && window.i18n.t) btn.setAttribute("aria-label", window.i18n.t("nav.copied"));
        setTimeout(() => {
          btn.classList.remove("copied");
          if (prev) btn.setAttribute("aria-label", prev);
        }, 1400);
      };
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done, () => fallback(text, done));
      } else fallback(text, done);
    });
    function fallback(text, cb) {
      const ta = document.createElement("textarea");
      ta.value = text; ta.style.position = "fixed"; ta.style.opacity = "0";
      document.body.appendChild(ta); ta.select();
      try { document.execCommand("copy"); cb(); } catch (e) {}
      document.body.removeChild(ta);
    }
  })();

  /* ---- install tabs (ARIA tablist + roving tabindex + arrow keys) ---- */
  (function tabs() {
    const wrap = document.getElementById("installTabs");
    if (!wrap) return;
    const bar = wrap.querySelector(".tabs__bar");
    const tabs = Array.from(wrap.querySelectorAll(".tab"));
    const panels = Array.from(wrap.querySelectorAll(".tabs__panel"));
    function activate(tab) {
      const id = tab.getAttribute("data-tab");
      tabs.forEach((t) => {
        const on = t === tab;
        t.classList.toggle("tab--active", on);
        t.setAttribute("aria-selected", on ? "true" : "false");
        t.setAttribute("tabindex", on ? "0" : "-1");
      });
      panels.forEach((p) => {
        const on = p.getAttribute("data-panel") === id;
        p.classList.toggle("tab--active", on);
        if (on) p.removeAttribute("hidden"); else p.setAttribute("hidden", "");
      });
    }
    tabs.forEach((tab) => tab.addEventListener("click", () => { activate(tab); tab.focus(); }));
    if (bar) {
      bar.addEventListener("keydown", (e) => {
        const i = tabs.indexOf(document.activeElement);
        if (i === -1) return;
        let n = i;
        if (e.key === "ArrowRight") n = (i + 1) % tabs.length;
        else if (e.key === "ArrowLeft") n = (i - 1 + tabs.length) % tabs.length;
        else if (e.key === "Home") n = 0;
        else if (e.key === "End") n = tabs.length - 1;
        else return;
        e.preventDefault();
        activate(tabs[n]);
        tabs[n].focus();
      });
    }
  })();

  /* ---- hero terminal typing ---- */
  (function terminal() {
    const cmdEl = document.getElementById("termCmd");
    const respEl = document.getElementById("termResp");
    const caret = document.getElementById("termCaret");
    if (!cmdEl || !respEl) return;
    const CMD = 'ask-bridge "Summarize Rust ownership in 3 lines"';
    const reduced = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced) {
      cmdEl.textContent = CMD;
      if (caret) caret.classList.add("done");
      respEl.classList.add("show");
      return;
    }
    let i = 0;
    const speed = 34;
    function tick() {
      cmdEl.textContent = CMD.slice(0, i);
      i++;
      if (i <= CMD.length) setTimeout(tick, speed);
      else {
        setTimeout(() => {
          if (caret) caret.classList.add("done");
          respEl.classList.add("show");
        }, 360);
      }
    }
    // start when hero terminal is visible (or immediately on small screens)
    const start = () => setTimeout(tick, 500);
    if ("IntersectionObserver" in window) {
      const io = new IntersectionObserver((entries) => {
        if (entries[0].isIntersecting) { io.disconnect(); start(); }
      }, { threshold: 0.4 });
      io.observe(document.getElementById("heroTerminal"));
    } else start();
  })();

  /* ---- hero entrance (page-load choreography; only when motion allowed) ---- */
  (function heroEnter() {
    if (!document.documentElement.classList.contains("hero-anim")) return;
    const hero = document.querySelector(".hero");
    if (!hero) return;
    const enter = () => hero.classList.add("hero-enter");
    const trigger = () => requestAnimationFrame(() => requestAnimationFrame(enter));
    if (document.readyState !== "loading") trigger();
    else document.addEventListener("DOMContentLoaded", trigger, { once: true });
    setTimeout(enter, 1400); // safety: never ship a blank hero
  })();

  /* ---- reveal + stagger on scroll (progressive; safety fallback) ---- */
  (function reveals() {
    const singles = document.querySelectorAll(".tabs");
    const staggers = document.querySelectorAll(".why__points, .features__ledger, .usage__grid, .how__timeline");
    singles.forEach((t) => t.classList.add("reveal"));
    staggers.forEach((c) => {
      c.classList.add("stagger");
      Array.from(c.children).forEach((child, i) => child.style.setProperty("--i", i));
    });
    const all = [...singles, ...staggers];
    if (!("IntersectionObserver" in window)) { all.forEach((t) => t.classList.add("in")); return; }
    const io = new IntersectionObserver((entries) => {
      entries.forEach((en) => {
        if (en.isIntersecting) { en.target.classList.add("in"); io.unobserve(en.target); }
      });
    }, { threshold: 0.12, rootMargin: "0px 0px -8% 0px" });
    all.forEach((t) => io.observe(t));
    // safety: if a target never intersects (hidden tab, etc.), show it after 2.5s
    setTimeout(() => all.forEach((t) => t.classList.add("in")), 2500);
  })();
})();