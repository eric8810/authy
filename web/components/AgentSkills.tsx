import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { TerminalWindow } from './TerminalWindow';
import { TerminalLine } from '../types';
import { FadeIn } from './FadeIn';
import { BookOpen, Puzzle, ShieldCheck } from 'lucide-react';

const installBlocks: Record<string, TerminalLine[]> = {
  skills: [
    { type: 'comment', content: 'Install authy skill for your AI agent' },
    { type: 'command', content: 'npx skills add eric8810/authy' },
    { type: 'output', content: '✔ Found 1 skill' },
    { type: 'output', content: '  authy — Inject secrets into subprocesses via environment variables' },
    { type: 'output', content: '✔ Installed authy skill' },
    { type: 'comment', content: 'Your agent now knows how to use authy run, authy resolve, and authy list' },
  ],
  clawhub: [
    { type: 'comment', content: 'Install from ClawHub registry' },
    { type: 'command', content: 'npx clawhub install authy' },
    { type: 'output', content: '✔ Installed authy@0.4.0' },
    { type: 'comment', content: 'Or browse at clawhub.dev/skills/authy' },
  ],
  result: [
    { type: 'comment', content: 'After installing the skill, your agent learns:' },
    { type: 'comment', content: '' },
    { type: 'output', content: '1. authy list --scope <policy> --json          → discover secret names' },
    { type: 'output', content: '2. authy run --scope <policy> -- <cmd>         → inject secrets into commands' },
    { type: 'output', content: '3. authy resolve <file> --scope <policy>       → resolve config placeholders' },
    { type: 'comment', content: '' },
    { type: 'comment', content: 'The agent never sees secret values — only names.' },
    { type: 'comment', content: 'Run-only tokens block get, env, and export.' },
  ],
};

const highlights = [
  { icon: Puzzle, key: 'install' },
  { icon: ShieldCheck, key: 'runOnly' },
  { icon: BookOpen, key: 'threeCommands' },
];

export const AgentSkills: React.FC = () => {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState('skills');

  const tabs = [
    { id: 'skills', label: t('agentSkills.tabs.npxSkills') },
    { id: 'clawhub', label: t('agentSkills.tabs.clawhub') },
    { id: 'result', label: t('agentSkills.tabs.whatAgentLearns') },
  ];

  return (
    <section id="agent-skills" className="py-24 border-t border-surfaceHighlight">
      <div className="max-w-7xl mx-auto px-6">
        <div className="text-center mb-16">
          <FadeIn>
            <h2 className="text-3xl md:text-4xl font-bold text-white mb-4">
              {t('agentSkills.title')}
            </h2>
          </FadeIn>
          <FadeIn delay={100}>
            <p className="text-lg text-secondary max-w-3xl mx-auto">
              {t('agentSkills.description')}
            </p>
          </FadeIn>
        </div>

        <div className="grid md:grid-cols-3 gap-6 mb-16">
          {highlights.map((item, idx) => (
            <FadeIn key={item.key} delay={idx * 100}>
              <div className="p-6 rounded-xl border border-surfaceHighlight bg-surface/30 hover:border-emerald-500/30 hover:-translate-y-1 transition-all duration-300">
                <div className="w-10 h-10 rounded-lg bg-surfaceHighlight flex items-center justify-center mb-4">
                  <item.icon size={20} className="text-emerald-400" />
                </div>
                <h3 className="text-white font-semibold mb-2">
                  {t(`agentSkills.highlights.${item.key}.title`)}
                </h3>
                <p className="text-secondary text-sm">
                  {t(`agentSkills.highlights.${item.key}.description`)}
                </p>
              </div>
            </FadeIn>
          ))}
        </div>

        <div className="grid lg:grid-cols-12 gap-12">
          <div className="lg:col-span-4 flex flex-col justify-center">
            <FadeIn direction="right">
              <h3 className="text-2xl font-bold text-white mb-4">
                {t('agentSkills.installTitle')}
              </h3>
              <p className="text-secondary mb-8">
                {t('agentSkills.installDescription')}
              </p>

              <div className="flex flex-col gap-2">
                {tabs.map(tab => (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`text-left px-4 py-3 rounded-lg transition-all duration-300 border group ${
                      activeTab === tab.id
                        ? 'bg-surfaceHighlight border-zinc-600 text-white translate-x-2'
                        : 'border-transparent text-secondary hover:bg-surfaceHighlight/50 hover:text-white hover:translate-x-1'
                    }`}
                  >
                    <span className={`font-mono text-sm mr-3 transition-colors ${
                      activeTab === tab.id ? 'text-emerald-400' : 'text-zinc-500 group-hover:text-zinc-400'
                    }`}>0{tabs.indexOf(tab) + 1}</span>
                    <span className="font-medium">{tab.label}</span>
                  </button>
                ))}
              </div>
            </FadeIn>
          </div>

          <div className="lg:col-span-8">
            <FadeIn direction="left" delay={200} className="h-full">
              <div className="relative h-full min-h-[320px]">
                {tabs.map((tab) => (
                  <div
                    key={tab.id}
                    className={`absolute inset-0 transition-all duration-500 ease-in-out transform ${
                      activeTab === tab.id
                        ? 'opacity-100 translate-x-0 z-10'
                        : 'opacity-0 translate-x-8 z-0 pointer-events-none'
                    }`}
                  >
                    <TerminalWindow
                      lines={installBlocks[tab.id]}
                      className="h-full shadow-2xl"
                      copyContent={installBlocks[tab.id]
                        .filter(l => l.type === 'command')
                        .map(l => l.content)
                        .join('\n')}
                    />
                  </div>
                ))}
              </div>
            </FadeIn>
          </div>
        </div>
      </div>
    </section>
  );
};
