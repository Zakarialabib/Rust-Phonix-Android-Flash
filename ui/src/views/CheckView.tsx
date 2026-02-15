import { createSignal, createResource, createEffect, onCleanup, Show, For } from 'solid-js';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { tauriApi } from '../api/tauri';
import { getAppErrorMessage } from '../errorCodes';
import { CompatibilityReport, PatchPlanResponse, WorkflowPhaseEvent } from '../types';
import { globalStore } from '../store';
import { useApp } from '../context/AppContext';
import { Card } from '../components/ui/Card';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';
import { Select } from '../components/ui/Select';
import { Badge } from '../components/ui/Badge';
import { Collapsible } from '../components/ui/Collapsible';
import { cn } from '../lib/utils';

const STANDARD_PROFILES = [
  { label: 'SELECT SYSTEM PROFILE...', value: '' },
  { label: 'AMLOGIC S905W (GENERIC)', value: 'configs/amlogic/s905w_generic.yaml' },
  { label: 'AMLOGIC S905X (GENERIC)', value: 'configs/amlogic/s905x_generic.yaml' },
  { label: 'ROCKCHIP RK3229 (MXQ)', value: 'configs/rockchip/rk3229.yaml' },
  { label: 'ROCKCHIP RK3328 (HK1)', value: 'configs/rockchip/rk3328.yaml' },
  { label: 'ALLWINNER H3 (SUNVELL)', value: 'configs/allwinner/h3_sunvell.yaml' },
];

