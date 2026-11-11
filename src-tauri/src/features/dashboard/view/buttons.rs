//! A small builder for the computed buttons, so every presenter writes them the same way.

use crate::features::dashboard::dto::{
    DashboardAction, DashboardButtonDto, DashboardButtonStyle, DashboardConfirmationDto,
};
use crate::session::Lane;

/// An outlined button that does `action` and waits for nothing.
pub fn button(label: impl Into<String>, action: DashboardAction) -> DashboardButtonDto {
    DashboardButtonDto {
        label: label.into(),
        icon: None,
        style: DashboardButtonStyle::Outlined,
        disabled_reason: None,
        lane: None,
        action,
        confirm: None,
    }
}

pub trait ButtonExt {
    fn style(self, style: DashboardButtonStyle) -> Self;
    fn icon(self, icon: &str) -> Self;
    fn disabled(self, reason: Option<&str>) -> Self;
    /// The button changes the library (or runs on the AI), so it waits while `lane` is busy.
    fn waits_for(self, lane: Lane) -> Self;
    fn confirm(self, confirmation: DashboardConfirmationDto) -> Self;
}

impl ButtonExt for DashboardButtonDto {
    fn style(mut self, style: DashboardButtonStyle) -> Self {
        self.style = style;
        self
    }

    fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_owned());
        self
    }

    fn disabled(mut self, reason: Option<&str>) -> Self {
        self.disabled_reason = reason.map(str::to_owned);
        self
    }

    fn waits_for(mut self, lane: Lane) -> Self {
        self.lane = Some(lane);
        self
    }

    fn confirm(mut self, confirmation: DashboardConfirmationDto) -> Self {
        self.confirm = Some(confirmation);
        self
    }
}

pub fn confirmation(
    title: String,
    message: String,
    affected: Vec<String>,
    confirm_label: &str,
) -> DashboardConfirmationDto {
    DashboardConfirmationDto { title, message, affected, confirm_label: confirm_label.to_owned() }
}
