import React, { useEffect, useMemo } from 'react';
import { useSettings } from '@/features/settings/hooks/useSettings';
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

/**
 * Converts hex to rgba for transparent borders and overlays.
 */
function hexToRgba(hex: string, opacity: number) {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${opacity})`;
}

/**
 * Blends two hex colors by a weight (0 = all color1, 1 = all color2).
 * Used to derive readable muted colors from foreground + background.
 */
function blendColor(color1: string, color2: string, weight: number) {
  const c1 = { r: parseInt(color1.slice(1, 3), 16), g: parseInt(color1.slice(3, 5), 16), b: parseInt(color1.slice(5, 7), 16) };
  const c2 = { r: parseInt(color2.slice(1, 3), 16), g: parseInt(color2.slice(3, 5), 16), b: parseInt(color2.slice(5, 7), 16) };
  const r = Math.round(c1.r * weight + c2.r * (1 - weight));
  const g = Math.round(c1.g * weight + c2.g * (1 - weight));
  const b = Math.round(c1.b * weight + c2.b * (1 - weight));
  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
}

export const AppThemeProvider: React.FC<AppThemeProviderProps> = ({ children }) => {
  const { data: settings } = useSettings();

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
    
    // UI Polish: Use RGBA for subtle borders instead of hard terminal colors
    const subtleBorder = hexToRgba(colors.foreground, 0.1);
    const subtleAccent = hexToRgba(colors.foreground, 0.05);

    setVar('--secondary', subtleAccent);
    setVar('--secondary-foreground', colors.foreground);
    
    setVar('--muted', subtleAccent);
    setVar('--muted-foreground', blendColor(colors.foreground, colors.background, 0.4));
    
    setVar('--accent', subtleAccent);
    setVar('--accent-foreground', colors.foreground);
    
    setVar('--border', subtleBorder);
    setVar('--input', subtleBorder);
    setVar('--ring', colors.blue);

    setVar('--sidebar', colors.background);
    setVar('--sidebar-foreground', colors.foreground);
    setVar('--sidebar-primary', colors.blue);
    setVar('--sidebar-primary-foreground', primaryForeground);
    setVar('--sidebar-accent', subtleAccent);
    setVar('--sidebar-accent-foreground', colors.foreground);
    setVar('--file-binary', colors.brightBlack);
    setVar('--file-symlink', colors.cyan);

    // Force .dark class based on background luminance
    const bgLum = getLuminance(colors.background);
    const isDark = bgLum < 0.5;
    root.classList.toggle('dark', isDark);
    }, [activeTerminalTheme]);

    return <>{children}</>;
    };
