import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import { listen } from '@tauri-apps/api/event';
import { tauriApi } from '../api/tauri';
import { getAppErrorMessage } from '../errorCodes';
import { Card } from '../components/ui/Card';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';
import { Badge } from '../components/ui/Badge';
import { AmlogicChipInfo, FlashProgress, FirmwareRecommendation, HardwareProfile } from '../types';
import { cn } from '../lib/utils';
import { useApp } from '../context/AppContext';

type Step = 'connect' | 'identify' | 'flash' | 'provision' | 'done';

export default function AmlogicBurnView() {
  const { t } = useApp();
  const [imagePath, setImagePath] = createSignal('');
  const [chipInfo, setChipInfo] = createSignal<AmlogicChipInfo | null>(null);
  const [isFlashing, setIsFlashing] = createSignal(false);
  const [progress, setProgress] = createSignal<FlashProgress | null>(null);
  const [status, setStatus] = createSignal('');
  const [currentStep, setCurrentStep] = createSignal<Step>('connect');
  const [logs, setLogs] = createSignal<string[]>([]);
  const [isExtracting, setIsExtracting] = createSignal(false);
  const [recommendations, setRecommendations] = createSignal<FirmwareRecommendation[]>([]);

  const addLog = (msg: string) => {
    setLogs(prev => [...prev.slice(-49), `[${new Date().toLocaleTimeString()}] ${msg}`]);
  };

  const steps = [
    { id: 'connect', label: 'Connect' },
    { id: 'identify', label: 'Identify' },
    { id: 'flash', label: 'Flash' },
    { id: 'provision', label: 'Provision' },
    { id: 'done', label: 'Ready' }
  ];

  createEffect(() => {
    const unlisten = listen<FlashProgress>('amlogic:progress', (event) => {
      const p = event.payload;
      setProgress(p);

      if (p.operation.toLowerCase().includes('provision') || p.partition?.includes('mac') || p.partition?.includes('hdcp')) {
        setCurrentStep('provision');
      } else if (p.operation.toLowerCase().includes('flash')) {
        setCurrentStep('flash');
      }

      setStatus(`${p.operation} ${p.partition ? `(${p.partition})` : ''}: ${p.percent}%`);

      if (p.percent === 0 || p.percent === 100) {
        addLog(`${p.operation}: ${p.percent}%`);
      }
    });

    onCleanup(() => {
      unlisten.then(f => f());
    });
  });

  const handleDetect = async () => {
    setStatus('Polling WorldCup bus...');
    setCurrentStep('connect');
    try {
      const info = await tauriApi.amlogicDetect();
      setChipInfo(info);
      addLog(`Hardware Handshake: ${info.chipId} detected (ROM v${info.romVersion})`);
      setStatus('Device Authenticated');
      setCurrentStep('identify');

      const profile: HardwareProfile = {
        soc: info.chipId,
        pcbVariant: 'unknown',
        ramVendor: 'unknown',
        wifiChip: 'unknown',
        emmcVendor: 'unknown',
        hdmiPhy: 'unknown'
      };

      const recs = await tauriApi.getFirmwareRecommendations(profile);
      setRecommendations(recs);
    } catch (e) {
      const msg = getAppErrorMessage(e);
      setStatus(`Error: ${msg}`);
      addLog(`Discovery Fail: ${msg}`);
    }
  };

  const handleFlash = async () => {
    if (!imagePath()) return;
    setIsFlashing(true);
    setCurrentStep('flash');
    addLog(`Initiating Partition Sequence: ${imagePath()}`);
    try {
      await tauriApi.amlogicFlashImage(imagePath());
      setStatus('Liberation Successful');
      setCurrentStep('done');
      addLog('Flashing completed. Device is ready for reboot.');
    } catch (e) {
      const msg = getAppErrorMessage(e);
      setStatus(`System Error: ${msg}`);
      addLog(`Flash Critical Fail: ${msg}`);
    } finally {
      setIsFlashing(false);
    }
  };

  const handleUnpack = async () => {
    if (!imagePath()) return;
    setIsExtracting(true);
    addLog(`Unpacking image container: ${imagePath()}`);
    try {
      const outDir = imagePath() + "_unpacked";
      await tauriApi.amlogicExtractImage(imagePath(), outDir);
      setStatus('Deconstruction Complete');
      addLog(`Partitions extracted to workspace: ${outDir}`);
    } catch (e) {
      const msg = getAppErrorMessage(e);
      addLog(`Extract Fail: ${msg}`);
    } finally {
      setIsExtracting(false);
    }
  };

  return (
    <div class="h-full flex flex-col gap-6 font-mono">
      <header class="flex flex-col gap-1 text-left">
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-accent animate-pulse shadow-[0_0_8px_rgba(var(--accent-rgb),0.4)]" />
          <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase">{t('amlogic.title')}</h2>
          <Badge variant={chipInfo() ? 'success' : 'warning'} class="rounded-none px-4 font-black ml-2">
            {chipInfo() ? 'Device Online' : 'Awaiting Connection'}
          </Badge>
        </div>
        <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">{t('amlogic.subtitle')}</p>
      </header>

      {/* Stepper */}
      <div class="grid grid-cols-5 gap-1 shrink-0 px-1">
        <For each={steps}>
          {(step, index) => {
            const isActive = steps.findIndex(s => s.id === currentStep()) >= index();
            const isCurrent = currentStep() === step.id;
            return (
              <div class="relative group">
                <div class={cn(
                  "h-1.5 transition-all duration-700 ease-out rounded-full",
                  isActive ? "bg-accent shadow-[0_0_10px_rgba(var(--accent-rgb),0.3)]" : "bg-sidebar/50"
                )} />
                <div class="mt-3 flex items-center justify-between px-1">
                  <span class={cn(
                    "text-[9px] font-black uppercase tracking-widest transition-colors duration-300",
                    isActive ? "text-accent" : "text-text-muted opacity-40 hover:opacity-100"
                  )}>
                    {index() + 1}. {step.label}
                  </span>
                  <Show when={isCurrent}>
                    <span class="w-2 h-2 rounded-full bg-accent animate-ping" />
                  </Show>
                </div>
              </div>
            );
          }}
        </For>
      </div>

      <div class="grid lg:grid-cols-12 gap-6 flex-1 min-h-0">
        {/* Main Controls */}
        <div class="lg:col-span-8 flex flex-col gap-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
          <Card glow="amber" title={t('amlogic.card_ops_title')} subtitle={t('amlogic.card_ops_subtitle')} class="border-border-subtle">
            <div class="space-y-6">
              <div class="grid grid-cols-2 gap-4">
                <div class="space-y-2">
                  <label class="text-[9px] text-text-muted uppercase tracking-[0.3em] font-black opacity-60">{t('amlogic.op_handshake_label')}</label>
                  <Button onClick={handleDetect} class="w-full h-11 border-border-subtle bg-sidebar/20 hover:bg-sidebar/40 rounded-none font-black text-xs tracking-widest text-text-muted hover:text-text-primary transition-all shadow-none">
                    {t('amlogic.btn_poll')}
                  </Button>
                </div>
                <div class="space-y-2">
                  <label class="text-[9px] text-text-muted uppercase tracking-[0.3em] font-black opacity-60">{t('amlogic.op_image_label')}</label>
                  <Button onClick={handleUnpack} disabled={!imagePath() || isExtracting()} isLoading={isExtracting()} class="w-full h-11 border-border-subtle bg-sidebar/20 hover:bg-sidebar/40 rounded-none font-black text-xs tracking-widest text-text-muted hover:text-text-primary transition-all shadow-none">
                    {t('amlogic.btn_partition')}
                  </Button>
                </div>
              </div>

              <div class="space-y-3">
                <label class="text-[9px] text-text-muted uppercase tracking-[0.3em] font-black opacity-60">{t('amlogic.op_firmware_label')}</label>
                <div class="flex gap-3">
                  <Input
                    value={imagePath()}
                    onInput={(e) => setImagePath(e.currentTarget.value)}
                    placeholder="C:\RESOURCES\FIRMWARE.IMG"
                    class="flex-1 bg-sidebar/40 border-border-subtle rounded-none text-xs h-11 font-bold tracking-tight text-text-secondary"
                  />
                  <Button
                    onClick={handleFlash}
                    disabled={!chipInfo() || isFlashing() || !imagePath()}
                    isLoading={isFlashing()}
                    class="h-11 px-10 rounded-none font-black text-xs border-none tracking-widest shadow-[0_5px_15px_rgba(var(--accent-rgb),0.2)]"
                  >
                    {t('amlogic.btn_flash')}
                  </Button>
                </div>
              </div>

              <Show when={imagePath().toLowerCase().includes('a11') || imagePath().toLowerCase().includes('android11')}>
                <div class="p-5 bg-rose-500/5 border border-rose-500/20 rounded-sm">
                  <div class="flex gap-4">
                    <span class="text-rose-500 font-black animate-pulse text-lg">!</span>
                    <div class="space-y-1.5">
                      <h4 class="text-[10px] font-black text-rose-400 uppercase tracking-widest leading-none">Incompatibility Detected (p282)</h4>
                      <p class="text-[9px] text-text-muted leading-relaxed font-bold opacity-60">
                        Android 11 kernels use a revised HDMI PHY driver. Older p282 boards (Samsung RAM) will experience black screens without a custom DTB overlay.
                      </p>
                    </div>
                  </div>
                </div>
              </Show>
            </div>
          </Card>

          {/* Progress & Logs */}
          <div class="flex-1 flex flex-col gap-6 min-h-[300px]">
            <Card glow="indigo" title={t('amlogic.card_log_title')} subtitle={t('amlogic.card_log_subtitle')} class="flex-1 flex flex-col overflow-hidden border-border-subtle">
              <div class="flex flex-col h-full gap-6">
                <Show when={progress()} fallback={
                  <div class="flex-1 border border-dashed border-border-subtle rounded-sm flex flex-col items-center justify-center text-[10px] text-text-muted uppercase tracking-[0.4em] font-black opacity-20 bg-sidebar/10">
                    <span class="text-4xl mb-4 grayscale opacity-40">⚒️</span>
                    {t('amlogic.log_placeholder')}
                  </div>
                }>
                  <div class="space-y-5 shrink-0 px-1 pt-1">
                    <div class="flex justify-between items-end">
                      <div class="space-y-2">
                        <span class="text-[10px] font-black text-accent uppercase tracking-[0.3em]">{progress()?.operation}</span>
                        <h4 class="text-2xl font-black text-text-primary tracking-tighter leading-none">{progress()?.partition || 'INITIALIZING'}</h4>
                      </div>
                      <div class="text-4xl font-black text-accent tracking-tighter drop-shadow-sm">{progress()?.percent}%</div>
                    </div>
                    <div class="h-2 bg-sidebar/50 relative overflow-hidden rounded-full">
                      <div class="absolute h-full bg-accent transition-all duration-500 ease-out shadow-[0_0_10px_rgba(var(--accent-rgb),0.5)]" style={{ width: `${progress()?.percent}%` }} />
                    </div>
                    <div class="flex justify-between text-[9px] text-text-muted uppercase font-black tracking-widest opacity-60">
                      <span>IO Trace: <span class="text-text-secondary">{(progress()!.bytesTransferred / 1024 / 1024).toFixed(1)} / {(progress()!.totalBytes / 1024 / 1024).toFixed(1)} MB</span></span>
                      <span>Link: <span class="text-text-secondary">{(progress()!.speedBps / 1024 / 1024).toFixed(2)} MB/S</span></span>
                    </div>
                  </div>
                </Show>

                <div class="flex-1 overflow-y-auto bg-black/20 border border-border-subtle p-5 space-y-2 custom-scrollbar rounded-sm">
                  <For each={logs()}>
                    {(log) => (
                      <div class="text-[10px] text-text-muted border-l-2 border-border-subtle pl-4 py-0.5 leading-relaxed hover:bg-white/[0.02] transition-colors font-bold opacity-70 hover:opacity-100 group">
                        <span class="text-accent mr-2 opacity-0 group-hover:opacity-100 transition-opacity">{" >> "}</span>
                        {log}
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </Card>
          </div>
        </div>

        {/* Info Column */}
        <div class="lg:col-span-4 flex flex-col gap-6 overflow-hidden pb-4">
          <Card glow="amber" title={t('amlogic.card_inspector_title')} subtitle={t('amlogic.card_inspector_subtitle')} class="border-border-subtle">
            <Show when={chipInfo()} fallback={
              <div class="py-14 border border-dashed border-border-subtle rounded-sm flex flex-col items-center justify-center text-[10px] text-text-muted uppercase text-center tracking-widest font-black opacity-20 bg-sidebar/5">
                {t('amlogic.inspector_placeholder')}
              </div>
            }>
              <div class="space-y-4 font-mono text-[10px] py-1 font-bold uppercase tracking-wider">
                <div class="flex justify-between border-b border-border-subtle pb-3">
                  <span class="text-text-muted opacity-60">{t('amlogic.label_chip')}</span>
                  <span class="text-accent underline decoration-accent/20 underline-offset-4">{chipInfo()?.chipId}</span>
                </div>
                <div class="flex justify-between border-b border-border-subtle pb-3">
                  <span class="text-text-muted opacity-60">{t('amlogic.label_rom')}</span>
                  <span class="text-text-secondary">{chipInfo()?.romVersion}</span>
                </div>
                <div class="flex justify-between border-b border-border-subtle pb-3">
                  <span class="text-text-muted opacity-60">{t('amlogic.label_ram')}</span>
                  <span class="text-text-secondary">{(chipInfo()!.ramSize / 1024 / 1024).toFixed(0)} MB SDRAM</span>
                </div>
                <div class="flex justify-between items-center pt-1">
                  <span class="text-text-muted opacity-60 font-black">{t('amlogic.label_secure')}</span>
                  <Badge variant={chipInfo()?.secureBoot ? 'error' : 'default'} class="rounded-none font-black px-3 py-1 text-[9px]">
                    {chipInfo()?.secureBoot ? 'LOCKED (SIGNED)' : 'OPEN (DEFAULT)'}
                  </Badge>
                </div>
              </div>
            </Show>
          </Card>

          <Card glow="teal" title={t('amlogic.card_stages_title')} subtitle={t('amlogic.card_stages_subtitle')} class="border-border-subtle bg-sidebar/20">
            <div class="space-y-5">
              <div class="space-y-2 opacity-60 hover:opacity-100 transition-opacity cursor-default group">
                <div class="flex items-center gap-3">
                  <div class="w-2 h-2 bg-blue-500 rounded-full shadow-[0_0_8px_rgba(59,130,246,0.6)] group-hover:animate-pulse" />
                  <span class="text-[10px] font-black text-text-primary uppercase tracking-widest leading-none">{t('amlogic.stage_1_title')}</span>
                </div>
                <p class="text-[9px] text-text-muted pl-5 leading-relaxed font-bold">
                  {t('amlogic.stage_1_desc')}
                </p>
              </div>
              <div class="space-y-2 opacity-60 hover:opacity-100 transition-opacity cursor-default group">
                <div class="flex items-center gap-3">
                  <div class="w-2 h-2 bg-purple-500 rounded-full shadow-[0_0_8px_rgba(168,85,247,0.6)] group-hover:animate-pulse" />
                  <span class="text-[10px] font-black text-text-primary uppercase tracking-widest leading-none">{t('amlogic.stage_2_title')}</span>
                </div>
                <p class="text-[9px] text-text-muted pl-5 leading-relaxed font-bold">
                  {t('amlogic.stage_2_desc')}
                </p>
              </div>
              <div class="space-y-2 opacity-60 hover:opacity-100 transition-opacity cursor-default group">
                <div class="flex items-center gap-3">
                  <div class="w-2 h-2 bg-emerald-500 rounded-full shadow-[0_0_8px_rgba(16,185,129,0.6)] group-hover:animate-pulse" />
                  <span class="text-[10px] font-black text-text-primary uppercase tracking-widest leading-none">{t('amlogic.stage_3_title')}</span>
                </div>
                <p class="text-[9px] text-text-muted pl-5 leading-relaxed font-bold">
                  {t('amlogic.stage_3_desc')}
                </p>
              </div>
              <div class="pt-3 border-t border-border-subtle opacity-30 mt-2">
                <p class="text-[8px] text-text-muted uppercase font-black tracking-tighter">Trace: Derived from 'usb_flow/key_flow.aml' reverse engineering logs.</p>
              </div>
            </div>
          </Card>

          <Show when={recommendations().length > 0}>
            <Card glow="indigo" title={t('amlogic.card_artifacts_title')} subtitle={t('amlogic.card_artifacts_subtitle')} class="border-border-subtle bg-accent/5">
              <div class="space-y-4 py-1">
                <For each={recommendations().slice(0, 2)}>
                  {(rec) => (
                    <div class="group block space-y-2 p-4 bg-sidebar/50 border border-border-subtle hover:border-accent/30 transition-all rounded-sm cursor-pointer">
                      <div class="flex justify-between items-center leading-none">
                        <span class="text-[10px] font-black text-text-primary uppercase tracking-tight">{rec.name}</span>
                        <Badge variant="secondary" class="text-[8px] font-black rounded-none px-2">{rec.version}</Badge>
                      </div>
                      <p class="text-[9px] text-text-muted leading-tight font-bold opacity-60 truncate">{rec.notes}</p>
                    </div>
                  )}
                </For>
                <div class="text-center pt-1">
                  <span class="text-[8px] text-text-muted uppercase tracking-[0.3em] font-black opacity-30">Ref: firmware_sources.md v2.1</span>
                </div>
              </div>
            </Card>
          </Show>
        </div>
      </div>
    </div>
  );
}
