import { ParentComponent, JSX, splitProps } from 'solid-js';
import { cn } from '../../lib/utils';

interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger' | 'outline';
  size?: 'xs' | 'sm' | 'md' | 'lg';
  isLoading?: boolean;
  icon?: JSX.Element;
  glow?: boolean;
}

export const Button: ParentComponent<ButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['variant', 'size', 'isLoading', 'icon', 'glow', 'class', 'children', 'disabled']);

  const variants = {
    primary: "bg-accent text-white hover:opacity-90 border-transparent",
    secondary: "bg-sidebar text-text-secondary border-border-subtle hover:bg-sidebar/80 hover:text-text-primary hover:border-accent/40",
    outline: "bg-transparent text-accent border-accent hover:bg-accent/10",
    ghost: "bg-transparent text-text-muted hover:text-accent hover:bg-accent/10 border-transparent",
    danger: "bg-red-900/10 text-red-500 border-red-900/30 hover:bg-red-500 hover:text-white"
  };

  const sizes = {
    xs: "text-[9px] px-2 py-1 gap-1",
    sm: "text-[10px] px-4 py-2 gap-2",
    md: "text-xs px-6 py-3 gap-2.5",
    lg: "text-sm px-8 py-4 gap-3"
  };

  return (
    <button
      {...others}
      disabled={local.disabled || local.isLoading}
      class={cn(
        "inline-flex items-center justify-center font-mono font-black uppercase tracking-widest transition-ui disabled:opacity-50 disabled:cursor-not-allowed focus:outline-none focus:ring-1 focus:ring-accent/40 active:translate-y-px border",
        variants[local.variant || 'primary'],
        sizes[local.size || 'md'],
        local.glow && "shadow-glow hover:shadow-glow-strong",
        local.class
      )}
    >
      {local.isLoading ? (
        <svg class="animate-spin h-3 w-3" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      ) : local.icon}
      {local.children}
    </button>
  );
};
