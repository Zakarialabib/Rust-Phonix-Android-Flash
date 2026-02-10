import { Component, JSX, Show } from 'solid-js';
import { cn } from '../lib/utils';

interface NavButtonProps {
  label: string | undefined;
  active: boolean;
  onClick: () => void;
  icon: JSX.Element;
  collapsed?: boolean;
}

export const NavButton: Component<NavButtonProps> = (props) => {
  return (
    <button
      onClick={props.onClick}
      title={props.collapsed ? props.label : ''}
      class={cn(
        "group flex items-center gap-3 px-4 py-3 text-[10px] font-black font-mono transition-ui border-r-2 uppercase tracking-[0.2em] italic rounded-none",
        props.collapsed ? "justify-center w-full" : "w-full",
        props.active
          ? "bg-accent/10 text-accent border-accent shadow-glow"
          : "text-text-muted border-transparent hover:bg-white/[0.03] hover:text-text-primary hover:border-white/5"
      )}
    >
      <span class={cn(
        "transition-colors duration-200 shrink-0",
        props.active ? "text-accent drop-shadow-sm" : "text-text-muted group-hover:text-text-secondary"
      )}>
        {props.icon}
      </span>
      {!props.collapsed && (
        <span class="whitespace-nowrap overflow-hidden text-ellipsis">
          {props.label}
        </span>
      )}
    </button>
  );
};
