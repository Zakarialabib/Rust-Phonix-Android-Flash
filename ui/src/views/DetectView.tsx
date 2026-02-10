import { createSignal, Show, For } from 'solid-js';
import { produce } from 'solid-js/store';
import { tauriApi } from '../api/tauri';
import { getAppErrorMessage } from '../errorCodes';
import { DetectedDevice, DeviceProfile } from '../types';
import { globalStore, setGlobalStore } from '../store';
import { Card } from '../components/ui/Card';
import { Button } from '../components/ui/Button';
import { Badge } from '../components/ui/Badge';
import { Collapsible } from '../components/ui/Collapsible';
import { Property, PropertyGrid } from '../components/ui/Property';
import { cn } from '../lib/utils';
import { useApp } from '../context/AppContext';

export default function DetectView() {
  const { t } = useApp();
  const [scanning, setScanning] = createSignal(false);
  const [uartScanning, setUartScanning] = createSignal(false);
  const [profile, setProfile] = createSignal<DeviceProfile | null>(null);

  const appendLog = (line: string) => {
    setGlobalStore('buildStatus', produce((state) => {
      state.log.push(`[${new Date().toLocaleTimeString()}] ${line}`);
      if (state.log.length > 100) state.log.shift();
    }));
  };

  const scan = async () => {
    setScanning(true);
    appendLog('USB scan initiated via rusb engine');
    try {
      const results = await tauriApi.detectDevices();

      setGlobalStore('detectedDevices', {});
      const devicesMap: Record<string, DetectedDevice> = {};
      results.forEach(d => {
        const key = `${d.busNumber}:${d.deviceAddress}`;
        devicesMap[key] = d;
      });
      setGlobalStore('detectedDevices', devicesMap);

      if (results.length > 0 && !globalStore.lastDetected) {
        selectDevice(results[0]);
      }
      appendLog(`Scan complete: ${results.length} hardware nodes found`);
    } catch (e) {
      appendLog(`USB subsystem error: ${getAppErrorMessage(e)}`);
    } finally {
      setScanning(false);
    }
  };

  const scanUart = async () => {
    setUartScanning(true);
    appendLog('Polling serial bus for TTL/UART interrupts...');
    try {
      const ports = await tauriApi.listSerialPorts();
      if (ports.length === 0) {
        appendLog('No active UART bridges detected');
      } else {
        appendLog(`Found ${ports.length} COM port(s): ${ports.join(', ')}`);
      }
    } catch (e) {
      appendLog(`UART subsystem error: ${getAppErrorMessage(e)}`);
    } finally {
      setUartScanning(false);
    }
  };

  const selectDevice = async (device: DetectedDevice) => {
    setGlobalStore('lastDetected', device);
    setProfile(null);
    appendLog(`Probing Device: VID=${device.vendorId.toString(16).toUpperCase()} PID=${device.productId.toString(16).toUpperCase()}`);
    try {
      const p = await tauriApi.resolveProfile(device.vendorId, device.productId);
      setProfile(p);
      if (p) {
        appendLog(`Hardware resolved to profile: ${p.name} [${p.soc.toUpperCase()}]`);
      }
    } catch (e) {
      appendLog(`Profile resolution failed: ${getAppErrorMessage(e)}`);
    }
  };

  const handleDownload = async (p: DeviceProfile) => {
    try {
      appendLog(`Fetching liberation assets for ${p.name}...`);
      const path = await tauriApi.downloadAssets(p);
      appendLog(`Assets localized to: ${path}`);
    } catch (e) {
      appendLog(`Download operation aborted: ${getAppErrorMessage(e)}`);
    }
  };

  const deviceList = () => Object.values(globalStore.detectedDevices);

  return (
    <div class="h-full flex flex-col gap-6 font-mono">
      <header class="flex flex-col gap-1">
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-accent animate-pulse shadow-glow" />
          <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase italic">Hardware Archaeology</h2>
        </div>
        <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5 italic">Identify target hardware and resolve SoC profiles</p>
      </header>

      <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 flex-1 min-h-0">
        {/* Left Column: List & Logs */}
        <div class="lg:col-span-8 flex flex-col gap-6 overflow-hidden">
          <Card
            glow="amber"
            title="Scanner Control"
            subtitle="Poll USB & UART buses for liberation-ready hardware"
            class="flex-1 flex flex-col overflow-hidden"
            actions={
              <div class="flex gap-2">
                <Button
                  onClick={scan}
                  isLoading={scanning()}
                  disabled={scanning()}
                  variant="primary"
                  size="sm"
                  class="px-5 italic"
                >
                  USB SCAN
                </Button>
                <Button
                  onClick={scanUart}
                  isLoading={uartScanning()}
                  disabled={uartScanning()}
                  variant="secondary"
                  size="sm"
                  class="px-5 italic"
                >
                  UART SCAN
                </Button>
              </div>
            }
          >
            <div class="space-y-4 flex-1 flex flex-col overflow-hidden pt-2">
              <div class="flex-1 min-h-[220px] bg-sidebar/30 border border-border-subtle rounded-sm p-4 overflow-y-auto custom-scrollbar">
                <Show
                  when={deviceList().length > 0}
                  fallback={
                    <div class="h-full min-h-[188px] flex flex-col items-center justify-center text-text-muted space-y-3 border border-dashed border-border-subtle rounded-sm opacity-60 italic">
                      <div class="terminal-pulse text-2xl opacity-20 group-hover:opacity-40 transition-opacity">📡</div>
                      <span class="text-[9px] uppercase tracking-[0.4em] font-black">Awaiting hardware connection...</span>
                    </div>
                  }
                >
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <For each={deviceList()}>{(d) => (
                      <button
                        onClick={() => selectDevice(d)}
                        class={cn(
                          "group relative flex flex-col p-4 border rounded-sm transition-ui text-left overflow-hidden italic",
                          globalStore.lastDetected === d
                            ? "border-accent bg-accent/5 shadow-glow shadow-accent/10"
                            : "border-border-subtle bg-sidebar/40 hover:border-accent/40 hover:bg-sidebar/60"
                        )}
                      >
                        <div class="flex items-center justify-between mb-3">
                          <span class="text-xs font-black text-text-primary uppercase truncate">{d.vendorName || "Unknown Vendor"}</span>
                          <Badge
                            variant={d.mode === 'Maskrom' ? 'error' : 'secondary'}
                            size="sm"
                            class="font-black"
                          >
                            {d.mode.toUpperCase()}
                          </Badge>
                        </div>

                        <div class="space-y-2">
                          <div class="flex justify-between items-center text-[9px] font-bold uppercase tracking-wider">
                            <span class="text-text-muted opacity-40">Bus Topology</span>
                            <span class="text-text-secondary">{d.busNumber}:{d.deviceAddress}</span>
                          </div>
                          <div class="flex justify-between items-center text-[9px] font-bold uppercase tracking-wider">
                            <span class="text-text-muted opacity-40">Hardware ID</span>
                            <span class="text-accent underline decoration-accent/20 underline-offset-4">{d.vendorId.toString(16).padStart(4, '0')}:{d.productId.toString(16).padStart(4, '0')}</span>
                          </div>
                        </div>

                        {globalStore.lastDetected === d && (
                          <div class="absolute bottom-0 left-0 h-[3px] bg-accent w-full shadow-[0_-2px_6px_rgba(var(--accent-rgb),0.3)] animate-pulse" />
                        )}
                      </button>
                    )}</For>
                  </div>
                </Show>
              </div>
            </div>
          </Card>

          <Collapsible title="Archaeology Log" subtitle="Real-time hardware discovery events" defaultOpen={true}>
            <div class="h-[180px] overflow-y-auto font-mono text-[9px] text-text-muted custom-scrollbar p-1 space-y-1 bg-black/[0.05]">
              <For each={globalStore.buildStatus.log} fallback={<span class="italic opacity-30 px-4 py-3 block uppercase tracking-widest font-black text-[8px]">No events logged in current session.</span>}>
                {(line) => (
                  <div class="flex gap-4 px-4 py-1.5 border-b border-white/[0.02] hover:bg-white/[0.01] transition-ui group">
                    <span class="text-text-muted opacity-30 shrink-0 font-bold group-hover:opacity-50 transition-opacity">[{line.split(']')[0].replace('[', '').trim()}]</span>
                    <span class="text-text-secondary font-black uppercase tracking-tight group-hover:text-text-primary transition-colors">{line.split(']')[1]}</span>
                  </div>
                )}
              </For>
            </div>
          </Collapsible>
        </div>

        {/* Right Column: Inspector */}
        <div class="lg:col-span-4 flex flex-col gap-6 overflow-hidden">
          <Show
            when={globalStore.lastDetected}
            fallback={
              <Card glow="slate" class="p-8 border-dashed flex flex-col items-center justify-center text-center h-[340px] opacity-40 italic">
                <div class="text-2xl mb-4 grayscale opacity-20">🔍</div>
                <span class="text-[9px] text-text-muted font-black uppercase tracking-[0.3em] leading-relaxed">Select device node<br />to begin inspection</span>
              </Card>
            }
          >
            <Card glow="amber" title="Device Inspector" subtitle="Resolved hardware capabilities">
              <div class="space-y-6">
                <div class="space-y-2">
                  <p class="text-[9px] text-text-muted font-black uppercase tracking-widest italic opacity-40">Hardware Signature</p>
                  <div class="bg-sidebar p-5 border border-border-subtle font-black text-2xl text-accent tracking-tighter italic shadow-inner group">
                    <span class="opacity-80 group-hover:opacity-100 transition-opacity">{globalStore.lastDetected?.vendorId.toString(16).padStart(4, '0').toUpperCase()}</span>
                    <span class="text-text-muted opacity-20 mx-3">:</span>
                    <span class="opacity-80 group-hover:opacity-100 transition-opacity">{globalStore.lastDetected?.productId.toString(16).padStart(4, '0').toUpperCase()}</span>
                  </div>
                </div>

                <PropertyGrid>
                  <Property label="Bus Topology" value={`BUS ${globalStore.lastDetected?.busNumber} / ADDR ${globalStore.lastDetected?.deviceAddress}`} />
                  <Property label="Detection Mode" value={globalStore.lastDetected?.mode} accent />
                  <Property label="Subsystem" value={globalStore.lastDetected?.vendorName || "Standard USB Hub"} />
                </PropertyGrid>

                <Show
                  when={profile()}
                  fallback={
                    <div class="py-10 flex flex-col items-center gap-4 border border-dashed border-border-subtle bg-sidebar/20 rounded-sm italic opacity-50">
                      <div class="w-5 h-5 border-2 border-accent/20 border-t-accent rounded-full animate-spin" />
                      <div class="text-[9px] text-text-muted font-black uppercase tracking-[0.2em]">Resolving Profiling Matrix...</div>
                    </div>
                  }
                >
                  <div class="space-y-4 pt-4 border-t border-accent/10 animate-in fade-in slide-in-from-bottom-2">
                    <div class="p-5 bg-accent/5 border border-accent/10 italic rounded-sm relative overflow-hidden group">
                      <div class="absolute inset-0 bg-accent/5 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none" />
                      <h4 class="text-[10px] font-black text-accent uppercase tracking-[0.2em] mb-4 underline decoration-accent/20 underline-offset-4 relative z-10">MATCHED_PROFILE: {profile()?.name}</h4>
                      <PropertyGrid cols={2} class="relative z-10">
                        <Property label="Platform SoC" value={profile()?.soc} />
                        <Property label="Memory Node" value={`${profile()?.ramMb} MB`} />
                      </PropertyGrid>
                    </div>

                    <Button
                      variant="primary"
                      class="w-full italic font-black text-[10px] tracking-widest h-12"
                      onClick={() => handleDownload(profile()!)}
                    >
                      Fetch Liberation Assets
                    </Button>
                  </div>
                </Show>
              </div>
            </Card>
          </Show>

          <Collapsible title="Discovery Guide" subtitle="Manual protocol escalation">
            <div class="space-y-5 p-2 italic">
              <div class="space-y-2.5">
                <span class="text-accent uppercase font-black text-[9px] tracking-widest leading-none block border-l-2 border-accent pl-2">Amlogic Recovery (1B8E)</span>
                <p class="text-[10px] text-text-muted leading-relaxed uppercase font-bold opacity-70">
                  Hold 'Recovery' button (RESET) via AV port. Power on. Device enters <span class="text-accent font-black">WorldCup</span> mode.
                  <br /><span class="text-accent/60 font-black text-[9px] mt-1 block">Serial Baud: 115200</span>
                </p>
              </div>
              <div class="space-y-2.5 border-t border-border-subtle pt-4">
                <span class="text-indigo-400 uppercase font-black text-[9px] tracking-widest leading-none block border-l-2 border-indigo-400 pl-2">Rockchip Maskrom (2207)</span>
                <p class="text-[10px] text-text-muted leading-relaxed uppercase font-bold opacity-70">
                  Short CLK to GND on eMMC or hold recovery button. Device enters <span class="text-indigo-400 font-black">Maskrom</span> mode.
                  <br /><span class="text-indigo-400/60 font-black text-[9px] mt-1 block">Serial Baud: 1,500,000</span>
                </p>
              </div>
            </div>
          </Collapsible>
        </div>
      </div>
    </div>
  );
}
