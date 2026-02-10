export type AppErrorCode =
  | 'CONFIG_ERROR'
  | 'HARDWARE_ERROR'
  | 'IO_ERROR'
  | 'NETWORK_ERROR'
  | 'VALIDATION_ERROR'
  | 'COMMAND_FAILED'
  | 'DEVICE_NOT_FOUND'
  | 'BUILD_FAILED'
  | 'SETTINGS_LOAD_FAILED'
  | 'SETTINGS_SAVE_FAILED'
  | 'ASSET_BASE_URL_MISSING'
  | 'UNKNOWN'
  | (string & {});

export interface AppError {
  code: AppErrorCode;
  message: string;
}

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null;

export const isAppError = (value: unknown): value is AppError =>
  isRecord(value) && typeof value.code === 'string' && typeof value.message === 'string';

export const toAppError = (error: unknown): AppError => {
  if (isAppError(error)) {
    return error;
  }

  if (error instanceof Error) {
    return { code: 'UNKNOWN', message: error.message };
  }

  if (isRecord(error)) {
    const code = typeof error.code === 'string' ? error.code : 'UNKNOWN';
    const message = typeof error.message === 'string' ? error.message : JSON.stringify(error);
    return { code, message };
  }

  return { code: 'UNKNOWN', message: String(error) };
};

export const getAppErrorMessage = (error: unknown): string => {
  const appError = toAppError(error);

  switch (appError.code) {
    case 'ASSET_BASE_URL_MISSING':
      return `Asset base URL missing. Set ${appError.message}.`;
    case 'SETTINGS_LOAD_FAILED':
      return `Failed to load settings. ${appError.message}`;
    case 'SETTINGS_SAVE_FAILED':
      return `Failed to save settings. ${appError.message}`;
    case 'BUILD_FAILED':
      return `Build failed. ${appError.message}`;
    case 'DEVICE_NOT_FOUND':
      return `Device not found. ${appError.message}`;
    case 'VALIDATION_ERROR':
      return `Validation failed. ${appError.message}`;
    case 'CONFIG_ERROR':
      return `Configuration error. ${appError.message}`;
    case 'HARDWARE_ERROR':
      return `Hardware error. ${appError.message}`;
    case 'IO_ERROR':
      return `I/O error. ${appError.message}`;
    case 'NETWORK_ERROR':
      return `Network error. ${appError.message}`;
    case 'COMMAND_FAILED':
      return `Command failed. ${appError.message}`;
    default:
      return appError.message || 'Unknown error.';
  }
};
