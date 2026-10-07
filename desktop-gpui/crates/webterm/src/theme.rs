//! Theme management and multi-theme presets mirroring ../web-tmux fe/ client
//! (`fe/src/features/settings/data/ui-themes.ts`).

use gpui::{rgb, rgba, App, Hsla, Rgba};
use gpui_component::{Theme, ThemeColor, ThemeMode};
use webterm_settings::Theme as SettingsTheme;

/// Complete theme color palette mirroring web client ThemePreset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemePreset {
    pub id: &'static str,
    pub label: &'static str,
    pub is_dark: bool,
    pub background: u32,
    pub foreground: u32,
    pub card: u32,
    pub card_foreground: u32,
    pub primary: u32,
    pub primary_foreground: u32,
    pub secondary: u32,
    pub secondary_foreground: u32,
    pub muted: u32,
    pub muted_foreground: u32,
    pub accent: u32,
    pub accent_foreground: u32,
    pub destructive: u32,
    pub destructive_foreground: u32,
    pub border: u32,
    pub input: u32,
    pub ring: u32,
}

impl ThemePreset {
    pub fn bg(&self) -> Rgba {
        rgb(self.background)
    }
    pub fn fg(&self) -> Rgba {
        rgb(self.foreground)
    }
    pub fn card_bg(&self) -> Rgba {
        rgb(self.card)
    }
    pub fn card_fg(&self) -> Rgba {
        rgb(self.card_foreground)
    }
    pub fn primary(&self) -> Rgba {
        rgb(self.primary)
    }
    pub fn primary_fg(&self) -> Rgba {
        rgb(self.primary_foreground)
    }
    pub fn secondary(&self) -> Rgba {
        rgb(self.secondary)
    }
    pub fn secondary_fg(&self) -> Rgba {
        rgb(self.secondary_foreground)
    }
    pub fn muted(&self) -> Rgba {
        rgb(self.muted)
    }
    pub fn muted_fg(&self) -> Rgba {
        rgb(self.muted_foreground)
    }
    pub fn accent(&self) -> Rgba {
        rgb(self.accent)
    }
    pub fn accent_fg(&self) -> Rgba {
        rgb(self.accent_foreground)
    }
    pub fn destructive(&self) -> Rgba {
        rgb(self.destructive)
    }
    pub fn border(&self) -> Rgba {
        rgb(self.border)
    }
}

/// Preset id / label the desktop GTK palette reports as its own.
pub const GTK_PRESET_ID: &str = "gtk";
pub const GTK_PRESET_LABEL: &str = "Desktop (GTK)";

/// The desktop (GTK) palette, resolved from the running theme.
///
/// Unlike [`ThemePreset`] this is *dynamic*: every field is a plain `0xRRGGBB`
/// value produced by [`crate::gtk_theme`] from the user's `~/.config/gtk-*/`
/// CSS (falling back to the GTK3 style engine). It is `Copy` + `Send` so render
/// paths only ever read cached data — GTK itself is never touched there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GtkPalette {
    pub is_dark: bool,
    pub background: u32,
    pub foreground: u32,
    pub card: u32,
    pub card_foreground: u32,
    pub primary: u32,
    pub primary_foreground: u32,
    pub secondary: u32,
    pub secondary_foreground: u32,
    pub muted: u32,
    pub muted_foreground: u32,
    pub accent: u32,
    pub accent_foreground: u32,
    pub destructive: u32,
    /// Status hues (`error_color` / `warning_color` / `success_color`).
    pub warning: u32,
    pub success: u32,
    pub border: u32,
}

impl GtkPalette {
    /// The same palette as a [`ThemePreset`], so every app accessor that already
    /// reads `current_theme()` follows the desktop with no extra plumbing.
    pub fn to_preset(&self) -> ThemePreset {
        ThemePreset {
            id: GTK_PRESET_ID,
            label: GTK_PRESET_LABEL,
            is_dark: self.is_dark,
            background: self.background,
            foreground: self.foreground,
            card: self.card,
            card_foreground: self.card_foreground,
            primary: self.primary,
            primary_foreground: self.primary_foreground,
            secondary: self.secondary,
            secondary_foreground: self.secondary_foreground,
            muted: self.muted,
            muted_foreground: self.muted_foreground,
            accent: self.accent,
            accent_foreground: self.accent_foreground,
            destructive: self.destructive,
            destructive_foreground: contrast_hex(self.destructive),
            // The desktop palette carries no dedicated input/ring tokens: the
            // field keeps the card's border and the ring follows the accent,
            // matching what `apply_gtk_to_component` used to derive.
            border: self.border,
            input: self.border,
            ring: self.accent,
        }
    }
}

// --- Colour helpers used by the GTK palette mapping (pure; unit-tested) ---

/// `0xRRGGBB` (alpha dropped) from an 8-bit-per-channel colour.
pub fn hex_from_rgba(color: Rgba) -> u32 {
    let channel = |v: f32| ((v.clamp(0.0, 1.0) * 255.0).round() as u32) & 0xff;
    (channel(color.r) << 16) | (channel(color.g) << 8) | channel(color.b)
}

/// Linear interpolation between two `0xRRGGBB` values (`t = 0` → `a`).
pub fn mix_hex(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |shift: u32| {
        let ca = ((a >> shift) & 0xff) as f32;
        let cb = ((b >> shift) & 0xff) as f32;
        ((ca + (cb - ca) * t).round() as u32) & 0xff
    };
    (lerp(16) << 16) | (lerp(8) << 8) | lerp(0)
}

/// Rec. 709 relative luminance of an `0xRRGGBB` colour (0 = black, 1 = white).
pub fn hex_luminance(hex: u32) -> f32 {
    let srgb_to_linear = |component: f32| {
        if component <= 0.04045 {
            component / 12.92
        } else {
            ((component + 0.055) / 1.055).powf(2.4)
        }
    };
    let c = |shift: u32| srgb_to_linear(((hex >> shift) & 0xff) as f32 / 255.0);
    0.2126 * c(16) + 0.7152 * c(8) + 0.0722 * c(0)
}

/// Readable foreground on top of `hex` (black on light, white on dark).
pub fn contrast_hex(hex: u32) -> u32 {
    if hex_luminance(hex) > 0.45 {
        0x000000
    } else {
        0xffffff
    }
}