export default function CheckView() {
  const { t } = useApp();
  const [profilePath, setProfilePath] = createSignal('');
  const [firmwarePath, setFirmwarePath] = createSignal('');
  const [osType, setOsType] = createSignal('android');
  const [version, setVersion] = createSignal('9');
  const [kernel, setKernel] = createSignal('');
  const [checkRunId, setCheckRunId] = createSignal(0);
  const [planRunId, setPlanRunId] = createSignal(0);
  const [phaseLabel, setPhaseLabel] = createSignal('IDLE');

  const browseProfile = async () => {
    const selected = await open({ filters: [{ name: 'YAML Profile', extensions: ['yaml', 'yml'] }] });
    if (selected && typeof selected === 'string') setProfilePath(selected);
  };

  const browseFirmware = async () => {
    const selected = await open({ filters: [{ name: 'Firmware', extensions: ['img', 'zip', 'bin'] }] });
    if (selected && typeof selected === 'string') setFirmwarePath(selected);
  };

  const [report] = createResource(() => {
    if (checkRunId() === 0) return null;
    return { profile: profilePath(), firmware: firmwarePath(), os: osType(), version: version(), kernel: kernel() };
  }, async (params) => {
    return await tauriApi.checkCompatibility(params.profile, params.firmware, params.os, params.version, params.kernel || undefined);
  });

  const [plan] = createResource(() => {
    if (planRunId() === 0) return null;
    return { profile: profilePath(), firmware: firmwarePath(), os: osType(), version: version(), kernel: kernel() };
  }, async (params) => {
    return await tauriApi.planPatches(params.profile, params.firmware, params.os, params.version, params.kernel || undefined);
  });

  createEffect(() => {
    let unlisten: UnlistenFn | undefined;
    const setupListener = async () => {
      unlisten = await listen<WorkflowPhaseEvent>('workflow:phase', (event) => {
        const detail = event.payload.detail ? ` [${event.payload.detail}]` : '';
        setPhaseLabel(`${event.payload.phase} >> ${event.payload.status}${detail}`.toUpperCase());
      });
    };
    setupListener();
    onCleanup(() => { if (unlisten) unlisten(); });
  });

  const runCheck = () => { if (profilePath() && firmwarePath()) setCheckRunId(id => id + 1); };
  const runPlan = () => { if (profilePath() && firmwarePath()) setPlanRunId(id => id + 1); };

  return (
    <div class="h-full flex flex-col gap-6 font-mono">
      <header class="flex flex-col gap-1 text-left">
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-accent animate-pulse shadow-[0_0_8px_rgba(var(--accent-rgb),0.4)]" />
          <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase">{t('check.title') || 'Compatibility Matrix'}</h2>
        </div>
        <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">{t('check.subtitle') || 'Pre-Flight Analysis | Layer 2 — Hardware-Firmware Mapping'}</p>
      </header>

      <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 flex-1 min-h-auto">
        <div class="lg:col-span-12 flex flex-col gap-6 overflow-y-auto custom-scrollbar pr-2 leading-none pb-4">
          <Card glow="amber" title={t('check.card_input_title') || 'Matrix Input'} subtitle={t('check.card_input_desc') || 'Verify firmware architecture against target SoC blueprint'} class="border-border-subtle">
            <div class="grid lg:grid-cols-2 gap-x-12 gap-y-8 mb-8">
              <div class="space-y-8">
                <div class="space-y-3">
                  <label class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] opacity-60 leading-none">{t('check.lbl_blueprint') || 'Hardware SoC Blueprint'}</label>
                  <div class="flex gap-3">
                    <div class="flex-1 relative">
                      <select
                        class="w-full h-11 px-5 bg-sidebar/40 border border-border-subtle rounded-none text-xs font-bold text-text-secondary focus:border-accent/50 focus:outline-none uppercase tracking-tight appearance-none cursor-pointer"
                        value={profilePath()}
                        onChange={e => setProfilePath(e.currentTarget.value)}
                      >
                        <For each={STANDARD_PROFILES}>{(p) => (
                          <option value={p.value} class="bg-sidebar text-text-primary">{p.value === '' ? (t('check.placeholder_select_profile') || p.label) : p.label}</option>
                        )}</For>
                      </select>
                      <div class="absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none text-text-muted/40 text-[10px]">▼</div>
                    </div>
                    <Button variant="ghost" onClick={browseProfile} class="h-11 px-8 font-black text-[10px] border-border-subtle bg-sidebar/20 hover:bg-sidebar/40 rounded-none uppercase tracking-widest text-text-muted hover:text-text-primary transition-all">{t('check.btn_browse') || 'BROWSE'}</Button>
                  </div>
                  <Show when={globalStore.lastDetected}>
                    <div class="flex items-center gap-2 mt-3 px-1">
                      <div class="w-1 h-1 rounded-full bg-accent animate-pulse" />
                      <span class="text-[9px] text-accent font-black uppercase tracking-widest opacity-80">
                        {t('check.lbl_link_established') || 'Link Established:'} {globalStore.lastDetected?.model} / {globalStore.lastDetected?.vendorName}
                      </span>
                    </div>
                  </Show>
                </div>

                <div class="space-y-3">
                  <label class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] opacity-60 leading-none">{t('check.lbl_firmware') || 'Firmware Artifact Path'}</label>
                  <div class="flex gap-3">
                    <Input
                      placeholder={t('check.placeholder_select_firmware') || "SELECT .IMG OR .ZIP SOURCE"}
                      value={firmwarePath()}
                      onInput={e => setFirmwarePath(e.currentTarget.value)}
                      class="flex-1 bg-sidebar/40 border-border-subtle rounded-none h-11 text-xs font-bold tracking-tight text-text-secondary"
                    />
                    <Button variant="ghost" onClick={browseFirmware} class="h-11 px-8 font-black text-[10px] border-border-subtle bg-sidebar/20 hover:bg-sidebar/40 rounded-none uppercase tracking-widest text-text-muted hover:text-text-primary transition-all">{t('check.btn_locate') || 'LOCATE'}</Button>
                  </div>
                </div>
              </div>

              <div class="grid grid-cols-2 gap-8">
                <div class="space-y-3">
                  <label class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] opacity-60 leading-none">{t('check.lbl_metadata') || 'Ecosystem Metadata'}</label>
                  <Select
                    value={osType()}
                    onInput={e => setOsType(e.currentTarget.value)}
                    class="bg-sidebar/40 border-border-subtle rounded-none h-11 text-xs font-bold tracking-tight text-text-secondary"
                  >
                    <option value="android">{t('check.os_android') || 'Android Open Source Project'}</option>
                    <option value="linux">{t('check.os_linux') || 'Mainline Linux (Armbian)'}</option>
                    <option value="emuelec">{t('check.os_emuelec') || 'EmuELEC / CoreELEC'}</option>
                  </Select>
                </div>
                <div class="space-y-3">
                  <label class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] opacity-60 leading-none">{t('check.lbl_version') || 'Version Strategy'}</label>
                  <Select
                    value={version()}
                    onInput={e => setVersion(e.currentTarget.value)}
                    class="bg-sidebar/40 border-border-subtle rounded-none h-11 text-xs font-bold tracking-tight text-text-secondary"
                  >
                    <optgroup label={t('check.grp_android') || "ANDROID"} class="bg-sidebar">
                      <option value="7">7.1.2 (NOUGAT)</option>
                      <option value="9">9.0.0 (PIE)</option>
                      <option value="11">11.0.0 (RDV)</option>
                    </optgroup>
                    <optgroup label={t('check.grp_linux') || "LINUX KERNEL"} class="bg-sidebar">
                      <option value="5.10">5.10 LTS (MAINLINE)</option>
                      <option value="6.1">6.1 LTS (MODERN)</option>
                    </optgroup>
                  </Select>
                </div>
              </div>
            </div>

            <div class="flex items-center justify-between border-t border-border-subtle pt-8">
              <div class="flex gap-4">
                <Button
                  onClick={runCheck}
                  disabled={!profilePath() || !firmwarePath() || report.loading}
                  isLoading={report.loading}
                  class="bg-accent hover:bg-accent/90 border-none font-black px-10 h-14 rounded-none text-xs tracking-[0.2em] shadow-[0_10px_30px_rgba(var(--accent-rgb),0.2)] text-white"
                >
                  {t('check.btn_scan') || 'INITIATE SCAN'}
                </Button>
                <Button
                  onClick={runPlan}
                  disabled={!profilePath() || !firmwarePath() || plan.loading}
                  isLoading={plan.loading}
                  class="font-black px-10 h-14 border-border-subtle rounded-none text-xs tracking-[0.1em] bg-sidebar/20 hover:bg-sidebar/40 text-text-muted hover:text-text-primary transition-all shadow-none"
                >
                  {t('check.btn_plan') || 'GENERATE PATCH PLAN'}
                </Button>
              </div>
              <div class="flex flex-col items-end gap-2">
                <span class="text-[10px] font-black text-text-muted uppercase tracking-[0.3em] opacity-40">{t('check.phase_progression') || 'Phase Progression'}:</span>
                <Badge variant={phaseLabel() === 'IDLE' ? 'secondary' : 'success'} class="rounded-none px-6 font-black py-2 tracking-widest leading-none">
                  {phaseLabel()}
                </Badge>
              </div>
            </div>
          </Card>

          <div class="grid gap-6 lg:grid-cols-2 pb-4">
            <Show when={report.error}>
              <div class="rounded-none border-l-2 border-rose-500 bg-rose-500/5 p-6 font-mono text-[11px] text-rose-500 uppercase leading-relaxed font-bold">
                <div class="flex items-center gap-2 mb-1">
                  <span class="text-lg">⚠</span>
                  <span class="tracking-widest">{t('check.critical_signal_loss') || 'CRITICAL SIGNAL_LOSS'}</span>
                </div>
                <p class="opacity-80">{getAppErrorMessage(report.error)}</p>
              </div>
            </Show>

            <Show when={report()}>
              <Card glow="teal" title={t('check.card_conflict_title') || 'Conflict Analysis'} subtitle={t('check.card_conflict_desc') || 'Hardware/Firmware register collisions'} class="border-border-subtle">
                <div class="relative group">
                  <div class="max-h-80 overflow-auto bg-black/20 border border-border-subtle p-6 font-mono text-[10px] text-text-secondary custom-scrollbar selection:bg-accent/20 font-bold">
                    <For each={Object.entries(report()!)}>
                      {([key, val]) => (
                        <div class="flex gap-6 mb-3 border-b border-border-subtle pb-2 group/row hover:bg-white/[0.02] transition-colors rounded-sm px-2">
                          <span class="text-accent shrink-0 font-black uppercase text-[9px] w-28 tracking-widest mt-0.5">[{key}]</span>
                          <span class="text-text-muted opacity-80 group-hover/row:opacity-100 transition-opacity leading-relaxed">{typeof val === 'object' ? JSON.stringify(val) : String(val)}</span>
                        </div>
                      )}
                    </For>
                  </div>
                  <div class="absolute top-4 right-4 opacity-40 group-hover:opacity-100 transition-all scale-90 group-hover:scale-100">
                    <Badge variant="secondary" class="border-none rounded-none text-[9px] font-black tracking-widest py-1 px-4">{t('check.hwmap_v1') || 'HWMAP_V1'}</Badge>
                  </div>
                </div>
              </Card>
            </Show>

            <Show when={plan.error}>
              <div class="rounded-none border-l-2 border-rose-500 bg-rose-500/5 p-6 font-mono text-[11px] text-rose-500 uppercase leading-relaxed font-bold">
                <div class="flex items-center gap-2 mb-1">
                  <span class="text-lg">⚠</span>
                  <span class="tracking-widest">{t('check.patch_planning_failure') || 'PATCH_PLANNING_FAILURE'}</span>
                </div>
                <p class="opacity-80">{getAppErrorMessage(plan.error)}</p>
              </div>
            </Show>

            <Show when={plan()}>
              <Card glow="indigo" title={t('check.card_patch_title') || 'Patch Logic Cache'} subtitle={t('check.card_patch_desc') || 'Planned DTB surgery & Blob injection'} class="border-border-subtle">
                <div class="relative group">
                  <div class="max-h-80 overflow-auto bg-black/20 border border-border-subtle p-6 font-mono text-[10px] text-text-secondary custom-scrollbar selection:bg-accent/20 font-bold">
                    <For each={Object.entries(plan()!)}>
                      {([key, val]) => (
                        <div class="flex gap-6 mb-3 border-b border-border-subtle pb-2 group/row hover:bg-white/[0.02] transition-colors rounded-sm px-2">
                          <span class="text-accent shrink-0 font-black uppercase text-[9px] w-28 tracking-widest mt-0.5">[{key}]</span>
                          <span class="text-text-muted opacity-80 group-hover/row:opacity-100 transition-opacity leading-relaxed">{typeof val === 'object' ? JSON.stringify(val) : String(val)}</span>
                        </div>
                      )}
                    </For>
                  </div>
                  <div class="absolute top-4 right-4 opacity-40 group-hover:opacity-100 transition-all scale-90 group-hover:scale-100">
                    <Badge variant="secondary" class="border-none rounded-none text-[9px] font-black tracking-widest py-1 px-4">{t('check.dtb_patch_set') || 'DTB_PATCH_SET'}</Badge>
                  </div>
                </div>
              </Card>
            </Show>
          </div>
        </div>
      </div>
    </div>
  );
}
