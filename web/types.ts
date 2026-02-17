import { LucideIcon } from 'lucide-react';

export interface NavItem {
  label: string;
  href: string;
}

export interface Feature {
  title: string;
  description: string;
  icon: LucideIcon;
}

export interface TerminalLine {
  type: 'command' | 'output' | 'comment';
  content: string;
  prompt?: string;
}

export interface ComparisonRow {
  feature: string;
  authy: boolean;
  pass: boolean;
  vault: boolean;
  onepassword: boolean;
}