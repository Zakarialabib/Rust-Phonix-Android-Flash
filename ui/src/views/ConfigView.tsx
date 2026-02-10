import { createSignal, For, Show } from 'solid-js';
import { Card } from '../components/ui/Card';
import { Button } from '../components/ui/Button';
import { Badge } from '../components/ui/Badge';
import { Property, PropertyGrid } from '../components/ui/Property';
import { cn } from '../lib/utils';
import { useApp } from '../context/AppContext';

export default function ConfigView() {
  const { t } = useApp();
  const [selectedSoc, setSelectedSoc] = createSignal<string>('s905x4');
  const [selectedRam, setSelectedRam] = createSignal<string>('4gb');

  const socPresets: Record<string, { family: string, model: string, arch: string, nodes: number }> = {
    's905x4': { family: 'Amlogic G12B', model: 'S905X4-B', arch: 'Cortex-A55', nodes: 4 },
    's905x3': { family: 'Amlogic G12A', model: 'S905X3', arch: 'Cortex-A55', nodes: 4 },
    's922x': { family: 'Amlogic G12B', model: 'S922X-H', arch: 'A73+A53', nodes: 6 },
    'rk3566': { family: 'Rockchip RK35', model: 'RK3566', arch: 'Cortex-A55', nodes: 4 },
  };

  return (
    <div class="h-full flex flex-col gap-6 font-mono">
      <header class="flex flex-col gap-1">
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-accent animate-pulse shadow-glow" />
          <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase italic">Blueprint Architect</h2>
        </div>
        <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5 italic">Hardware Abstraction Layer | Firmware Synthesis Engine</p>
      </header>

      <div class="grid lg:grid-cols-12 gap-6 flex-1 min-h-0">
        {/* Left: Component Selection */}
        <div class="lg:col-span-8 space-y-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
          <Card glow="accent" title="Silicon Parameters" subtitle="Core SoC Selection & Architectural Bounds">
            <div class="grid sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
              <For each={Object.entries(socPresets)}>
                {([id, data]) => (
                  <button
                    onClick={() => setSelectedSoc(id)}
                    class={cn(
                      "flex flex-col p-4 border border-border-subtle transition-ui text-left rounded-none group relative overflow-hidden",
                      selectedSoc() === id
                        ? "bg-accent/10 border-accent shadow-glow"
                        : "bg-sidebar/30 hover:bg-sidebar/60"
                    )}
                  >
                    <div class="flex justify-between items-center mb-2">
                      <span class={cn(
                        "text-[8px] font-black uppercase tracking-widest",
                        selectedSoc() === id ? "text-accent" : "text-text-muted"
                      )}>{id}</span>
                      {selectedSoc() === id && <div class="w-1 h-1 bg-accent rounded-full animate-pulse shadow-glow" />}
                    </div>
                    <span class="text-[11px] font-black uppercase italic group-hover:text-text-primary transition-colors">{data.model}</span>
                    <span class="text-[9px] text-text-muted mt-1 uppercase opacity-60 group-hover:opacity-100">{data.arch}</span>

                    {/* Interaction Indicator */}
                    <div class={cn(
                      "absolute bottom-0 right-0 w-4 h-4 translate-x-2 translate-y-2 rotate-45 border-t border-l border-accent/20 transition-all",
                      selectedSoc() === id ? "bg-accent/40" : "opacity-0"
                    )} />
                  </button>
                )}
              </For>
            </div>

            <PropertyGrid cols={2} class="bg-sidebar/10 p-1 border border-border-subtle rounded-sm">
              <Property label="Micro-Architecture" value={socPresets[selectedSoc()].family} />
              <Property label="Silicon Revision" value="A-01-RE" accent />
              <Property label="Fabrication Node" value="12nm FinFET" />
              <Property label="Logic Units" value={`${socPresets[selectedSoc()].nodes} Cores`} accent />
              <Property label="Instruction Set" value="ARMv8.2-A" />
              <Property label="Thermal Trip" value="95°C" />
            </PropertyGrid>
          </Card>

          <Card glow="accent" title="Hardware Constraints" subtitle="Memory & I/O Physical Topology">
            <div class="space-y-6">
              <div class="flex flex-col gap-3">
                <span class="text-[10px] text-text-muted font-black uppercase tracking-widest italic opacity-60">Memory Density Configuration</span>
                <div class="flex flex-wrap gap-3">
                  <For each={['1gb', '2gb', '4gb', '8gb']}>
                    {(ram) => (
                      <Button
                        variant={selectedRam() === ram ? 'primary' : 'secondary'}
                        size="sm"
                        onClick={() => setSelectedRam(ram)}
                        class="rounded-none font-black italic tracking-tighter"
                        glow={selectedRam() === ram}
                      >
                        {ram} DDR4
                      </Button>
                    )}
                  </For>
                </div>
              </div>

              <PropertyGrid cols={2} class="pt-4 border-t border-border-subtle">
                <Property label="DRAM Protocol" value="LPDDR4x @ 2133MHz" />
                <Property label="BUS Width" value="32-bit Single Channel" />
                <Property label="EMMC Mode" value="HS400 (Enhanced)" accent />
                <Property label="SPI Support" value="NOR Flash / 16MB" />
              </PropertyGrid>
            </div>
          </Card>

          <Card glow="accent" title="Peripheral Matrix" subtitle="Network & Multimedia I/O Nodes">
            <PropertyGrid cols={2}>
              <Property label="HDMI Controller" value="2.1b / HDCP 2.3" accent />
              <Property label="CVBS Output" value="ENABLED / 480i" />
              <Property label="Ethernet Node" value="GIGABIT RGMII" accent />
              <Property label="USB Subsystem" value="1x 3.0 / 2x 2.0" />
              <Property label="Wi-Fi Interface" value="SDIO 3.0" />
              <Property label="Bluetooth" value="v5.0 LE" />
            </PropertyGrid>
          </Card>
        </div>

        {/* Right: Validation & Save */}
        <div class="lg:col-span-4 space-y-6 overflow-y-auto custom-scrollbar">
          <Card glow="accent" title="Synthesis Check" class="bg-accent/5 border-accent/20">
            <div class="space-y-4">
              <div class="flex items-center gap-3 p-4 bg-yellow-500/10 border border-yellow-500/20 italic rounded-sm">
                <div class="w-1.5 h-1.5 bg-yellow-500 rounded-full animate-pulse shadow-[0_0_8px_rgba(234,179,8,0.4)]" />
                <div class="space-y-1">
                  <p class="text-[10px] font-black text-yellow-500 uppercase tracking-widest italic leading-none">Topology Warning</p>
                  <p class="text-[9px] text-yellow-500/70 uppercase font-bold tracking-tighter">S905X4 thermal profile exceeds Passive limit.</p>
                </div>
              </div>

              <div class="space-y-2 pt-2">
                <div class="flex justify-between items-center text-[9px] font-black uppercase tracking-widest italic">
                  <span class="text-text-muted">Manifest Integrity</span>
                  <span class="text-emerald-500">98.2%</span>
                </div>
                <div class="h-1 bg-sidebar overflow-hidden rounded-full">
                  <div class="h-full bg-accent animate-draw shadow-glow" style="width: 98%" />
                </div>
              </div>

              <div class="grid grid-cols-2 gap-3 pt-4">
                <Button class="w-full text-[9px] h-12 rounded-none bg-sidebar text-text-primary hover:bg-sidebar/80 border-border-subtle lowercase italic font-black">
                  export_blueprint.json
                </Button>
                <Button class="w-full text-[9px] h-12 rounded-none bg-accent text-white shadow-glow hover:shadow-glow-strong border-none italic font-black">
                  INITIATE BUILD ⚡
                </Button>
              </div>
            </div>
          </Card>

          <Card glow="accent" title="Project Metadata" subtitle="Logical Workspace Bindings">
            <div class="space-y-4">
              <div class="space-y-2">
                <span class="text-[9px] text-text-muted font-black uppercase tracking-widest italic opacity-60">Active Workspace</span>
                <div class="p-3 bg-sidebar/50 border border-dashed border-border-subtle group hover:border-accent/40 transition-colors cursor-pointer rounded-sm">
                  <code class="text-[9px] text-text-secondary group-hover:text-accent font-black block truncate italic">/user/phoenix_os/stable_v1</code>
                </div>
              </div>
              <div class="space-y-2 pt-2">
                <span class="text-[9px] text-text-muted font-black uppercase tracking-widest italic opacity-60">Security Signer</span>
                <Badge variant="secondary" class="w-full justify-center py-2 italic font-black opacity-80 shadow-sm">
                  RSA_4096_SHA256
                </Badge>
              </div>
            </div>
          </Card>

          <div class="border border-dashed border-border-subtle p-5 bg-white/[0.01] rounded-sm italic">
            <p class="text-[9px] text-text-muted leading-relaxed uppercase font-bold opacity-60">
              Synthesis engine will automatically map identified hardware traits to the selected silicon blueprint.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
