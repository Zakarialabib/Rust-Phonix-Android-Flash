import { ParentComponent, splitProps } from 'solid-js';
import { cn } from '../../lib/utils';

interface BadgeProps {
  variant?: 'default' | 'success' | 'warning' | 'error' | 'outline' | 'secondary' | 'accent';
  size?: 'sm' | 'md';
  class?: string;
  children: any;
}

export const Badge: ParentComponent<BadgeProps> = (props) => {
  const [local, others] = splitProps(props, ['variant', 'size', 'class', 'children']);

  const variants = {
    default: "bg-sidebar/50 text-text-muted border-border-subtle",
    success: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
    warning: "bg-amber-500/10 text-amber-400 border-amber-500/20",
    error: "bg-red-500/10 text-red-400 border-red-500/20",
    outline: "bg-transparent border-border-subtle text-text-muted",
    secondary: "bg-sidebar text-text-secondary border-border-subtle",
    accent: "bg-accent/10 text-accent border-accent/20"
  };

  const sizes = {
    sm: "px-2 py-0.5 text-[8px]",
    md: "px-3 py-1 text-[9px]"
  };

  return (
    <span class={cn(
      "inline-flex items-center rounded-none font-mono font-black uppercase tracking-widest border transition-ui",
      variants[local.variant || 'default'],
      sizes[local.size || 'md'],
      local.class
    )}>
      {local.children}
    </span>
  );
};
