import React from 'react';
import { useTranslation } from 'react-i18next';
import { Navbar } from './components/Navbar';
import { Hero } from './components/Hero';
import { Features } from './components/Features';
import { QuickStart } from './components/QuickStart';
import { Comparison } from './components/Comparison';
import { Footer } from './components/Footer';
import { FadeIn } from './components/FadeIn';

const App: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="min-h-screen bg-background text-primary selection:bg-emerald-500/30 font-sans">
      <Navbar />
      <main>
        <Hero />
        <Features />
        <QuickStart />
        <Comparison />
        
        <section className="py-24 px-6 border-y border-surfaceHighlight bg-surfaceHighlight/5">
          <div className="max-w-4xl mx-auto text-center space-y-8">
            <FadeIn>
              <h2 className="text-3xl md:text-4xl font-bold text-white">
                {t('securityBanner.title')}
              </h2>
            </FadeIn>
            <FadeIn delay={100}>
              <p className="text-xl text-secondary">
                {t('securityBanner.description', {
                  code: <code className="bg-surfaceHighlight px-2 py-1 rounded text-white font-mono text-base">echo "secret" | tool</code>
                })}
              </p>
            </FadeIn>
            <FadeIn delay={200}>
              <div className="pt-4">
                <div className="inline-block p-4 rounded-xl bg-black border border-surfaceHighlight shadow-2xl font-mono text-left text-sm text-secondary hover:scale-105 transition-transform duration-500 hover:border-emerald-500/30">
                  <div className="flex gap-2 mb-2">
                     <div className="w-3 h-3 rounded-full bg-red-500"></div>
                     <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
                     <div className="w-3 h-3 rounded-full bg-green-500"></div>
                  </div>
                  <div className="text-white">? Select action: <span className="text-emerald-400">Store new secret</span></div>
                  <div>? Name: <span className="text-white">openai-key</span></div>
                  <div>? Value: <span className="text-secondary">********</span> <span className="text-zinc-600 italic">{t('securityBanner.masked')}</span></div>
                  <div className="mt-2 text-emerald-500">✔ {t('securityBanner.successMessage')}</div>
                </div>
              </div>
            </FadeIn>
          </div>
        </section>
      </main>
      <Footer />
    </div>
  );
};

export default App;
