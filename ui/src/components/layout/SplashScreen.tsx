import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import { setGlobalStore } from '../../store';
import { useApp } from '../../context/AppContext';

export const SplashScreen: Component = () => {
    const { t } = useApp();
    const [loadingText, setLoadingText] = createSignal(t('splash.initializing'));
    const [progress, setProgress] = createSignal(0);

    const sequence = [
        { delay: 400, text: t('splash.loading_matrix') },
        { delay: 800, text: t('splash.mounting_engine') },
        { delay: 1200, text: t('splash.scanning_usb') },
        { delay: 1600, text: t('splash.reading_db') },
        { delay: 2000, text: t('splash.system_ready') },
    ];

    onMount(() => {
        let currentIdx = 0;
        const interval = setInterval(() => {
            if (currentIdx < sequence.length) {
                setLoadingText(sequence[currentIdx].text);
                setProgress((currentIdx + 1) * 20);
                currentIdx++;
            } else {
                clearInterval(interval);
                setTimeout(() => {
                    const isComplete = localStorage.getItem('phoenix_onboarding_complete') === 'true';
                    setGlobalStore('setupStatus', isComplete ? 'ready' : 'onboarding');
                }, 600);
            }
        }, 500);

        onCleanup(() => clearInterval(interval));
    });

    return (
        <div class="fixed inset-0 z-[9999] flex flex-col items-center justify-center bg-[#020617] text-white font-mono overflow-hidden">
            {/* Background Matrix Effect */}
            <div class="absolute inset-0 opacity-10 bg-dot-matrix pointer-events-none" />

            {/* Scanline Effect */}
            <div class="absolute inset-0 scanline pointer-events-none" />

            {/* Content */}
            <div class="relative flex flex-col items-center max-w-md w-full px-8">
                {/* Animated Phoenix Logo */}
                <div class="mb-12 relative">
                    <svg viewBox="0 0 100 100" class="w-24 h-24 text-amber-500 terminal-pulse">
                        <path d="M50 5 L95 50 L50 95 L5 50 Z" fill="none" stroke="currentColor" stroke-width="2" />
                        <path d="M50 20 L80 50 L50 80 L20 50 Z" fill="currentColor" opacity="0.5" />
                        <circle cx="50" cy="50" r="5" fill="currentColor" />
                    </svg>
                    <div class="absolute -inset-4 border border-amber-500/20 animate-spin-slow rounded-full pointer-events-none" />
                </div>

                {/* Brand */}
                <div class="text-center mb-8">
                    <h1 class="text-3xl font-bold tracking-[0.2em] mb-1 glitch-hover">PHOENIX</h1>
                    <p class="text-[10px] text-amber-500/60 tracking-[0.5em] uppercase">{t('splash.tagline')}</p>
                </div>

                {/* Loading Bar Container */}
                <div class="w-full h-[2px] bg-slate-900 mb-4 relative overflow-hidden">
                    <div
                        class="absolute top-0 left-0 h-full bg-amber-500 transition-all duration-300"
                        style={{ width: `${progress()}%` }}
                    />
                </div>

                {/* Loading Status */}
                <div class="flex items-center justify-between w-full text-[10px]">
                    <span class="text-amber-500/80 animate-pulse uppercase tracking-widest">{loadingText()}</span>
                    <span class="font-bold">{progress()}%</span>
                </div>

                {/* Technical Deco */}
                <div class="mt-12 opacity-30 text-[8px] flex gap-4 uppercase tracking-[0.2em]">
                    <span>{t('splash.ver')}</span>
                    <span>{t('splash.kernel')}</span>
                    <span>{t('splash.ui')}</span>
                </div>
            </div>
        </div>
    );
};

export default SplashScreen;
