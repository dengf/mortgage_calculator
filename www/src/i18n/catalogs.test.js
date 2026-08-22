import { describe, expect, it } from 'vitest';
import en from './en';
import zhHans from './zh-Hans';
import zhHant from './zh-Hant';
import { DEFAULT_LOCALE, LOCALES, matchLocale, translate } from './index.jsx';

const TRANSLATIONS = { 'zh-Hans': zhHans, 'zh-Hant': zhHant };

/** Placeholder names a template interpolates, e.g. `{count}`. */
function placeholders(template) {
  return new Set([...template.matchAll(/\{(\w+)\}/g)].map((m) => m[1]));
}

describe('catalogs', () => {
  it.each(Object.keys(TRANSLATIONS))('%s covers every English key', (locale) => {
    const missing = Object.keys(en).filter((k) => !(k in TRANSLATIONS[locale]));
    expect(missing).toEqual([]);
  });

  it.each(Object.keys(TRANSLATIONS))('%s has no keys English lacks', (locale) => {
    // A stale key is dead weight, and usually means a rename landed in one
    // catalog but not the source of truth.
    const extra = Object.keys(TRANSLATIONS[locale]).filter((k) => !(k in en));
    expect(extra).toEqual([]);
  });

  it.each(Object.keys(TRANSLATIONS))('%s interpolates the same values as English', (locale) => {
    // A dropped placeholder renders a sentence with a hole in it; an
    // invented one renders a literal "{foo}" to the user.
    const mismatched = Object.entries(en)
      .filter(([key, english]) => {
        const translated = TRANSLATIONS[locale][key];
        if (typeof translated !== 'string') return false;
        const a = placeholders(english);
        const b = placeholders(translated);
        return a.size !== b.size || [...a].some((p) => !b.has(p));
      })
      .map(([key]) => key);

    expect(mismatched).toEqual([]);
  });

  it('every locale offered in the switcher has a catalog', () => {
    for (const { id } of LOCALES) {
      expect(translate(id, 'app.title')).toBeTruthy();
    }
  });
});

describe('matchLocale', () => {
  it('reads the script subtag when one is present', () => {
    expect(matchLocale('zh-Hans-SG')).toBe('zh-Hans');
    expect(matchLocale('zh-Hant-TW')).toBe('zh-Hant');
  });

  it('treats Taiwan, Hong Kong and Macau as Traditional', () => {
    expect(matchLocale('zh-TW')).toBe('zh-Hant');
    expect(matchLocale('zh-HK')).toBe('zh-Hant');
    expect(matchLocale('zh-MO')).toBe('zh-Hant');
  });

  it('treats other Chinese regions as Simplified', () => {
    expect(matchLocale('zh-CN')).toBe('zh-Hans');
    expect(matchLocale('zh-SG')).toBe('zh-Hans');
    expect(matchLocale('zh')).toBe('zh-Hans');
  });

  it('ignores languages it has no catalog for', () => {
    expect(matchLocale('fr-FR')).toBeNull();
    expect(matchLocale(undefined)).toBeNull();
  });
});

describe('translate', () => {
  it('interpolates named values', () => {
    expect(translate('en', 'payment.totalOf', { count: 360 })).toBe('Total of 360 payments');
  });

  it('falls back to English rather than rendering blank', () => {
    // Simulates a key present in en but not yet in a translation.
    expect(translate('zh-Hans', '__missing__')).toBe('__missing__');
    expect(translate('en', 'app.title')).toBe(en['app.title']);
  });

  it('leaves an unmatched placeholder visible instead of blanking it', () => {
    // A silently-empty gap in a sentence is harder to spot in review than
    // a literal {count}.
    expect(translate('en', 'payment.totalOf', {})).toContain('{count}');
  });

  it('falls back to the default locale for an unknown one', () => {
    expect(translate('xx-YY', 'app.title')).toBe(en['app.title']);
    expect(DEFAULT_LOCALE).toBe('en');
  });
});

// UTF-8 bytes decoded as Latin-1 turn every CJK character into a run of
// Latin-1 supplement characters: 新加坡 becomes something like "\u00e6\u00b0\u00e5 \u00e5\u00a1".
// No catalog in this app legitimately contains any character in that range —
// verified across all three — so its presence is proof of a mis-encoded
// write and nothing else.
const MOJIBAKE = /[\u0080-\u00ff]/;

/** Han characters plus the CJK punctuation the catalogs use (，。？：、). */
const CJK = /[\u3000-\u303f\u4e00-\u9fff\uff00-\uffef]/;

/**
 * Prose keys that must be translated. Short labels are deliberately left in
 * English in places — regulatory scheme names with no authoritative Chinese
 * rendering — but a paragraph never is.
 */
const PROSE = /^(about|intro|meta)\./;

describe('catalog encoding', () => {
  it.each(Object.keys(TRANSLATIONS))('%s survived the file round-trip intact', (locale) => {
    // A single mis-encoded write turns 新加坡 into mojibake. It reads as
    // garbage to a user but passes every other test here, since the key set
    // and the placeholders are untouched.
    const corrupted = Object.entries(TRANSLATIONS[locale])
      .filter(([, value]) => MOJIBAKE.test(value))
      .map(([key]) => key);
    expect(corrupted).toEqual([]);
  });

  it('holds English to the same encoding rule', () => {
    const corrupted = Object.entries(en)
      .filter(([, value]) => MOJIBAKE.test(value))
      .map(([key]) => key);
    expect(corrupted).toEqual([]);
  });

  it.each(Object.keys(TRANSLATIONS))('%s actually contains Chinese', (locale) => {
    const chinese = Object.values(TRANSLATIONS[locale]).filter((v) => CJK.test(v));
    // Well over half of any real Chinese catalog. A wholesale corruption, or
    // an accidental copy of the English file, falls far below this.
    expect(chinese.length).toBeGreaterThan(Object.keys(en).length * 0.6);
  });

  it.each(Object.keys(TRANSLATIONS))('%s translates every prose string', (locale) => {
    const untranslated = Object.entries(TRANSLATIONS[locale])
      .filter(([key, value]) => PROSE.test(key) && !CJK.test(value))
      .map(([key]) => key);
    expect(untranslated).toEqual([]);
  });
});
