import { invoke } from '@tauri-apps/api/core';
import { toAppError } from '../errorCodes';
import {
  DetectedDevice,
  DeviceConfig,
  AppSettings,
  SystemInfo,
  DeviceProfile,
  CompatibilityReport,
  PatchPlanResponse,
  HardwareProfile,
  FirmwareRecommendation,
  AmlogicChipInfo,
  RockchipChipInfo,
  RkImageHeader,
  RkParameter,
  AllwinnerVersion,
  AllwinnerImageHeader,
  ForensicsReport,
  SecurityReport,
  RemoteConfig,
} from '../types';

const invokeTauri = async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toAppError(error);
  }
};

export const tauriApi = {
  resolveProfile: async (vid: number, pid: number): Promise<DeviceProfile | null> => {
    return await invokeTauri('cmd_resolve_profile', { vid, pid });
  },

  detectDevices: async (): Promise<DetectedDevice[]> => {
    return await invokeTauri('cmd_detect_devices');
  },

  listSerialPorts: async (): Promise<string[]> => {
    return await invokeTauri('cmd_list_serial_ports');
  },

  flashImage: async (imagePath: string, targetDevice: string): Promise<void> => {
    return await invokeTauri('cmd_flash_image', { imagePath, targetDevice });
  },

  downloadAssets: async (profile: DeviceProfile): Promise<string> => {
    return await invokeTauri('cmd_download_assets', { profile });
  },

  createConfig: async (soc: string, name: string): Promise<DeviceConfig> => {
    return await invokeTauri('cmd_create_config', { soc, name });
  },

  loadConfig: async (path: string): Promise<DeviceConfig> => {
    return await invokeTauri('cmd_load_config', { path });
  },

  saveConfig: async (config: DeviceConfig, path: string): Promise<void> => {
    return await invokeTauri('cmd_save_config', { config, path });
  },

  validateConfig: async (config: DeviceConfig): Promise<void> => {
    return await invokeTauri('cmd_validate_config', { config });
  },

  // Amlogic Commands
  amlogicDetect: async (): Promise<AmlogicChipInfo> => {
    return await invokeTauri('cmd_amlogic_detect');
  },

  amlogicFlashImage: async (imagePath: string): Promise<void> => {
    return await invokeTauri('cmd_amlogic_flash_image', { imagePath });
  },

  amlogicExtractImage: async (imagePath: string, outputDir: string): Promise<void> => {
    return await invokeTauri('cmd_amlogic_extract_image', { imagePath, outputDir });
  },

  checkCompatibility: async (profile: string, firmware: string, os?: string, version?: string, kernel?: string): Promise<CompatibilityReport> => {
    return await invokeTauri('cmd_check_compatibility', { profile, firmware, os, version, kernel });
  },

  planPatches: async (profile: string, firmware: string, os?: string, version?: string, kernel?: string): Promise<PatchPlanResponse> => {
    return await invokeTauri('cmd_plan_patches', { profile, firmware, os, version, kernel });
  },

  startBuild: async (profile: string, board: string, outputDir: string): Promise<void> => {
    return await invokeTauri('cmd_start_build', { profile, board, outputDir });
  },

  getSystemInfo: async (): Promise<SystemInfo> => {
    return await invokeTauri('cmd_get_system_info');
  },

  getSettings: async (): Promise<AppSettings> => {
    return await invokeTauri('cmd_get_settings');
  },

  saveSettings: async (settings: AppSettings): Promise<void> => {
    return await invokeTauri('cmd_save_settings', { settings });
  },

  forensicsDeepScan: async (device?: string): Promise<ForensicsReport> => {
    return await invokeTauri('cmd_forensics_deep_scan', { device });
  },

  securityScan: async (imagePath: string): Promise<SecurityReport> => {
    return await invokeTauri('cmd_security_scan', { imagePath });
  },

  listRemoteConfigs: async (): Promise<RemoteConfig[]> => {
    return await invokeTauri('cmd_list_remote_configs');
  },

  generateRemoteConf: async (name: string): Promise<string> => {
    return await invokeTauri('cmd_generate_remote_conf', { name });
  },

  getFirmwareRecommendations: async (profile: HardwareProfile): Promise<FirmwareRecommendation[]> => {
    return await invokeTauri('cmd_get_firmware_recommendations', { profile });
  },

  // Rockchip Commands
  rockchipDetect: async (): Promise<RockchipChipInfo> => {
    return await invokeTauri('cmd_rockchip_detect');
  },

  rockchipParseImage: async (imagePath: string): Promise<RkImageHeader> => {
    return await invokeTauri('cmd_rockchip_parse_image', { imagePath });
  },

  rockchipExtractImage: async (imagePath: string, outputDir: string): Promise<void> => {
    return await invokeTauri('cmd_rockchip_extract_image', { imagePath, outputDir });
  },

  rockchipParseParameter: async (content: string): Promise<RkParameter> => {
    return await invokeTauri('cmd_rockchip_parse_parameter', { content });
  },

  // Allwinner API
  allwinnerDetect: async (): Promise<AllwinnerVersion> => {
    return await invokeTauri('cmd_allwinner_detect');
  },

  allwinnerParseImage: async (imagePath: string): Promise<AllwinnerImageHeader> => {
    return await invokeTauri('cmd_allwinner_parse_image', { imagePath });
  },

  allwinnerFlashImage: async (imagePath: string): Promise<void> => {
    return await invokeTauri('cmd_allwinner_flash_image', { imagePath });
  },
};
