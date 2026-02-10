import { createSignal, Show, For, createEffect, Switch, Match } from 'solid-js';
import { tauriApi } from '../api/tauri';
import { getAppErrorMessage } from '../errorCodes';
import { Card } from '../components/ui/Card';
import { Button } from '../components/ui/Button';
import { Badge } from '../components/ui/Badge';
import { FirmwareRecommendation } from '../types';
import { cn } from '../lib/utils';
import { useApp } from '../context/AppContext';

type WorkflowStep = 'detect' | 'scan' | 'vault' | 'configure' | 'build' | 'flash' | 'verify';

interface StepConfig {
    id: WorkflowStep;
    title: string;
    description: string;
    icon: string;
}

const WORKFLOW_STEPS: StepConfig[] = [
    { id: 'detect', title: 'Hardware Discovery', description: 'Silicon ID & Handshake', icon: '🔌' },
    { id: 'scan', title: 'Threat Analysis', description: 'Malware Forensics', icon: '🔒' },
    { id: 'vault', title: 'NVRAM Vault', description: 'Unique ID Preservation', icon: '🛡️' },
    { id: 'configure', title: 'Intent Resolution', description: 'Blueprint Selection', icon: '⚙️' },
    { id: 'build', title: 'Synthesis', description: 'Image Construction', icon: '🔧' },
    { id: 'flash', title: 'Deployment', description: 'Sector Ignition', icon: '⚡' },
    { id: 'verify', title: 'Confirmation', description: 'Integrity Check', icon: '✅' },
];

interface StepStatus { completed: boolean; inProgress: boolean; error?: string; result?: any; }

