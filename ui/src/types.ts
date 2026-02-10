export interface DeviceProfile {
  vendorId: number;
  productId: number;
  name: string;
  soc: string;
  ramMb: number;
  storageType: string;
  bootloaderOffset: number;
  supportedModes: string[];
}

export interface DetectedDevice {
  vendorId: number;
  productId: number;
  vendorName: string;
  socFamily: string;
  socModel?: string;
  model?: string;     // Added for generic model display
  chipId?: string;    // Added for specific chip ID (e.g., Rockchip)
  mode: 'Maskrom' | 'Adb' | 'Fastboot' | 'Fel' | 'Unknown';
  busNumber: number;
  deviceAddress: number;
}

export interface DeviceInfo {
  name: string;
  manufacturer: string;
  soc: string;
  variant: string;
}

export interface MemoryConfig {
  type: string;
  sizeMb: number;
  chip: string;
}

export interface StorageConfig {
  type: string;
  sizeGb: number;
  chip: string;
}

export interface WifiConfig {
  chip: string;
  driver: string;
  firmware: string;
  nvram: string;
}

export interface EthernetConfig {
  type: string;
  speed: string;
}

export interface Capabilities {
  gpu?: string | null;
  vpu?: string | null;
  wifiSupported: boolean;
  ethernetSupported: boolean;
  hdmiCecSupported: boolean;
  hasEmmc: boolean;
  hasSdSlot: boolean;
}

export interface HardwareConfig {
  memory: MemoryConfig;
  storage: StorageConfig;
  wifi?: WifiConfig | null;
  ethernet?: EthernetConfig | null;
  capabilities?: Capabilities | null;
}

export interface UartConfig {
  port: string;
  baud: number;
}

export interface BootConfig {
  secureBoot: boolean;
  referenceDtb: string;
  uart?: UartConfig | null;
}

export interface BuildProfile {
  rootfs: string;
  kernel: string;
  packages: string[];
}

export interface BuildConfig {
  buildrootDefconfig: string;
  kernelFragments: string[];
  ubootConfig: string;
}

export interface DeviceConfig {
  device: DeviceInfo;
  hardware: HardwareConfig;
  boot: BootConfig;
  profiles: Record<string, BuildProfile>;
  build: BuildConfig;
}

export type CompatibilityStatus = 'Works' | 'WorksWithPatches' | 'Broken' | 'Untested';

export type KnownIssue =
  | 'Wifi5GhzIntermittent'
  | 'HdmiCecNoAudio'
  | 'DdrTrainingFail'
  | 'GpuKernelPanic'
  | 'GpuBlobMissing'
  | 'WifiCalibrationMissing';

export type PatchId =
  | 'BrcmfmacFix5Ghz'
  | 'HdmiCecAudioWorkaround'
  | 'MaliBlobExtract'
  | 'Ap6212NvramFix';

export interface HardwareProfile {
  soc: string;
  pcbVariant: string;
  ramVendor: string;
  wifiChip: string;
  emmcVendor: string;
  hdmiPhy: string;
}

export interface FirmwareTarget {
  osType: string;
  version: string;
  kernel: string;
}

export interface CompatibilityReport {
  hardware: HardwareProfile;
  firmware: FirmwareTarget;
  status: CompatibilityStatus;
  issues: KnownIssue[];
  requiredPatches: PatchId[];
  confidence: number;
}

export interface PatchPlanStep {
  step: number;
  patch: PatchId;
  description: string;
}

export interface PatchPlan {
  steps: PatchPlanStep[];
  riskLevel: string;
  successProbability: number;
}

export interface PatchPlanResponse {
  report: CompatibilityReport;
  plan: PatchPlan;
}

export type Phase =
  | 'Detect'
  | 'Backup'
  | 'Unlock'
  | 'Extract'
  | 'Build'
  | 'Flash'
  | 'Validate'
  | 'Check'
  | 'PatchPlan';

export type PhaseStatus = 'Started' | 'Completed' | 'Failed';

export interface WorkflowPhaseEvent {
  phase: Phase;
  status: PhaseStatus;
  detail?: string | null;
}

export interface AppSettings {
  toolsPath: string;
  cachePath: string;
  outputPath: string;
  language: string;
  themeMode: string;
  themeColor: string;
  uiScale: string;
  typography: string;
}

export interface BuildProgress {
  step: string;
  progress: number;
  message: string;
  logLine?: string;
}

export interface SystemInfo {
  os: string;
  arch: string;
  rustAvailable: boolean;
  hasUsbAccess: boolean;
}

export interface AmlogicChipInfo {
  chipId: string;
  romVersion: number;
  protocolVersion: number;
  secureBoot: boolean;
  ramSize: number;
  ddrType: string;
}

export interface FlashProgress {
  operation: string;
  partition?: string;
  percent: number;
  bytesTransferred: number;
  totalBytes: number;
  speedBps: number;
}

export interface FirmwareRecommendation {
  name: string;
  version: string;
  url: string;
  notes: string;
}

// Allwinner Types
export interface AllwinnerVersion {
  socId: number;
  socName: string;
  protocolVersion: number;
  scratchpad: number;
  sramBase: number;
  sramSize: number;
}

