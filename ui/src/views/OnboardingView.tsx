import { Component, createSignal, Switch, Match, For, Show } from 'solid-js';
import { setGlobalStore } from '../store';
import { Button } from '../components/ui/Button';
import { Card } from '../components/ui/Card';
import { Badge } from '../components/ui/Badge';
import { useApp, Language } from '../context/AppContext';
import { cn } from '../lib/utils';

export const OnboardingView: Component = () => {
    const { state, t, setLanguage } = useApp();
    const [currentStep, setCurrentStep] = createSignal(1);
    const totalSteps = 6;

    const nextStep = () => {
        if (currentStep() < totalSteps) {
            setCurrentStep(currentStep() + 1);
        } else {
            completeOnboarding();
        }
    };

    const prevStep = () => {
        if (currentStep() > 1) {
            setCurrentStep(currentStep() - 1);
        }
    };

    const completeOnboarding = () => {
        localStorage.setItem('phoenix_onboarding_complete', 'true');
        setGlobalStore('setupStatus', 'ready');
    };

    const languages: { label: string; value: Language; flag: string; desc: string }[] = [
        { label: 'English', value: 'en', flag: '🇺🇸', desc: 'Default system language' },
        { label: 'العربية', value: 'ar', flag: '🇲🇦', desc: 'دعم كامل للكتابة من اليمين إلى اليسار' },
        { label: 'Français', value: 'fr', flag: '🇫🇷', desc: 'Support complet de la langue française' },
    ];

    return (
        <div class="fixed inset-0 z-[9990] flex items-center justify-center bg-bg-primary p-8 overflow-hidden font-mono transition-ui" dir={state.language === 'ar' ? 'rtl' : 'ltr'}>
            {/* Background */}
            <div class="absolute inset-0 opacity-[0.4] bg-dot-matrix pointer-events-none transition-ui duration-700" />
            <div class="absolute -top-48 -right-48 w-[600px] h-[600px] bg-accent/10 blur-[150px] rounded-full transition-ui duration-1000" />
            <div class="absolute -bottom-48 -left-48 w-[600px] h-[600px] bg-accent/5 blur-[150px] rounded-full transition-ui duration-1000" />

            <div class="relative w-full max-w-5xl max-h-[850px] flex flex-col items-center animate-in fade-in zoom-in duration-500">
                {/* Progress Header */}
                <div class="w-full flex justify-between items-end mb-10 border-b border-border-subtle pb-6 transition-ui">
                    <div class="space-y-1">
                        <h2 class="text-[10px] text-accent font-black uppercase tracking-[0.5em] leading-none animate-pulse">Ignition Sequence</h2>
                        <div class="text-3xl font-black tracking-tight text-text-primary uppercase">STAGE 0{currentStep()} <span class="text-text-muted opacity-20">/ 0{totalSteps}</span></div>
                    </div>
                    <div class="flex gap-2 mb-1.5">
                        <For each={Array.from({ length: totalSteps })}>
                            {(_, i) => (
                                <div
                                    class={cn(
                                        "h-1.5 w-12 transition-ui cursor-pointer rounded-none",
                                        i() + 1 <= currentStep() ? "bg-accent shadow-glow" : "bg-border-subtle opacity-10 hover:opacity-20"
                                    )}
                                    onClick={() => setCurrentStep(i() + 1)}
                                />
                            )}
                        </For>
                    </div>
                </div>

                {/* Step Content */}
                <div class="w-full flex-1 overflow-y-auto custom-scrollbar pr-2 py-4">
                    <Switch>
                        {/* STEP 1 — LANGUAGE */}
                        <Match when={currentStep() === 1}>
                            <div class="flex flex-col items-center text-center">
                                <h1 class="text-5xl font-black leading-tight mb-4 text-text-primary uppercase tracking-tighter">Choose your <span class="text-accent underline decoration-accent/20 underline-offset-8 shadow-glow shadow-accent/5">Linguistic Framework</span>.</h1>
                                <p class="text-text-muted text-base mb-12 max-w-2xl uppercase tracking-widest opacity-60 font-bold">
                                    Phoenix supports multiple locales. Select your preferred environment interface.
                                </p>

                                <div class="grid grid-cols-3 gap-8 w-full max-w-4xl">
                                    <For each={languages}>
                                        {(l) => (
                                            <button
                                                onClick={() => { setLanguage(l.value); }}
                                                class={cn(
                                                    "relative group p-10 border transition-ui text-left flex flex-col items-start gap-4 h-80 rounded-none",
                                                    state.language === l.value
                                                        ? "bg-accent/5 border-accent shadow-glow shadow-accent/10"
                                                        : "bg-sidebar/40 border-border-subtle hover:border-accent/40 hover:bg-sidebar/60"
                                                )}
                                            >
                                                <span class="text-5xl filter transition-ui group-hover:scale-110 drop-shadow-sm">{l.flag}</span>
                                                <div class="mt-4">
                                                    <div class={cn("text-2xl font-black mb-1.5 uppercase tracking-widest", state.language === l.value ? "text-accent" : "text-text-primary")}>
                                                        {l.label}
                                                    </div>
                                                    <div class="text-[10px] text-text-muted leading-relaxed uppercase tracking-[0.2em] font-black opacity-50">{l.desc}</div>
                                                </div>
                                                <Show when={state.language === l.value}>
                                                    <div class="mt-auto self-end text-accent animate-in fade-in zoom-in duration-300 shadow-glow">
                                                        <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5" /></svg>
                                                    </div>
                                                </Show>
                                            </button>
                                        )}
                                    </For>
                                </div>

                                <div class="mt-14">
                                    <Button onClick={nextStep} size="lg" glow class="px-16 font-black uppercase tracking-widest">COMMIT SELECTION →</Button>
                                </div>
                            </div>
                        </Match>

                        {/* STEP 2 — WELCOME */}
                        <Match when={currentStep() === 2}>
                            <div class="grid grid-cols-2 gap-16 items-center">
                                <div>
                                    <h1 class="text-6xl font-black leading-[1.05] mb-8 text-text-primary uppercase tracking-tighter">
                                        Greetings from <span class="text-accent underline decoration-accent/20 underline-offset-8 shadow-glow shadow-accent/5">Phoenix</span>.
                                    </h1>
                                    <p class="text-text-secondary text-lg mb-8 leading-relaxed uppercase tracking-widest font-black opacity-80">
                                        An open-source <strong class="text-accent">hardware liberation platform</strong> that transforms legacy Android TV boxes into sovereign compute infrastructure.
                                    </p>
                                    <div class="p-8 bg-sidebar/50 border-l-4 border-accent mb-10 shadow-inner group">
                                        <p class="text-text-muted text-[11px] leading-relaxed uppercase font-black opacity-60 group-hover:opacity-80 transition-opacity">
                                            "Every Android box liberated is $35 not spent on a Raspberry Pi, 200g of e-waste diverted, and a new node in a sovereign compute mesh."
                                        </p>
                                    </div>
                                    <Button onClick={nextStep} size="lg" glow class="px-12 font-black">{t('common.initialize')?.toUpperCase() || 'INITIALIZE SYSTEM'} →</Button>
                                </div>
                                <div class="space-y-6">
                                    <Card glow="amber">
                                        <div class="flex gap-6 items-start p-2">
                                            <span class="text-accent font-black text-3xl leading-none opacity-20 mt-1 shrink-0">01</span>
                                            <div>
                                                <div class="font-black text-[11px] mb-2 uppercase tracking-widest text-text-primary">{t('onboarding.step1_title') || 'UNIVERSAL DISCOVERY'}</div>
                                                <div class="text-[10px] text-text-muted leading-relaxed uppercase font-bold opacity-60">{t('onboarding.step1_desc') || 'Full support for Amlogic, Rockchip, and Allwinner SoCs.'}</div>
                                            </div>
                                        </div>
                                    </Card>
                                    <Card glow="indigo">
                                        <div class="flex gap-6 items-start p-2">
                                            <span class="text-indigo-400 font-black text-3xl leading-none opacity-20 mt-1 shrink-0">02</span>
                                            <div>
                                                <div class="font-black text-[11px] mb-2 uppercase tracking-widest text-text-primary">{t('onboarding.step2_title') || 'DECLARATIVE SYNTHESIS'}</div>
                                                <div class="text-[10px] text-text-muted leading-relaxed uppercase font-bold opacity-60">{t('onboarding.step2_desc') || 'Build custom Linux environments with one-click automation.'}</div>
                                            </div>
                                        </div>
                                    </Card>
                                    <Card glow="teal">
                                        <div class="flex gap-6 items-start p-2">
                                            <span class="text-emerald-400 font-black text-3xl leading-none opacity-20 mt-1 shrink-0">03</span>
                                            <div>
                                                <div class="font-black text-[11px] mb-2 uppercase tracking-widest text-text-primary">{t('onboarding.step3_title') || 'RAW SECTOR BURN'}</div>
                                                <div class="text-[10px] text-text-muted leading-relaxed uppercase font-bold opacity-60">{t('onboarding.step3_desc') || 'Direct eMMC flashing via low-level hardware protocols.'}</div>
                                            </div>
                                        </div>
                                    </Card>
                                </div>
                            </div>
                        </Match>

                        {/* STEP 3 — ENVIRONMENT */}
                        <Match when={currentStep() === 3}>
                            <div class="flex flex-col">
                                <h2 class="text-4xl font-black mb-3 text-text-primary uppercase tracking-tighter">Foundational Verification</h2>
                                <p class="text-text-muted mb-10 text-[11px] uppercase font-black tracking-[0.3em] opacity-50">Phoenix requires these technical bridges for USB communication and declarative synthesis.</p>

                                <div class="grid grid-cols-3 gap-8 mb-10">
                                    <Card glow="amber">
                                        <div class="flex flex-col items-center text-center py-6 h-full">
                                            <div class="text-4xl mb-6 shadow-glow shadow-accent/10">🦀</div>
                                            <div class="font-black text-[12px] mb-2 uppercase tracking-widest text-text-primary">Rust Runtime</div>
                                            <div class="text-[10px] text-text-muted mb-8 leading-relaxed font-bold uppercase opacity-50 px-4 line-clamp-2">phoenix-lib core, rusb bindings, archive extraction engine</div>
                                            <Badge variant="accent" size="sm" class="px-6 font-black shadow-glow">OPERATIONAL</Badge>
                                        </div>
                                    </Card>
                                    <Card glow="indigo">
                                        <div class="flex flex-col items-center text-center py-6 h-full">
                                            <div class="text-4xl mb-6 shadow-glow shadow-indigo-500/10">⬢</div>
                                            <div class="font-black text-[12px] mb-2 uppercase tracking-widest text-text-primary">Node.js 18+</div>
                                            <div class="text-[10px] text-text-muted mb-8 leading-relaxed font-bold uppercase opacity-50 px-4 line-clamp-2">Vite + SolidJS interface layer, Tauri v2 bridge engine</div>
                                            <Badge variant="accent" size="sm" class="px-6 font-black shadow-glow">OPERATIONAL</Badge>
                                        </div>
                                    </Card>
                                    <Card glow="teal">
                                        <div class="flex flex-col items-center text-center py-6 h-full">
                                            <div class="text-4xl mb-6 font-black text-text-secondary drop-shadow-md">C++</div>
                                            <div class="font-black text-[12px] mb-2 uppercase tracking-widest text-text-primary">VS Toolchain</div>
                                            <div class="text-[10px] text-text-muted mb-8 leading-relaxed font-bold uppercase opacity-50 px-4 line-clamp-2">MSVC linker for Tauri native compilation on Windows hosts</div>
                                            <Badge variant="accent" size="sm" class="px-6 font-black shadow-glow">OPERATIONAL</Badge>
                                        </div>
                                    </Card>
                                </div>

                                <div class="mt-10 flex justify-between gap-6">
                                    <Button variant="secondary" onClick={prevStep} class="h-16 px-10 font-black text-[11px] tracking-[0.2em]">← {t('common.back')?.toUpperCase() || 'BACK'}</Button>
                                    <Button onClick={nextStep} size="lg" glow class="px-16 font-black text-[11px] tracking-[0.2em] bg-accent">ALL BRIDGES OPERATIONAL →</Button>
                                </div>
                            </div>
                        </Match>

                        {/* STEP 4 — ARCHITECTURE */}
                        <Match when={currentStep() === 4}>
                            <div class="flex flex-col">
                                <h2 class="text-4xl font-black mb-3 text-text-primary uppercase tracking-tighter">The Triple Hook Architecture</h2>
                                <p class="text-text-muted mb-10 text-[11px] uppercase font-black tracking-[0.3em] opacity-50">Phoenix is organized into three composable primitives forming the liberation loop.</p>
                                <div class="grid grid-cols-3 gap-6">
                                    <Card glow="amber">
                                        <div class="font-black text-[12px] mb-3 uppercase tracking-widest text-accent underline decoration-accent/20 underline-offset-4">Primitive 01: Ignition</div>
                                        <div class="text-[10px] font-bold text-text-muted leading-relaxed uppercase opacity-60">Hardware discovery, SoC fingerprinting, and low-level USB protocol handshakes.</div>
                                    </Card>
                                    <Card glow="indigo">
                                        <div class="font-black text-[12px] mb-3 uppercase tracking-widest text-indigo-400 underline decoration-indigo-400/20 underline-offset-4">Primitive 02: Synthesis</div>
                                        <div class="text-[10px] font-bold text-text-muted leading-relaxed uppercase opacity-60">Firmware building, DTB surgery, and declarative image orchestration from source.</div>
                                    </Card>
                                    <Card glow="teal">
                                        <div class="font-black text-[12px] mb-3 uppercase tracking-widest text-emerald-400 underline decoration-emerald-400/20 underline-offset-4">Primitive 03: Deployment</div>
                                        <div class="text-[10px] font-bold text-text-muted leading-relaxed uppercase opacity-60">Flashing via WorldCup, RockUSB, and FEL protocols directly to eMMC partitions.</div>
                                    </Card>
                                </div>
                                <div class="mt-14 flex justify-between gap-6">
                                    <Button variant="secondary" onClick={prevStep} class="h-16 px-10 font-black text-[11px] tracking-[0.2em]">← {t('common.back')?.toUpperCase() || 'BACK'}</Button>
                                    <Button onClick={nextStep} size="lg" glow class="px-12 font-black text-[11px] tracking-[0.2em]">{t('common.next')?.toUpperCase() || 'NEXT MODULE'} →</Button>
                                </div>
                            </div>
                        </Match>

                        {/* STEP 5 — HARDWARE MATRIX */}
                        <Match when={currentStep() === 5}>
                            <div class="flex flex-col">
                                <h2 class="text-4xl font-black mb-3 text-text-primary uppercase tracking-tighter">Silicon Fingerprint Matrix</h2>
                                <p class="text-text-muted mb-10 text-[11px] uppercase font-black tracking-[0.3em] opacity-50">Verified protocol hooks for the most pervasive ARM SoCs found in legacy Android TV infrastructure.</p>
                                <div class="grid grid-cols-5 gap-4">
                                    <For each={['S905W', 'S905X', 'RK3229', 'RK3328', 'H3']}>
                                        {(soc) => (
                                            <Card glow="amber" class="hover:bg-accent/5 transition-ui">
                                                <div class="text-center py-6">
                                                    <div class="font-black text-sm text-text-primary tracking-[0.2em] mb-1.5">{soc}</div>
                                                    <Badge variant="accent" size="sm" class="opacity-50 group-hover:opacity-100 transition-opacity font-black">STABLE</Badge>
                                                </div>
                                            </Card>
                                        )}
                                    </For>
                                </div>
                                <div class="mt-14 flex justify-between gap-6">
                                    <Button variant="secondary" onClick={prevStep} class="h-16 px-10 font-black text-[11px] tracking-[0.2em]">← {t('common.back')?.toUpperCase() || 'BACK'}</Button>
                                    <Button onClick={nextStep} size="lg" glow class="px-12 font-black text-[11px] tracking-[0.2em]">{t('common.next')?.toUpperCase() || 'NEXT MODULE'} →</Button>
                                </div>
                            </div>
                        </Match>

                        {/* STEP 6 — IGNITION */}
                        <Match when={currentStep() === 6}>
                            <div class="flex flex-col items-center">
                                <h1 class="text-6xl font-black mb-6 text-text-primary uppercase tracking-tighter">Workbench <span class="text-accent underline decoration-accent/20 underline-offset-8 shadow-glow shadow-accent/5">Primed.</span></h1>
                                <p class="text-text-muted text-xl mb-14 text-center max-w-xl uppercase tracking-widest font-black opacity-60">
                                    All modules initialized. Your terminal is ready for irreversible hardware liberation.
                                </p>

                                <div class="w-40 h-40 rounded-none border-[1px] border-accent/30 flex items-center justify-center mb-20 relative bg-sidebar rotate-45 shadow-glow shadow-accent/10 group overflow-hidden">
                                    <div class="absolute inset-0 bg-accent/5 opacity-0 group-hover:opacity-100 transition-opacity" />
                                    <div class="absolute inset-0 border-[2px] border-accent animate-pulse shadow-glow" />
                                    <div class="absolute -inset-8 border-[1px] border-accent/10 animate-ping opacity-20" />
                                    <span class="text-7xl -rotate-45 drop-shadow-[0_0_20px_rgba(var(--accent-rgb),0.6)] animate-in fade-in zoom-in-50 duration-700">🔥</span>
                                </div>

                                <Button onClick={completeOnboarding} size="lg" glow class="px-24 h-20 text-3xl font-black tracking-[0.5em] bg-accent transition-ui hover:scale-105">
                                    IGNITION
                                </Button>
                            </div>
                        </Match>
                    </Switch>
                </div>

                {/* Footer */}
                <div class="w-full h-px bg-border-subtle opacity-20 mt-10 mb-6 transition-ui" />
                <div class="w-full flex justify-between items-center text-[10px] text-text-muted font-black uppercase tracking-[0.4em] shrink-0 opacity-30">
                    <span class="hover:text-text-primary cursor-default transition-colors">PHOENIX // SYSTEM_RECOVERED</span>
                    <span class="hidden md:block">GPL-3.0 · APACHE 2.0 · CC-BY-SA 4.0</span>
                    <span class="hover:text-accent cursor-default transition-colors">EST. 2026 // NODE_0x01</span>
                </div>
            </div>
        </div>
    );
};

export default OnboardingView;
