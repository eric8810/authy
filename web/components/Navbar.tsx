import React, { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Terminal, Menu, X, Github } from 'lucide-react';
import { LanguageSwitcher } from './LanguageSwitcher';

export const Navbar: React.FC = () => {
  const { t } = useTranslation();
  const [isScrolled, setIsScrolled] = useState(false);
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  
  const navItems = [
    { label: t('nav.features'), href: '#features' },
    { label: t('nav.howItWorks'), href: '#quick-start' },
    { label: t('nav.comparison'), href: '#comparison' },
    { label: t('nav.documentation'), href: '#' },
  ];

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 20);
    };
    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <nav
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 border-b ${
        isScrolled
          ? 'bg-background/80 backdrop-blur-md border-surfaceHighlight py-4 shadow-lg'
          : 'bg-transparent border-transparent py-6'
      }`}
    >
      <div className="max-w-7xl mx-auto px-6 flex items-center justify-between">
        <div className="flex items-center gap-2 group cursor-pointer" onClick={() => window.scrollTo({top: 0, behavior: 'smooth'})}>
          <div className="p-2 bg-surfaceHighlight rounded-lg group-hover:bg-zinc-700 transition-all duration-300 group-hover:scale-110 group-hover:rotate-3">
            <Terminal size={20} className="text-white group-hover:text-emerald-400 transition-colors" />
          </div>
          <span className="text-xl font-bold tracking-tight font-mono group-hover:text-white transition-colors">Authy</span>
        </div>

        <div className="hidden md:flex items-center gap-8">
          {navItems.map((item) => (
            <a
              key={item.label}
              href={item.href}
              className="text-sm font-medium text-secondary hover:text-white transition-colors relative group"
            >
              {item.label}
              <span className="absolute -bottom-1 left-0 w-0 h-0.5 bg-emerald-500 transition-all duration-300 group-hover:w-full"></span>
            </a>
          ))}
          <LanguageSwitcher />
          <a
            href="https://github.com/eric8810/authy"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 text-sm font-medium bg-white text-black px-4 py-2 rounded-full hover:bg-zinc-200 transition-all duration-300 hover:scale-105 active:scale-95"
          >
            <Github size={16} />
            <span>{t('nav.starOnGitHub')}</span>
          </a>
        </div>

        <button
          className="md:hidden text-secondary hover:text-white transition-colors"
          onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
        >
          {isMobileMenuOpen ? <X size={24} /> : <Menu size={24} />}
        </button>
      </div>

      <div className={`md:hidden absolute top-full left-0 right-0 bg-surface border-b border-surfaceHighlight transition-all duration-300 overflow-hidden ${isMobileMenuOpen ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0'}`}>
        <div className="p-6 flex flex-col gap-4">
          {navItems.map((item) => (
            <a
              key={item.label}
              href={item.href}
              className="text-base font-medium text-secondary hover:text-white hover:translate-x-2 transition-all"
              onClick={() => setIsMobileMenuOpen(false)}
            >
              {item.label}
            </a>
          ))}
          <div className="pt-2 border-t border-surfaceHighlight">
            <LanguageSwitcher />
          </div>
          <a
            href="https://github.com/eric8810/authy"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-center gap-2 text-sm font-medium bg-white text-black px-4 py-3 rounded-lg hover:bg-zinc-200 transition-colors"
            onClick={() => setIsMobileMenuOpen(false)}
          >
            <Github size={16} />
            <span>{t('nav.viewOnGitHub')}</span>
          </a>
        </div>
      </div>
    </nav>
  );
};