/// WCAG contrast ratio between two `0xRRGGBB` colours (1.0 – 21.0).
pub fn hex_contrast(a: u32, b: u32) -> f32 {
    let (l1, l2) = (hex_luminance(a), hex_luminance(b));
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

/// Minimum contrast ratio secondary ("muted") text must hold against the
/// surfaces it is painted on. Below this the text reads as invisible — the
/// complaint that drove the gallery rework — while 4.5:1 (WCAG body text)
/// would repaint half the gallery and flatten each theme's dimmed look.
pub const MIN_MUTED_CONTRAST: f32 = 2.4;

/// Nudge `muted` toward `fg` (preserving its hue) until it reaches
/// [`MIN_MUTED_CONTRAST`] against every surface in `surfaces`.
///
/// The smallest blend step (5%) that satisfies all surfaces wins, so themes
/// already at or above the floor come back untouched and borderline ones move
/// barely perceptibly (`#565F89` → `#5B648E`). The floor test over
/// [`THEME_PRESETS`] keeps hand-edited literals honest.
pub fn readable_muted(muted: u32, surfaces: &[u32], fg: u32) -> u32 {
    if surfaces.iter().all(|s| hex_contrast(muted, *s) >= MIN_MUTED_CONTRAST) {
        return muted;
    }
    for step in 1..=20 {
        let t = step as f32 / 20.0;
        let candidate = mix_hex(muted, fg, t);
        if surfaces
            .iter()
            .all(|s| hex_contrast(candidate, *s) >= MIN_MUTED_CONTRAST)
        {
            return candidate;
        }
    }
    fg
}

/// The full preset gallery, ported verbatim from web-tmux
/// (fe/src/features/settings/data/ui-themes.ts): every dark theme ships a
/// light counterpart, and `input` / `ring` / `destructive_foreground` carry
/// the web client's exact tokens. One deliberate deviation: where the
/// source's `mutedForeground` has less than 2.4:1 contrast on card/bg (16
/// presets — mint-light, one-dark, kanagawa-dark, ...) it is nudged toward
/// `foreground` until it reads; see [`readable_muted`].
pub const THEME_PRESETS: &[ThemePreset] = &[
    ThemePreset {
        id: "default-dark",
        label: "Default",
        is_dark: true,
        background: 0x1e1e1e,
        foreground: 0xd4d4d4,
        card: 0x1e1e1e,
        card_foreground: 0xd4d4d4,
        primary: 0xd4d4d4,
        primary_foreground: 0x1e1e1e,
        secondary: 0x2d2d2d,
        secondary_foreground: 0xd4d4d4,
        muted: 0x2d2d2d,
        muted_foreground: 0x808080,
        accent: 0x2d2d2d,
        accent_foreground: 0xd4d4d4,
        destructive: 0x7f1d1d,
        destructive_foreground: 0xffffff,
        border: 0x3c3c3c,
        input: 0x3c3c3c,
        ring: 0x808080,
    },
    ThemePreset {
        id: "default-light",
        label: "Default",
        is_dark: false,
        background: 0xfafafa,
        foreground: 0x383a42,
        card: 0xffffff,
        card_foreground: 0x383a42,
        primary: 0x383a42,
        primary_foreground: 0xffffff,
        secondary: 0xf0f0f0,
        secondary_foreground: 0x383a42,
        muted: 0xf0f0f0,
        muted_foreground: 0xa0a1a7,
        accent: 0xf0f0f0,
        accent_foreground: 0x383a42,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xdbdbdc,
        input: 0xdbdbdc,
        ring: 0xa0a1a7,
    },
    ThemePreset {
        id: "ocean-dark",
        label: "Ocean",
        is_dark: true,
        background: 0x0c1929,
        foreground: 0xffffff,
        card: 0x162940,
        card_foreground: 0xffffff,
        primary: 0x2563eb,
        primary_foreground: 0xffffff,
        secondary: 0x162940,
        secondary_foreground: 0xffffff,
        muted: 0x1e2b3b,
        muted_foreground: 0x94a3b8,
        accent: 0x1e3a5f,
        accent_foreground: 0xffffff,
        destructive: 0x7f1d1d,
        destructive_foreground: 0xffffff,
        border: 0x274768,
        input: 0x274768,
        ring: 0x2563eb,
    },
    ThemePreset {
        id: "ocean-light",
        label: "Ocean",
        is_dark: false,
        background: 0xf0f9ff,
        foreground: 0x0f172a,
        card: 0xffffff,
        card_foreground: 0x0f172a,
        primary: 0x2563eb,
        primary_foreground: 0xffffff,
        secondary: 0xeff6ff,
        secondary_foreground: 0x0f172a,
        muted: 0xe3ecf2,
        muted_foreground: 0x64748b,
        accent: 0xeff6ff,
        accent_foreground: 0x0f172a,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xbfdbfe,
        input: 0xbfdbfe,
        ring: 0x3b82f6,
    },
    ThemePreset {
        id: "forest-dark",
        label: "Forest",
        is_dark: true,
        background: 0x0c1f17,
        foreground: 0xffffff,
        card: 0x173025,
        card_foreground: 0xffffff,
        primary: 0x059669,
        primary_foreground: 0xffffff,
        secondary: 0x173025,
        secondary_foreground: 0xffffff,
        muted: 0x1e3129,
        muted_foreground: 0x94a3b8,
        accent: 0x064e3b,
        accent_foreground: 0xffffff,
        destructive: 0x7f1d1d,
        destructive_foreground: 0xffffff,
        border: 0x1f4a32,
        input: 0x1f4a32,
        ring: 0x059669,
    },
    ThemePreset {
        id: "forest-light",
        label: "Forest",
        is_dark: false,
        background: 0xf0fdf4,
        foreground: 0x0f172a,
        card: 0xffffff,
        card_foreground: 0x0f172a,
        primary: 0x059669,
        primary_foreground: 0xffffff,
        secondary: 0xecfdf5,
        secondary_foreground: 0x0f172a,
        muted: 0xe3f0e7,
        muted_foreground: 0x64748b,
        accent: 0xecfdf5,
        accent_foreground: 0x0f172a,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xa7f3d0,
        input: 0xa7f3d0,
        ring: 0x10b981,
    },
    ThemePreset {
        id: "lavender-dark",
        label: "Lavender",
        is_dark: true,
        background: 0x131127,
        foreground: 0xffffff,
        card: 0x1f1b3d,
        card_foreground: 0xffffff,
        primary: 0x7c3aed,
        primary_foreground: 0xffffff,
        secondary: 0x1f1b3d,
        secondary_foreground: 0xffffff,
        muted: 0x252339,
        muted_foreground: 0x94a3b8,
        accent: 0x4c1d95,
        accent_foreground: 0xffffff,
        destructive: 0x7f1d1d,
        destructive_foreground: 0xffffff,
        border: 0x362d5c,
        input: 0x362d5c,
        ring: 0x7c3aed,
    },
    ThemePreset {
        id: "lavender-light",
        label: "Lavender",
        is_dark: false,
        background: 0xfaf5ff,
        foreground: 0x0f172a,
        card: 0xffffff,
        card_foreground: 0x0f172a,
        primary: 0x7c3aed,
        primary_foreground: 0xffffff,
        secondary: 0xf5f3ff,
        secondary_foreground: 0x0f172a,
        muted: 0xede8f2,
        muted_foreground: 0x64748b,
        accent: 0xf5f3ff,
        accent_foreground: 0x0f172a,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xc4b5fd,
        input: 0xc4b5fd,
        ring: 0x8b5cf6,
    },
    ThemePreset {
        id: "cherry-dark",
        label: "Cherry",
        is_dark: true,
        background: 0x1a0f14,
        foreground: 0xffffff,
        card: 0x271a22,
        card_foreground: 0xffffff,
        primary: 0xe11d48,
        primary_foreground: 0xffffff,
        secondary: 0x271a22,
        secondary_foreground: 0xffffff,
        muted: 0x2c2126,
        muted_foreground: 0x94a3b8,
        accent: 0x4c0519,
        accent_foreground: 0xffffff,
        destructive: 0x7f1d1d,
        destructive_foreground: 0xffffff,
        border: 0x442838,
        input: 0x442838,
        ring: 0xe11d48,
    },
    ThemePreset {
        id: "cherry-light",
        label: "Cherry",
        is_dark: false,
        background: 0xfff5f5,
        foreground: 0x0f172a,
        card: 0xffffff,
        card_foreground: 0x0f172a,
        primary: 0xe11d48,
        primary_foreground: 0xffffff,
        secondary: 0xfff1f2,
        secondary_foreground: 0x0f172a,
        muted: 0xf2e8e8,
        muted_foreground: 0x64748b,
        accent: 0xfff1f2,
        accent_foreground: 0x0f172a,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xfecdd3,
        input: 0xfecdd3,
        ring: 0xf43f5e,
    },
    ThemePreset {
        id: "sunset-dark",
        label: "Sunset",
        is_dark: true,
        background: 0x19140d,
        foreground: 0xffffff,
        card: 0x262016,
        card_foreground: 0xffffff,
        primary: 0xb45309,
        primary_foreground: 0xffffff,
        secondary: 0x262016,
        secondary_foreground: 0xffffff,
        muted: 0x2b261f,
        muted_foreground: 0x94a3b8,
        accent: 0x78350f,
        accent_foreground: 0xffffff,
        destructive: 0x7f1d1d,
        destructive_foreground: 0xffffff,
        border: 0x403828,
        input: 0x403828,
        ring: 0xb45309,
    },
    ThemePreset {
        id: "sunset-light",
        label: "Sunset",
        is_dark: false,
        background: 0xfff7ed,
        foreground: 0x0f172a,
        card: 0xffffff,
        card_foreground: 0x0f172a,
        primary: 0xd97706,
        primary_foreground: 0xffffff,
        secondary: 0xfffbeb,
        secondary_foreground: 0x0f172a,
        muted: 0xf2eae0,
        muted_foreground: 0x64748b,
        accent: 0xfffbeb,
        accent_foreground: 0x0f172a,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xfde68a,
        input: 0xfde68a,
        ring: 0xf59e0b,
    },
    ThemePreset {
        id: "mono-dark",
        label: "Monochrome",
        is_dark: true,
        background: 0x09090b,
        foreground: 0xfafafa,
        card: 0x18181b,
        card_foreground: 0xfafafa,
        primary: 0xa1a1aa,
        primary_foreground: 0x18181b,
        secondary: 0x27272a,
        secondary_foreground: 0xfafafa,
        muted: 0x1b1b1d,
        muted_foreground: 0xa1a1aa,
        accent: 0x3f3f46,
        accent_foreground: 0xfafafa,
        destructive: 0x7f1d1d,
        destructive_foreground: 0xfafafa,
        border: 0x27272a,
        input: 0x27272a,
        ring: 0xa1a1aa,
    },
    ThemePreset {
        id: "mono-light",
        label: "Monochrome",
        is_dark: false,
        background: 0xfafafa,
        foreground: 0x09090b,
        card: 0xffffff,
        card_foreground: 0x09090b,
        primary: 0x52525b,
        primary_foreground: 0xffffff,
        secondary: 0xf4f4f5,
        secondary_foreground: 0x09090b,
        muted: 0xededed,
        muted_foreground: 0x71717a,
        accent: 0xf4f4f5,
        accent_foreground: 0x09090b,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xe4e4e7,
        input: 0xe4e4e7,
        ring: 0x52525b,
    },
    ThemePreset {
        id: "mint-dark",
        label: "Mint",
        is_dark: true,
        background: 0x022c22,
        foreground: 0xffffff,
        card: 0x064e3b,
        card_foreground: 0xffffff,
        primary: 0x34d399,
        primary_foreground: 0x022c22,
        secondary: 0x064e3b,
        secondary_foreground: 0xffffff,
        muted: 0x143e34,
        muted_foreground: 0x6ee7b7,
        accent: 0x047857,
        accent_foreground: 0xffffff,
        destructive: 0x7f1d1d,
        destructive_foreground: 0xffffff,
        border: 0x065f46,
        input: 0x065f46,
        ring: 0x34d399,
    },
    ThemePreset {
        id: "mint-light",
        label: "Mint",
        is_dark: false,
        background: 0xf0fdfa,
        foreground: 0x0f172a,
        card: 0xffffff,
        card_foreground: 0x0f172a,
        primary: 0x0d9488,
        primary_foreground: 0xffffff,
        secondary: 0xccfbf1,
        secondary_foreground: 0x0f172a,
        muted: 0xe3f0ed,
        muted_foreground: 0x46aba1,
        accent: 0xccfbf1,
        accent_foreground: 0x0f172a,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0x99f6e4,
        input: 0x99f6e4,
        ring: 0x14b8a6,
    },
    ThemePreset {
        id: "rose-dark",
        label: "Rose",
        is_dark: true,
        background: 0x1c0d11,
        foreground: 0xffffff,
        card: 0x2d1520,
        card_foreground: 0xffffff,
        primary: 0xf43f5e,
        primary_foreground: 0xffffff,
        secondary: 0x2d1520,
        secondary_foreground: 0xffffff,
        muted: 0x2e1f23,
        muted_foreground: 0xfda4af,
        accent: 0x881337,
        accent_foreground: 0xffffff,
        destructive: 0x7f1d1d,
        destructive_foreground: 0xffffff,
        border: 0x4a1d30,
        input: 0x4a1d30,
        ring: 0xf43f5e,
    },
    ThemePreset {
        id: "rose-light",
        label: "Rose",
        is_dark: false,
        background: 0xfff1f2,
        foreground: 0x0f172a,
        card: 0xffffff,
        card_foreground: 0x0f172a,
        primary: 0xe11d48,
        primary_foreground: 0xffffff,
        secondary: 0xffe4e6,
        secondary_foreground: 0x0f172a,
        muted: 0xf2e4e5,
        muted_foreground: 0xcd8894,
        accent: 0xffe4e6,
        accent_foreground: 0x0f172a,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xfecdd3,
        input: 0xfecdd3,
        ring: 0xf43f5e,
    },
    ThemePreset {
        id: "amber-dark",
        label: "Amber",
        is_dark: true,
        background: 0x1c1408,
        foreground: 0xffffff,
        card: 0x2b1f0e,
        card_foreground: 0xffffff,
        primary: 0xf59e0b,
        primary_foreground: 0x1c1408,
        secondary: 0x2b1f0e,
        secondary_foreground: 0xffffff,
        muted: 0x2e261a,
        muted_foreground: 0xfcd34d,
        accent: 0x92400e,
        accent_foreground: 0xffffff,
        destructive: 0x7f1d1d,
        destructive_foreground: 0xffffff,
        border: 0x4a3510,
        input: 0x4a3510,
        ring: 0xf59e0b,
    },
    ThemePreset {
        id: "amber-light",
        label: "Amber",
        is_dark: false,
        background: 0xfffbeb,
        foreground: 0x0f172a,
        card: 0xffffff,
        card_foreground: 0x0f172a,
        primary: 0xd97706,
        primary_foreground: 0xffffff,
        secondary: 0xfef3c7,
        secondary_foreground: 0x0f172a,
        muted: 0xf2eede,
        muted_foreground: 0xb45309,
        accent: 0xfef3c7,
        accent_foreground: 0x0f172a,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xfde68a,
        input: 0xfde68a,
        ring: 0xf59e0b,
    },
    ThemePreset {
        id: "nord-dark",
        label: "Nord",
        is_dark: true,
        background: 0x2e3440,
        foreground: 0xeceff4,
        card: 0x3b4252,
        card_foreground: 0xeceff4,
        primary: 0x88c0d0,
        primary_foreground: 0x2e3440,
        secondary: 0x434c5e,
        secondary_foreground: 0xeceff4,
        muted: 0x404652,
        muted_foreground: 0xd8dee9,
        accent: 0x5e81ac,
        accent_foreground: 0xeceff4,
        destructive: 0xbf616a,
        destructive_foreground: 0xeceff4,
        border: 0x4c566a,
        input: 0x4c566a,
        ring: 0x88c0d0,
    },
    ThemePreset {
        id: "nord-light",
        label: "Nord",
        is_dark: false,
        background: 0xeceff4,
        foreground: 0x2e3440,
        card: 0xffffff,
        card_foreground: 0x2e3440,
        primary: 0x5e81ac,
        primary_foreground: 0xffffff,
        secondary: 0xe5e9f0,
        secondary_foreground: 0x2e3440,
        muted: 0xdfe2e7,
        muted_foreground: 0x4c566a,
        accent: 0xd8dee9,
        accent_foreground: 0x2e3440,
        destructive: 0xbf616a,
        destructive_foreground: 0xffffff,
        border: 0xd8dee9,
        input: 0xd8dee9,
        ring: 0x5e81ac,
    },
    ThemePreset {
        id: "midnight-dark",
        label: "Midnight Indigo",
        is_dark: true,
        background: 0x090b1e,
        foreground: 0xffffff,
        card: 0x131840,
        card_foreground: 0xffffff,
        primary: 0x6366f1,
        primary_foreground: 0xffffff,
        secondary: 0x131840,
        secondary_foreground: 0xffffff,
        muted: 0x1b1d30,
        muted_foreground: 0xa5b4fc,
        accent: 0x3730a3,
        accent_foreground: 0xffffff,
        destructive: 0x7f1d1d,
        destructive_foreground: 0xffffff,
        border: 0x232b5e,
        input: 0x232b5e,
        ring: 0x6366f1,
    },
    ThemePreset {
        id: "midnight-light",
        label: "Midnight",
        is_dark: false,
        background: 0xeef2ff,
        foreground: 0x0f172a,
        card: 0xffffff,
        card_foreground: 0x0f172a,
        primary: 0x4f46e5,
        primary_foreground: 0xffffff,
        secondary: 0xe0e7ff,
        secondary_foreground: 0x0f172a,
        muted: 0xe1e5f2,
        muted_foreground: 0x6366f1,
        accent: 0xe0e7ff,
        accent_foreground: 0x0f172a,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xc7d2fe,
        input: 0xc7d2fe,
        ring: 0x6366f1,
    },
    ThemePreset {
        id: "dracula-dark",
        label: "Dracula",
        is_dark: true,
        background: 0x282a36,
        foreground: 0xf8f8f2,
        card: 0x343746,
        card_foreground: 0xf8f8f2,
        primary: 0xbd93f9,
        primary_foreground: 0x282a36,
        secondary: 0x343746,
        secondary_foreground: 0xf8f8f2,
        muted: 0x3a3c48,
        muted_foreground: 0x6272a4,
        accent: 0xff79c6,
        accent_foreground: 0x282a36,
        destructive: 0xff5555,
        destructive_foreground: 0xf8f8f2,
        border: 0x44475a,
        input: 0x44475a,
        ring: 0xbd93f9,
    },
    ThemePreset {
        id: "dracula-light",
        label: "Dracula",
        is_dark: false,
        background: 0xf7f6fb,
        foreground: 0x282a36,
        card: 0xffffff,
        card_foreground: 0x282a36,
        primary: 0x8b5cf6,
        primary_foreground: 0xffffff,
        secondary: 0xefedf7,
        secondary_foreground: 0x282a36,
        muted: 0xedeaf4,
        muted_foreground: 0x64748b,
        accent: 0xe75fae,
        accent_foreground: 0x282a36,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xd9d5ea,
        input: 0xd9d5ea,
        ring: 0x8b5cf6,
    },
    ThemePreset {
        id: "monokai-dark",
        label: "Monokai",
        is_dark: true,
        background: 0x272822,
        foreground: 0xf8f8f2,
        card: 0x3d3e34,
        card_foreground: 0xf8f8f2,
        primary: 0xa6e22e,
        primary_foreground: 0x272822,
        secondary: 0x3d3e34,
        secondary_foreground: 0xf8f8f2,
        muted: 0x393a34,
        muted_foreground: 0x7c7865,
        accent: 0xf92672,
        accent_foreground: 0xf8f8f2,
        destructive: 0xf92672,
        destructive_foreground: 0xf8f8f2,
        border: 0x49483e,
        input: 0x49483e,
        ring: 0xa6e22e,
    },
    ThemePreset {
        id: "monokai-light",
        label: "Monokai",
        is_dark: false,
        background: 0xf8f8f2,
        foreground: 0x272822,
        card: 0xffffff,
        card_foreground: 0x272822,
        primary: 0x86a92b,
        primary_foreground: 0xffffff,
        secondary: 0xf0f0e6,
        secondary_foreground: 0x272822,
        muted: 0xebebdf,
        muted_foreground: 0x75715e,
        accent: 0xd91e63,
        accent_foreground: 0xffffff,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xdfdfd0,
        input: 0xdfdfd0,
        ring: 0x86a92b,
    },
    ThemePreset {
        id: "catppuccin-dark",
        label: "Catppuccin Mocha",
        is_dark: true,
        background: 0x1e1e2e,
        foreground: 0xcdd6f4,
        card: 0x313244,
        card_foreground: 0xcdd6f4,
        primary: 0xcba6f7,
        primary_foreground: 0x1e1e2e,
        secondary: 0x313244,
        secondary_foreground: 0xcdd6f4,
        muted: 0x303040,
        muted_foreground: 0x6c7086,
        accent: 0x89b4fa,
        accent_foreground: 0x1e1e2e,
        destructive: 0xf38ba8,
        destructive_foreground: 0x1e1e2e,
        border: 0x45475a,
        input: 0x45475a,
        ring: 0xcba6f7,
    },
    ThemePreset {
        id: "catppuccin-light",
        label: "Catppuccin Latte",
        is_dark: false,
        background: 0xeff1f5,
        foreground: 0x4c4f69,
        card: 0xffffff,
        card_foreground: 0x4c4f69,
        primary: 0x8839ef,
        primary_foreground: 0xffffff,
        secondary: 0xe6e9ef,
        secondary_foreground: 0x4c4f69,
        muted: 0xe2e5ec,
        muted_foreground: 0x6c7086,
        accent: 0xdd7878,
        accent_foreground: 0x4c4f69,
        destructive: 0xd20f39,
        destructive_foreground: 0xffffff,
        border: 0xccd0da,
        input: 0xccd0da,
        ring: 0x8839ef,
    },
    ThemePreset {
        id: "gruvbox-dark",
        label: "Gruvbox",
        is_dark: true,
        background: 0x282828,
        foreground: 0xebdbb2,
        card: 0x3c3836,
        card_foreground: 0xebdbb2,
        primary: 0xfabd2f,
        primary_foreground: 0x282828,
        secondary: 0x3c3836,
        secondary_foreground: 0xebdbb2,
        muted: 0x3a3a3a,
        muted_foreground: 0xa89984,
        accent: 0x83a598,
        accent_foreground: 0x282828,
        destructive: 0xfb4934,
        destructive_foreground: 0xebdbb2,
        border: 0x504945,
        input: 0x504945,
        ring: 0xfabd2f,
    },
    ThemePreset {
        id: "gruvbox-light",
        label: "Gruvbox",
        is_dark: false,
        background: 0xfbf1c7,
        foreground: 0x3c3836,
        card: 0xebdbb2,
        card_foreground: 0x3c3836,
        primary: 0xd79921,
        primary_foreground: 0xfbf1c7,
        secondary: 0xebdbb2,
        secondary_foreground: 0x3c3836,
        muted: 0xeee4ba,
        muted_foreground: 0x7c6f64,
        accent: 0x076678,
        accent_foreground: 0xfbf1c7,
        destructive: 0xcc241d,
        destructive_foreground: 0xfbf1c7,
        border: 0xd5c4a1,
        input: 0xd5c4a1,
        ring: 0xd79921,
    },
    ThemePreset {
        id: "tokyo-night-dark",
        label: "Tokyo Night",
        is_dark: true,
        background: 0x1a1b26,
        foreground: 0xc0caf5,
        card: 0x24283b,
        card_foreground: 0xc0caf5,
        primary: 0x7aa2f7,
        primary_foreground: 0x1a1b26,
        secondary: 0x24283b,
        secondary_foreground: 0xc0caf5,
        muted: 0x2c2d38,
        muted_foreground: 0x5b648e,
        accent: 0xbb9af7,
        accent_foreground: 0x1a1b26,
        destructive: 0xf7768e,
        destructive_foreground: 0x1a1b26,
        border: 0x292e42,
        input: 0x292e42,
        ring: 0x7aa2f7,
    },
    ThemePreset {
        id: "tokyo-night-light",
        label: "Tokyo Night",
        is_dark: false,
        background: 0xe8eaf0,
        foreground: 0x1a1b26,
        card: 0xffffff,
        card_foreground: 0x1a1b26,
        primary: 0x3b6fe0,
        primary_foreground: 0xffffff,
        secondary: 0xdfe2ec,
        secondary_foreground: 0x1a1b26,
        muted: 0xdadde8,
        muted_foreground: 0x565f89,
        accent: 0x6a4fe8,
        accent_foreground: 0x1a1b26,
        destructive: 0xf7768e,
        destructive_foreground: 0x1a1b26,
        border: 0xc9cddc,
        input: 0xc9cddc,
        ring: 0x3b6fe0,
    },
    ThemePreset {
        id: "everforest-dark",
        label: "Everforest",
        is_dark: true,
        background: 0x2b3339,
        foreground: 0xd3c6aa,
        card: 0x343f44,
        card_foreground: 0xd3c6aa,
        primary: 0xa7c080,
        primary_foreground: 0x2b3339,
        secondary: 0x343f44,
        secondary_foreground: 0xd3c6aa,
        muted: 0x3d454b,
        muted_foreground: 0x859289,
        accent: 0x84a598,
        accent_foreground: 0x2b3339,
        destructive: 0xe67e80,
        destructive_foreground: 0xd3c6aa,
        border: 0x3d484d,
        input: 0x3d484d,
        ring: 0xa7c080,
    },
    ThemePreset {
        id: "everforest-light",
        label: "Everforest",
        is_dark: false,
        background: 0xfdf6e3,
        foreground: 0x5c6a72,
        card: 0xf2efdf,
        card_foreground: 0x5c6a72,
        primary: 0x8da101,
        primary_foreground: 0xfdf6e3,
        secondary: 0xf2efdf,
        secondary_foreground: 0x5c6a72,
        muted: 0xf0e9d6,
        muted_foreground: 0x829184,
        accent: 0x3a94c5,
        accent_foreground: 0xfdf6e3,
        destructive: 0xeb5757,
        destructive_foreground: 0xfdf6e3,
        border: 0xe0dcc7,
        input: 0xe0dcc7,
        ring: 0x8da101,
    },
    ThemePreset {
        id: "rose-pine-dark",
        label: "Rosé Pine",
        is_dark: true,
        background: 0x191724,
        foreground: 0xe0def4,
        card: 0x26233a,
        card_foreground: 0xe0def4,
        primary: 0xebbcba,
        primary_foreground: 0x191724,
        secondary: 0x26233a,
        secondary_foreground: 0xe0def4,
        muted: 0x2b2936,
        muted_foreground: 0x908caa,
        accent: 0x31748f,
        accent_foreground: 0x191724,
        destructive: 0xeb6f92,
        destructive_foreground: 0xe0def4,
        border: 0x312e42,
        input: 0x312e42,
        ring: 0xebbcba,
    },
    ThemePreset {
        id: "rose-pine-light",
        label: "Rosé Pine Dawn",
        is_dark: false,
        background: 0xfaf4ed,
        foreground: 0x575279,
        card: 0xffffff,
        card_foreground: 0x575279,
        primary: 0x907aa9,
        primary_foreground: 0xffffff,
        secondary: 0xf2e9e1,
        secondary_foreground: 0x575279,
        muted: 0xefe7dc,
        muted_foreground: 0x797593,
        accent: 0x56949f,
        accent_foreground: 0x575279,
        destructive: 0xb4637a,
        destructive_foreground: 0xffffff,
        border: 0xe4dcd0,
        input: 0xe4dcd0,
        ring: 0x907aa9,
    },
    ThemePreset {
        id: "solarized-dark",
        label: "Solarized",
        is_dark: true,
        background: 0x002b36,
        foreground: 0x839496,
        card: 0x073642,
        card_foreground: 0x839496,
        primary: 0x268bd2,
        primary_foreground: 0xfdf6e3,
        secondary: 0x073642,
        secondary_foreground: 0x839496,
        muted: 0x123d48,
        muted_foreground: 0x586e75,
        accent: 0x2aa198,
        accent_foreground: 0xfdf6e3,
        destructive: 0xdc322f,
        destructive_foreground: 0xfdf6e3,
        border: 0x586e75,
        input: 0x586e75,
        ring: 0x268bd2,
    },
    ThemePreset {
        id: "solarized-light",
        label: "Solarized",
        is_dark: false,
        background: 0xfdf6e3,
        foreground: 0x657b83,
        card: 0xeee8d5,
        card_foreground: 0x657b83,
        primary: 0x268bd2,
        primary_foreground: 0xfdf6e3,
        secondary: 0xeee8d5,
        secondary_foreground: 0x657b83,
        muted: 0xf0e9d6,
        muted_foreground: 0x8a999b,
        accent: 0x2aa198,
        accent_foreground: 0xfdf6e3,
        destructive: 0xdc322f,
        destructive_foreground: 0xfdf6e3,
        border: 0xd3cbb4,
        input: 0xd3cbb4,
        ring: 0x268bd2,
    },
    ThemePreset {
        id: "one-dark",
        label: "One",
        is_dark: true,
        background: 0x282c34,
        foreground: 0xabb2bf,
        card: 0x353a44,
        card_foreground: 0xabb2bf,
        primary: 0x61afef,
        primary_foreground: 0x282c34,
        secondary: 0x353a44,
        secondary_foreground: 0xabb2bf,
        muted: 0x3a3e46,
        muted_foreground: 0x707784,
        accent: 0xe5c07b,
        accent_foreground: 0x282c34,
        destructive: 0xe06c75,
        destructive_foreground: 0x282c34,
        border: 0x3e4451,
        input: 0x3e4451,
        ring: 0x61afef,
    },
    ThemePreset {
        id: "one-light",
        label: "One",
        is_dark: false,
        background: 0xfafafa,
        foreground: 0x383a42,
        card: 0xffffff,
        card_foreground: 0x383a42,
        primary: 0x4078f2,
        primary_foreground: 0xffffff,
        secondary: 0xf0f0f0,
        secondary_foreground: 0x383a42,
        muted: 0xededed,
        muted_foreground: 0xa0a1a7,
        accent: 0xe45649,
        accent_foreground: 0xffffff,
        destructive: 0xca1243,
        destructive_foreground: 0xffffff,
        border: 0xdbdbdc,
        input: 0xdbdbdc,
        ring: 0x4078f2,
    },
    ThemePreset {
        id: "github-dark",
        label: "GitHub",
        is_dark: true,
        background: 0x0d1117,
        foreground: 0xc9d1d9,
        card: 0x161b22,
        card_foreground: 0xc9d1d9,
        primary: 0x58a6ff,
        primary_foreground: 0x0d1117,
        secondary: 0x161b22,
        secondary_foreground: 0xc9d1d9,
        muted: 0x1f2329,
        muted_foreground: 0x8b949e,
        accent: 0xf78166,
        accent_foreground: 0x0d1117,
        destructive: 0xf85149,
        destructive_foreground: 0x0d1117,
        border: 0x30363d,
        input: 0x30363d,
        ring: 0x58a6ff,
    },
    ThemePreset {
        id: "github-light",
        label: "GitHub",
        is_dark: false,
        background: 0xffffff,
        foreground: 0x24292f,
        card: 0xf6f8fa,
        card_foreground: 0x24292f,
        primary: 0x0969da,
        primary_foreground: 0xffffff,
        secondary: 0xf6f8fa,
        secondary_foreground: 0x24292f,
        muted: 0xf2f2f2,
        muted_foreground: 0x656d76,
        accent: 0xcf222e,
        accent_foreground: 0xffffff,
        destructive: 0xcf222e,
        destructive_foreground: 0xffffff,
        border: 0xd0d7de,
        input: 0xd0d7de,
        ring: 0x0969da,
    },
    ThemePreset {
        id: "ayu-dark",
        label: "Ayu",
        is_dark: true,
        background: 0x0a0e14,
        foreground: 0xb3b1ad,
        card: 0x131721,
        card_foreground: 0xb3b1ad,
        primary: 0xffb454,
        primary_foreground: 0x0a0e14,
        secondary: 0x131721,
        secondary_foreground: 0xb3b1ad,
        muted: 0x1c2026,
        muted_foreground: 0x5c6773,
        accent: 0x59c2ff,
        accent_foreground: 0x0a0e14,
        destructive: 0xff8f40,
        destructive_foreground: 0x0a0e14,
        border: 0x1e2531,
        input: 0x1e2531,
        ring: 0xffb454,
    },
    ThemePreset {
        id: "ayu-light",
        label: "Ayu",
        is_dark: false,
        background: 0xfafafa,
        foreground: 0x575f66,
        card: 0xffffff,
        card_foreground: 0x575f66,
        primary: 0xff6a00,
        primary_foreground: 0xffffff,
        secondary: 0xf0f0f0,
        secondary_foreground: 0x575f66,
        muted: 0xededed,
        muted_foreground: 0x8a9199,
        accent: 0x55b4d4,
        accent_foreground: 0xffffff,
        destructive: 0xf27983,
        destructive_foreground: 0xffffff,
        border: 0xd9d8d7,
        input: 0xd9d8d7,
        ring: 0xff6a00,
    },
    ThemePreset {
        id: "kanagawa-dark",
        label: "Kanagawa",
        is_dark: true,
        background: 0x1f1f28,
        foreground: 0xdcd7ba,
        card: 0x2a2a37,
        card_foreground: 0xdcd7ba,
        primary: 0x7e9cd8,
        primary_foreground: 0x1f1f28,
        secondary: 0x2a2a37,
        secondary_foreground: 0xdcd7ba,
        muted: 0x31313a,
        muted_foreground: 0x686879,
        accent: 0xc34043,
        accent_foreground: 0x1f1f28,
        destructive: 0xe82424,
        destructive_foreground: 0xdcd7ba,
        border: 0x363646,
        input: 0x363646,
        ring: 0x7e9cd8,
    },
    ThemePreset {
        id: "kanagawa-light",
        label: "Kanagawa",
        is_dark: false,
        background: 0xf2ecbc,
        foreground: 0x69604d,
        card: 0xe8e0b6,
        card_foreground: 0x69604d,
        primary: 0x5e81ac,
        primary_foreground: 0xf2ecbc,
        secondary: 0xe8e0b6,
        secondary_foreground: 0x69604d,
        muted: 0xe5dfaf,
        muted_foreground: 0x9d8c5d,
        accent: 0xc34043,
        accent_foreground: 0xf2ecbc,
        destructive: 0xe82424,
        destructive_foreground: 0xf2ecbc,
        border: 0xd5cda4,
        input: 0xd5cda4,
        ring: 0x5e81ac,
    },
    ThemePreset {
        id: "zenburn-dark",
        label: "Zenburn",
        is_dark: true,
        background: 0x3f3f3f,
        foreground: 0xdcdccc,
        card: 0x4a4a4a,
        card_foreground: 0xdcdccc,
        primary: 0x8cd0d3,
        primary_foreground: 0x3f3f3f,
        secondary: 0x4a4a4a,
        secondary_foreground: 0xdcdccc,
        muted: 0x515151,
        muted_foreground: 0x7f9f7f,
        accent: 0xffcfaf,
        accent_foreground: 0x3f3f3f,
        destructive: 0xcc9393,
        destructive_foreground: 0x3f3f3f,
        border: 0x505050,
        input: 0x505050,
        ring: 0x8cd0d3,
    },
    ThemePreset {
        id: "zenburn-light",
        label: "Zenburn",
        is_dark: false,
        background: 0xf0f0e8,
        foreground: 0x3f3f3f,
        card: 0xffffff,
        card_foreground: 0x3f3f3f,
        primary: 0x4a9a9e,
        primary_foreground: 0xffffff,
        secondary: 0xe8e8de,
        secondary_foreground: 0x3f3f3f,
        muted: 0xe3e3d9,
        muted_foreground: 0x7f9f7f,
        accent: 0xc08040,
        accent_foreground: 0x3f3f3f,
        destructive: 0xcc9393,
        destructive_foreground: 0x3f3f3f,
        border: 0xd8d8ca,
        input: 0xd8d8ca,
        ring: 0x4a9a9e,
    },
    ThemePreset {
        id: "material-dark",
        label: "Material",
        is_dark: true,
        background: 0x263238,
        foreground: 0xeeffff,
        card: 0x37474f,
        card_foreground: 0xeeffff,
        primary: 0x82aaff,
        primary_foreground: 0x263238,
        secondary: 0x37474f,
        secondary_foreground: 0xeeffff,
        muted: 0x38444a,
        muted_foreground: 0x6b848e,
        accent: 0xc792ea,
        accent_foreground: 0x263238,
        destructive: 0xff5370,
        destructive_foreground: 0xeeffff,
        border: 0x465a64,
        input: 0x465a64,
        ring: 0x82aaff,
    },
    ThemePreset {
        id: "material-light",
        label: "Material",
        is_dark: false,
        background: 0xfafafa,
        foreground: 0x546e7a,
        card: 0xffffff,
        card_foreground: 0x546e7a,
        primary: 0x2e7d32,
        primary_foreground: 0xffffff,
        secondary: 0xf5f5f5,
        secondary_foreground: 0x546e7a,
        muted: 0xededed,
        muted_foreground: 0x90a4ae,
        accent: 0x7c4dff,
        accent_foreground: 0xffffff,
        destructive: 0xd32f2f,
        destructive_foreground: 0xffffff,
        border: 0xe0e0e0,
        input: 0xe0e0e0,
        ring: 0x2e7d32,
    },
    ThemePreset {
        id: "palenight-dark",
        label: "Palenight",
        is_dark: true,
        background: 0x292d3e,
        foreground: 0xbfc7d5,
        card: 0x303348,
        card_foreground: 0xbfc7d5,
        primary: 0xc792ea,
        primary_foreground: 0x292d3e,
        secondary: 0x303348,
        secondary_foreground: 0xbfc7d5,
        muted: 0x3b3f50,
        muted_foreground: 0x676e95,
        accent: 0x82aaff,
        accent_foreground: 0x292d3e,
        destructive: 0xff5370,
        destructive_foreground: 0x292d3e,
        border: 0x404560,
        input: 0x404560,
        ring: 0xc792ea,
    },
    ThemePreset {
        id: "palenight-light",
        label: "Palenight",
        is_dark: false,
        background: 0xeceef7,
        foreground: 0x292d3e,
        card: 0xffffff,
        card_foreground: 0x292d3e,
        primary: 0x7a6ce0,
        primary_foreground: 0xffffff,
        secondary: 0xe2e4f0,
        secondary_foreground: 0x292d3e,
        muted: 0xdedfec,
        muted_foreground: 0x676e95,
        accent: 0x4f7fe8,
        accent_foreground: 0x292d3e,
        destructive: 0xff5370,
        destructive_foreground: 0x292d3e,
        border: 0xc9cce0,
        input: 0xc9cce0,
        ring: 0x7a6ce0,
    },
    ThemePreset {
        id: "darcula-dark",
        label: "Darcula",
        is_dark: true,
        background: 0x2b2b2b,
        foreground: 0xa9b7c6,
        card: 0x3c3f41,
        card_foreground: 0xa9b7c6,
        primary: 0xcc7832,
        primary_foreground: 0x2b2b2b,
        secondary: 0x3c3f41,
        secondary_foreground: 0xa9b7c6,
        muted: 0x3d3d3d,
        muted_foreground: 0x6a7c8e,
        accent: 0x6897bb,
        accent_foreground: 0x2b2b2b,
        destructive: 0xe95353,
        destructive_foreground: 0x2b2b2b,
        border: 0x4b4e50,
        input: 0x4b4e50,
        ring: 0xcc7832,
    },
    ThemePreset {
        id: "darcula-light",
        label: "Darcula",
        is_dark: false,
        background: 0xf5f5f5,
        foreground: 0x2b2b2b,
        card: 0xffffff,
        card_foreground: 0x2b2b2b,
        primary: 0xb26a28,
        primary_foreground: 0xffffff,
        secondary: 0xececec,
        secondary_foreground: 0x2b2b2b,
        muted: 0xe8e8e8,
        muted_foreground: 0x6a7c8e,
        accent: 0x4a7bae,
        accent_foreground: 0x2b2b2b,
        destructive: 0xe95353,
        destructive_foreground: 0x2b2b2b,
        border: 0xd8d8d8,
        input: 0xd8d8d8,
        ring: 0xb26a28,
    },
    ThemePreset {
        id: "cobalt2-dark",
        label: "Cobalt2",
        is_dark: true,
        background: 0x193549,
        foreground: 0xffffff,
        card: 0x1f4060,
        card_foreground: 0xffffff,
        primary: 0xffc600,
        primary_foreground: 0x193549,
        secondary: 0x1f4060,
        secondary_foreground: 0xffffff,
        muted: 0x2b475b,
        muted_foreground: 0x5a7d95,
        accent: 0xff628c,
        accent_foreground: 0x193549,
        destructive: 0xff628c,
        destructive_foreground: 0x193549,
        border: 0x2c5676,
        input: 0x2c5676,
        ring: 0xffc600,
    },
    ThemePreset {
        id: "cobalt2-light",
        label: "Cobalt2",
        is_dark: false,
        background: 0xf0f6fd,
        foreground: 0x193549,
        card: 0xffffff,
        card_foreground: 0x193549,
        primary: 0xd9a800,
        primary_foreground: 0xffffff,
        secondary: 0xe2edf8,
        secondary_foreground: 0x193549,
        muted: 0xdee9f5,
        muted_foreground: 0x5a7d95,
        accent: 0xd6436c,
        accent_foreground: 0x193549,
        destructive: 0xff628c,
        destructive_foreground: 0x193549,
        border: 0xc9dbee,
        input: 0xc9dbee,
        ring: 0xd9a800,
    },
    ThemePreset {
        id: "night-owl-dark",
        label: "Night Owl",
        is_dark: true,
        background: 0x011627,
        foreground: 0xd6deeb,
        card: 0x0b2942,
        card_foreground: 0xd6deeb,
        primary: 0x82aaff,
        primary_foreground: 0x011627,
        secondary: 0x0b2942,
        secondary_foreground: 0xd6deeb,
        muted: 0x132839,
        muted_foreground: 0x5f7e97,
        accent: 0xc792ea,
        accent_foreground: 0x011627,
        destructive: 0xef5350,
        destructive_foreground: 0x011627,
        border: 0x1d3b53,
        input: 0x1d3b53,
        ring: 0x82aaff,
    },
    ThemePreset {
        id: "night-owl-light",
        label: "Night Owl",
        is_dark: false,
        background: 0xf5f7fa,
        foreground: 0x011627,
        card: 0xffffff,
        card_foreground: 0x011627,
        primary: 0x2e6fd8,
        primary_foreground: 0xffffff,
        secondary: 0xe8eef5,
        secondary_foreground: 0x011627,
        muted: 0xe3eaf2,
        muted_foreground: 0x5f7e97,
        accent: 0x6e4ec0,
        accent_foreground: 0x011627,
        destructive: 0xef5350,
        destructive_foreground: 0x011627,
        border: 0xcfdae6,
        input: 0xcfdae6,
        ring: 0x2e6fd8,
    },
    ThemePreset {
        id: "horizon-dark",
        label: "Horizon",
        is_dark: true,
        background: 0x1c1e26,
        foreground: 0xdbd6d2,
        card: 0x252834,
        card_foreground: 0xdbd6d2,
        primary: 0xe95678,
        primary_foreground: 0x1c1e26,
        secondary: 0x252834,
        secondary_foreground: 0xdbd6d2,
        muted: 0x2e3038,
        muted_foreground: 0x6c6f93,
        accent: 0xfac29a,
        accent_foreground: 0x1c1e26,
        destructive: 0xe95678,
        destructive_foreground: 0xfdf0ed,
        border: 0x2e303e,
        input: 0x2e303e,
        ring: 0xe95678,
    },
    ThemePreset {
        id: "horizon-light",
        label: "Horizon",
        is_dark: false,
        background: 0xfdf6f5,
        foreground: 0x1c1e26,
        card: 0xffffff,
        card_foreground: 0x1c1e26,
        primary: 0xe95678,
        primary_foreground: 0xffffff,
        secondary: 0xf6e9e7,
        secondary_foreground: 0x1c1e26,
        muted: 0xf1e4e2,
        muted_foreground: 0x6c6f93,
        accent: 0xe09a66,
        accent_foreground: 0x1c1e26,
        destructive: 0xe95678,
        destructive_foreground: 0xfdf0ed,
        border: 0xead8d5,
        input: 0xead8d5,
        ring: 0xe95678,
    },
    ThemePreset {
        id: "synthwave-dark",
        label: "Synthwave '84",
        is_dark: true,
        background: 0x262335,
        foreground: 0xe0e0e0,
        card: 0x34294f,
        card_foreground: 0xe0e0e0,
        primary: 0xf92aad,
        primary_foreground: 0xffffff,
        secondary: 0x34294f,
        secondary_foreground: 0xe0e0e0,
        muted: 0x383547,
        muted_foreground: 0x848bbd,
        accent: 0x36f9f6,
        accent_foreground: 0x262335,
        destructive: 0xff8b39,
        destructive_foreground: 0xffffff,
        border: 0x495495,
        input: 0x495495,
        ring: 0xf92aad,
    },
    ThemePreset {
        id: "synthwave-light",
        label: "Synthwave",
        is_dark: false,
        background: 0xf7f4fc,
        foreground: 0x262335,
        card: 0xffffff,
        card_foreground: 0x262335,
        primary: 0xc01e86,
        primary_foreground: 0xffffff,
        secondary: 0xefe9f8,
        secondary_foreground: 0x262335,
        muted: 0xeae4f5,
        muted_foreground: 0x848bbd,
        accent: 0x0fa8a5,
        accent_foreground: 0x262335,
        destructive: 0xff8b39,
        destructive_foreground: 0xffffff,
        border: 0xdad2ec,
        input: 0xdad2ec,
        ring: 0xc01e86,
    },
    ThemePreset {
        id: "shades-of-purple-dark",
        label: "Shades of Purple",
        is_dark: true,
        background: 0x2d2b55,
        foreground: 0xffffff,
        card: 0x3a375d,
        card_foreground: 0xffffff,
        primary: 0xfad000,
        primary_foreground: 0x2d2b55,
        secondary: 0x3a375d,
        secondary_foreground: 0xffffff,
        muted: 0x3f3d67,
        muted_foreground: 0xa599e9,
        accent: 0x9e86ff,
        accent_foreground: 0x2d2b55,
        destructive: 0xff628c,
        destructive_foreground: 0xffffff,
        border: 0x554d8c,
        input: 0x554d8c,
        ring: 0xfad000,
    },
    ThemePreset {
        id: "shades-of-purple-light",
        label: "Shades of Purple",
        is_dark: false,
        background: 0xf6f4ff,
        foreground: 0x2d2b55,
        card: 0xffffff,
        card_foreground: 0x2d2b55,
        primary: 0xc4a800,
        primary_foreground: 0xffffff,
        secondary: 0xeeeafc,
        secondary_foreground: 0x2d2b55,
        muted: 0xe9e5f9,
        muted_foreground: 0x9f94e2,
        accent: 0x6b55e0,
        accent_foreground: 0x2d2b55,
        destructive: 0xff628c,
        destructive_foreground: 0xffffff,
        border: 0xd9d2f4,
        input: 0xd9d2f4,
        ring: 0xc4a800,
    },
    ThemePreset {
        id: "oceanic-next-dark",
        label: "Oceanic Next",
        is_dark: true,
        background: 0x1b2b34,
        foreground: 0xcdd3de,
        card: 0x243d4a,
        card_foreground: 0xcdd3de,
        primary: 0xec5f67,
        primary_foreground: 0xffffff,
        secondary: 0x243d4a,
        secondary_foreground: 0xcdd3de,
        muted: 0x2d3d46,
        muted_foreground: 0x6a7883,
        accent: 0x6699cc,
        accent_foreground: 0x1b2b34,
        destructive: 0xec5f67,
        destructive_foreground: 0xffffff,
        border: 0x304c5a,
        input: 0x304c5a,
        ring: 0xec5f67,
    },
    ThemePreset {
        id: "oceanic-next-light",
        label: "Oceanic Next",
        is_dark: false,
        background: 0xf0f5f7,
        foreground: 0x1b2b34,
        card: 0xffffff,
        card_foreground: 0x1b2b34,
        primary: 0xd94f57,
        primary_foreground: 0xffffff,
        secondary: 0xe6eef1,
        secondary_foreground: 0x1b2b34,
        muted: 0xe1e9ec,
        muted_foreground: 0x65737e,
        accent: 0x4f84b4,
        accent_foreground: 0x1b2b34,
        destructive: 0xec5f67,
        destructive_foreground: 0xffffff,
        border: 0xcfdde3,
        input: 0xcfdde3,
        ring: 0xd94f57,
    },
    ThemePreset {
        id: "panda-dark",
        label: "Panda",
        is_dark: true,
        background: 0x292a2b,
        foreground: 0xe6e6e6,
        card: 0x313335,
        card_foreground: 0xe6e6e6,
        primary: 0x19f9d8,
        primary_foreground: 0x292a2b,
        secondary: 0x313335,
        secondary_foreground: 0xe6e6e6,
        muted: 0x3b3c3d,
        muted_foreground: 0x6b6c6d,
        accent: 0xff6c6b,
        accent_foreground: 0x292a2b,
        destructive: 0xff6c6b,
        destructive_foreground: 0x292a2b,
        border: 0x444648,
        input: 0x444648,
        ring: 0x19f9d8,
    },
    ThemePreset {
        id: "panda-light",
        label: "Panda",
        is_dark: false,
        background: 0xf4f4f5,
        foreground: 0x292a2b,
        card: 0xffffff,
        card_foreground: 0x292a2b,
        primary: 0x0eb9a0,
        primary_foreground: 0xffffff,
        secondary: 0xebebec,
        secondary_foreground: 0x292a2b,
        muted: 0xe6e6e7,
        muted_foreground: 0x6b6c6d,
        accent: 0xe04443,
        accent_foreground: 0x292a2b,
        destructive: 0xff6c6b,
        destructive_foreground: 0x292a2b,
        border: 0xd7d7d9,
        input: 0xd7d7d9,
        ring: 0x0eb9a0,
    },
    ThemePreset {
        id: "cyberpunk-dark",
        label: "Cyberpunk",
        is_dark: true,
        background: 0x0d0a1a,
        foreground: 0xffffff,
        card: 0x1a1533,
        card_foreground: 0xffffff,
        primary: 0xff007f,
        primary_foreground: 0xffffff,
        secondary: 0x1a1533,
        secondary_foreground: 0xffffff,
        muted: 0x1f1c2c,
        muted_foreground: 0x8880b8,
        accent: 0x00f0ff,
        accent_foreground: 0x0d0a1a,
        destructive: 0xff3333,
        destructive_foreground: 0xffffff,
        border: 0x332d66,
        input: 0x332d66,
        ring: 0xff007f,
    },
    ThemePreset {
        id: "cyberpunk-light",
        label: "Cyberpunk",
        is_dark: false,
        background: 0xf6f4fe,
        foreground: 0x0d0a1a,
        card: 0xffffff,
        card_foreground: 0x0d0a1a,
        primary: 0xe80074,
        primary_foreground: 0xffffff,
        secondary: 0xede9fb,
        secondary_foreground: 0x0d0a1a,
        muted: 0xe8e4f8,
        muted_foreground: 0x8880b8,
        accent: 0x00a0ae,
        accent_foreground: 0x0d0a1a,
        destructive: 0xff3333,
        destructive_foreground: 0xffffff,
        border: 0xd8d0f2,
        input: 0xd8d0f2,
        ring: 0xe80074,
    },
    ThemePreset {
        id: "obsidian-dark",
        label: "Obsidian",
        is_dark: true,
        background: 0x0c0c0f,
        foreground: 0xebebeb,
        card: 0x18181f,
        card_foreground: 0xebebeb,
        primary: 0x8b5cf6,
        primary_foreground: 0xffffff,
        secondary: 0x18181f,
        secondary_foreground: 0xebebeb,
        muted: 0x1e1e21,
        muted_foreground: 0x6b6b7b,
        accent: 0x10b981,
        accent_foreground: 0x0c0c0f,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0x2a2a33,
        input: 0x2a2a33,
        ring: 0x8b5cf6,
    },
    ThemePreset {
        id: "obsidian-light",
        label: "Obsidian",
        is_dark: false,
        background: 0xf4f4f7,
        foreground: 0x0c0c0f,
        card: 0xffffff,
        card_foreground: 0x0c0c0f,
        primary: 0x7c4df5,
        primary_foreground: 0xffffff,
        secondary: 0xebebf0,
        secondary_foreground: 0x0c0c0f,
        muted: 0xe6e6ec,
        muted_foreground: 0x6b6b7b,
        accent: 0x0c946a,
        accent_foreground: 0x0c0c0f,
        destructive: 0xef4444,
        destructive_foreground: 0xffffff,
        border: 0xd9d9e0,
        input: 0xd9d9e0,
        ring: 0x7c4df5,
    },
    ThemePreset {
        id: "arc-dark",
        label: "Arc",
        is_dark: true,
        background: 0x2f343f,
        foreground: 0xd3dae3,
        card: 0x383c4a,
        card_foreground: 0xd3dae3,
        primary: 0x5294e2,
        primary_foreground: 0x2f343f,
        secondary: 0x383c4a,
        secondary_foreground: 0xd3dae3,
        muted: 0x414651,
        muted_foreground: 0x7c818c,
        accent: 0xc678dd,
        accent_foreground: 0x2f343f,
        destructive: 0xe06c75,
        destructive_foreground: 0x2f343f,
        border: 0x404552,
        input: 0x404552,
        ring: 0x5294e2,
    },
    ThemePreset {
        id: "arc-light",
        label: "Arc",
        is_dark: false,
        background: 0xf0f3f8,
        foreground: 0x2f343f,
        card: 0xffffff,
        card_foreground: 0x2f343f,
        primary: 0x4a86d0,
        primary_foreground: 0xffffff,
        secondary: 0xe7ebf2,
        secondary_foreground: 0x2f343f,
        muted: 0xe2e7ee,
        muted_foreground: 0x7c818c,
        accent: 0xa755c4,
        accent_foreground: 0x2f343f,
        destructive: 0xe06c75,
        destructive_foreground: 0x2f343f,
        border: 0xd3d9e2,
        input: 0xd3d9e2,
        ring: 0x4a86d0,
    },
    ThemePreset {
        id: "noctis-dark",
        label: "Noctis",
        is_dark: true,
        background: 0x1e2430,
        foreground: 0xd3d8e0,
        card: 0x293141,
        card_foreground: 0xd3d8e0,
        primary: 0x7cb7ff,
        primary_foreground: 0x1e2430,
        secondary: 0x293141,
        secondary_foreground: 0xd3d8e0,
        muted: 0x303642,
        muted_foreground: 0x626c7d,
        accent: 0x49acca,
        accent_foreground: 0x1e2430,
        destructive: 0xff8282,
        destructive_foreground: 0x1e2430,
        border: 0x323b4f,
        input: 0x323b4f,
        ring: 0x7cb7ff,
    },
    ThemePreset {
        id: "noctis-light",
        label: "Noctis",
        is_dark: false,
        background: 0xf0f3f8,
        foreground: 0x1e2430,
        card: 0xffffff,
        card_foreground: 0x1e2430,
        primary: 0x4a90e2,
        primary_foreground: 0xffffff,
        secondary: 0xe7ebf2,
        secondary_foreground: 0x1e2430,
        muted: 0xe2e7ee,
        muted_foreground: 0x5c6678,
        accent: 0x37879f,
        accent_foreground: 0x1e2430,
        destructive: 0xff8282,
        destructive_foreground: 0x1e2430,
        border: 0xd5dee8,
        input: 0xd5dee8,
        ring: 0x4a90e2,
    },
    ThemePreset {
        id: "blush-dark",
        label: "Blush",
        is_dark: true,
        background: 0x1e151f,
        foreground: 0xffffff,
        card: 0x2a1f2b,
        card_foreground: 0xffffff,
        primary: 0xd18ea6,
        primary_foreground: 0x1e151f,
        secondary: 0x2a1f2b,
        secondary_foreground: 0xffffff,
        muted: 0x302731,
        muted_foreground: 0xa98a99,
        accent: 0x8b6b82,
        accent_foreground: 0xffffff,
        destructive: 0xc7344b,
        destructive_foreground: 0xffffff,
        border: 0x3b2b3c,
        input: 0x3b2b3c,
        ring: 0xd18ea6,
    },
    ThemePreset {
        id: "blush-light",
        label: "Blush",
        is_dark: false,
        background: 0xfff5f7,
        foreground: 0x2c1a24,
        card: 0xffffff,
        card_foreground: 0x2c1a24,
        primary: 0xc5657c,
        primary_foreground: 0xffffff,
        secondary: 0xfde8ec,
        secondary_foreground: 0x2c1a24,
        muted: 0xf2e8ea,
        muted_foreground: 0x9d7b87,
        accent: 0xfde8ec,
        accent_foreground: 0x2c1a24,
        destructive: 0xe74c3c,
        destructive_foreground: 0xffffff,
        border: 0xf0d4dc,
        input: 0xf0d4dc,
        ring: 0xc5657c,
    },
    ThemePreset {
        id: "sand-dark",
        label: "Sand",
        is_dark: true,
        background: 0x1d1a14,
        foreground: 0xffffff,
        card: 0x29251b,
        card_foreground: 0xffffff,
        primary: 0xd4a96a,
        primary_foreground: 0x1d1a14,
        secondary: 0x29251b,
        secondary_foreground: 0xffffff,
        muted: 0x2f2c26,
        muted_foreground: 0x8c8269,
        accent: 0x7b6640,
        accent_foreground: 0xffffff,
        destructive: 0xb3463d,
        destructive_foreground: 0xffffff,
        border: 0x3d3526,
        input: 0x3d3526,
        ring: 0xd4a96a,
    },
    ThemePreset {
        id: "sand-light",
        label: "Sand",
        is_dark: false,
        background: 0xfdfbf7,
        foreground: 0x3d3627,
        card: 0xffffff,
        card_foreground: 0x3d3627,
        primary: 0xb8914a,
        primary_foreground: 0xffffff,
        secondary: 0xf5f0e5,
        secondary_foreground: 0x3d3627,
        muted: 0xf0eeea,
        muted_foreground: 0x8c8269,
        accent: 0xf5f0e5,
        accent_foreground: 0x3d3627,
        destructive: 0xd95040,
        destructive_foreground: 0xffffff,
        border: 0xe5dcc8,
        input: 0xe5dcc8,
        ring: 0xb8914a,
    },
    ThemePreset {
        id: "sky-dark",
        label: "Sky",
        is_dark: true,
        background: 0x0a1628,
        foreground: 0xffffff,
        card: 0x142440,
        card_foreground: 0xffffff,
        primary: 0x38bdf8,
        primary_foreground: 0x0a1628,
        secondary: 0x142440,
        secondary_foreground: 0xffffff,
        muted: 0x1c283a,
        muted_foreground: 0x7aaed6,
        accent: 0x0284c7,
        accent_foreground: 0xffffff,
        destructive: 0xc0392b,
        destructive_foreground: 0xffffff,
        border: 0x1d3a5c,
        input: 0x1d3a5c,
        ring: 0x38bdf8,
    },
    ThemePreset {
        id: "sky-light",
        label: "Sky",
        is_dark: false,
        background: 0xf0f9ff,
        foreground: 0x0f172a,
        card: 0xffffff,
        card_foreground: 0x0f172a,
        primary: 0x0284c7,
        primary_foreground: 0xffffff,
        secondary: 0xe0f2fe,
        secondary_foreground: 0x0f172a,
        muted: 0xe3ecf2,
        muted_foreground: 0x34ace3,
        accent: 0xe0f2fe,
        accent_foreground: 0x0f172a,
        destructive: 0xe74c3c,
        destructive_foreground: 0xffffff,
        border: 0xbae6fd,
        input: 0xbae6fd,
        ring: 0x0ea5e9,
    },
    ThemePreset {
        id: "plum-dark",
        label: "Plum",
        is_dark: true,
        background: 0x1a1423,
        foreground: 0xffffff,
        card: 0x261f33,
        card_foreground: 0xffffff,
        primary: 0xc084fc,
        primary_foreground: 0x1a1423,
        secondary: 0x261f33,
        secondary_foreground: 0xffffff,
        muted: 0x2c2635,
        muted_foreground: 0xa491c1,
        accent: 0x6b21a8,
        accent_foreground: 0xffffff,
        destructive: 0xb91c1c,
        destructive_foreground: 0xffffff,
        border: 0x3b2e4f,
        input: 0x3b2e4f,
        ring: 0xc084fc,
    },
    ThemePreset {
        id: "plum-light",
        label: "Plum",
        is_dark: false,
        background: 0xfdf4ff,
        foreground: 0x1a1423,
        card: 0xffffff,
        card_foreground: 0x1a1423,
        primary: 0x9333ea,
        primary_foreground: 0xffffff,
        secondary: 0xfae8ff,
        secondary_foreground: 0x1a1423,
        muted: 0xf0e7f2,
        muted_foreground: 0xc084fc,
        accent: 0xfae8ff,
        accent_foreground: 0x1a1423,
        destructive: 0xe11d48,
        destructive_foreground: 0xffffff,
        border: 0xe9d5ff,
        input: 0xe9d5ff,
        ring: 0xa855f7,
    },
    ThemePreset {
        id: "crimson-dark",
        label: "Crimson",
        is_dark: true,
        background: 0x1b0d0f,
        foreground: 0xffffff,
        card: 0x2a1518,
        card_foreground: 0xffffff,
        primary: 0xef4444,
        primary_foreground: 0xffffff,
        secondary: 0x2a1518,
        secondary_foreground: 0xffffff,
        muted: 0x2d1f21,
        muted_foreground: 0xc9818a,
        accent: 0x991b1b,
        accent_foreground: 0xffffff,
        destructive: 0xdc2626,
        destructive_foreground: 0xffffff,
        border: 0x441f22,
        input: 0x441f22,
        ring: 0xef4444,
    },
    ThemePreset {
        id: "crimson-light",
        label: "Crimson",
        is_dark: false,
        background: 0xfef2f2,
        foreground: 0x1b0d0f,
        card: 0xffffff,
        card_foreground: 0x1b0d0f,
        primary: 0xdc2626,
        primary_foreground: 0xffffff,
        secondary: 0xfee2e2,
        secondary_foreground: 0x1b0d0f,
        muted: 0xf1e5e5,
        muted_foreground: 0xf87171,
        accent: 0xfee2e2,
        accent_foreground: 0x1b0d0f,
        destructive: 0xb91c1c,
        destructive_foreground: 0xffffff,
        border: 0xfecaca,
        input: 0xfecaca,
        ring: 0xef4444,
    },
    ThemePreset {
        id: "lime-dark",
        label: "Lime",
        is_dark: true,
        background: 0x0f1a0b,
        foreground: 0xffffff,
        card: 0x1a2c15,
        card_foreground: 0xffffff,
        primary: 0x84cc16,
        primary_foreground: 0x0f1a0b,
        secondary: 0x1a2c15,
        secondary_foreground: 0xffffff,
        muted: 0x212c1d,
        muted_foreground: 0x8ab860,
        accent: 0x4d7c0f,
        accent_foreground: 0xffffff,
        destructive: 0xb91c1c,
        destructive_foreground: 0xffffff,
        border: 0x253b1e,
        input: 0x253b1e,
        ring: 0x84cc16,
    },
    ThemePreset {
        id: "lime-light",
        label: "Lime",
        is_dark: false,
        background: 0xf7fee7,
        foreground: 0x0f1a0b,
        card: 0xffffff,
        card_foreground: 0x0f1a0b,
        primary: 0x65a30d,
        primary_foreground: 0xffffff,
        secondary: 0xecfccb,
        secondary_foreground: 0x0f1a0b,
        muted: 0xeaf1da,
        muted_foreground: 0x72b114,
        accent: 0xecfccb,
        accent_foreground: 0x0f1a0b,
        destructive: 0xdc2626,
        destructive_foreground: 0xffffff,
        border: 0xd9f99d,
        input: 0xd9f99d,
        ring: 0x84cc16,
    },
    ThemePreset {
        id: "orchid-dark",
        label: "Orchid",
        is_dark: true,
        background: 0x15111b,
        foreground: 0xffffff,
        card: 0x211c2c,
        card_foreground: 0xffffff,
        primary: 0xd946ef,
        primary_foreground: 0xffffff,
        secondary: 0x211c2c,
        secondary_foreground: 0xffffff,
        muted: 0x27232d,
        muted_foreground: 0xb89ecf,
        accent: 0x86198f,
        accent_foreground: 0xffffff,
        destructive: 0xbe123c,
        destructive_foreground: 0xffffff,
        border: 0x342b44,
        input: 0x342b44,
        ring: 0xd946ef,
    },
    ThemePreset {
        id: "orchid-light",
        label: "Orchid",
        is_dark: false,
        background: 0xfdf4ff,
        foreground: 0x15111b,
        card: 0xffffff,
        card_foreground: 0x15111b,
        primary: 0xa21caf,
        primary_foreground: 0xffffff,
        secondary: 0xfae8ff,
        secondary_foreground: 0x15111b,
        muted: 0xf0e7f2,
        muted_foreground: 0xd946ef,
        accent: 0xfae8ff,
        accent_foreground: 0x15111b,
        destructive: 0xe11d48,
        destructive_foreground: 0xffffff,
        border: 0xf0abfc,
        input: 0xf0abfc,
        ring: 0xd946ef,
    },
    ThemePreset {
        id: "teal-dark",
        label: "Teal",
        is_dark: true,
        background: 0x081a17,
        foreground: 0xffffff,
        card: 0x0f2d26,
        card_foreground: 0xffffff,
        primary: 0x14b8a6,
        primary_foreground: 0x081a17,
        secondary: 0x0f2d26,
        secondary_foreground: 0xffffff,
        muted: 0x1a2c29,
        muted_foreground: 0x5eead4,
        accent: 0x0f766e,
        accent_foreground: 0xffffff,
        destructive: 0xb91c1c,
        destructive_foreground: 0xffffff,
        border: 0x1c4037,
        input: 0x1c4037,
        ring: 0x14b8a6,
    },
    ThemePreset {
        id: "teal-light",
        label: "Teal",
        is_dark: false,
        background: 0xf0fdfa,
        foreground: 0x081a17,
        card: 0xffffff,
        card_foreground: 0x081a17,
        primary: 0x0d9488,
        primary_foreground: 0xffffff,
        secondary: 0xccfbf1,
        secondary_foreground: 0x081a17,
        muted: 0xe3f0ed,
        muted_foreground: 0x13b09f,
        accent: 0xccfbf1,
        accent_foreground: 0x081a17,
        destructive: 0xdc2626,
        destructive_foreground: 0xffffff,
        border: 0x99f6e4,
        input: 0x99f6e4,
        ring: 0x14b8a6,
    },
    ThemePreset {
        id: "charcoal-dark",
        label: "Charcoal",
        is_dark: true,
        background: 0x121214,
        foreground: 0xcccccc,
        card: 0x1e1e20,
        card_foreground: 0xcccccc,
        primary: 0x4b9cd3,
        primary_foreground: 0xffffff,
        secondary: 0x1e1e20,
        secondary_foreground: 0xcccccc,
        muted: 0x242426,
        muted_foreground: 0x6a6a6e,
        accent: 0x3a6d8c,
        accent_foreground: 0xffffff,
        destructive: 0xcc3333,
        destructive_foreground: 0xffffff,
        border: 0x2e2e32,
        input: 0x2e2e32,
        ring: 0x4b9cd3,
    },
    ThemePreset {
        id: "charcoal-light",
        label: "Charcoal",
        is_dark: false,
        background: 0xf2f2f3,
        foreground: 0x121214,
        card: 0xffffff,
        card_foreground: 0x121214,
        primary: 0x4489bc,
        primary_foreground: 0xffffff,
        secondary: 0xe8e8ea,
        secondary_foreground: 0x121214,
        muted: 0xe3e3e5,
        muted_foreground: 0x6a6a6e,
        accent: 0x2f5870,
        accent_foreground: 0x121214,
        destructive: 0xcc3333,
        destructive_foreground: 0xffffff,
        border: 0xd5d5d8,
        input: 0xd5d5d8,
        ring: 0x4489bc,
    },
    ThemePreset {
        id: "coffee-dark",
        label: "Coffee",
        is_dark: true,
        background: 0x1e1710,
        foreground: 0xede0d4,
        card: 0x2b2018,
        card_foreground: 0xede0d4,
        primary: 0xc49a6c,
        primary_foreground: 0x1e1710,
        secondary: 0x2b2018,
        secondary_foreground: 0xede0d4,
        muted: 0x302922,
        muted_foreground: 0x8c7a69,
        accent: 0x7b5b3a,
        accent_foreground: 0xffffff,
        destructive: 0xb95646,
        destructive_foreground: 0xffffff,
        border: 0x3d2e22,
        input: 0x3d2e22,
        ring: 0xc49a6c,
    },
    ThemePreset {
        id: "coffee-light",
        label: "Coffee",
        is_dark: false,
        background: 0xfbf7f2,
        foreground: 0x4a3728,
        card: 0xffffff,
        card_foreground: 0x4a3728,
        primary: 0xa67b5b,
        primary_foreground: 0xffffff,
        secondary: 0xf3ebe1,
        secondary_foreground: 0x4a3728,
        muted: 0xeeeae5,
        muted_foreground: 0x8c7a69,
        accent: 0xf3ebe1,
        accent_foreground: 0x4a3728,
        destructive: 0xd64b3b,
        destructive_foreground: 0xffffff,
        border: 0xe4d5c3,
        input: 0xe4d5c3,
        ring: 0xa67b5b,
    },
    ThemePreset {
        id: "frost-dark",
        label: "Frost",
        is_dark: true,
        background: 0x0f1923,
        foreground: 0xffffff,
        card: 0x1a2b3f,
        card_foreground: 0xffffff,
        primary: 0x7dd3fc,
        primary_foreground: 0x0f1923,
        secondary: 0x1a2b3f,
        secondary_foreground: 0xffffff,
        muted: 0x212b35,
        muted_foreground: 0x8bb8d6,
        accent: 0x0c4a6e,
        accent_foreground: 0xffffff,
        destructive: 0xb91c1c,
        destructive_foreground: 0xffffff,
        border: 0x253e57,
        input: 0x253e57,
        ring: 0x7dd3fc,
    },
    ThemePreset {
        id: "frost-light",
        label: "Frost",
        is_dark: false,
        background: 0xf0f9ff,
        foreground: 0x0f1923,
        card: 0xffffff,
        card_foreground: 0x0f1923,
        primary: 0x0369a1,
        primary_foreground: 0xffffff,
        secondary: 0xe0f2fe,
        secondary_foreground: 0x0f1923,
        muted: 0xe3ecf2,
        muted_foreground: 0x62a5c6,
        accent: 0xe0f2fe,
        accent_foreground: 0x0f1923,
        destructive: 0xdc2626,
        destructive_foreground: 0xffffff,
        border: 0xbae6fd,
        input: 0xbae6fd,
        ring: 0x0ea5e9,
    },
    ThemePreset {
        id: "everglow-dark",
        label: "Everglow",
        is_dark: true,
        background: 0x121212,
        foreground: 0xe8e6e3,
        card: 0x202020,
        card_foreground: 0xe8e6e3,
        primary: 0xbb86fc,
        primary_foreground: 0x121212,
        secondary: 0x202020,
        secondary_foreground: 0xe8e6e3,
        muted: 0x242424,
        muted_foreground: 0x908e8b,
        accent: 0x03dac5,
        accent_foreground: 0x121212,
        destructive: 0xcf6679,
        destructive_foreground: 0x121212,
        border: 0x333333,
        input: 0x333333,
        ring: 0xbb86fc,
    },
    ThemePreset {
        id: "everglow-light",
        label: "Everglow",
        is_dark: false,
        background: 0xf0efee,
        foreground: 0x121212,
        card: 0xffffff,
        card_foreground: 0x121212,
        primary: 0x9a66e3,
        primary_foreground: 0xffffff,
        secondary: 0xe8e7e5,
        secondary_foreground: 0x121212,
        muted: 0xe3e2e0,
        muted_foreground: 0x908e8b,
        accent: 0x00a896,
        accent_foreground: 0x121212,
        destructive: 0xcf6679,
        destructive_foreground: 0x121212,
        border: 0xd6d5d3,
        input: 0xd6d5d3,
        ring: 0x9a66e3,
    },
];

/// Find a ThemePreset by its ID. Falls back to "default-dark" if not found.
pub fn find_theme_preset(id: &str) -> &'static ThemePreset {
    THEME_PRESETS
        .iter()
        .find(|t| t.id == id)
        .unwrap_or(&THEME_PRESETS[0])
}

