import { createStore } from 'solid-js/store';
import { DetectedDevice } from './types';

interface GlobalStore {
    activeTab: string;
    sidebarCollapsed: boolean;
    lastDetected: DetectedDevice | null;
    detectedDevices: Record<string, DetectedDevice>;
    buildStatus: {
        inProgress: boolean;
        percent: number;
        currentStage: string;
        log: string[];
    };
    preferences: {
        theme: 'dark' | 'retro' | 'minimal';
        advancedMode: boolean;
        onboardingComplete: boolean;
    };
    setupStatus: 'splash' | 'onboarding' | 'ready';
}

const [globalStore, setGlobalStore] = createStore<GlobalStore>({
    activeTab: 'detect',
    sidebarCollapsed: false,
    lastDetected: null,
    detectedDevices: {},
    buildStatus: {
        inProgress: false,
        percent: 0,
        currentStage: '',
        log: [],
    },
    preferences: {
        theme: 'retro',
        advancedMode: false,
        onboardingComplete: false,
    },
    setupStatus: 'splash',
});

export { globalStore, setGlobalStore };
