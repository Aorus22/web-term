//! Keyboard shortcut action definitions for tab management.

use gpui::*;

actions!(
    terminal_tabs,
    [
        NewTab,
        CloseTab,
        NextTab,
        PrevTab,
        JumpTab1,
        JumpTab2,
        JumpTab3,
        JumpTab4,
        JumpTab5,
        JumpTab6,
        JumpTab7,
        JumpTab8,
        JumpTab9,
    ]
);

/// Register standard tab navigation keybindings.
pub fn bind_tab_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-t", NewTab, None),
        KeyBinding::new("ctrl-w", CloseTab, None),
        KeyBinding::new("ctrl-tab", NextTab, None),
        KeyBinding::new("ctrl-shift-tab", PrevTab, None),
        KeyBinding::new("alt-1", JumpTab1, None),
        KeyBinding::new("alt-2", JumpTab2, None),
        KeyBinding::new("alt-3", JumpTab3, None),
        KeyBinding::new("alt-4", JumpTab4, None),
        KeyBinding::new("alt-5", JumpTab5, None),
        KeyBinding::new("alt-6", JumpTab6, None),
        KeyBinding::new("alt-7", JumpTab7, None),
        KeyBinding::new("alt-8", JumpTab8, None),
        KeyBinding::new("alt-9", JumpTab9, None),
    ]);
}