/// Apply the given settings theme to the GPUI application context.
///
/// `preset_id` is the active gallery preset (settings `theme_preset`); its
/// colours are pushed into the `gpui_component` tokens so inputs, popovers,
/// menus and buttons follow the gallery theme instead of the built-in
/// Adwaita defaults. Ignored in [`SettingsTheme::Gtk`], which maps the
/// probed desktop palette instead.
///
/// For [`SettingsTheme::Gtk`] the polarity follows the *palette* (`is_dark`),
/// never the GTK theme's name — and when no palette could be probed the
/// platform appearance decides, so a headless user still gets a sane mode.
pub fn apply_theme(theme: SettingsTheme, preset_id: &str, cx: &mut App) {
    match theme {
        SettingsTheme::Light => {
            apply_preset_to_component(find_theme_preset(preset_id), cx);
        }
        SettingsTheme::Gtk => {
            let dark = crate::gtk_theme::cached_is_dark()
                .unwrap_or_else(|| ThemeMode::from(cx.window_appearance()).is_dark());
            Theme::change(
                if dark { ThemeMode::Dark } else { ThemeMode::Light },
                None,
                cx,
            );
            if let Some(palette) = crate::gtk_theme::cached_palette() {
                apply_gtk_to_component(&palette, cx);
            }
        }
        SettingsTheme::Dark | SettingsTheme::System => {
            apply_preset_to_component(find_theme_preset(preset_id), cx);
        }
    }
}

