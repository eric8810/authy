import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Globe, Check } from 'lucide-react';

const languages = [
  { code: 'en', name: 'English' },
  { code: 'zh', name: '简体中文' },
  { code: 'zh-TW', name: '繁體中文' },
  { code: 'ja', name: '日本語' },
  { code: 'ko', name: '한국어' },
  { code: 'es', name: 'Español' },
  { code: 'pt', name: 'Português' },
  { code: 'fr', name: 'Français' },
  { code: 'de', name: 'Deutsch' },
];

export const LanguageSwitcher: React.FC = () => {
  const { i18n } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);

  const getCurrentLang = () => {
    const lang = i18n.language;
    const matched = languages.find(l => l.code === lang);
    if (matched) return matched;
    
    const baseLang = lang.split('-')[0];
    
    if (lang.startsWith('zh-TW') || lang.startsWith('zh-Hant') || lang.startsWith('zh-HK')) {
      return languages.find(l => l.code === 'zh-TW') || languages[0];
    }
    
    if (baseLang === 'zh') {
      return languages.find(l => l.code === 'zh') || languages[0];
    }
    
    return languages.find(l => l.code === baseLang) || languages[0];
  };

  const currentLang = getCurrentLang();

  const changeLanguage = (code: string) => {
    i18n.changeLanguage(code);
    setIsOpen(false);
  };

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2 text-sm font-medium text-secondary hover:text-white transition-colors"
      >
        <Globe size={18} />
        <span>{currentLang.name}</span>
      </button>

      {isOpen && (
        <>
          <div 
            className="fixed inset-0 z-10" 
            onClick={() => setIsOpen(false)}
          />
          <div className="absolute right-0 top-full mt-2 py-2 w-40 bg-surface border border-surfaceHighlight rounded-lg shadow-xl z-20">
            {languages.map((lang) => (
              <button
                key={lang.code}
                onClick={() => changeLanguage(lang.code)}
                className="w-full px-4 py-2 text-left text-sm text-secondary hover:text-white hover:bg-surfaceHighlight flex items-center justify-between"
              >
                <span>{lang.name}</span>
                {currentLang.code === lang.code && (
                  <Check size={14} className="text-emerald-400" />
                )}
              </button>
            ))}
          </div>
        </>
      )}
    </div>
  );
};
