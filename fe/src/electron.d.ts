declare global {
  interface Window {
    electron?: {
      minimize: () => void;
      maximize: () => void;
      close: () => void;
      getWindowState: () => Promise<'maximized' | 'restored'>;
      onWindowStateChange: (callback: (state: 'maximized' | 'restored') => void) => void;
      isElectron: boolean;
      platform: string;
    }
  }
}

export {}
