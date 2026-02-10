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

const AllwinnerBurnView: Component = () => {
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
                    <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase italic">Allwinner FEL Tool</h2>
                    <Badge variant={isAwDevice() ? 'success' : 'warning'} class="rounded-none px-4 font-black italic ml-2">
                        {isAwDevice() ? 'FEL ACTIVE' : 'NO HANDSHAKE'}
                    </Badge>
                </div>
                <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">Protocol: Allwinner FEL (VID 1F3A) | PhoenixSuit / LiveSuit Engine</p>
            </header>

            <div class="grid lg:grid-cols-12 gap-6 flex-1 min-h-0">
                {/* Control Column */}
                <div class="lg:col-span-8 flex flex-col gap-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
                    <Card glow="rose" title="Deployment Engine" subtitle="LiveSuit-compatible firmware flashing" class="border-border-subtle">
                        <div class="space-y-8">
                            <div class="space-y-3">
                                <label class="text-[10px] text-text-muted uppercase tracking-[0.3em] font-black opacity-60 italic">Firmware Container (.img)</label>
                                <div class="flex gap-3">
                                    <div class="flex-1 bg-sidebar/40 border border-border-subtle rounded-none px-5 py-3 text-xs text-text-secondary truncate flex items-center italic font-bold tracking-tight">
                                        {imagePath() || 'AWAITING IMAGE SELECTION...'}
                                    </div>
                                    <Button onClick={selectImage} class="font-black text-[10px] h-11 px-8 bg-sidebar/20 hover:bg-sidebar/40 border-border-subtle rounded-none uppercase italic tracking-widest text-text-muted hover:text-text-primary transition-all shadow-none">EXPLORE</Button>
                                </div>
                            </div>

                            <div class="flex justify-between items-center pt-8 border-t border-border-subtle">
                                <div class="flex flex-col gap-2 italic">
                                    <span class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] opacity-40">Protocol State</span>
                                    <span class="text-[11px] text-accent font-black uppercase italic tracking-widest">{isFlashing() ? 'Writing Sector Grids...' : 'System Idle'}</span>
                                </div>
                                <Button
                                    onClick={startFlash}
                                    disabled={!imagePath() || !isAwDevice() || isFlashing()}
                                    class="bg-accent hover:bg-accent/90 border-none px-14 h-14 font-black italic rounded-none tracking-widest shadow-[0_5px_20px_rgba(var(--accent-rgb),0.3)] text-white"
                                >
                                    IGNITE FLASH
                                </Button>
                            </div>
                        </div>
                    </Card>

                    <Collapsible title="FEL Discovery Matrix" subtitle="Hardware-level state induction" class="border-border-subtle bg-sidebar/30 rounded-sm italic">
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-8 p-4 text-[10px] font-bold text-text-muted lowercase leading-relaxed">
                            <div class="space-y-3">
                                <span class="text-accent uppercase font-black text-[9px] tracking-[0.3em] italic leading-none opacity-60">01: FEL Marker</span>
                                <p class="italic border-l border-border-subtle pl-4 py-1">Locate recovery button (inside AV jack or separate 'U-Boot' pin). hold while cold booting via OTG port.</p>
                            </div>
                            <div class="space-y-3">
                                <span class="text-accent uppercase font-black text-[9px] tracking-[0.3em] italic leading-none opacity-60">02: Hub Polling</span>
                                <p class="italic border-l border-border-subtle pl-4 py-1">Device will enumerate as <code class="text-text-primary bg-accent/5 px-2 rounded-sm font-black">1f3a:efe8</code>. standard for H3, H5, H6, and A64 architectures.</p>
                            </div>
                            <div class="space-y-3">
                                <span class="text-accent uppercase font-black text-[9px] tracking-[0.3em] italic leading-none opacity-60">03: Phoenix Bridge</span>
                                <p class="italic border-l border-border-subtle pl-4 py-1">Once identified, Phoenix maps the SoC registers and initializes the SDRAM controller for flashing.</p>
                            </div>
                        </div>
                    </Collapsible>
                </div>

                {/* Status Column */}
                <div class="lg:col-span-4 flex flex-col gap-6 overflow-hidden pb-4">
                    <Card glow="rose" title="Node Telemetry" subtitle="Active SoC probing" class="border-border-subtle">
                        <div class="space-y-8">
                            <div class="flex flex-col items-center py-10 bg-black/20 border border-border-subtle relative overflow-hidden rounded-sm italic">
                                <Badge
                                    variant={isAwDevice() ? 'success' : 'default'}
                                    class="text-[11px] px-8 py-2 font-black italic tracking-[0.4em] rounded-none shadow-lg"
                                >
                                    {isAwDevice() ? 'FEL ACTIVE' : 'DISCONNECTED'}
                                </Badge>
                                <span class="text-[9px] font-black font-mono text-text-muted uppercase tracking-[0.3em] mt-4 italic opacity-40">{isAwDevice() ? 'Link Established' : 'Awaiting Signal'}</span>
                                <div class="absolute inset-x-0 bottom-0 h-1 bg-accent/20 animate-pulse" />
                            </div>

                            <div class="space-y-4 px-1 italic text-[11px] font-black uppercase tracking-widest">
                                <div class="flex justify-between border-b border-border-subtle pb-3">
                                    <span class="text-text-muted opacity-60">Silicon Node</span>
                                    <span class="text-text-primary underline decoration-text-primary/10 underline-offset-4">{isAwDevice() ? globalStore.lastDetected?.model : '-----'}</span>
                                </div>
                                <div class="flex justify-between border-b border-border-subtle pb-3">
                                    <span class="text-text-muted opacity-60">Interface</span>
                                    <span class="text-text-primary">USB-FEL</span>
                                </div>
                                <Button
                                    class="w-full h-11 font-black uppercase border-border-subtle bg-sidebar/20 hover:bg-sidebar/40 rounded-none text-[10px] italic tracking-widest text-text-muted hover:text-text-primary transition-all shadow-none mt-2"
                                    onClick={detectDevice}
                                    isLoading={isDetecting()}
                                >
                                    RE-SCAN FEL BUS
                                </Button>
                            </div>
                        </div>
                    </Card>

                    <div class="flex-1 flex flex-col bg-sidebar/30 border border-border-subtle rounded-none overflow-hidden min-h-0 italic">
                        <div class="px-5 py-3 bg-sidebar/50 border-b border-border-subtle flex justify-between items-center shrink-0 italic">
                            <span class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] italic opacity-60">Protocol Trace</span>
                            <button class="text-[9px] text-text-muted hover:text-rose-500 font-black tracking-widest transition-colors opacity-40 hover:opacity-100 italic" onClick={() => setLogs([])}>RESET</button>
                        </div>
                        <div class="flex-1 overflow-y-auto p-6 space-y-2 font-mono text-[10px] custom-scrollbar selection:bg-accent/20 italic font-bold">
                            <For each={logs()} fallback={<div class="text-text-muted italic opacity-20 font-black tracking-widest uppercase py-10 text-center">Monitoring FEL traffic...</div>}>
                                {log => (
                                    <div class="flex gap-4 border-l-2 border-border-subtle pl-5 py-1 hover:bg-white/[0.01] transition-colors leading-relaxed group">
                                        <span class="text-text-muted shrink-0 text-[10px] italic opacity-40 group-hover:opacity-100 transition-opacity">[{log.time}]</span>
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
