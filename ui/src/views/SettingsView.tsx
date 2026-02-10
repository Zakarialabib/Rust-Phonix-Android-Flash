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
    { label: t('common.amber') || 'Amber', value: 'amber', bg: 'bg-amber-500' },
    { label: t('common.indigo') || 'Indigo', value: 'indigo', bg: 'bg-indigo-500' },
    { label: t('common.rose') || 'Rose', value: 'rose', bg: 'bg-rose-500' },
    { label: t('common.teal') || 'Teal', value: 'teal', bg: 'bg-teal-500' },
    { label: t('common.slate') || 'Slate', value: 'slate', bg: 'bg-slate-500' },
  ];

  return (
    <div class="h-full flex flex-col gap-6 font-mono">
      <header class="flex flex-col gap-1 text-left">
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-accent animate-pulse" />
          <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase">{t('nav.infrastructure') || 'Infrastructure Config'}</h2>
        </div>
        <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">Environment Variables | Interface Protocol</p>
      </header>

      <div class="max-w-4xl grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
        {/* Appearance Section */}
        <Card title={t('common.theme') || 'Interface Aesthetics'} subtitle="UI aesthetics and linguistic framework" class="h-full">
          <div class="space-y-8">
            <div class="grid grid-cols-1 gap-8">
              {/* Language Selection */}
              <div class="space-y-4">
                <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">{t('common.language') || 'Language Hook'}</label>
                <div class="flex flex-wrap gap-1">
                  <For each={languages}>
                    {(l) => (
                      <button
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
                <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">{t('common.theme') || 'Color Primitive'}</label>
                <div class="flex gap-2">
                  <button
                    onClick={() => setThemeMode('dark')}
                    class={cn(
                      "flex-1 px-4 py-2 text-[10px] font-bold uppercase tracking-widest border transition-all",
                      state.themeMode === 'dark' ? "bg-accent/5 border-accent text-accent shadow-glow shadow-accent/10" : "bg-sidebar/30 border-border-subtle text-text-muted"
                    )}
                  >
                    {t('common.dark') || 'Dark Mode'}
                  </button>
                  <button
                    onClick={() => setThemeMode('light')}
                    class={cn(
                      "flex-1 px-4 py-2 text-[10px] font-bold uppercase tracking-widest border transition-all",
                      state.themeMode === 'light' ? "bg-accent/5 border-accent text-accent shadow-glow shadow-accent/10" : "bg-sidebar/30 border-border-subtle text-text-muted"
                    )}
                  >
                    {t('common.light') || 'Light Mode'}
                  </button>
                </div>

                <div class="grid grid-cols-5 gap-2 pt-2">
                  <For each={colors}>
                    {(c) => (
                      <button
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
        <Card title="Interface Logic" subtitle="Viewport scaling and typography architecture" class="h-full">
          <div class="space-y-8">
            {/* UI Scaling */}
            <div class="space-y-4">
              <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">Interface Scaling (HUD)</label>
              <div class="grid grid-cols-3 gap-1">
                <For each={['compact', 'normal', 'large'] as const}>
                  {(scale) => (
                    <button
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
              <p class="text-[8px] text-text-muted uppercase opacity-40 leading-relaxed">Adjusts font size and spatial density for high-DPI displays.</p>
            </div>

            {/* Typography Selection */}
            <div class="space-y-4">
              <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">Typography Architecture</label>
              <div class="flex flex-col gap-1">
                <For each={['technical', 'modern', 'classic'] as const}>
                  {(style) => (
                    <button
                      onClick={() => setTypography(style)}
                      class={cn(
                        "px-4 py-2 text-left text-[10px] font-bold uppercase tracking-widest transition-all border flex justify-between items-center group",
                        state.typography === style
                          ? "bg-accent/10 text-accent border-accent shadow-glow shadow-accent/10"
                          : "bg-sidebar/50 border-border-subtle text-text-muted hover:bg-sidebar"
                      )}
                    >
                      <span>{style === 'technical' ? 'Blueprint (Mono)' : style === 'modern' ? 'Synthetic (Sans)' : 'Archeology (Serif)'}</span>
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
          <Card title="Workspace Paths" subtitle="FileSystem mappings for synthesis and toolchains">
            <Show when={settings.error}>
              <div class="py-6 px-4 mb-6 border-l-2 border-accent bg-accent/5 text-[10px] text-accent font-black uppercase">
                SIGNAL_LOSS: {errorMessage()}
              </div>
            </Show>

            <Show when={!settings.error && settings()} fallback={<div class="py-20 text-center text-[10px] text-text-muted uppercase tracking-widest animate-pulse">Polling local configuration...</div>}>
              <div class="space-y-8">
                <div class="grid gap-6">
                  <div class="space-y-2">
                    <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">Toolchain Binary Root</label>
                    <Input
                      placeholder="C:\PHOENIX\BIN"
                      value={settings()!.toolsPath}
                      onInput={e => mutate({ ...settings()!, toolsPath: e.currentTarget.value })}
                      class="bg-sidebar/50 border-border-subtle rounded-none h-11 text-xs"
                    />
                  </div>
                  <div class="space-y-2">
                    <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">Synthesis Cache Buffer</label>
                    <Input
                      placeholder="C:\PHOENIX\CACHE"
                      value={settings()!.cachePath}
                      onInput={e => mutate({ ...settings()!, cachePath: e.currentTarget.value })}
                      class="bg-sidebar/50 border-border-subtle rounded-none h-11 text-xs"
                    />
                  </div>
                  <div class="space-y-2">
                    <label class="text-[9px] font-black text-text-muted uppercase tracking-widest">Deployment Output Dir</label>
                    <Input
                      placeholder="C:\PHOENIX\OUT"
                      value={settings()!.outputPath}
                      onInput={e => mutate({ ...settings()!, outputPath: e.currentTarget.value })}
                      class="bg-sidebar/50 border-border-subtle rounded-none h-11 text-xs"
                    />
                  </div>
                </div>

                <div class="pt-6 border-t border-border-subtle flex flex-col gap-6">
                  <Button onClick={save} class="h-12 w-full font-black text-xs uppercase rounded-none tracking-widest">COMMIT CHANGES</Button>

                  <div class="p-5 border border-red-900/20 bg-red-900/5 space-y-4">
                    <div>
                      <h4 class="text-[10px] font-black text-red-500 uppercase tracking-widest mb-1">Erasure Protocol</h4>
                      <p class="text-[9px] text-text-muted leading-relaxed uppercase opacity-60">Wipe all local node identity and restart discovery handshake.</p>
                    </div>
                    <Button
                      variant="ghost"
                      class="w-full h-11 border-red-900/30 text-red-500 font-black text-[10px] hover:bg-red-500/10 rounded-none uppercase"
                      onClick={() => {
                        if (confirm('PROCEED WITH FULL NODE ERASURE?')) {
                          localStorage.removeItem('phoenix_onboarding_complete');
                          window.location.reload();
                        }
                      }}
                    >
                      INITIATE FACTORY RESET
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
