import { createResource, Show, For } from 'solid-js';
import { tauriApi } from '../api/tauri';
import { getAppErrorMessage } from '../errorCodes';
import { Card } from '../components/ui/Card';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';
import { useApp, Language, ThemeColor, ThemeMode } from '../context/AppContext';
import { cn } from '../lib/utils';

export default function SettingsView() {
  const { state, t, setLanguage, setThemeMode, setThemeColor, setUIScale, setTypography } = useApp();
  const [settings, { mutate }] = createResource(tauriApi.getSettings);
  const errorMessage = () => (settings.error ? getAppErrorMessage(settings.error) : '');

  const save = async () => {
    const s = settings();
    if (!s) return;
    try {
      await tauriApi.saveSettings(s);
      addLog('Settings cached to local node.', 'success');
    } catch (e) {
      alert(getAppErrorMessage(e));
    }
  };

  const addLog = (msg: string, level: string) => {
    console.log(`[SETTINGS] ${msg}`);
  };

  const languages: { label: string; value: Language }[] = [
    { label: 'English', value: 'en' },
    { label: 'العربية', value: 'ar' },
    { label: 'Français', value: 'fr' },
  ];

  const colors: { label: string; value: ThemeColor; bg: string }[] = [
    { label: t('common.amber'), value: 'amber', bg: 'bg-amber-500' },
    { label: t('common.indigo'), value: 'indigo', bg: 'bg-indigo-500' },
    { label: t('common.rose'), value: 'rose', bg: 'bg-rose-500' },
    { label: t('common.teal'), value: 'teal', bg: 'bg-teal-500' },
    { label: t('common.slate'), value: 'slate', bg: 'bg-slate-500' },
  ];

  return (
    <div class="h-full flex flex-col gap-6 font-mono">
      <header class="flex flex-col gap-1 text-left">
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-accent animate-pulse" />
          <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase">{t('settings.title')}</h2>
        </div>
        <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">{t('settings.subtitle')}</p>
      </header>

      <div class="max-w-4xl grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
        {/* Appearance Section */}
        <Card title={t('settings.sec_appearance_title')} subtitle={t('settings.sec_appearance_desc')} class="h-full">
          <div class="space-y-8">
            <div class="grid grid-cols-1 gap-8">
              {/* Language Selection */}
              <div class="space-y-4">
                <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">{t('settings.lbl_language')}</label>
                <div class="flex flex-wrap gap-1">
                  <For each={languages}>
                    {(l) => (
                      <button
                        aria-pressed={state.language === l.value}
                        onClick={() => setLanguage(l.value)}
                        class={cn(
                          "px-6 py-2 text-left text-[10px] font-bold uppercase tracking-widest transition-all border",
                          state.language === l.value
                            ? "bg-accent/10 text-accent border-accent shadow-glow shadow-accent/10"
                            : "bg-sidebar/50 border-border-subtle text-text-muted hover:text-text-secondary hover:bg-sidebar"
                        )}
                      >
                        {l.label}
                      </button>
                    )}
                  </For>
                </div>
              </div>

              {/* Theme Mode & Color */}
              <div class="space-y-4">
                <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">{t('settings.lbl_color')}</label>
                <div class="flex gap-2">
                  <button
                    aria-pressed={state.themeMode === 'dark'}
                    onClick={() => setThemeMode('dark')}
                    class={cn(
                      "flex-1 px-4 py-2 text-[10px] font-bold uppercase tracking-widest border transition-all",
                      state.themeMode === 'dark' ? "bg-accent/5 border-accent text-accent shadow-glow shadow-accent/10" : "bg-sidebar/30 border-border-subtle text-text-muted"
                    )}
                  >
                    {t('settings.lbl_theme_dark')}
                  </button>
                  <button
                    aria-pressed={state.themeMode === 'light'}
                    onClick={() => setThemeMode('light')}
                    class={cn(
                      "flex-1 px-4 py-2 text-[10px] font-bold uppercase tracking-widest border transition-all",
                      state.themeMode === 'light' ? "bg-accent/5 border-accent text-accent shadow-glow shadow-accent/10" : "bg-sidebar/30 border-border-subtle text-text-muted"
                    )}
                  >
                    {t('settings.lbl_theme_light')}
                  </button>
                </div>

                <div class="grid grid-cols-5 gap-2 pt-2">
                  <For each={colors}>
                    {(c) => (
                      <button
                        aria-label={c.label}
                        aria-pressed={state.themeColor === c.value}
                        onClick={() => setThemeColor(c.value)}
                        class={cn(
                          "aspect-square rounded-none border-2 transition-all flex items-center justify-center",
                          c.bg,
                          state.themeColor === c.value ? "border-text-primary scale-110 shadow-lg shadow-black/20" : "border-transparent opacity-40 hover:opacity-100"
                        )}
                        title={c.label}
                      >
                        <Show when={state.themeColor === c.value}>
                          <div class="w-1.5 h-1.5 bg-white rounded-full shadow-sm" />
                        </Show>
                      </button>
                    )}
                  </For>
                </div>
              </div>
            </div>
          </div>
        </Card>

        {/* HUD Scaling & Typography */}
        <Card title={t('settings.sec_logic_title')} subtitle={t('settings.sec_logic_desc')} class="h-full">
          <div class="space-y-8">
            {/* UI Scaling */}
            <div class="space-y-4">
              <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">{t('settings.lbl_scaling')}</label>
              <div class="grid grid-cols-3 gap-1">
                <For each={['compact', 'normal', 'large'] as const}>
                  {(scale) => (
                    <button
                      aria-pressed={state.uiScale === scale}
                      onClick={() => setUIScale(scale)}
                      class={cn(
                        "px-4 py-2 text-[10px] font-bold uppercase tracking-widest transition-all border text-center flex items-center justify-center",
                        state.uiScale === scale
                          ? "bg-accent/10 text-accent border-accent shadow-glow shadow-accent/10"
                          : "bg-sidebar/50 border-border-subtle text-text-muted hover:bg-sidebar"
                      )}
                    >
                      {scale}
                    </button>
                  )}
                </For>
              </div>
              <p class="text-[8px] text-text-muted uppercase opacity-40 leading-relaxed">{t('settings.scaling_desc')}</p>
            </div>

            {/* Typography Selection */}
            <div class="space-y-4">
              <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">{t('settings.lbl_typography')}</label>
              <div class="flex flex-col gap-1">
                <For each={['technical', 'modern', 'classic'] as const}>
                  {(style) => (
                    <button
                      aria-pressed={state.typography === style}
                      onClick={() => setTypography(style)}
                      class={cn(
                        "px-4 py-2 text-left text-[10px] font-bold uppercase tracking-widest transition-all border flex justify-between items-center group",
                        state.typography === style
                          ? "bg-accent/10 text-accent border-accent shadow-glow shadow-accent/10"
                          : "bg-sidebar/50 border-border-subtle text-text-muted hover:bg-sidebar"
                      )}
                    >
                      <span>{style === 'technical' ? t('settings.typo_technical') : style === 'modern' ? t('settings.typo_modern') : t('settings.typo_classic')}</span>
                      <span class={cn(
                        "text-[9px] opacity-30 group-hover:opacity-60",
                        style === 'technical' ? 'font-mono' : style === 'modern' ? 'font-sans' : 'font-serif'
                      )}>ABC 123</span>
                    </button>
                  )}
                </For>
              </div>
            </div>
          </div>
        </Card>

        {/* Path configuration and system reset */}
        <div class="lg:col-span-2 space-y-6 pt-2">
          <Card title={t('settings.sec_paths_title')} subtitle={t('settings.sec_paths_desc')}>
            <Show when={settings.error}>
              <div class="py-6 px-4 mb-6 border-l-2 border-accent bg-accent/5 text-[10px] text-accent font-black uppercase">
                {t('settings.signal_loss')}: {errorMessage()}
              </div>
            </Show>

            <Show when={!settings.error && settings()} fallback={<div class="py-20 text-center text-[10px] text-text-muted uppercase tracking-widest animate-pulse">{t('settings.polling_config')}</div>}>
              <div class="space-y-8">
                <div class="grid gap-6">
                  <div class="space-y-2">
                    <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">{t('settings.lbl_tools')}</label>
                    <Input
                      placeholder="C:\PHOENIX\BIN"
                      value={settings()!.toolsPath}
                      onInput={e => mutate({ ...settings()!, toolsPath: e.currentTarget.value })}
                      class="bg-sidebar/50 border-border-subtle rounded-none h-11 text-xs"
                    />
                  </div>
                  <div class="space-y-2">
                    <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">{t('settings.lbl_cache')}</label>
                    <Input
                      placeholder="C:\PHOENIX\CACHE"
                      value={settings()!.cachePath}
                      onInput={e => mutate({ ...settings()!, cachePath: e.currentTarget.value })}
                      class="bg-sidebar/50 border-border-subtle rounded-none h-11 text-xs"
                    />
                  </div>
                  <div class="space-y-2">
                    <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">{t('settings.lbl_output')}</label>
                    <Input
                      placeholder="C:\PHOENIX\OUT"
                      value={settings()!.outputPath}
                      onInput={e => mutate({ ...settings()!, outputPath: e.currentTarget.value })}
                      class="bg-sidebar/50 border-border-subtle rounded-none h-11 text-xs"
                    />
                  </div>
                </div>

                <div class="pt-6 border-t border-border-subtle flex flex-col gap-6">
                  <Button onClick={save} class="h-12 w-full font-black text-xs uppercase rounded-none tracking-widest">{t('settings.btn_save')}</Button>

                  <div class="p-5 border border-red-900/20 bg-red-900/5 space-y-4">
                    <div>
                      <h4 class="text-[10px] font-black text-red-500 uppercase tracking-widest mb-1">{t('settings.reset_title')}</h4>
                      <p class="text-[9px] text-text-muted leading-relaxed uppercase opacity-60">{t('settings.reset_desc')}</p>
                    </div>
                    <Button
                      variant="ghost"
                      class="w-full h-11 border-red-900/30 text-red-500 font-black text-[10px] hover:bg-red-500/10 rounded-none uppercase"
                      onClick={() => {
                        if (confirm(t('settings.reset_confirm'))) {
                          localStorage.removeItem('phoenix_onboarding_complete');
                          window.location.reload();
                        }
                      }}
                    >
                      {t('settings.btn_reset')}
                    </Button>
                  </div>
                </div>
              </div>
            </Show>
          </Card>
        </div>
      </div>
    </div>
  );
}
