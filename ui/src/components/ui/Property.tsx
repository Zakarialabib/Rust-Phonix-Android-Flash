import { JSX, Show } from 'solid-js';
import { cn } from '../../lib/utils';

interface PropertyProps {
    label: string;
    value?: string | number | JSX.Element;
    highlight?: boolean;
    class?: string;
    icon?: JSX.Element;
    accent?: boolean;
}

export const Property = (props: PropertyProps) => {
    return (
        <div class={cn(
            "flex items-center justify-between py-2.5 px-3 border-b border-border-subtle group transition-ui hover:bg-white/[0.02]",
            props.highlight && "bg-accent/5",
            props.class
        )}>
            <div class="flex items-center gap-2">
                {props.icon && <span class="text-text-muted group-hover:text-accent transition-colors">{props.icon}</span>}
                <span class="text-[10px] text-text-muted font-bold uppercase tracking-widest opacity-60 group-hover:opacity-100 transition-opacity">
                    {props.label}
                </span>
            </div>
            <div class={cn(
                "font-mono text-[10px] font-black tracking-tighter uppercase",
                props.accent ? "text-accent" : "text-text-secondary group-hover:text-text-primary"
            )}>
                {props.value ?? '---'}
            </div>
        </div>
    );
};

export const PropertyGrid = (props: { children: any; cols?: 1 | 2 | 3; class?: string }) => {
    const gridCols = {
        1: "grid-cols-1",
        2: "grid-cols-1 sm:grid-cols-2",
        3: "grid-cols-1 sm:grid-cols-2 lg:grid-cols-3"
    };

    return (
        <div class={cn("grid gap-x-6 gap-y-0", gridCols[props.cols || 1], props.class)}>
            {props.children}
        </div>
    );
};
