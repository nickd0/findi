use crate::network::ping_result::PingResult;
use crate::network::host::Host;
use crate::ui::pages::PageContent;
use crate::ui::notification::Notification;
use crate::ui::modal::Modal;

use std::net::Ipv4Addr;

pub trait Action {}

#[allow(dead_code)]
pub enum AppAction {
  BuildHosts(Vec<Ipv4Addr>),
  UpdatePingResult(Ipv4Addr, PingResult),
  UpdateHost(Host),
  SetQuery(String),
  SetInputErr(bool),
  SetHostSearchRun(bool),
  NewQuery(Vec<Ipv4Addr>),
  TableSelect(usize),
  ShiftFocus(PageContent),
  SetNotification(Option<Notification>),
  SetModal(Option<Modal>),
  RestartQuery,
  IterateFocus
}

impl Action for AppAction {}
