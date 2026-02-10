import { Component, createSignal, For, Show, createEffect, Switch, Match } from 'solid-js';
import { tauriApi } from '../api/tauri';
import { getAppErrorMessage } from '../errorCodes';
import { Card } from '../components/ui/Card';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';
import { Badge } from '../components/ui/Badge';
import { Collapsible } from '../components/ui/Collapsible';
import { globalStore, setGlobalStore } from '../store';
import { cn } from '../lib/utils';

interface LogEntry { time: string; msg: string; level: 'info' | 'warn' | 'error' | 'success'; }

const RockchipFlashView: Component = () => {
    const [imagePath, setImagePath] = createSignal('');
    const [imageInfo, setImageInfo] = createSignal<any>(null);
    const [paramInfo, setParamInfo] = createSignal<any>(null);
    const [logs, setLogs] = createSignal<LogEntry[]>([]);
    const [isDetecting, setIsDetecting] = createSignal(false);
    const [isParsing, setIsParsing] = createSignal(false);
    const [isExtracting, setIsExtracting] = createSignal(false);
    const [activeTab, setActiveTab] = createSignal<'flash' | 'analyze' | 'info'>('flash');

    const addLog = (msg: string, level: LogEntry['level'] = 'info') => {
        const time = new Date().toLocaleTimeString();
        setLogs(prev => {
            const next = [...prev, { time, msg, level }];
            return next.length > 100 ? next.slice(-100) : next;
        });
    };

    createEffect(() => {
        if (!globalStore.lastDetected) {
            handleDetect();
        }
    });

    const handleDetect = async () => {
        setIsDetecting(true);
        addLog('Polling Serial Bus (VID 0x2207)...', 'info');
        try {
            const info = await tauriApi.rockchipDetect();
            setGlobalStore('lastDetected', {
                vendorId: 0x2207,
                productId: 0x0,
                vendorName: 'Rockchip',
                model: info.chipId,
                mode: info.isMaskrom ? 'Maskrom' : 'Loader',
                busNumber: 0,
                deviceAddress: 0,
                chipId: info.chipId,
            } as any);
            addLog(`Node Discovery: ${info.chipId} via ${info.flashType} interface.`, 'success');
        } catch (e) {
            addLog(`Discovery Aborted: ${getAppErrorMessage(e)}`, 'error');
        } finally {
            setIsDetecting(false);
        }
    };

    const handleParseImage = async () => {
        if (!imagePath()) return;
        setIsParsing(true);
        addLog(`Analyzing Firmware Geometry: ${imagePath()}`, 'info');
        try {
            const path = imagePath().toLowerCase();
            if (path.endsWith('.txt')) {
                const f = await tauriApi.rockchipParseParameter(imagePath());
                setParamInfo(f);
                addLog('Parameter Mappings Resolved.', 'success');
            } else {
                const info = await tauriApi.rockchipParseImage(imagePath());
                setImageInfo(info);
                addLog(`Binary Header Valid: ${info.magic} with ${info.entries?.length ?? 0} partition segments`, 'success');
            }
        } catch (e) {
            addLog(`Structural Integrity Error: ${getAppErrorMessage(e)}`, 'error');
        } finally {
            setIsParsing(false);
        }
    };

    const handleExtract = async () => {
        if (!imagePath()) return;
        setIsExtracting(true);
        addLog(`Decompressing Image Clusters: ${imagePath()}`, 'info');
        try {
            const outDir = imagePath() + '_unpacked';
            await tauriApi.rockchipExtractImage(imagePath(), outDir);
            addLog(`Workspace Created: ${outDir}`, 'success');
        } catch (e) {
            addLog(`Extraction Collision: ${getAppErrorMessage(e)}`, 'error');
        } finally {
            setIsExtracting(false);
        }
    };

    const logColor = (l: LogEntry['level']) =>
        l === 'error' ? 'text-rose-500' : l === 'warn' ? 'text-amber-500' : l === 'success' ? 'text-accent' : 'text-text-muted';

    const isRkDevice = () => globalStore.lastDetected?.vendorId === 0x2207;

    return (
        <div class="h-full flex flex-col gap-6 font-mono">
            <header class="flex flex-col gap-1 text-left">
                <div class="flex items-center gap-3">
                    <div class="w-2 h-2 rounded-full bg-accent animate-pulse shadow-[0_0_8px_rgba(var(--accent-rgb),0.4)]" />
                    <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase">RockUSB Flash Suite</h2>
                    <Badge variant={isRkDevice() ? 'success' : 'warning'} class="rounded-none px-4 font-black ml-2">
                        {isRkDevice() ? globalStore.lastDetected?.mode.toUpperCase() : 'IDLE'}
                    </Badge>
                </div>
                <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">Protocol: RockUSB v3.0 (VID 2207) | Bulk Transfer Interface</p>
            </header>

            <div class="grid lg:grid-cols-12 gap-6 flex-1 min-h-0">
                {/* Control Column */}
                <div class="lg:col-span-8 flex flex-col gap-6 overflow-hidden pb-4">
                    <nav class="flex gap-1 p-1 bg-sidebar/30 border border-border-subtle rounded-none shrink-0">
                        <For each={['flash', 'analyze', 'info']}>{(tab) => (
                            <button
                                onClick={() => setActiveTab(tab as any)}
                                class={cn(
                                    "flex-1 py-2.5 text-[10px] font-black uppercase tracking-[0.2em] transition-all duration-300",
                                    activeTab() === tab
                                        ? "bg-accent text-white shadow-lg"
                                        : "text-text-muted hover:text-text-primary hover:bg-white/[0.02]"
                                )}
                            >
                                {tab}
                            </button>
                        )}</For>
                    </nav>

                    <main class="flex-1 overflow-y-auto custom-scrollbar space-y-6 pr-1">
                        <Switch>
                            <Match when={activeTab() === 'flash'}>
                                <Card glow="teal" title="Deployment Engine" subtitle="Burn firmware images or extract partitions" class="border-border-subtle">
                                    <div class="space-y-6">
                                        <div class="space-y-4">
                                            <div class="space-y-3">
                                                <label class="text-[10px] text-text-muted uppercase tracking-[0.3em] font-black opacity-60">Target Image Cluster</label>
                                                <div class="flex gap-3">
                                                    <Input
                                                        placeholder="C:\RESOURCES\UPDATE.IMG"
                                                        value={imagePath()}
                                                        onInput={e => setImagePath(e.currentTarget.value)}
                                                        class="flex-1 bg-sidebar/40 border-border-subtle rounded-none h-11 text-xs font-bold tracking-tight text-text-secondary"
                                                    />
                                                    <Button
                                                        onClick={handleParseImage}
                                                        isLoading={isParsing()}
                                                        class="h-11 px-8 rounded-none font-black text-[10px] border-border-subtle bg-sidebar/20 hover:bg-sidebar/40 text-text-muted hover:text-text-primary tracking-widest shadow-none"
                                                    >
                                                        ANALYZE BINARY
                                                    </Button>
                                                </div>
                                            </div>

                                            <div class="grid grid-cols-2 gap-4">
                                                <Button
                                                    onClick={handleExtract}
                                                    isLoading={isExtracting()}
                                                    class="h-11 rounded-none font-black text-[10px] border-border-subtle bg-sidebar/20 hover:bg-sidebar/40 text-text-muted hover:text-text-primary tracking-widest shadow-none"
                                                >
                                                    CREATE WORKSPACE (UNPACK)
                                                </Button>
                                                <Button
                                                    class="h-11 rounded-none font-black text-[10px] bg-rose-600 hover:bg-rose-500 border-none tracking-widest text-white shadow-[0_5px_15px_rgba(244,63,94,0.2)]"
                                                    disabled={!isRkDevice() || !imageInfo()}
                                                >
                                                    EXECUTE FLASH SEQUENCE
                                                </Button>
                                            </div>
                                        </div>

                                        <Show when={imageInfo()}>
                                            <div class="mt-4 border border-border-subtle bg-black/20 p-6 space-y-5 rounded-sm">
                                                <div class="flex justify-between items-center pb-3 border-b border-border-subtle">
                                                    <span class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] opacity-60">Image Structure</span>
                                                    <Badge variant="secondary" class="rounded-none px-4 font-black">{imageInfo().magic}</Badge>
                                                </div>
                                                <div class="max-h-72 overflow-y-auto custom-scrollbar space-y-2 pr-2">
                                                    <For each={imageInfo().entries}>{(entry) => (
                                                        <div class="flex justify-between text-[10px] p-3 hover:bg-white/[0.02] transition-colors border-b border-border-subtle last:border-0 group font-bold tracking-tight">
                                                            <span class="text-text-muted group-hover:text-accent transition-colors uppercase">{entry.name}</span>
                                                            <span class="text-text-secondary opacity-60 group-hover:opacity-100 transition-opacity">{(entry.fileSize / 1024 / 1024).toFixed(2)} MB</span>
                                                        </div>
                                                    )}</For>
                                                </div>
                                            </div>
                                        </Show>
                                    </div>
                                </Card>
                            </Match>

                            <Match when={activeTab() === 'analyze'}>
                                <Card glow="indigo" title="Forensic Analysis" subtitle="Parameter mapping & MTD geometry" class="border-border-subtle">
                                    <div class="space-y-6">
                                        <p class="text-[10px] text-text-muted uppercase tracking-[0.2em] leading-relaxed font-bold opacity-60">
                                            Analyzing <code class="text-accent bg-accent/5 px-2 py-0.5 rounded-sm">parameter.txt</code> logic. Resolving GPT offsets for mtdparts orchestration.
                                        </p>
                                        <div class="space-y-3">
                                            <label class="text-[10px] text-text-muted uppercase tracking-[0.3em] font-black opacity-60">Parameter File Path</label>
                                            <Input placeholder="C:\FIRMWARE\PARAMETER.TXT" value={imagePath()} onInput={e => setImagePath(e.currentTarget.value)} class="bg-sidebar/40 border-border-subtle rounded-none h-11 text-xs font-bold tracking-tight text-text-secondary" />
                                        </div>
                                        <Button onClick={handleParseImage} isLoading={isParsing()} class="w-full h-11 border-border-subtle bg-sidebar/20 hover:bg-sidebar/40 rounded-none font-black text-[10px] tracking-widest text-text-muted hover:text-text-primary transition-all shadow-none">
                                            RESOLVE MTD MAPPINGS
                                        </Button>

                                        <Show when={paramInfo()}>
                                            <div class="p-6 bg-black/20 border border-border-subtle space-y-5 rounded-sm">
                                                <h4 class="text-[10px] font-black text-accent uppercase tracking-[0.3em] leading-none">MTD Partition Map</h4>
                                                <div class="space-y-2 max-h-96 overflow-y-auto custom-scrollbar pr-2">
                                                    <For each={paramInfo().partitions}>{(p) => (
                                                        <div class="flex items-center justify-between text-[10px] p-3 hover:bg-accent/5 transition-colors border-b border-border-subtle font-bold tracking-tight">
                                                            <span class="text-text-primary font-black uppercase tracking-widest">{p.name}</span>
                                                            <div class="flex gap-6 text-text-muted opacity-60">
                                                                <span class="text-[9px]">LEN: 0x{p.size.toString(16).toUpperCase()}</span>
                                                                <span class="text-accent font-black tracking-tight underline decoration-accent/20 underline-offset-4">@ 0x{p.offset.toString(16).toUpperCase()}</span>
                                                            </div>
                                                        </div>
                                                    )}</For>
                                                </div>
                                            </div>
                                        </Show>
                                    </div>
                                </Card>
                            </Match>

                            <Match when={activeTab() === 'info'}>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                    <Card glow="teal" title="Protocol Reference" subtitle="RockUSB Transaction Layer" class="border-border-subtle bg-sidebar/20">
                                        <div class="space-y-5 py-1 text-[10px] font-black text-text-muted uppercase tracking-widest">
                                            <div class="flex justify-between border-b border-border-subtle pb-3">
                                                <span class="opacity-60">Interface</span>
                                                <span class="text-text-primary">BULK (CBW/CSW)</span>
                                            </div>
                                            <div class="flex justify-between border-b border-border-subtle pb-3">
                                                <span class="opacity-60">Endpoints</span>
                                                <span class="text-accent">OUT 0x02 | IN 0x81</span>
                                            </div>
                                            <div class="flex justify-between border-b border-border-subtle pb-3">
                                                <span class="opacity-60">Endianness</span>
                                                <span class="text-text-secondary">LITTLE ENDIAN</span>
                                            </div>
                                            <div class="flex justify-between pt-1">
                                                <span class="opacity-60">Packet CRC</span>
                                                <span class="text-accent underline decoration-accent/20 underline-offset-4">CRC-16 CCITT</span>
                                            </div>
                                        </div>
                                    </Card>
                                    <Card glow="amber" title="Loader Chain" subtitle="Boot Stages Management" class="border-border-subtle bg-sidebar/20">
                                        <div class="space-y-5 text-[10px] text-text-muted leading-relaxed font-bold py-1">
                                            <div class="p-4 border-l-2 border-accent bg-accent/5 rounded-sm rounded-l-none">
                                                <span class="text-accent font-black block mb-2 uppercase tracking-[0.2em]">STAGE 471 (miniloader)</span>
                                                <p class="opacity-70 leading-relaxed uppercase">SRAM stage handshake. Performs DRAM training and maps the flash controller.</p>
                                            </div>
                                            <div class="p-4 border-l-2 border-border-subtle bg-sidebar/5 rounded-sm rounded-l-none opacity-60 hover:opacity-100 transition-opacity">
                                                <span class="text-text-primary font-black block mb-2 uppercase tracking-[0.2em]">STAGE 472 (loader)</span>
                                                <p class="leading-relaxed uppercase">Main execution stage. Enables bulk flashing and interactive commands via RockUSB.</p>
                                            </div>
                                        </div>
                                    </Card>
                                </div>
                            </Match>
                        </Switch>

                        <Collapsible title="Recovery Mode Matrix" subtitle="Hardware-level state entry" class="border-accent/20 bg-accent/5 rounded-sm">
                            <div class="space-y-5 p-4 text-[10px] text-text-muted leading-relaxed font-bold">
                                <p class="border-l-2 border-accent/20 pl-4 py-1">1. <span class="text-accent font-black uppercase tracking-widest mr-2">Maskrom Entry</span>: Short eMMC <b class="text-text-primary">CLK</b> to <b class="text-text-primary">GND</b> (Tweezers method) while powering via OTG port. Required for blank or bricked devices.</p>
                                <p class="border-l-2 border-accent/20 pl-4 py-1">2. <span class="text-accent font-black uppercase tracking-widest mr-2">Loader Entry</span>: Hold <b class="text-text-primary">RECOVERY</b> (or ADKey) during power-on. Device enumerates as a RockUSB class device.</p>
                                <p class="border-l-2 border-accent/20 pl-4 py-1">3. <span class="text-accent font-black uppercase tracking-widest mr-2">Fingerprint</span>: Verify <b class="text-text-primary">VID 2207</b>. Masks IDs 0x00xx through 0x3xxx based on chip generation.</p>
                            </div>
                        </Collapsible>
                    </main>
                </div>

                {/* Status Column */}
                <div class="lg:col-span-4 flex flex-col gap-6 overflow-hidden pb-4">
                    <Card glow="teal" title="Hardware State" subtitle="Active Node Telemetry" class="border-border-subtle">
                        <div class="space-y-8">
                            <div class="flex flex-col items-center gap-3 py-10 bg-black/20 border border-border-subtle relative overflow-hidden group rounded-sm">
                                <Badge
                                    variant={isRkDevice() ? (globalStore.lastDetected?.mode === 'Maskrom' ? 'error' : 'secondary') : 'default'}
                                    class="text-[11px] px-8 py-2 font-black tracking-[0.4em] rounded-none shadow-lg"
                                >
                                    {isRkDevice() ? globalStore.lastDetected?.mode.toUpperCase() : 'WAITING'}
                                </Badge>
                                <span class="text-[9px] font-black text-text-muted uppercase tracking-[0.3em] mt-3 opacity-40">{isRkDevice() ? 'System Linked' : 'Polling Bus...'}</span>
                                <div class="absolute inset-x-0 bottom-0 h-1 bg-accent/20 animate-pulse" />
                            </div>

                            <div class="space-y-4 px-1 text-[11px] font-black uppercase tracking-widest">
                                <div class="flex justify-between border-b border-border-subtle pb-3">
                                    <span class="text-text-muted opacity-60">Detected Chip</span>
                                    <span class="text-text-primary underline decoration-text-primary/10 underline-offset-4">{isRkDevice() ? globalStore.lastDetected?.model : '-----'}</span>
                                </div>
                                <div class="flex justify-between border-b border-border-subtle pb-3">
                                    <span class="text-text-muted opacity-60">Node Bridge</span>
                                    <span class="text-text-primary">{isRkDevice() ? `USB 2.0 (OTG)` : '-----'}</span>
                                </div>
                                <Button
                                    class="w-full text-[10px] font-black uppercase h-11 rounded-none border-border-subtle bg-sidebar/20 hover:bg-sidebar/40 text-text-muted hover:text-text-primary tracking-widest shadow-none"
                                    onClick={handleDetect}
                                    isLoading={isDetecting()}
                                >
                                    PROBE SERIAL BUS
                                </Button>
                            </div>
                        </div>
                    </Card>

                    <div class="flex-1 flex flex-col bg-sidebar/30 border border-border-subtle rounded-none overflow-hidden min-h-0">
                        <div class="px-5 py-3 bg-sidebar/50 border-b border-border-subtle flex justify-between items-center shrink-0">
                            <span class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] opacity-60">Protocol Trace</span>
                            <button class="text-[9px] text-text-muted hover:text-rose-500 uppercase font-black tracking-widest transition-colors opacity-40 hover:opacity-100" onClick={() => setLogs([])}>Reset</button>
                        </div>
                        <div class="flex-1 overflow-y-auto p-6 space-y-2 font-mono text-[10px] custom-scrollbar selection:bg-accent/20">
                            <For each={logs()} fallback={<div class="text-text-muted opacity-20 font-black tracking-widest uppercase py-10 text-center">Monitoring RockUSB traffic...</div>}>
                                {log => (
                                    <div class="flex gap-4 border-l-2 border-border-subtle pl-5 py-1 hover:bg-white/[0.02] transition-colors leading-relaxed group">
                                        <span class="text-text-muted shrink-0 text-[10px] opacity-40 font-bold group-hover:opacity-100 transition-opacity">[{log.time}]</span>
                                        <div class="flex gap-2 items-center">
                                            <span class={cn("font-black tracking-tighter shrink-0", logColor(log.level))}>{" >> "}</span>
                                            <span class={cn(logColor(log.level), "italic font-bold tracking-tight")}>{log.msg}</span>
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

export default RockchipFlashView;
