import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { produce } from 'solid-js/store';
import { tauriApi } from '../api/tauri';
import { getAppErrorMessage } from '../errorCodes';
import { BuildProgress } from '../types';
import { globalStore, setGlobalStore } from '../store';
import { Card } from '../components/ui/Card';
import { Button } from '../components/ui/Button';
import { Select } from '../components/ui/Select';
import { Badge } from '../components/ui/Badge';
import { Collapsible } from '../components/ui/Collapsible';
import { cn } from '../lib/utils';

export default function BuildView() {
  const [profile, setProfile] = createSignal('ambient');
  const [variant, setVariant] = createSignal('beta');
  const [features, setFeatures] = createSignal({
    gapps: true,
    root: false,
    debloat: true
  });

  const toggleFeature = (key: 'gapps' | 'root' | 'debloat') => {
    setFeatures(prev => ({ ...prev, [key]: !prev[key] }));
  };

  createEffect(() => {
    let unlisten: UnlistenFn | undefined;
    const setupListener = async () => {
      try {
        unlisten = await listen<BuildProgress>('build-progress', (event) => {
          const { step, progress: pct, message, logLine } = event.payload;
          setGlobalStore('buildStatus', produce((state) => {
            state.percent = pct;
            state.currentStage = `${step}: ${message}`;
            if (logLine) {
              state.log.push(`[${new Date().toLocaleTimeString()}] ${logLine}`);
              if (state.log.length > 500) state.log.shift();
            }
          }));
        });
      } catch (err) {
        console.error('Failed to setup listener', err);
      }
    };
    setupListener();
    onCleanup(() => { if (unlisten) unlisten(); });
  });

  const startBuild = async () => {
    setGlobalStore('buildStatus', 'inProgress', true);
    setGlobalStore('buildStatus', 'percent', 0);
    setGlobalStore('buildStatus', 'currentStage', 'Initializing Materializing Sequence...');

    const initMsg = `Foundry Session Started | Profile: ${profile()} | Intent: ${variant()}`;
    setGlobalStore('buildStatus', produce(state => {
      state.log.push(`[${new Date().toLocaleTimeString()}] ${initMsg}`);
    }));

    try {
      await tauriApi.startBuild(profile(), variant(), 'output');
    } catch (e) {
      const message = getAppErrorMessage(e);
      setGlobalStore('buildStatus', produce(state => {
        state.log.push(`[${new Date().toLocaleTimeString()}] CRITICAL: ${message}`);
        state.currentStage = 'Materialization Failed';
      }));
    } finally {
      setGlobalStore('buildStatus', 'inProgress', false);
    }
  };

  return (
    <div class="h-full flex flex-col gap-6 font-mono">
      <header class="flex flex-col gap-1">
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-accent animate-pulse shadow-[0_0_8px_rgba(var(--accent-rgb),0.4)]" />
          <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase italic">The Foundry</h2>
        </div>
        <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">Intent-Based Build Pipeline | Layer 2 — Image Synthesis</p>
      </header>

      <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 flex-1 min-h-0">
        {/* Controls Column */}
        <div class="lg:col-span-4 flex flex-col gap-6 overflow-y-auto custom-scrollbar pr-2 leading-none pb-4">
          <Card glow="indigo" title="Synthesis Config" subtitle="Declare your intent for the target hardware" class="border-border-subtle">
            <div class="space-y-6">
              <Select
                label="Target Blueprint"
                value={profile()}
                onInput={e => setProfile(e.currentTarget.value)}
                class="bg-sidebar/40 border-border-subtle rounded-none h-11 text-[11px] font-bold italic tracking-wider text-text-secondary"
              >
                <option value="ambient">Ambient (Android TV + Leanback)</option>
                <option value="minimal">Minimal (Edge Server / CLI)</option>
                <option value="desktop">Desktop (Armbian X11/Wayland)</option>
                <option value="retro">Retro (EmuELEC / Gaming)</option>
              </Select>

              <div class="p-4 bg-accent/5 border border-accent/10 text-[9px] text-text-muted leading-relaxed uppercase italic font-bold rounded-sm">
                <Show when={profile() === 'ambient'}>
                  Full 10ft UI experience. Materializes leanback launcher, media codecs, and optional GMS blobs. Optimized for S905W/X.
                </Show>
                <Show when={profile() === 'minimal'}>
                  Alpine-based headless synthesis. materializes sshd, docker, and busybox. ideal for ARM cluster nodes.
                </Show>
                <Show when={profile() === 'desktop'}>
                  Materializes XFCE toolkit and mesa-gpu drivers. turns device into a sovereign terminal. requires 2gb+ ram.
                </Show>
                <Show when={profile() === 'retro'}>
                  Gaming-centric synthesis. materializes retroarch core and hardware-accelerated SDL2.
                </Show>
              </div>

              <div class="grid grid-cols-2 gap-4">
                <Select
                  label="Synthesis Channel"
                  value={variant()}
                  onInput={e => setVariant(e.currentTarget.value)}
                  class="bg-sidebar/40 border-border-subtle rounded-none h-11 text-[11px] font-bold italic tracking-wider text-text-secondary"
                >
                  <option value="stable">Stable (Production)</option>
                  <option value="beta">Beta (Experimental)</option>
                  <option value="nightly">Nightly (Raw Trace)</option>
                </Select>

                <div class="space-y-2">
                  <label class="text-[9px] text-text-muted uppercase tracking-[0.3em] font-black opacity-60">Compiler State</label>
                  <div class={cn(
                    "h-11 border border-border-subtle bg-sidebar/20 flex items-center justify-center text-[10px] font-black italic rounded-none tracking-widest",
                    globalStore.buildStatus.inProgress ? "text-accent bg-accent/5 animate-pulse" : "text-text-muted"
                  )}>
                    {globalStore.buildStatus.inProgress ? 'COMPILING' : 'IDLE'}
                  </div>
                </div>
              </div>

              <div class="space-y-3">
                <span class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] block opacity-60">Feature Synthesis Matrix</span>
                <div class="grid grid-cols-1 gap-2">
                  <button
                    onClick={() => toggleFeature('gapps')}
                    class={cn(
                      "group flex items-center justify-between p-4 border transition-all text-[10px] font-black uppercase tracking-widest italic rounded-sm",
                      features().gapps
                        ? "border-accent/50 bg-accent/10 text-accent"
                        : "border-border-subtle bg-sidebar/30 text-text-muted hover:border-border-strong hover:bg-sidebar/50"
                    )}
                  >
                    <span>Google Services</span>
                    <span class={cn("text-[9px] px-2 py-0.5 rounded-none", features().gapps ? "bg-accent text-white" : "bg-sidebar text-text-muted opacity-40")}>
                      {features().gapps ? 'INJECTED' : 'EXCLUDED'}
                    </span>
                  </button>
                  <button
                    onClick={() => toggleFeature('root')}
                    class={cn(
                      "group flex items-center justify-between p-4 border transition-all text-[10px] font-black uppercase tracking-widest italic rounded-sm",
                      features().root
                        ? "border-accent/50 bg-accent/10 text-accent"
                        : "border-border-subtle bg-sidebar/30 text-text-muted hover:border-border-strong hover:bg-sidebar/50"
                    )}
                  >
                    <span>Root Authorization</span>
                    <span class={cn("text-[9px] px-2 py-0.5 rounded-none", features().root ? "bg-accent text-white" : "bg-sidebar text-text-muted opacity-40")}>
                      {features().root ? 'MAGISK' : 'STOCK'}
                    </span>
                  </button>
                </div>
              </div>

              <div class="pt-2">
                <Button
                  onClick={startBuild}
                  isLoading={globalStore.buildStatus.inProgress}
                  class="w-full h-14 font-black italic text-md tracking-[0.2em] rounded-none shadow-[0_10px_30px_rgba(var(--accent-rgb),0.15)] ring-1 ring-accent/20"
                >
                  MATERIALIZE IMAGE
                </Button>
              </div>
            </div>
          </Card>

          <Card glow="indigo" title="Synthesis Estimate" subtitle="Foundry capacity evaluation" class="bg-sidebar/20 border-border-subtle">
            <div class="space-y-3 text-[10px] font-black text-text-muted uppercase tracking-widest italic leading-none py-1">
              <div class="flex justify-between border-b border-border-subtle/50 pb-2">
                <span class="opacity-60">Synthesis TTL</span>
                <span class="text-accent underline decoration-accent/20 underline-offset-4">~12 - 45 MINS</span>
              </div>
              <div class="flex justify-between border-b border-border-subtle/50 pb-2">
                <span class="opacity-60">Target Weight</span>
                <span class="text-text-secondary">{profile() === 'minimal' ? '450 MB' : '1.4 GB'}</span>
              </div>
              <div class="flex justify-between">
                <span class="opacity-60">Node Target</span>
                <span class="text-text-primary underline decoration-text-primary/10 underline-offset-4">{globalStore.lastDetected?.model || 'GENERIC-ARM64'}</span>
              </div>
            </div>
          </Card>
        </div>

        {/* Console Column */}
        <div class="lg:col-span-8 flex flex-col bg-sidebar/30 border border-border-subtle rounded-none overflow-hidden min-h-[500px]">
          <header class="h-16 bg-sidebar/50 border-b border-border-subtle px-6 flex items-center justify-between shrink-0">
            <div class="flex items-center gap-4">
              <div class={cn(
                "w-2 h-2 rounded-full",
                globalStore.buildStatus.inProgress ? "bg-accent animate-ping shadow-[0_0_10px_rgba(var(--accent-rgb),0.6)]" : "bg-slate-700 opacity-40"
              )} />
              <span class="text-[11px] font-black text-text-muted uppercase tracking-[0.3em] italic">Foundry Trace</span>
              <Badge variant={globalStore.buildStatus.inProgress ? "secondary" : "default"} class="rounded-none px-4 font-black tracking-widest italic py-1 text-[9px]">
                {globalStore.buildStatus.currentStage || 'AWAITING BLUEPRINT'}
              </Badge>
            </div>
            <button
              class="text-[10px] font-black text-text-muted hover:text-rose-500 uppercase tracking-widest transition-colors italic opacity-50 hover:opacity-100"
              onClick={() => setGlobalStore('buildStatus', 'log', [])}
            >
              Flush Cache
            </button>
          </header>

          <div class="flex-1 overflow-y-auto p-8 font-mono text-[10px] text-text-muted custom-scrollbar selection:bg-accent/20 italic leading-relaxed">
            <For each={globalStore.buildStatus.log} fallback={
              <div class="h-full flex flex-col items-center justify-center opacity-10 select-none grayscale">
                <div class="text-6xl mb-6">⚒️</div>
                <span class="text-xs tracking-[1em] font-black uppercase">Foundry Standby</span>
              </div>
            }>
              {(line) => (
                <div class="flex gap-4 mb-2 hover:bg-white/[0.02] transition-colors border-l-2 border-border-subtle pl-5 py-0.5 group">
                  <span class="text-accent shrink-0 font-black opacity-40 group-hover:opacity-100 transition-opacity">{" >> "}</span>
                  <span class="text-text-secondary font-bold tracking-tight">{line}</span>
                </div>
              )}
            </For>
          </div>

          <footer class="h-1.5 bg-sidebar/50 shrink-0 relative overflow-hidden">
            <div
              class="h-full bg-accent shadow-[0_0_15px_rgba(var(--accent-rgb),0.6)] transition-all duration-1000 ease-out"
              style={{ width: `${globalStore.buildStatus.percent}%` }}
            ></div>
          </footer>
        </div>
      </div>
    </div>
  );
}
