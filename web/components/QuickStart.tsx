import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { TerminalWindow } from './TerminalWindow';
import { TerminalLine } from '../types';
import { FadeIn } from './FadeIn';

const codeBlocks: Record<string, TerminalLine[]> = {
  install: [
    { type: 'comment', content: 'Install via npm (recommended)' },
    { type: 'command', content: 'npm install -g authy-cli' },
    { type: 'output', content: 'added 1 package in 1.2s' },
    { type: 'comment', content: 'Or use install script (Linux/macOS)' },
    { type: 'command', content: 'curl -fsSL https://raw.githubusercontent.com/eric8810/authy/main/install.sh | sh' },
    { type: 'output', content: 'Installed authy to /usr/local/bin/authy' },
  ],
  init: [
    { type: 'comment', content: 'Initialize a new vault with a keyfile' },
    { type: 'command', content: 'authy init --generate-keyfile ~/.authy/keys/master.key' },
    { type: 'output', content: 'Vault initialized at ~/.authy/vault.age' },
    { type: 'output', content: 'Master key saved to ~/.authy/keys/master.key' },
  ],
  store: [
    { type: 'comment', content: 'Recommended: Use TUI for secure masked input' },
    { type: 'command', content: 'authy admin --keyfile ~/.authy/keys/master.key' },
    { type: 'output', content: '# TUI opens → Select Secrets → Add (masked input)' },
    { type: 'comment', content: 'Or use CLI with interactive prompt' },
    { type: 'command', content: 'authy store openai-api-key' },
    { type: 'output', content: '# Type secret value, then Ctrl+D' },
    { type: 'output', content: "Secret 'openai-api-key' stored." },
    { type: 'comment', content: 'Or from file (for scripts)' },
    { type: 'command', content: 'authy store db-url < ~/.secrets/db-credential' },
  ],
  agent: [
    { type: 'comment', content: 'Create run-only policy — agent injects secrets but never reads them' },
    { type: 'command', content: 'authy policy create claude-code --allow "anthropic-*" --allow "github-*" --deny "prod-*" --run-only' },
    { type: 'comment', content: 'Launch Claude Code with scoped secrets injected' },
    { type: 'command', content: 'authy run --scope claude-code --uppercase --replace-dash _ -- claude' },
    { type: 'output', content: '[injected] ANTHROPIC_API_KEY, GITHUB_TOKEN (2 secrets)' },
  ]
};

export const QuickStart: React.FC = () => {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState('install');

  const tabs = [
    { id: 'install', label: t('quickStart.tabs.install') },
    { id: 'init', label: t('quickStart.tabs.initialize') },
    { id: 'store', label: t('quickStart.tabs.storeSecrets') },
    { id: 'agent', label: t('quickStart.tabs.agentAccess') },
  ];

  return (
    <section id="quick-start" className="py-24">
      <div className="max-w-7xl mx-auto px-6">
        <div className="grid lg:grid-cols-12 gap-12">
          
          <div className="lg:col-span-4 flex flex-col justify-center">
            <FadeIn direction="right">
              <h2 className="text-3xl font-bold text-white mb-6">{t('quickStart.title')}</h2>
              <p className="text-secondary mb-8">
                {t('quickStart.description')}
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
              <div className="relative h-full min-h-[400px]">
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
                      lines={codeBlocks[tab.id]} 
                      className="h-full shadow-2xl"
                      copyContent={codeBlocks[tab.id].map(l => l.content).join('\n')}
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
