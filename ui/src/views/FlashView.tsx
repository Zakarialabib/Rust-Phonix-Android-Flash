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
          <h2 class="text-2xl font-black tracking-tighter text-text-primary uppercase">{t('flash.title')}</h2>
        </div>
        <p class="text-[10px] text-text-muted uppercase tracking-[0.3em] pl-5">{t('flash.subtitle')}</p>
      </header>

      <div class="grid lg:grid-cols-12 gap-6 flex-1 min-h-0">
        <div class="lg:col-span-8 flex flex-col gap-6 overflow-y-auto custom-scrollbar pr-2 pb-4">
          <Card glow="rose" title={t('flash.card_write_title')} subtitle={t('flash.card_write_subtitle')}>
            <div class="space-y-8">
              <div class="space-y-6">
                <div class="flex gap-4 items-end">
                  <div class="flex-1">
                    <Input
                      label={t('flash.input_source_label')}
                      placeholder={t('flash.input_source_placeholder')}
                      value={imagePath()}
                      onInput={e => setImagePath(e.currentTarget.value)}
                    />
                  </div>
                  <Button variant="secondary" onClick={selectImage} class="h-10 px-8">{t('common.explore')}</Button>
                </div>

                <div class="space-y-4">
                  <Input
                    label={t('flash.input_target_label')}
                    placeholder={t('flash.input_target_placeholder')}
                    value={targetDevice()}
                    onInput={e => setTargetDevice(e.currentTarget.value)}
                  />
                  <div class="flex items-center gap-2 pt-1 border-l-2 border-rose-500/20 pl-3">
                    <span class="text-rose-500 font-black text-[9px] uppercase animate-pulse">{t('flash.warning_destruction_title')}</span>
                    <span class="text-[9px] text-text-muted uppercase font-bold opacity-40">{t('flash.warning_destruction_desc')}</span>
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
                  {flashing() ? t('flash.btn_flashing') : t('flash.btn_flash')}
                </Button>
                <Show when={status()}>
                  <Badge variant={status().includes('FAIL') ? 'error' : 'accent'} size="md" class="py-2">
                    {status()}
                  </Badge>
                </Show>
              </div>
            </div>
          </Card>

          <div class="p-6 border-l-2 border-accent bg-accent/5 space-y-3 rounded-sm rounded-l-none">
            <div class="flex items-center gap-2">
              <span class="text-accent font-black text-xs">!</span>
              <span class="text-[10px] font-black text-accent uppercase tracking-[0.2em]">{t('flash.alert_windows_title')}</span>
            </div>
            <p class="text-[10px] text-text-muted uppercase leading-relaxed font-bold opacity-60">
              {t('flash.alert_windows_desc')} <span class="text-text-primary underline underline-offset-4 decoration-accent/30 font-black opacity-100">IGNORE THESE DIALOGS.</span> The Linux partition structure is not natively readable by Windows Explorer.
            </p>
          </div>
        </div>

        <div class="lg:col-span-4 flex flex-col gap-6 overflow-hidden pb-4">
          <Card glow="indigo" title={t('flash.card_hardware_title')} subtitle={t('flash.card_hardware_subtitle')}>
            <div class="space-y-4">
              <div class="p-5 bg-sidebar/30 border border-border-subtle hover:border-accent/40 transition-ui group cursor-pointer rounded-sm">
                <div class="flex items-center gap-3 mb-3 leading-none">
                  <span class="text-accent text-lg opacity-40 group-hover:opacity-100 transition-opacity">⚡</span>
                  <h4 class="text-[10px] font-black text-text-primary uppercase tracking-widest leading-none">{t('flash.tool_amlogic_title')}</h4>
                </div>
                <p class="text-[9px] text-text-muted uppercase leading-relaxed font-bold opacity-50">{t('flash.tool_amlogic_desc')}</p>
              </div>
              <div class="p-5 bg-sidebar/30 border border-border-subtle hover:border-accent/40 transition-ui group cursor-pointer rounded-sm">
                <div class="flex items-center gap-3 mb-3 leading-none">
                  <span class="text-accent text-lg opacity-40 group-hover:opacity-100 transition-opacity">⚡</span>
                  <h4 class="text-[10px] font-black text-text-primary uppercase tracking-widest leading-none">{t('flash.tool_rockchip_title')}</h4>
                </div>
                <p class="text-[9px] text-text-muted uppercase leading-relaxed font-bold opacity-50">{t('flash.tool_rockchip_desc')}</p>
              </div>
            </div>
          </Card>

          <Card glow="slate" title={t('flash.card_stats_title')} subtitle={t('flash.card_stats_subtitle')} class="bg-sidebar/20">
            <PropertyGrid>
              <Property label={t('flash.stat_block_size')} value="4.0 KB (STD)" />
              <Property label={t('flash.stat_buffer')} value="RUST_NIO_V2" accent />
              <Property label={t('flash.stat_method')} value="RAW_SECTOR_DIRECT" />
              <Property label={t('flash.stat_verify')} value="ENABLED" accent />
            </PropertyGrid>
          </Card>
        </div>
      </div>
    </div>
  );
}
