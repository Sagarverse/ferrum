/** Type declarations for Electron preload bridge */
interface MetallicBridge {
  selectFolder: () => Promise<string | null>;
  selectFile: () => Promise<string | null>;
  getPlatform: () => Promise<string>;
}

interface Window {
  metallic?: MetallicBridge;
}
