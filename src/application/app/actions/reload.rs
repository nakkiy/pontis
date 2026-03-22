use crate::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReloadDecision {
    Start,
    Rejected,
}

impl App {
    pub(crate) fn request_reload(&mut self) -> ReloadDecision {
        if !self.reload_supported {
            self.set_temporary_status("reload not available in this mode");
            return ReloadDecision::Rejected;
        }

        if self.has_unsaved_changes() {
            self.set_temporary_status("reload unavailable with unsaved changes");
            return ReloadDecision::Rejected;
        }

        ReloadDecision::Start
    }
}
