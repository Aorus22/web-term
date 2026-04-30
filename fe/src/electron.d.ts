declare global {
  interface Window {
    electron?: {
      minimize: () => void;
      maximize: () => void;
      close: () => void;
      isElectron: boolean;
    }
  }
}

export {}
