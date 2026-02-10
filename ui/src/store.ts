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
    setupStatus: 'splash',
});

export { globalStore, setGlobalStore };
