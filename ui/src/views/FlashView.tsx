import { createSignal, Show } from 'solid-js';
import { tauriApi } from '../api/tauri';
import { getAppErrorMessage } from '../errorCodes';
import { Card } from '../components/ui/Card';
import { Button } from '../components/ui/Button';
import { Input } from '../components/ui/Input';
import { Badge } from '../components/ui/Badge';
import { Property, PropertyGrid } from '../components/ui/Property';
import { open } from '@tauri-apps/plugin-dialog';
import { cn } from '../lib/utils';
import { useApp } from '../context/AppContext';

export default function FlashView() {
  const { t } = useApp();
  const [imagePath, setImagePath] = createSignal('');
  const [targetDevice, setTargetDevice] = createSignal('');
  const [flashing, setFlashing] = createSignal(false);
  const [status, setStatus] = createSignal('');

  const selectImage = async () => {
    const selected = await open({
      filters: [{ name: 'Disk Image', extensions: ['img', 'iso', 'bin', 'sdcard'] }]
    });
    if (selected && typeof selected === 'string') {
      setImagePath(selected);
    }
  };

  const handleFlash = async () => {
    const image = imagePath().trim();
    const device = targetDevice().trim();
    if (!image || !device) {
      alert('Please provide both image path and target device.');
      return;
    }

    if (device.toLowerCase().includes('sda') || device.toLowerCase().includes('physicaldrive0')) {
      if (!confirm(`CRITICAL WARNING: ${device} appears to be a SYSTEM DRIVE. Proceeding will result in IRREVERSIBLE DATA LOSS. Continue?`)) {
        return;
      }
    }

    setFlashing(true);
    setStatus('IGNITION: SECTOR WRITE IN PROGRESS...');
    try {
      await tauriApi.flashImage(image, device);
      setStatus('SECTOR_VERIFY_OK: FLASH COMPLETE');
    } catch (e) {
      setStatus(`FAIL: ${getAppErrorMessage(e)}`);
    } finally {
      setFlashing(false);
    }
  };

  return (
    <div class="h-full flex flex-col gap-6 font-mono">
      <header class="flex flex-col gap-1 text-left">
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-accent animate-pulse shadow-glow" />
          <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase italic">Universal Image Burner</h2>
        </div>
        <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5 italic">Raw Partition Deployment | SD Card & USB Mass Storage</p>
      </header>

      <div class="grid lg:grid-cols-12 gap-6 flex-1 min-h-0">
        <div class="lg:col-span-8 flex flex-col gap-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
          <Card glow="rose" title="Destructive Write Configuration" subtitle="Define source blob and target physical node">
            <div class="space-y-8">
              <div class="space-y-6">
                <div class="flex gap-4 items-end">
                  <div class="flex-1">
                    <Input
                      label="Source Firmware Blob (.img / .iso)"
                      placeholder="C:\PHOENIX\IMAGES\ARMBIAN_S905W.IMG"
                      value={imagePath()}
                      onInput={e => setImagePath(e.currentTarget.value)}
                    />
                  </div>
                  <Button variant="secondary" onClick={selectImage} class="h-10 px-8">EXPLORE</Button>
                </div>

                <div class="space-y-4">
                  <Input
                    label="Target Physical Node (Local Device Path)"
                    placeholder="Windows: \\.\PhysicalDrive1 | Linux: /dev/sdb"
                    value={targetDevice()}
                    onInput={e => setTargetDevice(e.currentTarget.value)}
                  />
                  <div class="flex items-center gap-2 pt-1 border-l-2 border-rose-500/20 pl-3">
                    <span class="text-rose-500 font-black text-[9px] uppercase italic animate-pulse">! DATA_DESTRUCTION_WARNING:</span>
                    <span class="text-[9px] text-text-muted uppercase italic font-bold opacity-40">Confirm Device ID via PowerShell 'Get-Disk'</span>
                  </div>
                </div>
              </div>

              <div class="pt-6 border-t border-border-subtle flex items-center justify-between">
                <Button
                  onClick={handleFlash}
                  disabled={flashing()}
                  isLoading={flashing()}
                  size="lg"
                  glow={!flashing()}
                  class={cn(
                    "px-12",
                    flashing() ? "bg-sidebar text-text-muted" : "bg-accent"
                  )}
                >
                  {flashing() ? 'IGNITING SECTORS...' : 'START BURN SEQUENCE'}
                </Button>
                <Show when={status()}>
                  <Badge variant={status().includes('FAIL') ? 'error' : 'accent'} size="md" class="py-2">
                    {status()}
                  </Badge>
                </Show>
              </div>
            </div>
          </Card>

          <div class="p-6 border-l-2 border-accent bg-accent/5 space-y-3 italic rounded-sm rounded-l-none">
            <div class="flex items-center gap-2">
              <span class="text-accent font-black text-xs">!</span>
              <span class="text-[10px] font-black text-accent uppercase tracking-[0.2em] italic">Windows Mounting Conflict</span>
            </div>
            <p class="text-[10px] text-text-muted uppercase italic leading-relaxed font-bold opacity-60">
              Windows may attempt to mount the newly burned partition, requesting 'Formatting'. <span class="text-text-primary underline underline-offset-4 decoration-accent/30 font-black opacity-100">IGNORE THESE DIALOGS.</span> The Linux partition structure is not natively readable by Windows Explorer.
            </p>
          </div>
        </div>

        <div class="lg:col-span-4 flex flex-col gap-6 overflow-hidden pb-4">
          <Card glow="indigo" title="Hardware Specific Suites" subtitle="Protocol-level flashing tools">
            <div class="space-y-4">
              <div class="p-5 bg-sidebar/30 border border-border-subtle hover:border-accent/40 transition-ui group cursor-pointer rounded-sm italic">
                <div class="flex items-center gap-3 mb-3 leading-none">
                  <span class="text-accent text-lg opacity-40 group-hover:opacity-100 transition-opacity">⚡</span>
                  <h4 class="text-[10px] font-black text-text-primary uppercase tracking-widest italic leading-none">AMLOGIC WORLDCUP</h4>
                </div>
                <p class="text-[9px] text-text-muted uppercase italic leading-relaxed font-bold opacity-50">Direct eMMC flashing via USB OTG. Bypasses bootloader security.</p>
              </div>
              <div class="p-5 bg-sidebar/30 border border-border-subtle hover:border-accent/40 transition-ui group cursor-pointer rounded-sm italic">
                <div class="flex items-center gap-3 mb-3 leading-none">
                  <span class="text-accent text-lg opacity-40 group-hover:opacity-100 transition-opacity">⚡</span>
                  <h4 class="text-[10px] font-black text-text-primary uppercase tracking-widest italic leading-none">ROCKCHIP ROCKUSB</h4>
                </div>
                <p class="text-[9px] text-text-muted uppercase italic leading-relaxed font-bold opacity-50">High-speed loader-based deployment for RK35xx series.</p>
              </div>
            </div>
          </Card>

          <Card glow="slate" title="Burn Statics" subtitle="Transfer rate and integrity" class="bg-sidebar/20">
            <PropertyGrid>
              <Property label="Block Size" value="4.0 KB (STD)" />
              <Property label="Buffer Engine" value="RUST_NIO_V2" accent />
              <Property label="Write Method" value="RAW_SECTOR_DIRECT" />
              <Property label="Verify Stage" value="ENABLED" accent />
            </PropertyGrid>
          </Card>
        </div>
      </div>
    </div>
  );
}
