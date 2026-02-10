import { createSignal, Show, For, Switch, Match } from 'solid-js';
import { tauriApi } from '../api/tauri';
import { getAppErrorMessage } from '../errorCodes';
import { Card } from '../components/ui/Card';
import { Button } from '../components/ui/Button';
import { Badge } from '../components/ui/Badge';
import { Property, PropertyGrid } from '../components/ui/Property';
import { ForensicsReport, SecurityReport } from '../types';
import { cn } from '../lib/utils';
import { useApp } from '../context/AppContext';

export default function DiagnosticsView() {
    const { t } = useApp();
    const [loading, setLoading] = createSignal(false);
    const [report, setReport] = createSignal<ForensicsReport | null>(null);
    const [securityReport, setSecurityReport] = createSignal<SecurityReport | null>(null);
    const [error, setError] = createSignal<string | null>(null);
    const [activeTab, setActiveTab] = createSignal<'hardware' | 'security' | 'performance'>('hardware');

    const runDiagnostics = async () => {
        setLoading(true);
        setError(null);
        try {
            const forensicsResult = await tauriApi.forensicsDeepScan();
            setReport(forensicsResult);
        } catch (e) {
            setError(getAppErrorMessage(e));
        } finally {
            setLoading(false);
        }
    };

    const getSeverityColor = (severity: string): "error" | "warning" | "secondary" | "default" => {
        switch (severity.toLowerCase()) {
            case 'critical': return 'error';
            case 'high': return 'warning';
            case 'medium': return 'secondary';
            case 'low': return 'default';
            default: return 'default';
        }
    };

    return (
        <div class="h-full flex flex-col gap-6 font-mono">
            <header class="flex flex-col gap-1">
                <div class="flex items-center gap-3">
                    <div class="w-2 h-2 rounded-full bg-accent animate-pulse shadow-glow" />
                    <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase">Archaeological Forensics</h2>
                </div>
                <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">Deep Hardware Probing | Layer 1 — Hardware Liberation</p>
            </header>

            <div class="grid lg:grid-cols-12 gap-6 flex-1 min-h-0">
                {/* Master Tab Control */}
                <div class="lg:col-span-3 flex flex-col gap-2">
                    <Button
                        onClick={() => setActiveTab('hardware')}
                        variant={activeTab() === 'hardware' ? 'primary' : 'ghost'}
                        class={cn(
                            "h-12 w-full justify-start px-6 rounded-none border-none text-[10px] font-black uppercase tracking-widest",
                            activeTab() === 'hardware' && "shadow-glow"
                        )}
                    >
                        01: SILICON TRACE
                    </Button>
                    <Button
                        onClick={() => setActiveTab('security')}
                        variant={activeTab() === 'security' ? 'primary' : 'ghost'}
                        class={cn(
                            "h-12 w-full justify-start px-6 rounded-none border-none text-[10px] font-black uppercase tracking-widest",
                            activeTab() === 'security' && "bg-rose-600 border-rose-600 shadow-glow shadow-rose-600/20"
                        )}
                    >
                        02: SECURITY VAULT
                    </Button>
                    <Button
                        onClick={() => setActiveTab('performance')}
                        variant={activeTab() === 'performance' ? 'primary' : 'ghost'}
                        class={cn(
                            "h-12 w-full justify-start px-6 rounded-none border-none text-[10px] font-black uppercase tracking-widest",
                            activeTab() === 'performance' && "bg-indigo-600 border-indigo-600 shadow-glow shadow-indigo-600/20"
                        )}
                    >
                        03: THERMAL NODE
                    </Button>

                    <div class="mt-auto border border-dashed border-border-subtle p-5 space-y-4 bg-sidebar/20 rounded-sm font-bold">
                        <p class="text-[9px] text-text-muted leading-relaxed uppercase opacity-60">
                            Deep scan probes the SoC registers directly to bypass kernel-level obfuscation.
                        </p>
                        <Button
                            onClick={runDiagnostics}
                            isLoading={loading()}
                            glow
                            class="w-full h-11 text-[9px] font-black rounded-none transition-ui tracking-widest"
                        >
                            INITIATE DEEP PROBE
                        </Button>
                    </div>
                </div>

                {/* Content Area */}
                <div class="lg:col-span-9 flex flex-col gap-6 overflow-hidden">
                    <Switch>
                        <Match when={activeTab() === 'hardware'}>
                            <div class="grid md:grid-cols-2 gap-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
                                <Card glow="accent" title="System on Chip" subtitle="Silicon ID & register trace">
                                    <Show when={report()?.usbDevices?.[0]} fallback={<div class="py-12 text-center text-[9px] text-text-muted font-black uppercase opacity-30">Awaiting Silicon Probe...</div>}>
                                        <PropertyGrid>
                                            <Property label="Family" value={report()?.usbDevices[0]?.socFamily} accent />
                                            <Property label="Silicon" value={report()?.usbDevices[0]?.socModel} />
                                            <Property label="Board REV" value={report()?.pcbVariant} highlight={report()?.pcbVariant === 'P282'} />
                                            <Property label="Mode" value={report()?.usbDevices[0]?.mode} />
                                            <Property label="Vendor" value={report()?.usbDevices[0]?.vendorName} />
                                            <Property label="IO Bridge" value={`${report()?.usbDevices[0]?.vendorId?.toString(16)}:${report()?.usbDevices[0]?.productId?.toString(16)}`} />
                                        </PropertyGrid>
                                    </Show>
                                </Card>

                                <Card glow="accent" title="Memory Latency" subtitle="SDRAM timings & vendor match">
                                    <Show when={report()?.ddrTiming} fallback={<div class="py-12 text-center text-[9px] text-text-muted font-black uppercase opacity-30">Probe required...</div>}>
                                        <PropertyGrid>
                                            <Property label="DRAM Node" value={report()?.ddrTiming?.vendor} accent />
                                            <Property label="Clock" value={report()?.ddrTiming?.speed} />
                                            <Property label="Density" value={`${report()?.ddrTiming?.sizeMb} MB`} />
                                            <Property label="Pattern" value={report()?.ddrTiming?.timingPattern} />
                                        </PropertyGrid>
                                        <div class="mt-4 pt-4 border-t border-border-subtle bg-white/[0.01] p-3 rounded-sm">
                                            <span class="text-[8px] font-black text-text-muted uppercase opacity-60 block mb-3">Compatible Device Trees:</span>
                                            <div class="flex flex-wrap gap-2">
                                                <For each={report()?.ddrTiming?.compatibleDtbs}>
                                                    {(dtb) => <Badge variant="secondary" size="sm" class="italic">{dtb}</Badge>}
                                                </For>
                                            </div>
                                        </div>
                                    </Show>
                                </Card>

                                <Card glow="accent" title="Bootloader State" subtitle="Secure boot & signature verification">
                                    <Show when={report()?.bootloader} fallback={<div class="py-12 text-center text-[9px] text-text-muted font-black uppercase opacity-30">Awaiting stage analysis...</div>}>
                                        <PropertyGrid>
                                            <Property label="Stage" value={report()?.bootloader?.bootloaderType} />
                                            <Property label="Revision" value={report()?.bootloader?.version} />
                                            <Property label="Secure Boot" value={report()?.bootloader?.secureBoot ? 'LOCK_CONNECTED' : 'OPEN_GND'} accent={report()?.bootloader?.secureBoot} />
                                            <Property label="Key Presence" value={report()?.bootloader?.bl2Signed ? 'SIGNED' : 'RAW'} />
                                        </PropertyGrid>
                                        <Show when={report()?.bootloader?.notes?.length}>
                                            <div class="mt-4 border-t border-border-subtle pt-4 space-y-2">
                                                <For each={report()?.bootloader?.notes}>
                                                    {(note) => <p class="text-[9px] text-accent font-black uppercase tracking-tight pl-2 border-l-2 border-accent/20">Trace: {note}</p>}
                                                </For>
                                            </div>
                                        </Show>
                                    </Show>
                                </Card>

                                <Card glow="accent" title="IO Matrix" subtitle="Storage & Wireless controllers">
                                    <div class="space-y-6">
                                        <Show when={report()?.wifiChip} fallback={<div class="text-[9px] text-text-muted font-black uppercase opacity-30">Wireless node masked...</div>}>
                                            <div class="space-y-1">
                                                <span class="text-[8px] font-black text-text-muted uppercase opacity-60 block">Wireless Controller</span>
                                                <PropertyGrid>
                                                    <Property label="Silicon" value={report()?.wifiChip?.chip} accent />
                                                    <Property label="Driver" value={report()?.wifiChip?.mainlineDriver ? 'MAINLINE_UPSTREAM' : 'LEGACY_BLOB'} />
                                                </PropertyGrid>
                                            </div>
                                        </Show>
                                        <Show when={report()?.emmcInfo} fallback={<div class="text-[9px] text-text-muted font-black uppercase border-t border-border-subtle pt-4 opacity-30">Storage node masked...</div>}>
                                            <div class="space-y-1 pt-4 border-t border-border-subtle">
                                                <span class="text-[8px] font-black text-text-muted uppercase opacity-60 block">Flash Storage</span>
                                                <PropertyGrid>
                                                    <Property label="Vendor" value={report()?.emmcInfo?.vendor} />
                                                    <Property label="Capacity" value={`${report()?.emmcInfo?.capacityGb} GB`} accent />
                                                </PropertyGrid>
                                            </div>
                                        </Show>
                                    </div>
                                </Card>
                            </div>
                        </Match>

                        <Match when={activeTab() === 'security'}>
                            <div class="grid gap-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
                                <Card glow="rose" title="Threat Detection Vault" subtitle="Scanning for malicious OEM telemetry & backdoors">
                                    <Show when={securityReport()} fallback={
                                        <div class="flex flex-col items-center py-24 grayscale opacity-20 select-none">
                                            <div class="text-7xl mb-6">🛡️</div>
                                            <span class="text-[10px] tracking-[0.5em] font-black uppercase">Vault Analysis Required</span>
                                            <p class="text-[9px] text-text-muted mt-2 uppercase tracking-widest font-bold">Mount firmware blob to initiate forensics</p>
                                        </div>
                                    }>
                                        <Show when={securityReport()?.isInfected} fallback={
                                            <div class="flex flex-col items-center py-14 bg-emerald-500/5 border border-emerald-500/10 rounded-sm">
                                                <div class="w-3 h-3 rounded-full bg-emerald-500 animate-pulse mb-4 shadow-glow shadow-emerald-500/40" />
                                                <h3 class="text-xs font-black text-emerald-400 uppercase tracking-widest leading-none mb-1">Signal Integrity Green</h3>
                                                <p class="text-[9px] text-text-muted mt-2 uppercase font-bold opacity-60">No known malware signatures found</p>
                                            </div>
                                        }>
                                            <div class="space-y-4">
                                                <For each={securityReport()?.threats}>
                                                    {(threat) => (
                                                        <div class="p-4 border border-rose-500/20 bg-rose-500/5 space-y-3 group hover:bg-rose-500/10 transition-ui">
                                                            <div class="flex justify-between items-center">
                                                                <span class="text-[10px] font-black text-rose-500 uppercase tracking-widest">{threat.name}</span>
                                                                <Badge variant={getSeverityColor(threat.severity)} class="px-3 font-black tracking-widest">{threat.severity}</Badge>
                                                            </div>
                                                            <p class="text-[9px] text-text-secondary leading-relaxed uppercase font-black opacity-80">{threat.description}</p>
                                                            <div class="pt-2 border-t border-rose-500/10 text-[8px] text-text-muted font-bold tracking-wider truncate">
                                                                PATH: {threat.path}
                                                            </div>
                                                        </div>
                                                    )}
                                                </For>
                                            </div>
                                        </Show>
                                    </Show>
                                </Card>

                                <div class="grid sm:grid-cols-2 gap-6">
                                    <Card glow="rose" title="Known OEM Malware" subtitle="Common TV Box threats">
                                        <div class="space-y-4">
                                            <div class="p-4 bg-sidebar/50 border border-border-subtle hover:border-rose-500/30 transition-ui cursor-default rounded-sm group">
                                                <div class="flex items-center gap-2 mb-2">
                                                    <div class="w-1.5 h-1.5 bg-rose-500 group-hover:animate-pulse" />
                                                    <span class="text-[10px] font-black text-text-primary uppercase tracking-widest">Corejava Botnet</span>
                                                </div>
                                                <p class="text-[9px] text-text-muted uppercase leading-relaxed font-bold opacity-60">OEM-installed click-fraud malware found in generic H6/S905 Android builds.</p>
                                            </div>
                                            <div class="p-4 bg-sidebar/50 border border-border-subtle hover:border-orange-500/30 transition-ui cursor-default rounded-sm group">
                                                <div class="flex items-center gap-2 mb-2">
                                                    <div class="w-1.5 h-1.5 bg-orange-500 group-hover:animate-pulse" />
                                                    <span class="text-[10px] font-black text-text-primary uppercase tracking-widest">BadBox Proxy</span>
                                                </div>
                                                <p class="text-[9px] text-text-muted uppercase leading-relaxed font-bold opacity-60">Residential proxy exit node masked as an OTA update service.</p>
                                            </div>
                                        </div>
                                    </Card>
                                    <Card glow="accent" title="Forensic Techniques" subtitle="Register-level analysis">
                                        <div class="space-y-4 text-[10px] text-text-muted leading-relaxed font-bold uppercase py-1">
                                            <p><span class="text-accent font-black tracking-widest underline decoration-accent/20 underline-offset-4 mr-2">HEAP SCAN:</span> Analyzing SRAM for residue OEM keys.</p>
                                            <p><span class="text-accent font-black tracking-widest underline decoration-accent/20 underline-offset-4 mr-2">GIFT WRAP:</span> Identifying masked partitions in EMMC controller.</p>
                                            <p><span class="text-accent font-black tracking-widest underline decoration-accent/20 underline-offset-4 mr-2">DTB DUMP:</span> Reconstructing hardware tree from binary blobs.</p>
                                        </div>
                                    </Card>
                                </div>
                            </div>
                        </Match>

                        <Match when={activeTab() === 'performance'}>
                            <div class="space-y-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
                                <div class="grid sm:grid-cols-3 gap-6">
                                    <Card glow="accent" class="text-center py-10">
                                        <span class="text-[9px] font-black text-text-muted uppercase tracking-[0.3em] mb-4 block opacity-60">Memory Profile</span>
                                        <span class="text-3xl font-black text-accent tracking-tighter drop-shadow-sm uppercase">Low RAM</span>
                                        <p class="text-[9px] text-text-muted uppercase mt-3 font-bold">Target: 1GB LPDDR3</p>
                                    </Card>
                                    <Card glow="accent" class="text-center py-10">
                                        <span class="text-[9px] font-black text-text-muted uppercase tracking-[0.3em] mb-4 block opacity-60">Compcache Status</span>
                                        <span class="text-3xl font-black text-accent tracking-tighter drop-shadow-sm uppercase">512 MB</span>
                                        <p class="text-[9px] text-text-muted uppercase mt-3 font-bold">ZRAM Node Active</p>
                                    </Card>
                                    <Card glow="accent" class="text-center py-10">
                                        <span class="text-[9px] font-black text-text-muted uppercase tracking-[0.3em] mb-4 block opacity-60">Power Governor</span>
                                        <span class="text-3xl font-black text-accent tracking-tighter drop-shadow-sm uppercase">Balanced</span>
                                        <p class="text-[9px] text-text-muted uppercase mt-3 font-bold">Mode: Interactive</p>
                                    </Card>
                                </div>

                                <Card glow="accent" title="Optimization Matrix" subtitle="Proposed system-level tweaks">
                                    <div class="space-y-3">
                                        <For each={[
                                            { key: 'ro.config.low_ram', value: 'true', desc: 'Squeeze memory for 1GB/512MB nodes' },
                                            { key: 'ro.sys.fw.bg_apps_limit', value: '16', desc: 'Prevent memory thrashing in background' },
                                            { key: 'persist.sys.purgeable_assets', value: '1', desc: 'Aggressive asset purging from heap' }
                                        ]}>
                                            {(prop) => (
                                                <div class="flex items-center justify-between p-5 bg-sidebar/50 border border-border-subtle group hover:border-accent/30 transition-ui rounded-sm">
                                                    <div class="space-y-1.5">
                                                        <code class="text-[11px] font-black text-accent uppercase tracking-tight">{prop.key}={prop.value}</code>
                                                        <p class="text-[9px] text-text-muted font-bold uppercase opacity-60 tracking-wider leading-none">{prop.desc}</p>
                                                    </div>
                                                    <div class="w-5 h-5 border border-border-subtle flex items-center justify-center bg-black/20 group-hover:bg-accent/10 transition-colors">
                                                        <div class="w-2 h-2 bg-accent opacity-20 group-hover:opacity-100 transition-opacity" />
                                                    </div>
                                                </div>
                                            )}
                                        </For>
                                    </div>
                                </Card>
                            </div>
                        </Match>
                    </Switch>
                </div>
            </div>
        </div>
    );
}
