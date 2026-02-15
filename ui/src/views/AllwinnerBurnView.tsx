import { Component, createSignal, Show, For, createEffect } from 'solid-js';
import { Card } from '../components/ui/Card';
import { Button } from '../components/ui/Button';
import { Badge } from '../components/ui/Badge';
import { Collapsible } from '../components/ui/Collapsible';
import { open } from '@tauri-apps/plugin-dialog';
import { tauriApi } from '../api/tauri';
import { getAppErrorMessage } from '../errorCodes';
import { globalStore, setGlobalStore } from '../store';
import { cn } from '../lib/utils';
import { useApp } from '../context/AppContext';

const AllwinnerBurnView: Component = () => {
    const { t } = useApp();
    const [status, setStatus] = createSignal('Ready');
    const [imagePath, setImagePath] = createSignal('');
    const [isFlashing, setIsFlashing] = createSignal(false);
    const [isDetecting, setIsDetecting] = createSignal(false);
    const [logs, setLogs] = createSignal<{ time: string, msg: string, level: string }[]>([]);

    const addLog = (msg: string, level = 'info') => {
        setLogs(prev => [...prev.slice(-49), { time: new Date().toLocaleTimeString(), msg, level }]);
    };

    createEffect(() => {
        if (!globalStore.lastDetected && !isDetecting()) {
            detectDevice();
        }
    });

    const detectDevice = async () => {
        setIsDetecting(true);
        try {
            setStatus('Scanning for FEL devices...');
            addLog('Polling USB Bus for Allwinner FEL markers...', 'info');
            const info = await tauriApi.allwinnerDetect();
            setGlobalStore('lastDetected', {
                vendorId: 0x1f3a,
                productId: 0xefe8,
                vendorName: 'Allwinner',
                model: info.socName || 'Unknown Allwinner',
                mode: 'FEL',
                busNumber: 0,
                deviceAddress: 0,
            } as any);
            setStatus(`FEL: ${info.socName || 'AW-CHIP'}`);
            addLog(`Handshake Successful: ${info.socName} identified in FEL mode.`, 'success');
        } catch (err: any) {
            setStatus('Discovery Failed');
            addLog(`Discovery Error: ${getAppErrorMessage(err)}`, 'error');
        } finally {
            setIsDetecting(false);
        }
    };

    const selectImage = async () => {
        const selected = await open({
            filters: [{ name: 'Allwinner Image', extensions: ['img'] }]
        });
        if (selected && typeof selected === 'string') {
            setImagePath(selected);
            addLog(`Image Localized: ${selected}`);
            try {
                const header = await tauriApi.allwinnerParseImage(selected);
                addLog(`Header Match: Platform ${header.platform}, Revision ${header.version}`, 'success');
            } catch (e) {
                addLog(`Binary Trace Warning: Could not parse Allwinner header.`, 'warn');
            }
        }
    };

    const startFlash = async () => {
        if (!imagePath() || !globalStore.lastDetected) return;

        setIsFlashing(true);
        addLog('Initiating PhoenixSuite Compatible Sequence...', 'info');

        try {
            await tauriApi.allwinnerFlashImage(imagePath());
            addLog('Deployment Successful: NAND/EMMC written.', 'success');
            setStatus('Flash Successful');
        } catch (e: any) {
            addLog(`System Error: ${getAppErrorMessage(e)}`, 'error');
            setStatus('Critical Error');
        } finally {
            setIsFlashing(false);
        }
    };

    const isAwDevice = () => globalStore.lastDetected?.vendorId === 0x1f3a || globalStore.lastDetected?.productId === 0xefe8;

    return (
        <div class="h-full flex flex-col gap-6 font-mono">
            <header class="flex flex-col gap-1 text-left">
                <div class="flex items-center gap-3">
                    <div class="w-2 h-2 rounded-full bg-accent animate-pulse shadow-[0_0_8px_rgba(var(--accent-rgb),0.4)]" />
                    <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase">{t('allwinner.title')}</h2>
                    <Badge variant={isAwDevice() ? 'success' : 'warning'} class="rounded-none px-4 font-black ml-2">
                        {isAwDevice() ? t('allwinner.badge_active') : t('allwinner.badge_disconnected')}
                    </Badge>
                </div>
                <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">{t('allwinner.subtitle')}</p>
            </header>

            <div class="grid lg:grid-cols-12 gap-6 flex-1 min-h-0">
                {/* Control Column */}
                <div class="lg:col-span-8 flex flex-col gap-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
                    <Card glow="rose" title={t('allwinner.card_deployment_title')} subtitle={t('allwinner.card_deployment_subtitle')} class="border-border-subtle">
                        <div class="space-y-8">
                            <div class="space-y-3">
                                <label class="text-[10px] text-text-muted uppercase tracking-[0.3em] font-black opacity-60">{t('allwinner.label_firmware')}</label>
                                <div class="flex gap-3">
                                    <div class="flex-1 bg-sidebar/40 border border-border-subtle rounded-none px-5 py-3 text-xs text-text-secondary truncate flex items-center font-bold tracking-tight">
                                        {imagePath() || t('allwinner.placeholder_image')}
                                    </div>
                                    <Button onClick={selectImage} class="font-black text-[10px] h-11 px-8 bg-sidebar/20 hover:bg-sidebar/40 border-border-subtle rounded-none uppercase tracking-widest text-text-muted hover:text-text-primary transition-all shadow-none">{t('common.explore')}</Button>
                                </div>
                            </div>

                            <div class="flex justify-between items-center pt-8 border-t border-border-subtle">
                                <div class="flex flex-col gap-2">
                                    <span class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] opacity-40">{t('allwinner.status_protocol')}</span>
                                    <span class="text-[11px] text-accent font-black uppercase tracking-widest">{isFlashing() ? t('allwinner.status_writing') : t('allwinner.status_idle')}</span>
                                </div>
                                <Button
                                    onClick={startFlash}
                                    disabled={!imagePath() || !isAwDevice() || isFlashing()}
                                    class="bg-accent hover:bg-accent/90 border-none px-14 h-14 font-black rounded-none tracking-widest shadow-[0_5px_20px_rgba(var(--accent-rgb),0.3)] text-white"
                                >
                                    {t('allwinner.btn_flash')}
                                </Button>
                            </div>
                        </div>
                    </Card>

                    <Collapsible title={t('allwinner.card_discovery_title')} subtitle={t('allwinner.card_discovery_subtitle')} class="border-border-subtle bg-sidebar/30 rounded-sm">
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-8 p-4 text-[10px] font-bold text-text-muted lowercase leading-relaxed">
                            <div class="space-y-3">
                                <span class="text-accent uppercase font-black text-[9px] tracking-[0.3em] leading-none opacity-60">{t('allwinner.step_1_title')}</span>
                                <p class="italic border-l border-border-subtle pl-4 py-1">{t('allwinner.step_1_desc')}</p>
                            </div>
                            <div class="space-y-3">
                                <span class="text-accent uppercase font-black text-[9px] tracking-[0.3em] leading-none opacity-60">{t('allwinner.step_2_title')}</span>
                                <p class="italic border-l border-border-subtle pl-4 py-1">{t('allwinner.step_2_desc')}</p>
                            </div>
                            <div class="space-y-3">
                                <span class="text-accent uppercase font-black text-[9px] tracking-[0.3em] leading-none opacity-60">{t('allwinner.step_3_title')}</span>
                                <p class="italic border-l border-border-subtle pl-4 py-1">{t('allwinner.step_3_desc')}</p>
                            </div>
                        </div>
                    </Collapsible>
                </div>

                {/* Status Column */}
                <div class="lg:col-span-4 flex flex-col gap-6 overflow-hidden pb-4">
                    <Card glow="rose" title={t('allwinner.card_telemetry_title')} subtitle={t('allwinner.card_telemetry_subtitle')} class="border-border-subtle">
                        <div class="space-y-8">
                            <div class="flex flex-col items-center py-10 bg-black/20 border border-border-subtle relative overflow-hidden rounded-sm">
                                <Badge
                                    variant={isAwDevice() ? 'success' : 'default'}
                                    class="text-[11px] px-8 py-2 font-black tracking-[0.4em] rounded-none shadow-lg"
                                >
                                    {isAwDevice() ? t('allwinner.badge_active') : t('allwinner.badge_disconnected')}
                                </Badge>
                                <span class="text-[9px] font-black font-mono text-text-muted uppercase tracking-[0.3em] mt-4 opacity-40">{isAwDevice() ? t('allwinner.status_link_ok') : t('allwinner.status_link_wait')}</span>
                                <div class="absolute inset-x-0 bottom-0 h-1 bg-accent/20 animate-pulse" />
                            </div>

                            <div class="space-y-4 px-1 text-[11px] font-black uppercase tracking-widest">
                                <div class="flex justify-between border-b border-border-subtle pb-3">
                                    <span class="text-text-muted opacity-60">{t('allwinner.label_silicon')}</span>
                                    <span class="text-text-primary underline decoration-text-primary/10 underline-offset-4">{isAwDevice() ? globalStore.lastDetected?.model : '-----'}</span>
                                </div>
                                <div class="flex justify-between border-b border-border-subtle pb-3">
                                    <span class="text-text-muted opacity-60">{t('allwinner.label_interface')}</span>
                                    <span class="text-text-primary">USB-FEL</span>
                                </div>
                                <Button
                                    class="w-full h-11 font-black uppercase border-border-subtle bg-sidebar/20 hover:bg-sidebar/40 rounded-none text-[10px] tracking-widest text-text-muted hover:text-text-primary transition-all shadow-none mt-2"
                                    onClick={detectDevice}
                                    isLoading={isDetecting()}
                                >
                                    {t('allwinner.btn_rescan')}
                                </Button>
                            </div>
                        </div>
                    </Card>

                    <div class="flex-1 flex flex-col bg-sidebar/30 border border-border-subtle rounded-none overflow-hidden min-h-0">
                        <div class="px-5 py-3 bg-sidebar/50 border-b border-border-subtle flex justify-between items-center shrink-0">
                            <span class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] opacity-60">{t('allwinner.log_title')}</span>
                            <button class="text-[9px] text-text-muted hover:text-rose-500 font-black tracking-widest transition-colors opacity-40 hover:opacity-100" onClick={() => setLogs([])}>{t('allwinner.log_reset')}</button>
                        </div>
                        <div class="flex-1 overflow-y-auto p-6 space-y-2 font-mono text-[10px] custom-scrollbar selection:bg-accent/20 font-bold">
                            <For each={logs()} fallback={<div class="text-text-muted opacity-20 font-black tracking-widest uppercase py-10 text-center">{t('allwinner.log_placeholder')}</div>}>
                                {log => (
                                    <div class="flex gap-4 border-l-2 border-border-subtle pl-5 py-1 hover:bg-white/[0.01] transition-colors leading-relaxed group">
                                        <span class="text-text-muted shrink-0 text-[10px] opacity-40 group-hover:opacity-100 transition-opacity">[{log.time}]</span>
                                        <div class="flex gap-2 items-center">
                                            <span class={cn("font-black tracking-tighter shrink-0", log.level === 'error' ? 'text-rose-500' : log.level === 'success' ? 'text-accent' : 'text-text-muted opacity-60')}>{" >> "}</span>
                                            <span class={cn("italic tracking-tight", log.level === 'error' ? 'text-rose-500' : log.level === 'success' ? 'text-accent' : 'text-text-muted opacity-80')}>{log.msg}</span>
                                        </div>
                                    </div>
                                )}
                            </For>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default AllwinnerBurnView;
