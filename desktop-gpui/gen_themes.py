import io
import re

# Parse the web themes.ts
ts = io.open(r"..\fe\src\features\settings\data\themes.ts", encoding="utf-8").read()
web = []
for m in re.finditer(
    r'\{ name: "([^"]+)", label: "([^"]+)", colors: \{(.*?)\} \}',
    ts, re.DOTALL,
):
    name, label, body = m.group(1), m.group(2), m.group(3)
    colors = {}
    for cm in re.finditer(r"(\w+): \"#([0-9a-fA-F]{6})\"", body):
        colors[cm.group(1)] = cm.group(2)
    web.append((name, label, colors))
print(f"web themes: {len(web)}")

# Existing GPUI preset ids
rs = io.open(r"crates\webterm\src\theme.rs", encoding="utf-8").read()
existing = set(re.findall(r'id: "([^"]+)"', rs))
print(f"existing: {len(existing)}")

missing = [(n, l, c) for (n, l, c) in web if n not in existing]
print(f"missing: {len(missing)}")

# Emit Rust entries appended to THEME_PRESETS before its closing `];`
def to_field(colors):
    def h(k):
        v = colors.get(k)
        if v is None:
            return "0x000000"
        return "0x" + v.lower()
    return (
        f"        background: {h('background')},\n"
        f"        foreground: {h('foreground')},\n"
        f"        card: {h('card')},\n"
        f"        card_foreground: {h('cardForeground')},\n"
        f"        primary: {h('primary')},\n"
        f"        primary_foreground: {h('primaryForeground')},\n"
        f"        secondary: {h('secondary')},\n"
        f"        secondary_foreground: {h('secondaryForeground')},\n"
        f"        muted: {h('muted')},\n"
        f"        muted_foreground: {h('mutedForeground')},\n"
        f"        accent: {h('accent')},\n"
        f"        accent_foreground: {h('accentForeground')},\n"
        f"        destructive: {h('destructive')},\n"
        f"        border: {h('border')},\n"
    )


def is_dark(colors):
    bg = colors.get("background", "000000").lstrip("#")
    r, g, b = int(bg[0:2], 16), int(bg[2:4], 16), int(bg[4:6], 16)
    return (0.299 * r + 0.587 * g + 0.114 * b) < 128


entries = []
for name, label, colors in missing:
    entries.append(
        f'    ThemePreset {{\n'
        f'        id: "{name}",\n'
        f'        label: "{label}",\n'
        f'        is_dark: {"true" if is_dark(colors) else "false"},\n'
        f'{to_field(colors)}'
        f'    }},'
    )

new_block = "\n".join(entries) + "\n"
marker = "];\n\npub fn find_theme_preset"
idx = rs.index(marker)
rs = rs[:idx] + new_block + rs[idx:]
io.open(r"crates\webterm\src\theme.rs", "w", encoding="utf-8", newline="").write(rs)
print("theme.rs updated")
