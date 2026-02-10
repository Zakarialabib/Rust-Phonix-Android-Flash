import { createSignal, Switch, Match, ErrorBoundary, Show, createEffect } from 'solid-js';
import { cn } from './lib/utils';
import { NavButton } from './components/NavButton';
import { globalStore, setGlobalStore } from './store';

import DetectView from './views/DetectView';
import ConfigView from './views/ConfigView';
import BuildView from './views/BuildView';
import FlashView from './views/FlashView';
import CheckView from './views/CheckView';
import SettingsView from './views/SettingsView';
import DiagnosticsView from './views/DiagnosticsView';
import WizardView from './views/WizardView';
import AmlogicBurnView from './views/AmlogicBurnView';
import RockchipFlashView from './views/RockchipFlashView';
import AllwinnerBurnView from './views/AllwinnerBurnView';
import { Button } from './components/ui/Button';
import { Card } from './components/ui/Card';
import { Badge } from './components/ui/Badge';
import { getAppErrorMessage } from './errorCodes';
import SplashScreen from './components/layout/SplashScreen';
import OnboardingView from './views/OnboardingView';
import { TopNav } from './components/layout/TopNav';
import { useApp } from './context/AppContext';

function App() {
  const setActiveTab = (tab: string) => setGlobalStore('activeTab', tab);
  const activeTab = () => globalStore.activeTab;
  const { state, t } = useApp();

  return (
    <Switch>
      <Match when={globalStore.setupStatus === 'splash'}>
        <SplashScreen />
      </Match>
      <Match when={globalStore.setupStatus === 'onboarding'}>
        <OnboardingView />
      </Match>
      <Match when={globalStore.setupStatus === 'ready'}>
        <div class="flex h-screen w-screen bg-bg-primary text-text-primary font-mono selection:bg-accent/30 overflow-hidden no-select transition-ui">
          {/* Sidebar - Premium Minimalist Style */}
          <aside
            class={cn(
              "flex flex-col border-r border-border-subtle bg-sidebar z-20 transition-all duration-500 ease-in-out",
              globalStore.sidebarCollapsed ? "w-20" : "w-64"
            )}
          >
            <div class="flex h-16 items-center gap-3 border-b border-border-subtle px-6 shrink-0 bg-white/[0.01]">
              <button
                onClick={() => setGlobalStore('sidebarCollapsed', !globalStore.sidebarCollapsed)}
                class="flex h-8 w-8 shrink-0 items-center justify-center bg-accent text-white font-black rounded-none hover:brightness-110 active:scale-95 transition-ui shadow-glow italic"
              >
                P
              </button>
              {!globalStore.sidebarCollapsed && (
                <div class="flex flex-col overflow-hidden">
                  <h1 class="text-sm font-black tracking-widest text-text-primary uppercase whitespace-nowrap italic">Phoenix</h1>
                  <span class="text-[9px] text-text-muted whitespace-nowrap uppercase tracking-widest opacity-60 font-bold">System v0.1.0-alpha</span>
                </div>
              )}
            </div>

            <nav class="flex-1 overflow-y-auto py-6 custom-scrollbar">
              <div class="px-4 mb-2">
                {!globalStore.sidebarCollapsed && (
                  <p class="px-3 text-[9px] font-black uppercase tracking-[0.2em] text-text-muted mb-4 opacity-40 italic">Workflow Protocol</p>
                )}
                <div class="flex flex-col gap-1">
                  <NavButton
                    active={activeTab() === 'wizard'}
                    onClick={() => setActiveTab('wizard')}
                    collapsed={globalStore.sidebarCollapsed}
                    icon={<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>}
                    label={t('nav.wizard') || 'Auto Wizard'}
                  />
                  <NavButton
                    active={activeTab() === 'detect'}
                    onClick={() => setActiveTab('detect')}
                    collapsed={globalStore.sidebarCollapsed}
                    icon={<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" /></svg>}
                    label={t('nav.discovery') || 'Discovery'}
                  />
                  <NavButton
                    active={activeTab() === 'diagnostics'}
                    onClick={() => setActiveTab('diagnostics')}
                    collapsed={globalStore.sidebarCollapsed}
                    icon={<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" /></svg>}
                    label={t('nav.forensics') || 'Forensics'}
                  />
                  <NavButton
                    active={activeTab() === 'config'}
                    onClick={() => setActiveTab('config')}
                    collapsed={globalStore.sidebarCollapsed}
                    icon={<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" /></svg>}
                    label={t('nav.architect') || 'Architect'}
                  />
                  <NavButton
                    active={activeTab() === 'check'}
                    onClick={() => setActiveTab('check')}
                    collapsed={globalStore.sidebarCollapsed}
                    icon={<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>}
                    label={t('nav.compatibility') || 'Compatibility'}
                  />
                  <NavButton
                    active={activeTab() === 'build'}
                    onClick={() => setActiveTab('build')}
                    collapsed={globalStore.sidebarCollapsed}
                    icon={<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" /></svg>}
                    label={t('nav.foundry') || 'Foundry'}
                  />
                  <NavButton
                    active={activeTab() === 'flash'}
                    onClick={() => setActiveTab('flash')}
                    collapsed={globalStore.sidebarCollapsed}
                    icon={<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" /></svg>}
                    label={t('nav.burner') || 'Burner'}
                  />
                </div>
              </div>

              <div class="px-4 mb-2 mt-6">
                {!globalStore.sidebarCollapsed && (
                  <p class="px-3 text-[9px] font-black uppercase tracking-[0.2em] text-text-muted mb-4 opacity-40 italic">Low-Level Hooks</p>
                )}
                <div class="flex flex-col gap-1">
                  <NavButton
                    active={activeTab() === 'amlogic-burn'}
                    onClick={() => setActiveTab('amlogic-burn')}
                    collapsed={globalStore.sidebarCollapsed}
                    icon={<div class="h-4 w-4 flex items-center justify-center font-black text-[8px] border border-border-subtle rounded-none italic">AM</div>}
                    label="Amlogic Burn"
                  />
                  <NavButton
                    active={activeTab() === 'rockchip-flash'}
                    onClick={() => setActiveTab('rockchip-flash')}
                    collapsed={globalStore.sidebarCollapsed}
                    icon={<div class="h-4 w-4 flex items-center justify-center font-black text-[8px] border border-border-subtle rounded-none italic">RK</div>}
                    label="Rockchip Flash"
                  />
                </div>
              </div>
            </nav>

            <div class="border-t border-border-subtle p-4 shrink-0 bg-white/[0.01]">
              <NavButton
                active={activeTab() === 'settings'}
                onClick={() => setActiveTab('settings')}
                collapsed={globalStore.sidebarCollapsed}
                icon={<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /></svg>}
                label={t('common.settings') || 'Settings'}
              />
            </div>
          </aside>

          {/* Main Content Area */}
          <div class="flex-1 flex flex-col min-w-0 h-full relative overflow-hidden transition-ui">
            {/* Subtle technical background */}
            <div class="absolute inset-0 bg-dot-matrix opacity-[0.4] pointer-events-none" />
            <div class="absolute inset-0 bg-[radial-gradient(circle_800px_at_100%_200px,var(--accent-glow),transparent)] pointer-events-none transition-ui duration-700" />

            {/* Top Navigation Bar with Breadcrumbs & Switchers */}
            <TopNav />

            {/* View Main Content */}
            <main class="flex-1 overflow-auto p-8 relative z-0 custom-scrollbar">
              <div class="h-full max-w-7xl mx-auto">
                <ErrorBoundary fallback={(err) => (
                  <Card class="border-rose-500/20 bg-rose-500/5">
                    <div class="flex flex-col items-center justify-center p-12 text-center italic">
                      <div class="inline-flex h-16 w-16 items-center justify-center rounded-none bg-rose-500/10 text-rose-500 mb-6 border border-rose-500/20 shadow-glow shadow-rose-500/20 animate-pulse">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                        </svg>
                      </div>
                      <h3 class="text-xl font-black text-rose-500 mb-2 uppercase tracking-tighter italic">Kernel Panic</h3>
                      <p class="text-text-muted mb-8 text-[11px] max-w-md mx-auto font-mono bg-sidebar/50 p-6 border border-border-subtle uppercase font-bold leading-relaxed">{getAppErrorMessage(err)}</p>
                      <Button
                        onClick={() => window.location.reload()}
                        variant="primary"
                        class="bg-rose-600 hover:bg-rose-500 border-none shadow-glow shadow-rose-500/30 px-10 italic font-black"
                      >
                        REBOOT SYSTEM ⚡
                      </Button>
                    </div>
                  </Card>
                )}>
                  <Switch>
                    <Match when={activeTab() === 'detect'}><DetectView /></Match>
                    <Match when={activeTab() === 'config'}><ConfigView /></Match>
                    <Match when={activeTab() === 'check'}><CheckView /></Match>
                    <Match when={activeTab() === 'build'}><BuildView /></Match>
                    <Match when={activeTab() === 'flash'}><FlashView /></Match>
                    <Match when={activeTab() === 'amlogic-burn'}><AmlogicBurnView /></Match>
                    <Match when={activeTab() === 'rockchip-flash'}><RockchipFlashView /></Match>
                    <Match when={activeTab() === 'allwinner-burn'}><AllwinnerBurnView /></Match>
                    <Match when={activeTab() === 'settings'}><SettingsView /></Match>
                    <Match when={activeTab() === 'diagnostics'}><DiagnosticsView /></Match>
                    <Match when={activeTab() === 'wizard'}><WizardView /></Match>
                  </Switch>
                </ErrorBoundary>
              </div>
            </main>
          </div>
        </div>
      </Match>
    </Switch>
  );
}

export default App;
