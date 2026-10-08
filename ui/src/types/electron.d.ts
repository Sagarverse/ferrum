/** Type declarations for Electron preload bridge */
interface FerrumBridge {
  selectFolder: () => Promise<string | null>;
  selectFile: () => Promise<string | null>;
  getPlatform: () => Promise<string>;
  openDataDir: () => Promise<void>;
  api: (method: string, path: string, body?: any) => Promise<any>;
  subscribeEvents: (options?: any) => Promise<() => void>;
}

interface Window {
  ferrum?: FerrumBridge;
  metallic?: FerrumBridge;
}