/// Colour table shared by [`apply_gtk_to_component`] and
/// [`apply_preset_to_component`]: the fields both sources can supply, with
/// GTK-only extras (`warning` / `success`) filled by each caller.
#[derive(Debug, Clone, Copy)]
struct PaletteTokens {
    is_dark: bool,
    background: u32,
    foreground: u32,
    card: u32,
    muted: u32,
    muted_foreground: u32,
    primary: u32,
    primary_foreground: u32,
    accent: u32,
    accent_foreground: u32,
    destructive: u32,
    destructive_foreground: u32,
    warning: u32,
    success: u32,
    border: u32,
    input: u32,
    ring: u32,
}

/// Push a gallery [`ThemePreset`] into the global [`gpui_component::Theme`].
///
/// Without this the widget layer keeps the built-in Adwaita colours of the
/// mode alone: on most presets that leaves inputs, dropdown popovers and
/// modal buttons painted in the wrong palette — and on light presets the
/// dark defaults make their text unreadable.
pub fn apply_preset_to_component(preset: &ThemePreset, cx: &mut App) {
    Theme::change(
        if preset.is_dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        },
        None,
        cx,
    );

    // Status hues the web presets don't carry: neutral green/amber that read
    // on both polarities (same values the terminal ANSI fallback uses).
    let (warning, success) = if preset.is_dark {
        (0xeab308, 0x22c55e)
    } else {
        (0xca8a04, 0x16a34a)
    };

    apply_tokens(
        PaletteTokens {
            is_dark: preset.is_dark,
            background: preset.background,
            foreground: preset.foreground,
            card: preset.card,
            muted: preset.muted,
            muted_foreground: preset.muted_foreground,
            primary: preset.primary,
            primary_foreground: preset.primary_foreground,
            accent: preset.accent,
            accent_foreground: preset.accent_foreground,
            destructive: preset.destructive,
            destructive_foreground: preset.destructive_foreground,
            warning,
            success,
            border: preset.border,
            input: preset.input,
            ring: preset.ring,
        },
        cx,
    );
}

