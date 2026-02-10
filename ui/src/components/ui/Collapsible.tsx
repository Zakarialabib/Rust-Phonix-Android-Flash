import { Component, createSignal, JSX, Show } from 'solid-js';
import { cn } from '../../lib/utils';

interface CollapsibleProps {
    title: string;
    subtitle?: string;
    children: JSX.Element;
    defaultOpen?: boolean;
    class?: string;
}

export const Collapsible: Component<CollapsibleProps> = (props) => {
    const [isOpen, setIsOpen] = createSignal(props.defaultOpen ?? false);

    return (
        <div class={cn("border border-border-subtle rounded-sm overflow-hidden bg-sidebar/40", props.class)}>
            <button
                onClick={() => setIsOpen(!isOpen())}
                class="w-full flex items-center justify-between p-3 hover:bg-white/[0.02] transition-colors text-left"
            >
                <div class="flex flex-col">
                    <span class="text-xs font-black font-mono tracking-widest text-text-primary uppercase italic">
                        {props.title}
                    </span>
                    <Show when={props.subtitle}>
                        <span class="text-[10px] text-text-muted font-mono mt-0.5 uppercase tracking-wider opacity-60">{props.subtitle}</span>
                    </Show>
                </div>
                <div class={cn("transition-transform duration-300", isOpen() ? "rotate-180" : "")}>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                </div>
            </button>

            <div
                class={cn(
                    "transition-all duration-300 ease-in-out",
                    isOpen() ? "max-h-[2000px] border-t border-border-subtle" : "max-h-0 overflow-hidden"
                )}
            >
                <div class="p-4 bg-black/[0.05]">
                    {props.children}
                </div>
            </div>
        </div>
    );
};
