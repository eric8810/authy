import React from 'react';
import { useTranslation } from 'react-i18next';
import { Terminal } from 'lucide-react';

export const Footer: React.FC = () => {
  const { t } = useTranslation();

  return (
    <footer className="py-12 border-t border-surfaceHighlight bg-background">
      <div className="max-w-7xl mx-auto px-6">
        <div className="grid md:grid-cols-4 gap-8 mb-12">
          <div className="col-span-2">
            <div className="flex items-center gap-2 mb-4">
              <Terminal size={24} className="text-white" />
              <span className="text-xl font-bold font-mono text-white">Authy</span>
            </div>
            <p className="text-secondary max-w-sm">
              {t('footer.description')}
            </p>
          </div>
          
          <div>
            <h4 className="text-white font-semibold mb-4">{t('footer.guides')}</h4>
            <ul className="space-y-2 text-secondary text-sm">
              <li><a href="https://github.com/eric8810/authy/blob/main/AI_AGENT_GUIDE.md" className="hover:text-white transition-colors">{t('footer.aiAgentGuide')}</a></li>
              <li><a href="#" className="hover:text-white transition-colors">{t('footer.documentation')}</a></li>
              <li><a href="#" className="hover:text-white transition-colors">{t('footer.releases')}</a></li>
              <li><a href="#" className="hover:text-white transition-colors">{t('footer.license')}</a></li>
            </ul>
          </div>
          
          <div>
            <h4 className="text-white font-semibold mb-4">{t('footer.community')}</h4>
            <ul className="space-y-2 text-secondary text-sm">
              <li><a href="https://github.com/eric8810/authy/issues" target="_blank" rel="noopener noreferrer" className="hover:text-white transition-colors">{t('footer.issues')}</a></li>
              <li><a href="https://github.com/eric8810/authy/discussions" target="_blank" rel="noopener noreferrer" className="hover:text-white transition-colors">{t('footer.discussions')}</a></li>
            </ul>
          </div>
        </div>
        
        <div className="flex flex-col md:flex-row justify-between items-center pt-8 border-t border-surfaceHighlight text-sm text-secondary/60">
          <div>
            {t('footer.copyright', { year: new Date().getFullYear() })}
          </div>
          <div className="flex items-center gap-6 mt-4 md:mt-0">
             <span className="flex items-center gap-2">{t('footer.builtWithRust')} <span className="w-2 h-2 rounded-full bg-orange-600"></span></span>
             <span className="flex items-center gap-2">{t('footer.securedWithAge')} <span className="w-2 h-2 rounded-full bg-blue-600"></span></span>
          </div>
        </div>
      </div>
    </footer>
  );
};
