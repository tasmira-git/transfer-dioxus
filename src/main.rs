use dioxus::prelude::*;
use transfer_dioxus::ui::Route;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(|| {
        rsx! {
            Stylesheet{ href: TAILWIND_CSS }
            Router::<Route> {}
        }
    });
}
