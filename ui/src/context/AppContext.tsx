import { createContext, useContext, createSignal, createEffect, JSX } from 'solid-js';
import { createStore } from 'solid-js/store';
import * as i18n from '@solid-primitives/i18n';
import { en } from '../i18n/en';
import { ar } from '../i18n/ar';
import { fr } from '../i18n/fr';

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
}

const translations = {
    en: i18n.flatten(en),
    ar: i18n.flatten(ar),
    fr: i18n.flatten(fr),
};

interface AppContextType {
    state: AppState;
    t: i18n.NullableTranslator<ReturnType<typeof i18n.flatten<typeof en>>>;
    setLanguage: (lang: Language) => void;
    setThemeMode: (mode: ThemeMode) => void;
    setThemeColor: (color: ThemeColor) => void;
    setUIScale: (scale: UIScale) => void;
    setTypography: (style: TypographyStyle) => void;
}

const AppContext = createContext<AppContextType>();

export function AppProvider(props: { children: JSX.Element }) {
    const [state, setState] = createStore<AppState>({
        language: (localStorage.getItem('phoenix_lang') as Language) || 'en',
        themeMode: (localStorage.getItem('phoenix_theme_mode') as ThemeMode) || 'dark',
        themeColor: (localStorage.getItem('phoenix_theme_color') as ThemeColor) || 'amber',
        uiScale: (localStorage.getItem('phoenix_ui_scale') as UIScale) || 'normal',
        typography: (localStorage.getItem('phoenix_typography') as TypographyStyle) || 'technical',
    });

    const t = i18n.translator(() => translations[state.language], i18n.resolveTemplate);

    createEffect(() => {
        localStorage.setItem('phoenix_lang', state.language);
        document.documentElement.lang = state.language;
        document.documentElement.dir = state.language === 'ar' ? 'rtl' : 'ltr';
    });

    createEffect(() => {
        localStorage.setItem('phoenix_theme_mode', state.themeMode);
        if (state.themeMode === 'dark') {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
    });

    createEffect(() => {
        localStorage.setItem('phoenix_theme_color', state.themeColor);
        // Remove old accent classes
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

    const setLanguage = (lang: Language) => setState('language', lang);
    const setThemeMode = (mode: ThemeMode) => setState('themeMode', mode);
    const setThemeColor = (color: ThemeColor) => setState('themeColor', color);
    const setUIScale = (scale: UIScale) => setState('uiScale', scale);
    const setTypography = (style: TypographyStyle) => setState('typography', style);

    return (
        <AppContext.Provider value={{ state, t, setLanguage, setThemeMode, setThemeColor, setUIScale, setTypography }}>
            {props.children}
        </AppContext.Provider>
    );
}

export function useApp() {
    const context = useContext(AppContext);
    if (!context) throw new Error('useApp must be used within an AppProvider');
    return context;
}
