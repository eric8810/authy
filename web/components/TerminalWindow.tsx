import React from 'react';
import { Copy, Check } from 'lucide-react';
import { TerminalLine } from '../types';

interface TerminalWindowProps {
  title?: string;
  lines: TerminalLine[];
  className?: string;
  copyContent?: string;
}

export const TerminalWindow: React.FC<TerminalWindowProps> = ({ 
  title = "bash", 
  lines, 
  className = "",
  copyContent
}) => {
  const [copied, setCopied] = React.useState(false);

  const handleCopy = () => {
    if (copyContent) {
      navigator.clipboard.writeText(copyContent);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className={`rounded-xl overflow-hidden border border-surfaceHighlight bg-surface/50 backdrop-blur shadow-2xl hover:border-zinc-600 transition-all duration-300 ${className}`}>
      <div className="flex items-center justify-between px-4 py-3 bg-surface border-b border-surfaceHighlight">
        <div className="flex items-center gap-2">
          <div className="flex gap-1.5 group">
            <div className="w-3 h-3 rounded-full bg-red-500/20 border border-red-500/50 group-hover:bg-red-500 transition-colors duration-300" />
            <div className="w-3 h-3 rounded-full bg-yellow-500/20 border border-yellow-500/50 group-hover:bg-yellow-500 transition-colors duration-300" />
            <div className="w-3 h-3 rounded-full bg-green-500/20 border border-green-500/50 group-hover:bg-green-500 transition-colors duration-300" />
          </div>
          <span className="ml-2 text-xs font-mono text-secondary">{title}</span>
        </div>
        {copyContent && (
          <button 
            onClick={handleCopy}
            className="text-secondary hover:text-white transition-colors p-1 hover:bg-white/5 rounded"
            title="Copy to clipboard"
          >
            {copied ? <Check size={14} className="text-emerald-400" /> : <Copy size={14} />}
          </button>
        )}
      </div>
      <div className="p-4 md:p-6 font-mono text-sm md:text-sm overflow-x-auto custom-scrollbar h-full">
        {lines.map((line, idx) => (
          <div key={idx} className="mb-1 last:mb-0 whitespace-pre">
            {line.type === 'command' && (
              <div className="flex">
                <span className="text-secondary select-none mr-3">{line.prompt || '$'}</span>
                <span className="text-white">{line.content}</span>
              </div>
            )}
            {line.type === 'output' && (
              <div className="text-secondary/70">
                {line.content}
              </div>
            )}
            {line.type === 'comment' && (
              <div className="text-secondary/50 italic">
                # {line.content}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};