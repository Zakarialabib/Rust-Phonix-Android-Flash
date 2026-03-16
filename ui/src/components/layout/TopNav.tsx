import { Component, For, Show } from 'solid-js';
import { useApp, Language, ThemeColor, ThemeMode } from '../../context/AppContext';
import { globalStore } from '../../store';
import { Badge } from '../ui/Badge';
import { cn } from '../../lib/utils';

export const TopNav: Component = () => {
    
    const { state, t, setLanguage, setThemeMode, setThemeColor } = useApp();

    const languages: { label: string; value: Language; flag: string }[] = [
        { label: 'English', value: 'en', flag: '🇺🇸' },
        { label: 'العربية', value: 'ar', flag: '🇲🇦' },
        { label: 'Français', value: 'fr', flag: '🇫🇷' },
    ];

    const colors = () => [
        { label: t('common.amber'), value: 'amber' as ThemeColor, bg: 'bg-amber-500' },
        { label: t('common.indigo'), value: 'indigo' as ThemeColor, bg: 'bg-indigo-500' },
        { label: t('common.rose'), value: 'rose' as ThemeColor, bg: 'bg-rose-500' },
        { label: t('common.teal'), value: 'teal' as ThemeColor, bg: 'bg-teal-500' },
        { label: t('common.slate'), value: 'slate' as ThemeColor, bg: 'bg-slate-500' },
    ];

    return (
        <header class="h-16 border-b border-border-subtle bg-sidebar/50 backdrop-blur-xl flex items-center justify-between px-6 shrink-0 z-10 transition-ui">
            {/* Breadcrumbs / View Title */}
            <div class="flex items-center gap-4">
                <div class="flex items-center gap-2 text-[10px] font-black font-mono uppercase tracking-[0.2em] text-text-muted">
                    <span class="text-white hover:text-text-primary cursor-default transition-ui opacity-40">PHOENIX</span>
                    <span class="opacity-20 translate-y--1px">/</span>
                    <span class="text-accent underline underline-offset-4 decoration-accent/30 shadow-glow shadow-accent/5">{globalStore.activeTab.toUpperCase()}</span>
                </div>
                <Show when={globalStore.lastDetected}>
                    <div class="h-3 w-px bg-border-subtle mx-2" />
                    <Badge variant="accent" size="sm" class="italic px-3">
                        {globalStore.lastDetected?.model || 'DEVICE_ONLINE'}
                    </Badge>
                </Show>
            </div>

            {/* Control Cluster */}
            <div class="flex items-center gap-6">
                {/* Color Style Picker */}
                <div class="flex items-center gap-2.5 px-3.5 py-1.5 bg-sidebar/30 border border-border-subtle rounded-none">
                    <For each={colors()}>
                        {(c) => (
                            <button
                                onClick={() => setThemeColor(c.value)}
                                aria-label={c.label}
                                class={cn(
                                    "w-3 h-3 transition-ui hover:scale-125",
                                    c.bg,
                                    state.themeColor === c.value ? "ring-1 ring-offset-2 ring-offset-sidebar ring-text-primary scale-110 shadow-glow" : "opacity-20 grayscale-[50%] hover:opacity-100 hover:grayscale-0"
                                )}
                                title={c.label}
                            />
                        )}
                    </For>
                </div>

                {/* Theme Toggle */}
                <button
                    onClick={() => setThemeMode(state.themeMode === 'dark' ? 'light' : 'dark')}
                    aria-label={t('common.theme_toggle')}
                    class="p-2 border border-border-subtle bg-sidebar/20 hover:bg-accent/10 transition-ui text-text-muted hover:text-accent group rounded-none"
                    title={t('common.theme') || ''}
                >
                    <Show when={state.themeMode === 'dark'} fallback={
                        <svg class="group-hover:rotate-12 transition-transform" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" /></svg>
                    }>
                        <svg class="group-hover:rotate-90 transition-transform duration-500" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4" /><path d="M12 2v2" /><path d="M12 20v2" /><path d="m4.93 4.93 1.41 1.41" /><path d="m17.66 17.66 1.41 1.41" /><path d="M2 12h2" /><path d="M20 12h2" /><path d="m6.34 17.66-1.41 1.41" /><path d="m19.07 4.93-1.41 1.41" /></svg>
                    </Show>
                </button>

                {/* Language Selector */}
                <div class="relative group">
                    <button
                        aria-haspopup="true"
                        aria-expanded="false"
                        aria-label={t('common.language')}
                        class="flex items-center gap-3 px-4 py-2 bg-sidebar/30 hover:bg-sidebar/50 border border-border-subtle rounded-none transition-ui hover:border-accent/40 group"
                    >
                        <span class="text-[9px] font-black uppercase tracking-widest text-text-muted group-hover:text-accent transition-colors">
                            {state.language}
                        </span>
                    </button>

                    <div class="absolute right-0 top-full mt-0 bg-sidebar border border-border-subtle shadow-2xl rounded-none overflow-hidden opacity-0 scale-95 invisible group-hover:opacity-100 group-hover:visible group-hover:scale-100 group-focus-within:opacity-100 group-focus-within:visible group-focus-within:scale-100 transition-ui z-20 min-w-[140px] border-t-0">
                        <div class="p-1 flex flex-col">
                            <For each={languages}>
                                {(l) => (
                                    <button
                                        onClick={() => setLanguage(l.value)}
                                        class={cn(
                                            "w-full text-left px-4 py-2.5 text-[9px] font-black uppercase tracking-widest transition-ui flex items-center gap-3",
                                            state.language === l.value
                                                ? "bg-accent/10 text-accent shadow-glow"
                                                : "text-text-muted hover:bg-white/5 hover:text-text-primary"
                                        )}
                                    >
                                        <span class="text-xs opacity-40 group-hover:opacity-80">{l.flag}</span>
                                        {l.label}
                                    </button>
                                )}
                            </For>
                        </div>
                    </div>
                </div>
            </div>
        </header>
    );
};
