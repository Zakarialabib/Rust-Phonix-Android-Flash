import { ParentComponent, JSX, splitProps } from 'solid-js';
import { cn } from '../../lib/utils';

interface CardProps {
  title?: string;
  subtitle?: string;
  class?: string;
  actions?: JSX.Element;
  glow?: 'amber' | 'indigo' | 'teal' | 'rose' | 'slate' | 'accent';
  variant?: 'glass' | 'solid' | 'outline';
}

export const Card: ParentComponent<CardProps> = (props) => {
  const [local, others] = splitProps(props, ['title', 'subtitle', 'actions', 'glow', 'variant', 'class', 'children']);

  const glowStyles = {
    amber: "hover:border-amber-500/40 hover:shadow-[0_0_20px_rgba(245,158,11,0.1)]",
    indigo: "hover:border-indigo-500/40 hover:shadow-[0_0_20px_rgba(99,102,241,0.1)]",
    teal: "hover:border-teal-500/40 hover:shadow-[0_0_20px_rgba(20,184,166,0.1)]",
    rose: "hover:border-rose-500/40 hover:shadow-[0_0_20px_rgba(244,63,94,0.1)]",
    slate: "hover:border-slate-500/40 hover:shadow-[0_0_20px_rgba(100,116,139,0.1)]",
    accent: "hover:border-accent/40 shadow-glow hover:shadow-glow-strong",
  };

  const bgStyle = () => {
    if (local.variant === 'solid') return 'bg-sidebar';
    if (local.variant === 'outline') return 'bg-transparent';
    return 'bg-glass';
  };

  return (
    <div
      class={cn(
        "relative group flex flex-col border border-border-subtle transition-ui overflow-hidden",
        bgStyle(),
        local.glow && glowStyles[local.glow],
        local.class
      )}
    >
      {(local.title || local.subtitle || local.actions) && (
        <div class="flex items-start justify-between border-b border-border-subtle bg-white/[0.02] px-5 py-4">
          <div>
            {local.title && <h2 class="font-mono text-[10px] font-black uppercase tracking-[0.2em] text-text-primary italic">{local.title}</h2>}
            {local.subtitle && <p class="mt-1 font-mono text-[8px] text-text-muted uppercase tracking-[0.2em] italic opacity-60">{local.subtitle}</p>}
          </div>
          {local.actions && <div class="flex gap-2">{local.actions}</div>}
        </div>
      )}
      <div class="p-5 flex-1">
        {local.children}
      </div>
    </div>
  );
};
