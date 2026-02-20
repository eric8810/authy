import React from 'react';
import { useTranslation } from 'react-i18next';
import { Shield, Key, FileJson, TerminalSquare, Lock, Monitor, FileCode, Code2 } from 'lucide-react';
import { Feature } from '../types';
import { FadeIn } from './FadeIn';

export const Features: React.FC = () => {
  const { t } = useTranslation();

  const features: Feature[] = [
    {
      title: t('features.libraryApi.title'),
      description: t('features.libraryApi.description'),
      icon: Code2
    },
    {
      title: t('features.encryptedVault.title'),
      description: t('features.encryptedVault.description'),
      icon: Lock
    },
    {
      title: t('features.scopedPolicies.title'),
      description: t('features.scopedPolicies.description'),
      icon: Shield
    },
    {
      title: t('features.sessionTokens.title'),
      description: t('features.sessionTokens.description'),
      icon: Key
    },
    {
      title: t('features.subprocessInjection.title'),
      description: t('features.subprocessInjection.description'),
      icon: TerminalSquare
    },
    {
      title: t('features.filePlaceholders.title'),
      description: t('features.filePlaceholders.description'),
      icon: FileCode
    },
    {
      title: t('features.auditLogging.title'),
      description: t('features.auditLogging.description'),
      icon: FileJson
    },
    {
      title: t('features.adminTUI.title'),
      description: t('features.adminTUI.description'),
      icon: Monitor
    },
  ];

  return (
    <section id="features" className="py-24 bg-surface/30 border-y border-surfaceHighlight">
      <div className="max-w-7xl mx-auto px-6">
        <FadeIn>
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-white mb-4">{t('features.title')}</h2>
            <p className="text-secondary max-w-2xl mx-auto">
              {t('features.description')}
            </p>
          </div>
        </FadeIn>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
          {features.map((feature, idx) => (
            <FadeIn key={idx} delay={idx * 100}>
              <div 
                className="h-full p-6 rounded-2xl bg-surface border border-surfaceHighlight hover:border-emerald-500/30 hover:bg-surfaceHighlight/30 hover:-translate-y-2 hover:shadow-xl hover:shadow-emerald-500/5 transition-all duration-300 group cursor-default"
              >
                <div className="w-12 h-12 rounded-lg bg-surfaceHighlight flex items-center justify-center mb-5 group-hover:bg-emerald-500 group-hover:text-black transition-colors duration-300">
                  <feature.icon size={24} />
                </div>
                <h3 className="text-xl font-semibold text-white mb-2">{feature.title}</h3>
                <p className="text-secondary leading-relaxed text-sm">{feature.description}</p>
              </div>
            </FadeIn>
          ))}
        </div>
      </div>
    </section>
  );
};
