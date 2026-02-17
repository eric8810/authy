import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

import en from './locales/en.json';
import zh from './locales/zh.json';
import zhTW from './locales/zh-TW.json';
import ja from './locales/ja.json';
import es from './locales/es.json';
import pt from './locales/pt.json';
import ko from './locales/ko.json';
import fr from './locales/fr.json';
import de from './locales/de.json';

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { translation: en },
      zh: { translation: zh },
      'zh-TW': { translation: zhTW },
      ja: { translation: ja },
      es: { translation: es },
      pt: { translation: pt },
      ko: { translation: ko },
      fr: { translation: fr },
      de: { translation: de },
    },
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['navigator', 'htmlTag', 'localStorage', 'cookie'],
      caches: ['localStorage', 'cookie'],
      lookupLocalStorage: 'i18nextLng',
      lookupCookie: 'i18next',
      cookieMinutes: 10080,
      checkWhitelist: false,
    },
    supportedLngs: ['en', 'zh', 'zh-TW', 'ja', 'ko', 'es', 'pt', 'fr', 'de'],
  });

const supportedLangs = ['en', 'zh', 'zh-TW', 'ja', 'ko', 'es', 'pt', 'fr', 'de'];
const detectedLang = i18n.language;

if (!supportedLangs.includes(detectedLang)) {
  if (detectedLang.startsWith('zh-TW') || detectedLang.startsWith('zh-Hant') || detectedLang.startsWith('zh-HK')) {
    i18n.changeLanguage('zh-TW');
  } else if (detectedLang.startsWith('zh')) {
    i18n.changeLanguage('zh');
  }
}

export default i18n;
