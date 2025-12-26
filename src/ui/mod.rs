mod dashboard;
mod receiver;
mod sender;

use dashboard::DashboardLayout;
use dioxus::prelude::*;
use receiver::ReceiverPage;
use sender::SenderPage;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(DashboardLayout)]
    #[redirect("/", || Route::SenderPage)]
    #[route("/sender")]
    SenderPage,
    #[route("/receiver")]
    ReceiverPage,
}
