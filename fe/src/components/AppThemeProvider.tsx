import React, { useEffect, useMemo } from 'react';
import { useSettings } from '@/features/settings/hooks/useSettings';
import { useTheme } from '@/hooks/use-theme';
import { terminalThemes } from '@/features/settings/data/terminal-themes';

interface AppThemeProviderProps {
  children: React.ReactNode;
}

/**
 * Utility to calculate relative luminance of a hex color.
 * Used to determine if we should use light or dark text on top of a color.
 */
function getLuminance(hex: string) {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;

  const a = [r, g, b].map(v => {
    return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
  });
  return a[0] * 0.2126 + a[1] * 0.7152 + a[2] * 0.0722;
}

/**
 * Validates if a string is a valid hex color to prevent CSS injection.
 */
function isValidHex(color: string) {
  return /^#[0-9A-F]{6}$/i.test(color);
}

export const AppThemeProvider: React.FC<AppThemeProviderProps> = ({ children }) => {
  const { data: settings } = useSettings();
  const { theme: themeMode } = useTheme();

  const activeTerminalTheme = useMemo(() => {
    const themeName = settings?.terminal_color_theme || 'default';
    return terminalThemes.find(t => t.name === themeName) || terminalThemes[0];
  }, [settings?.terminal_color_theme]);

  useEffect(() => {
    const root = document.documentElement;
    const colors = activeTerminalTheme.colors;

    // Helper to set variable only if valid
    const setVar = (name: string, value: string) => {
      if (isValidHex(value)) {
        root.style.setProperty(name, value);
      }
    };

    // Calculate primary-foreground based on blue color luminance
    const primaryLum = getLuminance(colors.blue);
    const primaryForeground = primaryLum > 0.5 ? '#000000' : '#ffffff';

    // Map terminal colors to Shadcn/Tailwind CSS variables
    setVar('--background', colors.background);
    setVar('--foreground', colors.foreground);
    
    setVar('--card', colors.background);
    setVar('--card-foreground', colors.foreground);
    
    setVar('--popover', colors.background);
    setVar('--popover-foreground', colors.foreground);
    
    setVar('--primary', colors.blue);
    setVar('--primary-foreground', primaryForeground);
    
    // Secondary/Muted often use the 'black' color from terminal palettes (usually a variant of background)
    setVar('--secondary', colors.black);
    setVar('--secondary-foreground', colors.foreground);
    
    setVar('--muted', colors.black);
    setVar('--muted-foreground', colors.brightBlack);
    
    setVar('--accent', colors.brightBlack);
    setVar('--accent-foreground', colors.foreground);
    
    setVar('--border', colors.brightBlack);
    setVar('--input', colors.brightBlack);
    setVar('--ring', colors.blue);

    setVar('--sidebar', colors.background);
    setVar('--sidebar-foreground', colors.foreground);
    setVar('--sidebar-primary', colors.blue);
    setVar('--sidebar-primary-foreground', primaryForeground);
    setVar('--sidebar-accent', colors.brightBlack);
    setVar('--sidebar-accent-foreground', colors.foreground);
    setVar('--sidebar-border', colors.brightBlack);
    setVar('--sidebar-ring', colors.blue);

    // Force .dark class based on background luminance if theme is 'system'
    // This ensures that components relying on the .dark class for non-variable styles 
    // (like scrollbars or specific images) stay in sync with the terminal theme.
    const bgLum = getLuminance(colors.background);
    const isDark = bgLum < 0.5;
    
    root.classList.toggle('dark', isDark);

    return () => {
      // Clean up variables when unmounting (though this component usually lives for app lifetime)
      const vars = [
        '--background', '--foreground', '--card', '--card-foreground',
        '--popover', '--popover-foreground', '--primary', '--primary-foreground',
        '--secondary', '--secondary-foreground', '--muted', '--muted-foreground',
        '--accent', '--accent-foreground', '--border', '--input', '--ring',
        '--sidebar', '--sidebar-foreground', '--sidebar-primary', '--sidebar-primary-foreground',
        '--sidebar-accent', '--sidebar-accent-foreground', '--sidebar-border', '--sidebar-ring'
      ];
      vars.forEach(v => root.style.removeProperty(v));
    };
  }, [activeTerminalTheme]);

  return <>{children}</>;
};