export default function WizardView() {
    const { t } = useApp();
    const [currentStep, setCurrentStep] = createSignal<number>(0);
    const [stepStatuses, setStepStatuses] = createSignal<Record<WorkflowStep, StepStatus>>({
        detect: { completed: false, inProgress: false },
        scan: { completed: false, inProgress: false },
        vault: { completed: false, inProgress: false },
        configure: { completed: false, inProgress: false },
        build: { completed: false, inProgress: false },
        flash: { completed: false, inProgress: false },
        verify: { completed: false, inProgress: false },
    });

    const [recommendations, setRecommendations] = createSignal<FirmwareRecommendation[]>([]);

    const currentStepConfig = () => WORKFLOW_STEPS[currentStep()];

    createEffect(() => {
        const detectStatus = stepStatuses().detect;
        if (detectStatus.completed && detectStatus.result && detectStatus.result.length > 0) {
            const info = detectStatus.result[0];
            tauriApi.getFirmwareRecommendations({
                soc: info.socModel || info.socFamily,
                pcbVariant: 'unknown', ramVendor: 'unknown', wifiChip: 'unknown', emmcVendor: 'unknown', hdmiPhy: 'unknown'
            }).then(setRecommendations);
        }
    });

    const updateStepStatus = (step: WorkflowStep, status: Partial<StepStatus>) => {
        setStepStatuses(prev => ({ ...prev, [step]: { ...prev[step], ...status } }));
    };

    const handleDetect = async () => {
        updateStepStatus('detect', { inProgress: true, error: undefined });
        try {
            const devices = await tauriApi.detectDevices();
            if (devices.length === 0) throw new Error('NO HUB SIGNAL. ENSURE OTG LINK.');
            updateStepStatus('detect', { completed: true, inProgress: false, result: devices });
        } catch (e) {
            updateStepStatus('detect', { inProgress: false, error: getAppErrorMessage(e) });
        }
    };

    const handleVault = async () => {
        updateStepStatus('vault', { inProgress: true, error: undefined });
        try {
            await new Promise(resolve => setTimeout(resolve, 2000));
            updateStepStatus('vault', { completed: true, inProgress: false, result: { path: '/vault/p282_rsa.zip' } });
        } catch (e) { updateStepStatus('vault', { inProgress: false, error: getAppErrorMessage(e) }); }
    };

    return (
        <div class="h-full flex flex-col gap-6 font-mono">
            <header class="flex flex-col gap-1">
                <div class="flex items-center gap-3">
                    <div class="w-2 h-2 rounded-full bg-accent animate-pulse shadow-[0_0_8px_rgba(var(--accent-rgb),0.4)]" />
                    <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase italic">{t('wizard.title') || 'Liberation Pipeline'}</h2>
                </div>
                <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">Guided Hardware Transformation | Automated Protocol Orchestrator</p>
            </header>

            <div class="grid lg:grid-cols-12 gap-6 flex-1 min-h-0">
                {/* Master Pipeline Stepper */}
                <div class="lg:col-span-3 flex flex-col gap-1 bg-white/[0.01] border border-border-subtle p-2 overflow-y-auto custom-scrollbar">
                    <For each={WORKFLOW_STEPS}>{(step, index) => {
                        const status = stepStatuses()[step.id];
                        const isActive = index() === currentStep();
                        const isCompleted = status.completed;
                        return (
                            <button
                                onClick={() => index() <= currentStep() + 1 && setCurrentStep(index())}
                                disabled={index() > currentStep() + 1}
                                class={cn(
                                    "group flex flex-col p-4 transition-all border-l-2 text-left",
                                    isActive
                                        ? "bg-accent/5 border-accent italic"
                                        : isCompleted
                                            ? "border-emerald-500/40 opacity-60 hover:opacity-100"
                                            : "border-transparent opacity-30 cursor-not-allowed"
                                )}
                            >
                                <div class="flex justify-between items-center w-full">
                                    <span class={cn(
                                        "text-[8px] font-black uppercase tracking-[0.2em]",
                                        isActive ? "text-accent" : isCompleted ? "text-emerald-400" : "text-text-muted"
                                    )}>
                                        STAGE 0{index() + 1}
                                    </span>
                                    {isCompleted && <span class="text-emerald-500 text-[10px] font-black">✓</span>}
                                </div>
                                <span class={cn(
                                    "text-[10px] font-black uppercase mt-1",
                                    isActive ? "text-text-primary" : "text-text-secondary"
                                )}>{step.title}</span>
                                <span class="text-[8px] text-text-muted mt-1 uppercase tracking-widest opacity-60">{step.description}</span>
                            </button>
                        );
                    }}</For>
                </div>

                {/* Main Interaction Node */}
                <div class="lg:col-span-9 flex flex-col gap-6 overflow-hidden">
                    <Card glow="accent" title={currentStepConfig().title} subtitle={currentStepConfig().description} class="flex-1 flex flex-col overflow-hidden border-border-subtle">
                        <div class="flex-1 overflow-y-auto custom-scrollbar pr-2 py-2">
                            <Switch>
                                <Match when={currentStepConfig().id === 'detect'}>
                                    <div class="space-y-8">
                                        <div class="grid sm:grid-cols-2 gap-4 italic leading-loose">
                                            <div class="p-5 bg-sidebar/40 border border-border-subtle space-y-3 rounded-sm">
                                                <h4 class="text-[10px] font-black text-accent uppercase tracking-widest">Amlogic WorldCup</h4>
                                                <p class="text-[9px] text-text-muted uppercase font-bold opacity-60">Hold recovery pin or short eMMC CLK while cold booting OTG.</p>
                                                <Badge variant="secondary" class="rounded-none border-none text-[8px] opacity-60">VID 1B8E</Badge>
                                            </div>
                                            <div class="p-5 bg-sidebar/40 border border-border-subtle space-y-3 rounded-sm">
                                                <h4 class="text-[10px] font-black text-emerald-400 uppercase tracking-widest">Rockchip RockUSB</h4>
                                                <p class="text-[9px] text-text-muted uppercase font-bold opacity-60">Hold ADKey/Recovery during power-up sequence.</p>
                                                <Badge variant="secondary" class="rounded-none border-none text-[8px] opacity-60">VID 2207</Badge>
                                            </div>
                                        </div>
                                        <Button
                                            onClick={handleDetect}
                                            isLoading={stepStatuses().detect.inProgress}
                                            class="w-full h-14 font-black text-xs tracking-[0.2em] italic bg-accent hover:opacity-90 shadow-lg shadow-accent/20"
                                        >
                                            {stepStatuses().detect.completed ? 'RE-SCAN HUB' : 'INITIATE HARDWARE HANDSHAKE'}
                                        </Button>
                                    </div>
                                </Match>
                                <Match when={currentStepConfig().id === 'vault'}>
                                    <div class="space-y-8 italic">
                                        <div class="p-5 bg-accent/5 border border-accent/10 space-y-4 rounded-sm">
                                            <h3 class="text-[10px] font-black text-accent uppercase tracking-[0.2em]">Preservation Ethics</h3>
                                            <p class="text-[9px] text-text-secondary leading-relaxed uppercase font-bold opacity-80">
                                                The vault creates a bit-perfect clone of the original NAND/eMMC calibration blocks. This includes HDCP L1 keys, Widevine BLOBS, and MAC addresses. <b class="text-accent underline decoration-accent/30 underline-offset-4">REQUIRED FOR IRREVERSIBLE LIBERATION.</b>
                                            </p>
                                        </div>
                                        <Button onClick={handleVault} isLoading={stepStatuses().vault.inProgress} class="w-full h-14 bg-accent hover:opacity-90 border-none font-black text-xs tracking-[0.2em] rounded-none shadow-lg shadow-accent/20 italic">
                                            CREATE ENCRYPTED VAULT
                                        </Button>
                                    </div>
                                </Match>
                                <Match when={true}>
                                    <div class="py-20 flex flex-col items-center opacity-30 grayscale italic">
                                        <div class="text-4xl mb-4 text-accent">⚙️</div>
                                        <span class="text-[10px] font-black text-text-muted uppercase tracking-[0.5em]">Module Interface Standby</span>
                                    </div>
                                </Match>
                            </Switch>
                        </div>

                        {/* Node Footer Navigation */}
                        <div class="pt-6 border-t border-border-subtle flex justify-between items-center bg-white/[0.01]">
                            <Button
                                variant="ghost"
                                onClick={() => setCurrentStep(s => Math.max(0, s - 1))}
                                disabled={currentStep() === 0}
                                class="h-11 px-8 font-black text-[9px] uppercase border-border-subtle rounded-none italic tracking-widest"
                            >
                                « STAGE PREV
                            </Button>
                            <span class="text-[8px] font-black text-text-muted uppercase tracking-widest italic opacity-40">Node 0x{currentStep() + 1} // 0x07</span>
                            <Button
                                variant="ghost"
                                onClick={() => setCurrentStep(s => Math.min(WORKFLOW_STEPS.length - 1, s + 1))}
                                disabled={!stepStatuses()[currentStepConfig().id].completed}
                                class="h-11 px-8 font-black text-[9px] uppercase border-border-subtle rounded-none text-accent italic tracking-widest"
                            >
                                NEXT STAGE »
                            </Button>
                        </div>
                    </Card>
                </div>
            </div>
        </div>
    );
}
