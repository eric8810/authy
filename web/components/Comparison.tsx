import React from 'react';
import { useTranslation } from 'react-i18next';
import { Check, Minus } from 'lucide-react';
import { ComparisonRow } from '../types';
import { FadeIn } from './FadeIn';

export const Comparison: React.FC = () => {
  const { t } = useTranslation();

  const data: ComparisonRow[] = [
    { feature: t('comparison.features.singleBinary'), authy: true, pass: true, vault: false, onepassword: true },
    { feature: t('comparison.features.noServerRequired'), authy: true, pass: true, vault: false, onepassword: true },
    { feature: t('comparison.features.scopedAccess'), authy: true, pass: false, vault: true, onepassword: true },
    { feature: t('comparison.features.sessionTokens'), authy: true, pass: false, vault: true, onepassword: true },
    { feature: t('comparison.features.subprocessInjection'), authy: true, pass: false, vault: false, onepassword: true },
    { feature: t('comparison.features.interactiveTUI'), authy: true, pass: false, vault: false, onepassword: true },
    { feature: t('comparison.features.openSource'), authy: true, pass: true, vault: true, onepassword: false },
    { feature: t('comparison.features.noAccountNeeded'), authy: true, pass: true, vault: true, onepassword: false },
    { feature: t('comparison.features.auditLogging'), authy: true, pass: false, vault: true, onepassword: true },
    { feature: t('comparison.features.runOnlyMode'), authy: true, pass: false, vault: false, onepassword: false },
    { feature: t('comparison.features.jsonOutput'), authy: true, pass: false, vault: true, onepassword: true },
    { feature: t('comparison.features.libraryApi'), authy: true, pass: false, vault: true, onepassword: true },
    { feature: t('comparison.features.mcpServer'), authy: true, pass: false, vault: false, onepassword: false },
    { feature: t('comparison.features.builtForAgents'), authy: true, pass: false, vault: false, onepassword: false },
  ];

  return (
    <section id="comparison" className="py-24 bg-surface/30 border-y border-surfaceHighlight">
      <div className="max-w-7xl mx-auto px-6">
        <FadeIn>
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-white mb-4">{t('comparison.title')}</h2>
            <p className="text-secondary">{t('comparison.description')}</p>
          </div>
        </FadeIn>

        <FadeIn delay={200}>
          <div className="overflow-x-auto rounded-xl border border-surfaceHighlight/50">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="border-b border-surfaceHighlight bg-surface/50">
                  <th className="py-5 px-6 text-sm font-semibold text-secondary uppercase tracking-wider">{t('comparison.feature')}</th>
                  <th className="py-5 px-6 text-sm font-bold text-white uppercase tracking-wider bg-surfaceHighlight/20 rounded-t-lg">Authy</th>
                  <th className="py-5 px-6 text-sm font-semibold text-secondary uppercase tracking-wider">pass</th>
                  <th className="py-5 px-6 text-sm font-semibold text-secondary uppercase tracking-wider">Vault</th>
                  <th className="py-5 px-6 text-sm font-semibold text-secondary uppercase tracking-wider">1Password</th>
                </tr>
              </thead>
              <tbody>
                {data.map((row, idx) => (
                  <tr key={idx} className="border-b border-surfaceHighlight last:border-0 hover:bg-surfaceHighlight/20 transition-colors duration-200 group">
                    <td className="py-4 px-6 text-sm font-medium text-white group-hover:text-emerald-300 transition-colors">{row.feature}</td>
                    
                    <td className="py-4 px-6 bg-surfaceHighlight/10 group-hover:bg-surfaceHighlight/30 transition-colors">
                      <div className="flex items-center text-emerald-400">
                        <Check size={18} className="mr-2" />
                      </div>
                    </td>
                    
                    <td className="py-4 px-6">
                      {row.pass ? <Check size={18} className="text-zinc-500 group-hover:text-zinc-400" /> : <Minus size={18} className="text-zinc-700" />}
                    </td>
                    
                    <td className="py-4 px-6">
                      {row.vault ? <Check size={18} className="text-zinc-500 group-hover:text-zinc-400" /> : <Minus size={18} className="text-zinc-700" />}
                    </td>
                    
                    <td className="py-4 px-6">
                      {row.onepassword ? <Check size={18} className="text-zinc-500 group-hover:text-zinc-400" /> : <Minus size={18} className="text-zinc-700" />}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </FadeIn>
      </div>
    </section>
  );
};
