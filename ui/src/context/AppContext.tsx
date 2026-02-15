import { createContext, useContext, createSignal, createEffect, onMount, JSX } from 'solid-js';
import { createStore } from 'solid-js/store';
import * as i18n from '@solid-primitives/i18n';
import { en } from '../i18n/en';
import { ar } from '../i18n/ar';
import { fr } from '../i18n/fr';
import { tauriApi } from '../api/tauri';

export type Language = 'en' | 'ar' | 'fr';
export type ThemeMode = 'dark' | 'light';
export type ThemeColor = 'amber' | 'indigo' | 'rose' | 'teal' | 'slate';
export type UIScale = 'compact' | 'normal' | 'large';
export type TypographyStyle = 'technical' | 'modern' | 'classic';

interface AppState {
    language: Language;
    themeMode: ThemeMode;
    themeColor: ThemeColor;
    uiScale: UIScale;
    typography: TypographyStyle;
    // Infrastructure Paths
    toolsPath: string;
    cachePath: string;
    outputPath: string;
}

const translations = {
    en: i18n.flatten(en),
    ar: i18n.flatten(ar),
    fr: i18n.flatten(fr),
};

// Type-safe translator that always returns string
type TranslatorFunction = (key: string, params?: Record<string, string>) => string;

interface AppContextType {
    state: AppState;
    t: TranslatorFunction;
    setLanguage: (lang: Language) => void;
    setThemeMode: (mode: ThemeMode) => void;
    setThemeColor: (color: ThemeColor) => void;
    setUIScale: (scale: UIScale) => void;
    setTypography: (style: TypographyStyle) => void;
    syncToRust: () => Promise<void>;
}

const AppContext = createContext<AppContextType>();

export function AppProvider(props: { children: JSX.Element }) {
    const [state, setState] = createStore<AppState>({
        language: (localStorage.getItem('phoenix_lang') as Language) || 'en',
        themeMode: (localStorage.getItem('phoenix_theme_mode') as ThemeMode) || 'dark',
        themeColor: (localStorage.getItem('phoenix_theme_color') as ThemeColor) || 'amber',
        uiScale: (localStorage.getItem('phoenix_ui_scale') as UIScale) || 'normal',
        typography: (localStorage.getItem('phoenix_typography') as TypographyStyle) || 'technical',
        toolsPath: '',
        cachePath: '',
        outputPath: '',
    });

    const rawT = i18n.translator(() => translations[state.language], i18n.resolveTemplate);
    // Wrapper that always returns string (fallback to key if undefined)
    const t: TranslatorFunction = (key, params) => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const result = rawT(key as any, params);
        if (typeof result === 'string') return result;
        return String(key);
    };

    // Load settings from Rust on mount
    onMount(async () => {
        try {
            const settings = await tauriApi.getSettings();
            setState({
                language: (settings.language as Language) || 'en',
                themeMode: (settings.themeMode as ThemeMode) || 'dark',
                themeColor: (settings.themeColor as ThemeColor) || 'amber',
                uiScale: (settings.uiScale as UIScale) || 'normal',
                typography: (settings.typography as TypographyStyle) || 'technical',
                toolsPath: settings.toolsPath || '',
                cachePath: settings.cachePath || '',
                outputPath: settings.outputPath || '',
            });
            // Update localStorage to ensure sync
            localStorage.setItem('phoenix_lang', settings.language);
            localStorage.setItem('phoenix_theme_mode', settings.themeMode);
            localStorage.setItem('phoenix_theme_color', settings.themeColor);
            localStorage.setItem('phoenix_ui_scale', settings.uiScale);
            localStorage.setItem('phoenix_typography', settings.typography);
        } catch (e) {
            console.error("Failed to load settings from Rust node:", e);
        }
    });

    const syncToRust = async () => {
        if (!state.toolsPath) return; // Wait for initial load
        try {
            await tauriApi.saveSettings({
                toolsPath: state.toolsPath,
                cachePath: state.cachePath,
                outputPath: state.outputPath,
                language: state.language,
                themeMode: state.themeMode,
                themeColor: state.themeColor,
                uiScale: state.uiScale,
                typography: state.typography,
            });
        } catch (e) {
            console.error("Failed to persist settings to Rust node:", e);
        }
    };

    createEffect(() => {
        localStorage.setItem('phoenix_lang', state.language);
        document.documentElement.lang = state.language;
        document.documentElement.dir = state.language === 'ar' ? 'rtl' : 'ltr';
    });

    createEffect(() => {
        localStorage.setItem('phoenix_theme_mode', state.themeMode);
        if (state.themeMode === 'dark') {
            document.documentElement.classList.add('dark');
            document.documentElement.classList.remove('light');
        } else {
            document.documentElement.classList.remove('dark');
            document.documentElement.classList.add('light');
        }
    });

    createEffect(() => {
        localStorage.setItem('phoenix_theme_color', state.themeColor);
        document.documentElement.classList.forEach(cls => {
            if (cls.startsWith('accent-')) document.documentElement.classList.remove(cls);
        });
        document.documentElement.classList.add(`accent-${state.themeColor}`);
    });

    createEffect(() => {
        localStorage.setItem('phoenix_ui_scale', state.uiScale);
        document.documentElement.classList.forEach(cls => {
            if (cls.startsWith('scale-')) document.documentElement.classList.remove(cls);
        });
        document.documentElement.classList.add(`scale-${state.uiScale}`);
    });

    createEffect(() => {
        localStorage.setItem('phoenix_typography', state.typography);
        document.documentElement.classList.forEach(cls => {
            if (cls.startsWith('font-style-')) document.documentElement.classList.remove(cls);
        });
        document.documentElement.classList.add(`font-style-${state.typography}`);
    });

    // Auto-save to Rust when UI preferences change
    createEffect(() => {
        // Track dependencies
        state.language;
        state.themeMode;
        state.themeColor;
        state.uiScale;
        state.typography;
        // Trigger save
        syncToRust();
    });

    const setLanguage = (lang: Language) => setState('language', lang);
    const setThemeMode = (mode: ThemeMode) => setState('themeMode', mode);
    const setThemeColor = (color: ThemeColor) => setState('themeColor', color);
    const setUIScale = (scale: UIScale) => setState('uiScale', scale);
    const setTypography = (style: TypographyStyle) => setState('typography', style);

    return (
        <AppContext.Provider value={{ state, t, setLanguage, setThemeMode, setThemeColor, setUIScale, setTypography, syncToRust }}>
            {props.children}
        </AppContext.Provider>
    );
}

export function useApp() {
    const context = useContext(AppContext);
    if (!context) throw new Error('useApp must be used within an AppProvider');
    return context;
}
