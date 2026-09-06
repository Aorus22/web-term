use gpui::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    Connecting,
    Connected,
    Reconnecting { attempt: usize, next_retry_secs: usize },
    Disconnected(Option<String>),
}

pub struct TerminalTab {
    pub id: u64,
    pub session_id: Option<String>,
    pub title: String,
    pub status: SessionStatus,
    pub view: Option<AnyView>,
    pub session_type: String,
    pub connection_id: Option<String>,
}

impl TerminalTab {
    pub fn is_connected(&self) -> bool {
        self.status == SessionStatus::Connected
    }

    pub fn is_local(&self) -> bool {
        self.session_type == "local"
    }

    pub fn is_restartable(&self) -> bool {
        matches!(self.status, SessionStatus::Disconnected(_))
    }

    pub fn disconnect(&mut self) {
        self.status = SessionStatus::Disconnected(None);
    }
}

pub struct TerminalSessionManager {
    tabs: Vec<TerminalTab>,
    active_tab_index: usize,
    next_tab_id: u64,
}

impl Default for TerminalSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalSessionManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_index: 0,
            next_tab_id: 0,
        }
    }

    pub fn alloc_tab_id(&mut self) -> u64 {
        self.next_tab_id += 1;
        self.next_tab_id
    }

    pub fn tabs(&self) -> &[TerminalTab] {
        &self.tabs
    }

    pub fn tabs_mut(&mut self) -> &mut [TerminalTab] {
        &mut self.tabs
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    pub fn active_index(&self) -> usize {
        self.active_tab_index
    }

    pub fn active_tab(&self) -> Option<&TerminalTab> {
        self.tabs.get(self.active_tab_index)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut TerminalTab> {
        self.tabs.get_mut(self.active_tab_index)
    }

    pub fn add_tab(&mut self, tab: TerminalTab) -> usize {
        self.tabs.push(tab);
        self.active_tab_index = self.tabs.len() - 1;
        self.active_tab_index
    }

    pub fn switch_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active_tab_index = index;
            true
        } else {
            false
        }
    }

    pub fn close_tab(&mut self, index: usize) -> Option<TerminalTab> {
        if index >= self.tabs.len() {
            return None;
        }

        let mut tab = self.tabs.remove(index);
        tab.disconnect();

        if self.tabs.is_empty() {
            self.active_tab_index = 0;
        } else if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        }

        Some(tab)
    }

    pub fn next_tab(&mut self) -> usize {
        self.cycle_next()
    }

    pub fn prev_tab(&mut self) -> usize {
        self.cycle_prev()
    }

    pub fn cycle_next(&mut self) -> usize {
        if self.tabs.is_empty() {
            return 0;
        }
        self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
        self.active_tab_index
    }

    pub fn cycle_prev(&mut self) -> usize {
        if self.tabs.is_empty() {
            return 0;
        }
        self.active_tab_index = if self.active_tab_index == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab_index - 1
        };
        self.active_tab_index
    }

    pub fn jump_to(&mut self, index: usize) -> bool {
        self.switch_tab(index)
    }

    pub fn set_active(&mut self, index: usize) {
        self.switch_tab(index);
    }
}

