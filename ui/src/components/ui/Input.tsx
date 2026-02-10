import { JSX, splitProps } from 'solid-js';
import { cn } from '../../lib/utils';

interface InputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
}

export const Input = (props: InputProps) => {
  const [local, others] = splitProps(props, ['label', 'error', 'class', 'type']);

  return (
    <div class="w-full space-y-1.5">
      {local.label && (
        <label class="block text-[10px] font-mono font-black text-text-muted uppercase tracking-[0.2em]">
          {local.label}
        </label>
      )}
      <input
        {...others}
        type={local.type || 'text'}
        class={cn(
          "w-full rounded-none border bg-sidebar/50 px-4 py-3 text-xs font-mono text-text-primary placeholder:text-text-muted/50 transition-ui focus:outline-none focus:ring-1 font-bold",
          local.error
            ? "border-red-900/50 focus:border-red-500 focus:ring-red-500/30"
            : "border-border-subtle focus:border-accent/40 focus:ring-accent/20",
          local.class
        )}
      />
      {local.error && <span class="block text-[10px] font-mono font-black text-red-500 uppercase tracking-widest">{local.error}</span>}
    </div>
  );
};
