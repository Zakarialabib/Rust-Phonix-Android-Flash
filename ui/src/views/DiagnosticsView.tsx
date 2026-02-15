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
                    <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase">{t('diagnostics.title')}</h2>
                </div>
                <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">{t('diagnostics.subtitle')}</p>
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
                        {t('diagnostics.tab_silicon')}
                    </Button>
                    <Button
                        onClick={() => setActiveTab('security')}
                        variant={activeTab() === 'security' ? 'primary' : 'ghost'}
                        class={cn(
                            "h-12 w-full justify-start px-6 rounded-none border-none text-[10px] font-black uppercase tracking-widest",
                            activeTab() === 'security' && "bg-rose-600 border-rose-600 shadow-glow shadow-rose-600/20"
                        )}
                    >
                        {t('diagnostics.tab_security')}
                    </Button>
                    <Button
                        onClick={() => setActiveTab('performance')}
                        variant={activeTab() === 'performance' ? 'primary' : 'ghost'}
                        class={cn(
                            "h-12 w-full justify-start px-6 rounded-none border-none text-[10px] font-black uppercase tracking-widest",
                            activeTab() === 'performance' && "bg-indigo-600 border-indigo-600 shadow-glow shadow-indigo-600/20"
                        )}
                    >
                        {t('diagnostics.tab_thermal')}
                    </Button>

                    <div class="mt-auto border border-dashed border-border-subtle p-5 space-y-4 bg-sidebar/20 rounded-sm font-bold">
                        <p class="text-[9px] text-text-muted leading-relaxed uppercase opacity-60">
                            {t('diagnostics.side_desc')}
                        </p>
                        <Button
                            onClick={runDiagnostics}
                            isLoading={loading()}
                            glow
                            class="w-full h-11 text-[9px] font-black rounded-none transition-ui tracking-widest"
                        >
                            {t('diagnostics.btn_probe')}
                        </Button>
                    </div>
                </div>

                {/* Content Area */}
                <div class="lg:col-span-9 flex flex-col gap-6 overflow-hidden">
                    <Switch>
                        <Match when={activeTab() === 'hardware'}>
                            <div class="grid md:grid-cols-2 gap-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
                                <Card glow="accent" title={t('diagnostics.card_soc')} subtitle={t('diagnostics.card_soc_desc')}>
                                    <Show when={report()?.usbDevices?.[0]} fallback={<div class="py-12 text-center text-[9px] text-text-muted font-black uppercase opacity-30">{t('diagnostics.status_awaiting')}</div>}>
                                        <PropertyGrid>
                                            <Property label={t('diagnostics.lbl_family')} value={report()?.usbDevices[0]?.socFamily} accent />
                                            <Property label={t('diagnostics.lbl_silicon')} value={report()?.usbDevices[0]?.socModel} />
                                            <Property label={t('diagnostics.lbl_board_rev')} value={report()?.pcbVariant} highlight={report()?.pcbVariant === 'P282'} />
                                            <Property label={t('diagnostics.lbl_mode')} value={report()?.usbDevices[0]?.mode} />
                                            <Property label={t('diagnostics.lbl_vendor')} value={report()?.usbDevices[0]?.vendorName} />
                                            <Property label={t('diagnostics.lbl_io_bridge')} value={`${report()?.usbDevices[0]?.vendorId?.toString(16)}:${report()?.usbDevices[0]?.productId?.toString(16)}`} />
                                        </PropertyGrid>
                                    </Show>
                                </Card>

                                <Card glow="accent" title={t('diagnostics.card_memory')} subtitle={t('diagnostics.card_memory_desc')}>
                                    <Show when={report()?.ddrTiming} fallback={<div class="py-12 text-center text-[9px] text-text-muted font-black uppercase opacity-30">{t('diagnostics.status_probe_req')}</div>}>
                                        <PropertyGrid>
                                            <Property label={t('diagnostics.lbl_dram_node')} value={report()?.ddrTiming?.vendor} accent />
                                            <Property label={t('diagnostics.lbl_clock')} value={report()?.ddrTiming?.speed} />
                                            <Property label={t('diagnostics.lbl_density')} value={`${report()?.ddrTiming?.sizeMb} MB`} />
                                            <Property label={t('diagnostics.lbl_pattern')} value={report()?.ddrTiming?.timingPattern} />
                                        </PropertyGrid>
                                        <div class="mt-4 pt-4 border-t border-border-subtle bg-white/[0.01] p-3 rounded-sm">
                                            <span class="text-[8px] font-black text-text-muted uppercase opacity-60 block mb-3">{t('diagnostics.lbl_compatible_dtb')}</span>
                                            <div class="flex flex-wrap gap-2">
                                                <For each={report()?.ddrTiming?.compatibleDtbs}>
                                                    {(dtb) => <Badge variant="secondary" size="sm" class="italic">{dtb}</Badge>}
                                                </For>
                                            </div>
                                        </div>
                                    </Show>
                                </Card>

                                <Card glow="accent" title={t('diagnostics.card_bootloader')} subtitle={t('diagnostics.card_bootloader_desc')}>
                                    <Show when={report()?.bootloader} fallback={<div class="py-12 text-center text-[9px] text-text-muted font-black uppercase opacity-30">{t('diagnostics.status_stage_awaiting')}</div>}>
                                        <PropertyGrid>
                                            <Property label={t('diagnostics.lbl_stage')} value={report()?.bootloader?.bootloaderType} />
                                            <Property label={t('diagnostics.lbl_revision')} value={report()?.bootloader?.version} />
                                            <Property label={t('diagnostics.lbl_secure_boot')} value={report()?.bootloader?.secureBoot ? 'LOCK_CONNECTED' : 'OPEN_GND'} accent={report()?.bootloader?.secureBoot} />
                                            <Property label={t('diagnostics.lbl_key_presence')} value={report()?.bootloader?.bl2Signed ? 'SIGNED' : 'RAW'} />
                                        </PropertyGrid>
                                        <Show when={report()?.bootloader?.notes?.length}>
                                            <div class="mt-4 border-t border-border-subtle pt-4 space-y-2">
                                                <For each={report()?.bootloader?.notes}>
                                                    {(note) => <p class="text-[9px] text-accent font-black uppercase tracking-tight pl-2 border-l-2 border-accent/20">{t('diagnostics.lbl_trace')} {note}</p>}
                                                </For>
                                            </div>
                                        </Show>
                                    </Show>
                                </Card>

                                <Card glow="accent" title={t('diagnostics.card_io')} subtitle={t('diagnostics.card_io_desc')}>
                                    <div class="space-y-6">
                                        <Show when={report()?.wifiChip} fallback={<div class="text-[9px] text-text-muted font-black uppercase opacity-30">{t('diagnostics.status_wireless_masked')}</div>}>
                                            <div class="space-y-1">
                                                <span class="text-[8px] font-black text-text-muted uppercase opacity-60 block">{t('diagnostics.lbl_wireless_ctrl')}</span>
                                                <PropertyGrid>
                                                    <Property label={t('diagnostics.lbl_silicon')} value={report()?.wifiChip?.chip} accent />
                                                    <Property label={t('diagnostics.lbl_driver')} value={report()?.wifiChip?.mainlineDriver ? 'MAINLINE_UPSTREAM' : 'LEGACY_BLOB'} />
                                                </PropertyGrid>
                                            </div>
                                        </Show>
                                        <Show when={report()?.emmcInfo} fallback={<div class="text-[9px] text-text-muted font-black uppercase border-t border-border-subtle pt-4 opacity-30">{t('diagnostics.status_storage_masked')}</div>}>
                                            <div class="space-y-1 pt-4 border-t border-border-subtle">
                                                <span class="text-[8px] font-black text-text-muted uppercase opacity-60 block">{t('diagnostics.lbl_flash_storage')}</span>
                                                <PropertyGrid>
                                                    <Property label={t('diagnostics.lbl_vendor')} value={report()?.emmcInfo?.vendor} />
                                                    <Property label={t('diagnostics.lbl_capacity')} value={`${report()?.emmcInfo?.capacityGb} GB`} accent />
                                                </PropertyGrid>
                                            </div>
                                        </Show>
                                    </div>
                                </Card>
                            </div>
                        </Match>

                        <Match when={activeTab() === 'security'}>
                            <div class="grid gap-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
                                <Card glow="rose" title={t('diagnostics.card_threats')} subtitle={t('diagnostics.card_threats_desc')}>
                                    <Show when={securityReport()} fallback={
                                        <div class="flex flex-col items-center py-24 grayscale opacity-20 select-none">
                                            <div class="text-7xl mb-6">🛡️</div>
                                            <span class="text-[10px] tracking-[0.5em] font-black uppercase">{t('diagnostics.vault_req_title')}</span>
                                            <p class="text-[9px] text-text-muted mt-2 uppercase tracking-widest font-bold">{t('diagnostics.vault_req_desc')}</p>
                                        </div>
                                    }>
                                        <Show when={securityReport()?.isInfected} fallback={
                                            <div class="flex flex-col items-center py-14 bg-emerald-500/5 border border-emerald-500/10 rounded-sm">
                                                <div class="w-3 h-3 rounded-full bg-emerald-500 animate-pulse mb-4 shadow-glow shadow-emerald-500/40" />
                                                <h3 class="text-xs font-black text-emerald-400 uppercase tracking-widest leading-none mb-1">{t('diagnostics.signal_green')}</h3>
                                                <p class="text-[9px] text-text-muted mt-2 uppercase font-bold opacity-60">{t('diagnostics.signal_green_desc')}</p>
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
                                    <Card glow="rose" title={t('diagnostics.card_malware')} subtitle={t('diagnostics.card_malware_desc')}>
                                        <div class="space-y-4">
                                            <div class="p-4 bg-sidebar/50 border border-border-subtle hover:border-rose-500/30 transition-ui cursor-default rounded-sm group">
                                                <div class="flex items-center gap-2 mb-2">
                                                    <div class="w-1.5 h-1.5 bg-rose-500 group-hover:animate-pulse" />
                                                    <span class="text-[10px] font-black text-text-primary uppercase tracking-widest">{t('diagnostics.malware_corejava')}</span>
                                                </div>
                                                <p class="text-[9px] text-text-muted uppercase leading-relaxed font-bold opacity-60">{t('diagnostics.malware_corejava_desc')}</p>
                                            </div>
                                            <div class="p-4 bg-sidebar/50 border border-border-subtle hover:border-orange-500/30 transition-ui cursor-default rounded-sm group">
                                                <div class="flex items-center gap-2 mb-2">
                                                    <div class="w-1.5 h-1.5 bg-orange-500 group-hover:animate-pulse" />
                                                    <span class="text-[10px] font-black text-text-primary uppercase tracking-widest">{t('diagnostics.malware_badbox')}</span>
                                                </div>
                                                <p class="text-[9px] text-text-muted uppercase leading-relaxed font-bold opacity-60">{t('diagnostics.malware_badbox_desc')}</p>
                                            </div>
                                        </div>
                                    </Card>
                                    <Card glow="accent" title={t('diagnostics.card_techniques')} subtitle={t('diagnostics.card_techniques_desc')}>
                                        <div class="space-y-4 text-[10px] text-text-muted leading-relaxed font-bold uppercase py-1">
                                            <p><span class="text-accent font-black tracking-widest underline decoration-accent/20 underline-offset-4 mr-2">{t('diagnostics.tech_heap')}</span> {t('diagnostics.tech_heap_desc')}</p>
                                            <p><span class="text-accent font-black tracking-widest underline decoration-accent/20 underline-offset-4 mr-2">{t('diagnostics.tech_gift')}</span> {t('diagnostics.tech_gift_desc')}</p>
                                            <p><span class="text-accent font-black tracking-widest underline decoration-accent/20 underline-offset-4 mr-2">{t('diagnostics.tech_dtb')}</span> {t('diagnostics.tech_dtb_desc')}</p>
                                        </div>
                                    </Card>
                                </div>
                            </div>
                        </Match>

                        <Match when={activeTab() === 'performance'}>
                            <div class="space-y-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
                                <div class="grid sm:grid-cols-3 gap-6">
                                    <Card glow="accent" class="text-center py-10">
                                        <span class="text-[9px] font-black text-text-muted uppercase tracking-[0.3em] mb-4 block opacity-60">{t('diagnostics.card_memory_profile')}</span>
                                        <span class="text-3xl font-black text-accent tracking-tighter drop-shadow-sm uppercase">{t('diagnostics.perf_low_ram')}</span>
                                        <p class="text-[9px] text-text-muted uppercase mt-3 font-bold">{t('diagnostics.perf_low_ram_desc')}</p>
                                    </Card>
                                    <Card glow="accent" class="text-center py-10">
                                        <span class="text-[9px] font-black text-text-muted uppercase tracking-[0.3em] mb-4 block opacity-60">{t('diagnostics.card_zram')}</span>
                                        <span class="text-3xl font-black text-accent tracking-tighter drop-shadow-sm uppercase">{t('diagnostics.perf_zram_val')}</span>
                                        <p class="text-[9px] text-text-muted uppercase mt-3 font-bold">{t('diagnostics.perf_zram_desc')}</p>
                                    </Card>
                                    <Card glow="accent" class="text-center py-10">
                                        <span class="text-[9px] font-black text-text-muted uppercase tracking-[0.3em] mb-4 block opacity-60">{t('diagnostics.card_governor')}</span>
                                        <span class="text-3xl font-black text-accent tracking-tighter drop-shadow-sm uppercase">{t('diagnostics.perf_governor_val')}</span>
                                        <p class="text-[9px] text-text-muted uppercase mt-3 font-bold">{t('diagnostics.perf_governor_desc')}</p>
                                    </Card>
                                </div>

                                <Card glow="accent" title={t('diagnostics.card_matrix')} subtitle={t('diagnostics.card_matrix_desc')}>
                                    <div class="space-y-3">
                                        <For each={[
                                            { key: 'ro.config.low_ram', value: 'true', desc: t('diagnostics.opt_low_ram_desc') },
                                            { key: 'ro.sys.fw.bg_apps_limit', value: '16', desc: t('diagnostics.opt_bg_apps_desc') },
                                            { key: 'persist.sys.purgeable_assets', value: '1', desc: t('diagnostics.opt_purgeable_desc') }
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
