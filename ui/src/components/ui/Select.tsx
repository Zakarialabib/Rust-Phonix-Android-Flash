import { JSX, ParentComponent, splitProps } from 'solid-js';
import { cn } from '../../lib/utils';

interface SelectProps extends JSX.SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  error?: string;
}

export const Select: ParentComponent<SelectProps> = (props) => {
  const [local, others] = splitProps(props, ['label', 'error', 'class', 'children']);

  return (
    <div class="w-full space-y-1.5">
      {local.label && (
        <label class="block text-[10px] font-mono font-black text-text-muted uppercase tracking-[0.2em] italic">
          {local.label}
        </label>
      )}
      <div class="relative group">
        <select
          {...others}
          class={cn(
            "w-full appearance-none rounded-none border bg-sidebar/50 px-4 py-3 text-xs font-mono text-text-primary transition-ui focus:outline-none focus:ring-1 italic font-bold",
            local.error
              ? "border-red-900/50 focus:border-red-500 focus:ring-red-500/30"
              : "border-border-subtle focus:border-accent/40 focus:ring-accent/20",
            local.class
          )}
        >
          {local.children}
        </select>
        <div class="pointer-events-none absolute right-4 top-1/2 -translate-y-1/2 text-text-muted group-hover:text-accent transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M19 9l-7 7-7-7" />
          </svg>
        </div>
      </div>
      {local.error && <span class="block text-[10px] font-mono font-black text-red-500 uppercase tracking-widest italic">{local.error}</span>}
    </div>
  );
};
