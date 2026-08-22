import React, { createContext, useCallback, useContext, useMemo, useState } from 'react';
import en from './en';
import zhHans from './zh-Hans';
import zhHant from './zh-Hant';

// Hand-rolled rather than react-i18next: this is one app with three flat
// catalogs and no plurals to speak of, and the library costs more gzipped
// than the charting library we already declined for the same reason.

export const LOCALES = [
  { id: 'en', label: 'EN', name: 'English' },
  { id: 'zh-Hans', label: '简', name: '简体中文' },
  { id: 'zh-Hant', label: '繁', name: '繁體中文' },
];

const CATALOGS = { en, 'zh-Hans': zhHans, 'zh-Hant': zhHant };

export const DEFAULT_LOCALE = 'en';
const STORAGE_KEY = 'mc:locale';

/**
 * Maps a BCP-47 tag to one of our catalogs.
 *
 * Script subtags win where present (`zh-Hans-SG`). Otherwise fall back on
 * region, since that is how the tag usually arrives in practice: Taiwan,
 * Hong Kong and Macau are Traditional, and everywhere else Chinese is
 * spoken is Simplified.
 */
export function matchLocale(tag) {
  if (typeof tag !== 'string') return null;
  const lower = tag.toLowerCase();
  if (!lower.startsWith('zh')) return lower.startsWith('en') ? 'en' : null;
  if (lower.includes('hant')) return 'zh-Hant';
  if (lower.includes('hans')) return 'zh-Hans';
  return /-(tw|hk|mo)\b/.test(lower) ? 'zh-Hant' : 'zh-Hans';
}

export function detectLocale() {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored && CATALOGS[stored]) return stored;
  } catch {
    // Storage can be unavailable in private mode; fall through to the
    // browser's own preference.
  }
  const tags = navigator.languages?.length ? navigator.languages : [navigator.language];
  for (const tag of tags) {
    const match = matchLocale(tag);
    if (match) return match;
  }
  return DEFAULT_LOCALE;
}

/** Replaces `{name}` placeholders. Missing values are left visible rather
 *  than silently blanked, so a broken message is obvious in review. */
function interpolate(template, params) {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (whole, key) =>
    Object.prototype.hasOwnProperty.call(params, key) ? String(params[key]) : whole,
  );
}

/**
 * Looks a key up, falling back to English and then to the key itself.
 *
 * Falling back to English rather than rendering blank means a missing
 * translation degrades to a readable app, which matters while catalogs are
 * being filled in.
 */
export function translate(locale, key, params) {
  const catalog = CATALOGS[locale] ?? CATALOGS[DEFAULT_LOCALE];
  const template = catalog[key] ?? CATALOGS[DEFAULT_LOCALE][key] ?? key;
  return interpolate(template, params);
}

const I18nContext = createContext({
  locale: DEFAULT_LOCALE,
  setLocale: () => {},
  t: (key, params) => translate(DEFAULT_LOCALE, key, params),
});

export function I18nProvider({ initialLocale, children }) {
  const [locale, setLocaleState] = useState(initialLocale ?? DEFAULT_LOCALE);

  const setLocale = useCallback((next) => {
    setLocaleState(next);
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // Preference just won't persist; the session still switches.
    }
    // Assistive tech and browser features (translation prompts, hyphenation)
    // key off this, so it has to track the choice.
    document.documentElement.lang = next;
  }, []);

  const value = useMemo(
    () => ({ locale, setLocale, t: (key, params) => translate(locale, key, params) }),
    [locale, setLocale],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  return useContext(I18nContext);
}