/// Push a resolved desktop palette into the global [`gpui_component::Theme`].
///
/// This is layer 1 of 3 (see `gtk_theme` docs): dialogs, menus, inputs and the
/// command palette read `gpui_component`'s tokens, so without this they keep
/// painting the built-in Adwaita colours while the app shell follows the
/// desktop. `Theme::sync_base` is mandatory afterwards — the scrollbar and
/// resize handles only see new colours through the Base projection.
pub fn apply_gtk_to_component(palette: &GtkPalette, cx: &mut App) {
    Theme::change(
        if palette.is_dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        },
        None,
        cx,
    );

    apply_tokens(
        PaletteTokens {
            is_dark: palette.is_dark,
            background: palette.background,
            foreground: palette.foreground,
            card: palette.card,
            muted: palette.muted,
            muted_foreground: palette.muted_foreground,
            primary: palette.primary,
            primary_foreground: palette.primary_foreground,
            accent: palette.accent,
            accent_foreground: palette.accent_foreground,
            destructive: palette.destructive,
            destructive_foreground: contrast_hex(palette.destructive),
            warning: palette.warning,
            success: palette.success,
            border: palette.border,
            input: mix_hex(palette.card, palette.foreground, 0.10),
            ring: palette.accent,
        },
        cx,
    );
}

