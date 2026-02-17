import React from 'react';
import { useTranslation } from 'react-i18next';
import { ArrowRight, Terminal as TerminalIcon } from 'lucide-react';
import { TerminalWindow } from './TerminalWindow';
import { TerminalLine } from '../types';
import { FadeIn } from './FadeIn';

const heroDemoLines: TerminalLine[] = [
  { type: 'comment', content: 'Create a run-only policy — agent can never read secret values' },
  { type: 'command', content: 'authy policy create claude-code --allow "anthropic-*" --deny "prod-*" --run-only' },
  { type: 'output', content: "Policy 'claude-code' created." },
  { type: 'comment', content: 'Secrets injected into subprocess — agent never sees them' },
  { type: 'command', content: 'authy run --scope claude-code --uppercase --replace-dash _ -- claude' },
  { type: 'output', content: '[injected] ANTHROPIC_API_KEY, GITHUB_TOKEN (2 secrets)' },
  { type: 'output', content: '[claude] Session started.' },
];

export const Hero: React.FC = () => {
  const { t } = useTranslation();

  return (
    <section className="relative pt-32 pb-20 md:pt-48 md:pb-32 overflow-hidden">
      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-full h-[500px] bg-gradient-to-b from-white/[0.03] to-transparent pointer-events-none" />
      
      <div className="max-w-7xl mx-auto px-6 grid lg:grid-cols-2 gap-12 items-center relative z-10">
        <div className="flex flex-col gap-8">
          <FadeIn delay={0}>
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-surfaceHighlight border border-zinc-700 w-fit hover:border-emerald-500/50 transition-colors duration-300 cursor-default">
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
              </span>
              <span className="text-xs font-medium text-zinc-300">{t('hero.released')}</span>
            </div>
          </FadeIn>
          
          <FadeIn delay={100}>
            <h1 className="text-5xl md:text-6xl font-bold tracking-tight text-white leading-[1.1]">
              {t('hero.title')} <span className="text-transparent bg-clip-text bg-gradient-to-r from-white to-zinc-500">{t('hero.titleHighlight')}</span>
            </h1>
          </FadeIn>
          
          <FadeIn delay={200}>
            <p className="text-lg text-secondary max-w-xl leading-relaxed">
              {t('hero.description')}
            </p>
          </FadeIn>

          <FadeIn delay={300}>
            <div className="flex flex-col sm:flex-row gap-4">
              <a 
                href="#quick-start"
                className="inline-flex items-center justify-center gap-2 px-6 py-3 rounded-lg bg-white text-black font-semibold hover:bg-zinc-200 hover:scale-105 transition-all duration-300 shadow-lg shadow-white/5"
              >
                {t('hero.getStarted')}
                <ArrowRight size={18} />
              </a>
              <a 
                href="https://github.com/eric8810/authy/blob/main/AI_AGENT_GUIDE.md"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center justify-center gap-2 px-6 py-3 rounded-lg bg-surface border border-surfaceHighlight text-white font-medium hover:bg-surfaceHighlight hover:border-zinc-500 transition-all duration-300"
              >
                <TerminalIcon size={18} />
                {t('hero.aiAgentGuide')}
              </a>
            </div>
          </FadeIn>

          <FadeIn delay={400}>
            <div className="pt-4 flex items-center gap-6 text-sm text-secondary/60">
               <span>{t('hero.openSource')}</span>
               <span className="w-1 h-1 rounded-full bg-zinc-700" />
               <span>{t('hero.writtenInRust')}</span>
               <span className="w-1 h-1 rounded-full bg-zinc-700" />
               <span>{t('hero.zeroDependencies')}</span>
            </div>
          </FadeIn>
        </div>

        <FadeIn delay={200} direction="left" className="relative">
           <div className="absolute -inset-1 bg-gradient-to-r from-zinc-700 to-zinc-800 rounded-xl blur opacity-20 animate-pulse-slow" />
           <div className="animate-float">
             <TerminalWindow lines={heroDemoLines} className="relative transform md:rotate-1 md:translate-x-4 transition-transform hover:rotate-0 hover:translate-x-0 duration-500 hover:shadow-2xl hover:shadow-emerald-500/10" />
           </div>
        </FadeIn>
      </div>
    </section>
  );
};