export interface AllwinnerImageItem {
  name: string;
  path: string;
  offset: number;
  size: number;
  loadAddress: number;
}

export interface AllwinnerImageHeader {
  magic: string;
  version: number;
  platform: string;
  items: AllwinnerImageItem[];
}

// Rockchip Types
export interface RkPartition {
  name: string;
  sizeSectors: number;
  offsetSectors: number;
  grow: boolean;
}

export interface RkParameter {
  firmwareVer: string;
  machineModel: string;
  machineId: string;
  manufacturer: string;
  cmdline: string;
  partitions: RkPartition[];
}

// Security Types
export type ThreatLevel = 'Critical' | 'High' | 'Medium' | 'Low' | 'Info';

export interface ThreatDetection {
  name: string;
  severity: ThreatLevel;
  path: string;
  description: string;
  remediation: string;
}

export type ScanType = 'Image' | 'LiveDevice' | 'Backup';

export interface SecurityReport {
  isInfected: boolean;
  threats: ThreatDetection[];
  recommendations: string[];
  scanPath: string;
  scanType: ScanType;
}

// Rockchip Additional Types
export interface RockchipChipInfo {
  chipId: string;
  flashType: string;
  flashSize: number;
  bootRomVersion: string;
  loaderVersion?: string | null;
  isMaskrom: boolean;
}

export interface RkImageEntry {
  name: string;
  path: string;
  offset: number;
  size: number;
  fileSize: number;
}

export interface RkImageHeader {
  magic: string;
  manufacturer: string;
  model: string;
  version: string;
  chipFamily: string;
  entries: RkImageEntry[];
}

// Forensics Types
export type DeviceMode = 'Maskrom' | 'Adb' | 'Fastboot' | 'Fel' | 'Unknown';

export interface UartDetection {
  port: string;
  baud: number;
  bootloader?: string | null;
  isResponding: boolean;
}

export interface DdrTiming {
  vendor: string;
  speed: string;
  timingPattern: string;
  sizeMb: number;
  compatibleDtbs: string[];
}

export interface BootloaderInfo {
  version: string;
  bootloaderType: string;
  secureBoot: boolean;
  bl2Signed: boolean;
  unlockPossible: boolean;
  notes: string[];
}

export interface WifiChipInfo {
  chip: string;
  vendor: string;
  sdioVid?: number | null;
  sdioDid?: number | null;
  mainlineDriver: boolean;
  firmwareFiles: string[];
  nvramPath?: string | null;
}

export interface EmmcInfo {
  vendor: string;
  version: string;
  capacityGb: number;
  manufacturerId?: number | null;
  model?: string | null;
}

export type PcbVariant = 'P281' | 'P282' | 'EVB' | 'GENERIC';

export interface ForensicsReport {
  targetDevice?: string | null;
  usbDevices: DetectedDevice[];
  uartPorts: string[];
  uartProbe?: UartDetection | null;
  ddrTiming?: DdrTiming | null;
  bootloader?: BootloaderInfo | null;
  wifiChip?: WifiChipInfo | null;
  emmcInfo?: EmmcInfo | null;
  pcbVariant?: PcbVariant | null;
  variantId?: string | null;
}

// Remote Config Types
export type LinuxKeyCode =
  | 'KEY_POWER'
  | 'KEY_MUTE'
  | 'KEY_VOLUMEUP'
  | 'KEY_VOLUMEDOWN'
  | 'KEY_UP'
  | 'KEY_DOWN'
  | 'KEY_LEFT'
  | 'KEY_RIGHT'
  | 'KEY_OK'
  | 'KEY_ENTER'
  | 'KEY_BACK'
  | 'KEY_HOME'
  | 'KEY_MENU'
  | 'KEY_INFO'
  | 'KEY_CHANNELUP'
  | 'KEY_CHANNELDOWN'
  | 'KEY_1'
  | 'KEY_2'
  | 'KEY_3'
  | 'KEY_4'
  | 'KEY_5'
  | 'KEY_6'
  | 'KEY_7'
  | 'KEY_8'
  | 'KEY_9'
  | 'KEY_0'
  | 'KEY_RED'
  | 'KEY_GREEN'
  | 'KEY_YELLOW'
  | 'KEY_BLUE'
  | 'KEY_PLAY'
  | 'KEY_PAUSE'
  | 'KEY_STOP'
  | 'KEY_REWIND'
  | 'KEY_FASTFORWARD'
  | 'KEY_RECORD'
  | 'KEY_PREVIOUS'
  | 'KEY_NEXT'
  | 'KEY_SUBTITLE'
  | 'KEY_AUDIO'
  | 'KEY_TV'
  | 'KEY_EPG'
  | 'KEY_SLEEP'
  | 'KEY_SETUP'
  | 'KEY_FAVORITES';

export type RemoteSource =
  | 'coreElec'
  | 'libreElec'
  | 'slimBoxTv'
  | 'aidansRom'
  | 'community'
  | 'custom';

export interface RemoteConfig {
  name: string;
  factoryCode: number;
  protocol: string;
  repeatPeriod: number;
  releaseDelay: number;
  keymaps: Record<string, LinuxKeyCode>;
  source: RemoteSource;
  compatibleDevices: string[];
}