/// Write [`PaletteTokens`] over the global [`gpui_component::Theme`] colours.
///
/// The brand colour is `primary`; `accent` stays available for surfaces that
/// intentionally follow the desktop accent. `Theme::sync_base` is mandatory
/// afterwards — the scrollbar and resize handles only see new colours through
/// the Base projection.
fn apply_tokens(t: PaletteTokens, cx: &mut App) {
    let h = |hex: u32| -> Hsla { rgb(hex).into() };
    let toward = |hex: u32, amount: f32| {
        if t.is_dark {
            mix_hex(hex, 0xffffff, amount)
        } else {
            mix_hex(hex, 0x000000, amount)
        }
    };

    let bg = t.background;
    let fg = t.foreground;
    let card = t.card;
    let muted = t.muted;
    let border = t.border;
    let primary = t.primary;
    let primary_fg = t.primary_foreground;
    let accent = t.accent;
    let accent_fg = t.accent_foreground;
    let error = t.destructive;
    let warning = t.warning;
    let success = t.success;
    let hover = toward(primary, 0.12);
    let active = toward(primary, 0.22);
    let surface_hover = mix_hex(card, fg, 0.08);
    let surface_active = mix_hex(card, fg, 0.14);
    let subtle = mix_hex(bg, fg, 0.28);

    let colors: &mut ThemeColor = &mut Theme::global_mut(cx).colors;

    colors.background = h(bg);
    colors.foreground = h(fg);
    colors.border = h(border);
    colors.accent = h(accent);
    colors.accent_foreground = h(accent_fg);

    colors.primary = h(primary);
    colors.primary_foreground = h(primary_fg);
    colors.primary_hover = h(hover);
    colors.primary_active = h(active);

    colors.secondary = h(card);
    colors.secondary_foreground = h(fg);
    colors.secondary_hover = h(surface_hover);
    colors.secondary_active = h(surface_active);

    colors.muted = h(muted);
    colors.muted_foreground = h(t.muted_foreground);

    colors.danger = h(error);
    colors.danger_foreground = h(t.destructive_foreground);
    colors.danger_hover = h(toward(error, 0.12));
    colors.danger_active = h(toward(error, 0.22));
    colors.success = h(success);
    colors.success_foreground = h(contrast_hex(success));
    colors.success_hover = h(toward(success, 0.12));
    colors.success_active = h(toward(success, 0.22));
    colors.warning = h(warning);
    colors.warning_foreground = h(contrast_hex(warning));
    colors.warning_hover = h(toward(warning, 0.12));
    colors.warning_active = h(toward(warning, 0.22));
    colors.info = h(primary);
    colors.info_foreground = h(primary_fg);

    colors.popover = h(card);
    colors.popover_foreground = h(fg);
    colors.input = h(t.input);
    colors.caret = h(primary);
    colors.ring = h(t.ring);
    colors.selection = rgba((primary << 8) | 0x55).into();

    colors.list = h(card);
    colors.list_hover = h(surface_hover);
    colors.list_active = h(surface_active);
    colors.list_active_border = h(border);
    colors.list_even = h(mix_hex(card, bg, 0.5));
    colors.list_head = h(muted);

    colors.sidebar = h(bg);
    colors.sidebar_foreground = h(fg);
    colors.sidebar_border = h(border);
    colors.sidebar_accent = h(primary);
    colors.sidebar_accent_foreground = h(primary_fg);
    colors.sidebar_primary = h(primary);
    colors.sidebar_primary_foreground = h(primary_fg);

    colors.title_bar = h(bg);
    colors.title_bar_border = h(border);
    colors.tab_bar = h(card);
    colors.tab_bar_segmented = h(muted);
    colors.tab = h(card);
    colors.tab_active = h(surface_active);
    colors.tab_foreground = h(t.muted_foreground);
    colors.tab_active_foreground = h(fg);

    colors.button = h(card);
    colors.button_foreground = h(fg);
    colors.button_hover = h(surface_hover);
    colors.button_active = h(surface_active);
    colors.button_primary = h(primary);
    colors.button_primary_foreground = h(primary_fg);
    colors.button_primary_hover = h(hover);
    colors.button_primary_active = h(active);
    colors.button_secondary = h(muted);
    colors.button_secondary_foreground = h(fg);
    colors.button_secondary_hover = h(surface_hover);
    colors.button_secondary_active = h(surface_active);
    colors.button_danger = h(error);
    colors.button_danger_foreground = h(t.destructive_foreground);
    colors.button_danger_hover = h(toward(error, 0.12));
    colors.button_danger_active = h(toward(error, 0.22));
    colors.button_success = h(success);
    colors.button_success_foreground = h(contrast_hex(success));
    colors.button_warning = h(warning);
    colors.button_warning_foreground = h(contrast_hex(warning));
    colors.button_info = h(primary);
    colors.button_info_foreground = h(primary_fg);

    colors.switch = h(subtle);
    colors.switch_thumb = h(fg);
    colors.skeleton = h(surface_hover);
    colors.group_box = h(card);
    colors.group_box_foreground = h(fg);
    colors.link = h(primary);
    colors.link_hover = h(hover);
    colors.link_active = h(active);
    colors.progress_bar = h(primary);
    colors.slider_bar = h(subtle);
    colors.slider_thumb = h(primary);
    colors.scrollbar = h(bg);
    colors.scrollbar_thumb = h(subtle);
    colors.scrollbar_thumb_hover = h(mix_hex(bg, fg, 0.42));
    colors.overlay = rgba((bg << 8) | 0xcc).into();
    colors.window_border = h(border);
    colors.table = h(card);
    colors.table_head = h(muted);
    colors.table_head_foreground = h(fg);
    colors.table_hover = h(surface_hover);
    colors.table_even = h(mix_hex(card, bg, 0.5));
    colors.table_row_border = h(border);

    Theme::sync_base(cx);
}

