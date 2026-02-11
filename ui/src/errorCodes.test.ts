import { describe, it, expect } from 'vitest';
import { getAppErrorMessage, toAppError, isAppError, AppError } from './errorCodes';

describe('errorCodes', () => {
  describe('isAppError', () => {
    it('returns true for valid AppError objects', () => {
      const error: AppError = { code: 'UNKNOWN', message: 'test' };
      expect(isAppError(error)).toBe(true);
    });

    it('returns false for non-objects', () => {
      expect(isAppError(null)).toBe(false);
      expect(isAppError(undefined)).toBe(false);
      expect(isAppError('error')).toBe(false);
    });

    it('returns false for objects missing required properties', () => {
      expect(isAppError({})).toBe(false);
      expect(isAppError({ code: 'UNKNOWN' })).toBe(false);
      expect(isAppError({ message: 'test' })).toBe(false);
    });
  });

  describe('toAppError', () => {
    it('returns the input if it is already an AppError', () => {
      const error: AppError = { code: 'CONFIG_ERROR', message: 'Config missing' };
      expect(toAppError(error)).toBe(error);
    });

    it('converts Error objects to AppError', () => {
      const error = new Error('Something went wrong');
      const result = toAppError(error);
      expect(result).toEqual({ code: 'UNKNOWN', message: 'Something went wrong' });
    });

    it('converts plain objects with code/message to AppError', () => {
      const error = { code: 'IO_ERROR', message: 'Read failed' };
      const result = toAppError(error);
      expect(result).toEqual(error);
    });

    it('converts strings to AppError', () => {
      const result = toAppError('Something happened');
      expect(result).toEqual({ code: 'UNKNOWN', message: 'Something happened' });
    });
  });

  describe('getAppErrorMessage', () => {
    it('returns a formatted message for known error codes', () => {
      const error: AppError = { code: 'DEVICE_NOT_FOUND', message: 'Check USB cable' };
      expect(getAppErrorMessage(error)).toBe('Device not found. Check USB cable');
    });

    it('returns the raw message for UNKNOWN or unhandled codes', () => {
      const error: AppError = { code: 'UNKNOWN', message: 'Mysterious error' };
      // Default case in switch usually returns "message || 'Unknown error.'"
      // But verify specific implementation
      expect(getAppErrorMessage(error)).toBe('Mysterious error');
    });

    it('handles various error codes correctly', () => {
      expect(getAppErrorMessage({ code: 'CONFIG_ERROR', message: 'Invalid JSON' }))
        .toBe('Configuration error. Invalid JSON');

      expect(getAppErrorMessage({ code: 'NETWORK_ERROR', message: 'Offline' }))
        .toBe('Network error. Offline');
    });
  });
});