/// Flip between Dark and Light themes.
///
/// `Gtk` has no paired mode of its own, so toggling leaves desktop mode for a
/// concrete palette (Dark).
pub fn toggle_theme(current: SettingsTheme) -> SettingsTheme {
    match current {
        SettingsTheme::Light => SettingsTheme::Dark,
        SettingsTheme::Dark | SettingsTheme::System => SettingsTheme::Light,
        SettingsTheme::Gtk => SettingsTheme::Dark,
    }
}

/// Derive a full terminal ColorPalette for the given theme preset.
pub fn terminal_palette_for_preset(preset: &ThemePreset) -> webterm_terminal::ColorPalette {
    let fg = preset.foreground;
    let bg = preset.background;
    let cursor = preset.primary;
    let selection = preset.primary;

    // Desktop (GTK) mode: the user's CSS carries no ANSI table, so the 16
    // colours come from the nearest built-in preset by polarity while the
    // terminal *surface* (bg/fg/cursor/selection) is the desktop palette's own.
    // A terminal that follows the desktop background but keeps a close preset's
    // ANSI hues reads far better than one that ignores the desktop entirely.
    if preset.id == GTK_PRESET_ID {
        let fallback = if preset.is_dark {
            "tokyo-night-dark"
        } else {
            "default-light"
        };
        let mut palette = terminal_palette_for_preset(find_theme_preset(fallback));
        palette.background = rgb(bg).into();
        palette.foreground = rgb(fg).into();
        palette.cursor = rgb(cursor).into();
        palette.selection = Rgba {
            a: 0.40,
            ..rgb(selection)
        }
        .into();
        return palette;
    }

    let ansi = match preset.id {
        "dracula-dark" => [
            0x21222c, 0xff5555, 0x50fa7b, 0xf1fa8c, 0xbd93f9, 0xff79c6, 0x8be9fd, 0xf8f8f2,
            0x6272a4, 0xff6e6e, 0x69ff94, 0xffffa5, 0xd6acff, 0xff92df, 0xa4ffff, 0xffffff,
        ],
        "monokai-dark" => [
            0x272822, 0xf92672, 0xa6e22e, 0xf4bf75, 0x66d9ef, 0xae81ff, 0xa1efe4, 0xf8f8f2,
            0x75715e, 0xf92672, 0xa6e22e, 0xf4bf75, 0x66d9ef, 0xae81ff, 0xa1efe4, 0xf9f8f5,
        ],
        "tokyo-night-dark" => [
            0x15161e, 0xf7768e, 0x9ece6a, 0xe0af68, 0x7aa2f7, 0xbb9af7, 0x7dcfff, 0xa9b1d6,
            0x414868, 0xf7768e, 0x9ece6a, 0xe0af68, 0x7aa2f7, 0xbb9af7, 0x7dcfff, 0xc0caf5,
        ],
        "nord-dark" => [
            0x3b4252, 0xbf616a, 0xa3be8c, 0xebcb8b, 0x81a1c1, 0xb48ead, 0x88c0d0, 0xe5e9f0,
            0x4c566a, 0xbf616a, 0xa3be8c, 0xebcb8b, 0x81a1c1, 0xb48ead, 0x8fbcbb, 0xeceff4,
        ],
        "nord-light" => [
            0xe5e9f0, 0xbf616a, 0xa3be8c, 0xebcb8b, 0x81a1c1, 0xb48ead, 0x88c0d0, 0x3b4252,
            0x4c566a, 0xbf616a, 0xa3be8c, 0xebcb8b, 0x81a1c1, 0xb48ead, 0x8fbcbb, 0x2e3440,
        ],
        "gruvbox-dark" => [
            0x282828, 0xcc241d, 0x98971a, 0xd79921, 0x458588, 0xb16286, 0x689d6a, 0xa89984,
            0x928374, 0xfb4934, 0xb8bb26, 0xfabd2f, 0x83a598, 0xd3869b, 0x8ec07c, 0xebdbb2,
        ],
        "gruvbox-light" => [
            0xfbf1c7, 0xcc241d, 0x98971a, 0xd79921, 0x458588, 0xb16286, 0x689d6a, 0x7c6f64,
            0x928374, 0x9d0006, 0x79740e, 0xb57614, 0x076678, 0x8f3f71, 0x427b58, 0x3c3836,
        ],
        "catppuccin-dark" => [
            0x45475a, 0xf38ba8, 0xa6e3a1, 0xf9e2af, 0x89b4fa, 0xf5c2e7, 0x94e2d5, 0xbac2de,
            0x585b70, 0xf38ba8, 0xa6e3a1, 0xf9e2af, 0x89b4fa, 0xf5c2e7, 0x94e2d5, 0xa6adc8,
        ],
        "one-dark" => [
            0x282c34, 0xe06c75, 0x98c379, 0xe5c07b, 0x61afef, 0xc678dd, 0x56b6c2, 0xabb2bf,
            0x5c6370, 0xe06c75, 0x98c379, 0xe5c07b, 0x61afef, 0xc678dd, 0x56b6c2, 0xffffff,
        ],
        "github-dark" => [
            0x24292e, 0xea4a5a, 0x34d058, 0xffea7f, 0x2188ff, 0xb392f0, 0x39c5bb, 0xd1d5da,
            0x6a737d, 0xf97583, 0x85e89d, 0xffea7f, 0x79b8ff, 0xb392f0, 0x56d4dd, 0xfafbfc,
        ],
        "github-light" => [
            0x24292e, 0xd73a49, 0x28a745, 0xdbab09, 0x0366d6, 0x5a32a3, 0x0598bc, 0x6a737d,
            0x959da5, 0xcb2431, 0x22863a, 0xb08800, 0x005cc5, 0x4c2889, 0x005cc5, 0x24292e,
        ],
        "solarized-dark" => [
            0x073642, 0xdc322f, 0x859900, 0xb58900, 0x268bd2, 0xd33682, 0x2aa198, 0xeee8d5,
            0x586e75, 0xcb4b16, 0x586e75, 0x657b83, 0x839496, 0x6c71c4, 0x93a1a1, 0xfdf6e3,
        ],
        "solarized-light" => [
            0xeee8d5, 0xdc322f, 0x859900, 0xb58900, 0x268bd2, 0xd33682, 0x2aa198, 0x073642,
            0x93a1a1, 0xcb4b16, 0x93a1a1, 0x839496, 0x657b83, 0x6c71c4, 0x586e75, 0x002b36,
        ],
        _ => {
            let red = preset.destructive;
            let green = if preset.is_dark { 0x22c55e } else { 0x16a34a };
            let yellow = if preset.is_dark { 0xeab308 } else { 0xca8a04 };
            let blue = preset.primary;
            let magenta = preset.accent;
            let cyan = if preset.is_dark { 0x06b6d4 } else { 0x0891b2 };
            [
                bg,
                red,
                green,
                yellow,
                blue,
                magenta,
                cyan,
                fg,
                preset.muted_foreground,
                red,
                0x4ade80,
                0xfacc15,
                blue,
                magenta,
                0x22d3ee,
                fg,
            ]
        }
    };

    webterm_terminal::ColorPalette::from_rgb_u32(fg, bg, cursor, selection, ansi)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preset(id: &str) -> &'static ThemePreset {
        find_theme_preset(id)
    }

    /// A small fixture of the Tokyo Night GTK palette the dev machine resolves.
    fn tokyo_night() -> GtkPalette {
        GtkPalette {
            is_dark: true,
            background: 0x1a1b26,
            foreground: 0xa9b1d6,
            card: 0x282a38,
            card_foreground: 0xa9b1d6,
            primary: 0xf7768e,
            primary_foreground: 0x222222,
            secondary: 0x282a38,
            secondary_foreground: 0xa9b1d6,
            muted: 0x232433,
            muted_foreground: 0x646b8a,
            accent: 0xf7768e,
            accent_foreground: 0x222222,
            destructive: 0xf7768e,
            warning: 0xe0af68,
            success: 0x9ece6a,
            border: 0x373949,
        }
    }

    /// A5 parity lock: the built-in presets must keep their exact literal
    /// colours. These are the values web-tmux ships (`ui-themes.ts`), with the
    /// one documented deviation of the readability floor on `muted_foreground`
    /// — a refactor that reroutes them through the GTK palette would silently
    /// change every existing user's theme, so the literals are pinned here.
    #[test]
    fn builtin_preset_literals_are_locked() {
        // 102 presets ship today (51 dark + 51 light, the web-tmux gallery) —
        // keep the count so an accidental truncation fails loudly rather than
        // quietly shrinking the gallery.
        assert_eq!(THEME_PRESETS.len(), 102);

        // Every dark preset has a light counterpart and vice versa, so the
        // Settings mode filter can always flip a theme to its pair.
        for p in THEME_PRESETS {
            let counterpart = if p.is_dark {
                p.id.trim_end_matches("-dark").to_owned() + "-light"
            } else {
                p.id.trim_end_matches("-light").to_owned() + "-dark"
            };
            assert!(
                THEME_PRESETS.iter().any(|q| q.id == counterpart),
                "{} has no {} counterpart",
                p.id,
                counterpart
            );
            // Polarity must match the background the preset paints.
            assert_eq!(hex_luminance(p.background) > 0.5, !p.is_dark, "{}", p.id);
        }

        let dark = preset("default-dark");
        assert_eq!(dark.background, 0x1e1e1e);
        assert_eq!(dark.foreground, 0xd4d4d4);
        assert_eq!(dark.primary, 0xd4d4d4);
        assert_eq!(dark.primary_foreground, 0x1e1e1e);
        assert_eq!(dark.input, 0x3c3c3c);
        assert_eq!(dark.ring, 0x808080);
        assert!(dark.is_dark);

        let light = preset("default-light");
        assert_eq!(light.background, 0xfafafa);
        assert_eq!(light.foreground, 0x383a42);
        assert_eq!(light.destructive, 0xef4444);
        assert!(!light.is_dark);

        let dracula = preset("dracula-dark");
        assert_eq!(dracula.background, 0x282a36);
        assert_eq!(dracula.primary, 0xbd93f9);

        // Light counterparts added in the web-tmux gallery.
        let dracula_light = preset("dracula-light");
        assert_eq!(dracula_light.background, 0xf7f6fb);
        assert_eq!(dracula_light.primary, 0x8b5cf6);

        let cyberpunk_light = preset("cyberpunk-light");
        assert_eq!(cyberpunk_light.background, 0xf6f4fe);
        assert_eq!(cyberpunk_light.ring, 0xe80074);

        let tokyo = preset("tokyo-night-dark");
        assert_eq!(tokyo.background, 0x1a1b26);
        assert_eq!(tokyo.primary, 0x7aa2f7);
        // Web-tmux ships 0x565f89 (2.35:1 on its card) — below the readability
        // floor, so the port carries the nudged value. The web counterpart
        // stays visually identical at arm's length, but the text is legible.
        assert_eq!(tokyo.muted_foreground, 0x5b648e);
        assert_eq!(preset("one-dark").muted_foreground, 0x707784);
        assert_eq!(preset("mint-light").muted_foreground, 0x46aba1);

        let nord = preset("nord-dark");
        assert_eq!(nord.background, 0x2e3440);

        let catppuccin = preset("catppuccin-dark");
        assert_eq!(catppuccin.background, 0x1e1e2e);

        // Unknown ids still fall back to the first preset, unchanged.
        assert_eq!(find_theme_preset("does-not-exist").id, "default-dark");
    }

    /// The point of the gallery rework: secondary text must stay visible on
    /// every preset. `muted_foreground` drives the host-card subtitles, tab
    /// captions and search hints, so it must clear the floor against both
    /// surfaces it can be painted on.
    #[test]
    fn muted_foreground_meets_the_readability_floor() {
        for p in THEME_PRESETS {
            for surface in [p.card, p.background] {
                let ratio = hex_contrast(p.muted_foreground, surface);
                assert!(
                    ratio >= MIN_MUTED_CONTRAST,
                    "{}: muted {:06x} on {:06x} is {ratio:.2}:1",
                    p.id,
                    p.muted_foreground,
                    surface
                );
            }
        }
    }

    #[test]
    fn readable_muted_nudges_only_when_needed() {
        // At or above the floor: returned untouched.
        assert_eq!(readable_muted(0x808080, &[0x1e1e1e], 0xd4d4d4), 0x808080);
        // Below the floor: smallest 5% blend toward fg that clears it — the
        // exact values baked into the gallery (tokyo-night / one-dark).
        assert_eq!(
            readable_muted(0x565f89, &[0x24283b, 0x1a1b26], 0xc0caf5),
            0x5b648e
        );
        assert_eq!(readable_muted(0x5c6370, &[0x353a44, 0x282c34], 0xabb2bf), 0x707784);
        // A colour pinned to its own surface still converges — to the first
        // 5% blend that clears the floor (t=0.30), never further than needed.
        assert_eq!(readable_muted(0x202020, &[0x202020], 0xffffff), 0x636363);
    }

    #[test]
    fn gtk_preset_maps_onto_the_theme_preset_shape() {
        let p = tokyo_night();
        let mapped = p.to_preset();
        assert_eq!(mapped.id, GTK_PRESET_ID);
        assert_eq!(mapped.label, "Desktop (GTK)");
        assert!(mapped.is_dark);
        assert_eq!(mapped.background, 0x1a1b26);
        assert_eq!(mapped.foreground, 0xa9b1d6);
        assert_eq!(mapped.card, 0x282a38);
        assert_eq!(mapped.accent, 0xf7768e);
        assert_eq!(mapped.accent_foreground, 0x222222);
        assert_eq!(mapped.destructive, 0xf7768e);
        assert_eq!(mapped.border, 0x373949);
    }

    #[test]
    fn hex_helpers_mix_and_measure() {
        assert_eq!(mix_hex(0x000000, 0xffffff, 0.0), 0x000000);
        assert_eq!(mix_hex(0x000000, 0xffffff, 1.0), 0xffffff);
        assert_eq!(mix_hex(0x000000, 0xffffff, 0.5), 0x808080);
        assert!(hex_luminance(0xffffff) > hex_luminance(0x000000));
        // WCAG ratio anchors: black-on-white is the 21:1 ceiling, a colour
        // against itself the 1:1 floor.
        assert!((hex_contrast(0x000000, 0xffffff) - 21.0).abs() < 0.01);
        assert!((hex_contrast(0x1a1b26, 0x1a1b26) - 1.0).abs() < 0.01);
        assert!(hex_contrast(0xffffff, 0x1a1b26) > hex_contrast(0x808080, 0x1a1b26));
        assert_eq!(contrast_hex(0x1a1b26), 0xffffff);
        assert_eq!(contrast_hex(0xfafafa), 0x000000);
        assert_eq!(
            hex_from_rgba(Rgba {
                r: 1.0,
                g: 0.5,
                b: 0.0,
                a: 0.5,
            }),
            0xff8000
        );
    }

    /// GTK mode must keep the terminal's surface on the desktop palette while
    /// borrowing the ANSI table from the nearest built-in preset.
    #[test]
    fn gtk_terminal_palette_follows_desktop_surface() {
        let gtk = tokyo_night().to_preset();
        let palette = terminal_palette_for_preset(&gtk);

        assert_eq!(palette.background, rgb(0x1a1b26).into());
        assert_eq!(palette.foreground, rgb(0xa9b1d6).into());
        assert_eq!(palette.cursor, rgb(0xf7768e).into());

        // ANSI comes from tokyo-night-dark, the nearest dark preset.
        let fallback = terminal_palette_for_preset(preset("tokyo-night-dark"));
        assert_eq!(palette.ansi, fallback.ansi);

        // Selecting text must stay translucent.
        let selection: Rgba = palette.selection.into();
        assert!(selection.a < 1.0);
    }

    #[test]
    fn light_gtk_terminal_palette_borrows_light_ansi() {
        let mut gtk = tokyo_night();
        gtk.is_dark = false;
        gtk.background = 0xfafafa;
        gtk.foreground = 0x27272a;
        let palette = terminal_palette_for_preset(&gtk.to_preset());
        let fallback = terminal_palette_for_preset(preset("default-light"));
        assert_eq!(palette.ansi, fallback.ansi);
        assert_eq!(palette.background, rgb(0xfafafa).into());
    }

    #[test]
    fn toggle_theme_covers_every_variant() {
        assert_eq!(toggle_theme(SettingsTheme::Dark), SettingsTheme::Light);
        assert_eq!(toggle_theme(SettingsTheme::Light), SettingsTheme::Dark);
        assert_eq!(toggle_theme(SettingsTheme::System), SettingsTheme::Light);
        assert_eq!(toggle_theme(SettingsTheme::Gtk), SettingsTheme::Dark);
    }
}
