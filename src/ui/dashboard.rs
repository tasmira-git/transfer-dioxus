use crate::{
    app_state::{Language, ReceiverState, SenderState},
    ui::Route,
};
use dioxus::prelude::*;
use futures_util::StreamExt;
use rust_i18n::t;
use std::time::Duration;
use tokio::time::interval;

#[component]
pub fn DashboardLayout() -> Element {
    let current_route = use_route::<Route>();

    let receiver_logs = use_signal(Vec::new);
    let receiver_tx = use_coroutine(move |rx: UnboundedReceiver<String>| async move {
        handle_logs(rx, receiver_logs).await
    });

    let sender_logs = use_signal(Vec::new);
    let sender_log_tx = use_coroutine(move |rx: UnboundedReceiver<String>| async move {
        handle_logs(rx, sender_logs).await
    });
    let mut send_progress = use_signal(|| (0.0, "0.00MB/s".to_string()));
    let sender_progress_tx =
        use_coroutine(move |mut rx: UnboundedReceiver<(f64, String)>| async move {
            while let Some((percent, speed)) = rx.next().await {
                send_progress.set((percent, speed));
            }
        });

    use_context_provider(|| {
        SenderState::new(
            Signal::new(sender_log_tx.tx()),
            sender_logs,
            Signal::new(sender_progress_tx.tx()),
            send_progress,
        )
    });
    use_context_provider(|| ReceiverState::new(Signal::new(receiver_tx.tx()), receiver_logs));

    use_hook(|| rust_i18n::set_locale("en"));
    let language = use_signal(|| Language::English);
    use_context_provider(|| language);

    _ = language.read();

    rsx! {
        div { class: "flex h-screen gap-10 bg-base-300",
            div { class: "w-18 flex flex-col bg-base-100 shadow items-center pb-4",
                div { class: "flex-1 flex flex-col justify-evenly ",
                    Link {
                        class: if current_route == Route::SenderPage {
                            "btn btn-ghost btn-square btn-xl flex flex-col bg-blue-500/10 text-blue-500"
                        } else {
                            "btn btn-ghost btn-square btn-xl flex flex-col text-gray-500"
                        },
                        to: Route::SenderPage,
                        svg {
                            class: "lucide lucide-send-icon lucide-send",
                            fill: "none",
                            height: "24",
                            stroke: "currentColor",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            view_box: "0 0 24 24",
                            width: "24",
                            xmlns: "http://www.w3.org/2000/svg",
                            path { d: "M14.536 21.686a.5.5 0 0 0 .937-.024l6.5-19a.496.496 0 0 0-.635-.635l-19 6.5a.5.5 0 0 0-.024.937l7.93 3.18a2 2 0 0 1 1.112 1.11z" }
                            path { d: "m21.854 2.147-10.94 10.939" }
                        }
                        span { class: "text-xs", r#"{t!("send")}"# }
                    }
                    Link {
                        class: if current_route == Route::ReceiverPage {
                            "btn btn-ghost btn-square btn-xl flex flex-col bg-blue-500/10 text-blue-500"
                        } else {
                            "btn btn-ghost btn-square btn-xl flex flex-col text-gray-500"
                        },
                        to: Route::ReceiverPage,
                        svg {
                            class: "lucide lucide-download-icon lucide-download",
                            fill: "none",
                            height: "24",
                            stroke: "currentColor",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            view_box: "0 0 24 24",
                            width: "24",
                            xmlns: "http://www.w3.org/2000/svg",
                            path { d: "M12 15V3" }
                            path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                            path { d: "m7 10 5 5 5-5" }
                        }
                        span { class: "text-xs", r#"{t!("receive")}"# }
                    }
                }
                Settings { }
            }
            div { class: "flex-1 py-4 pr-8",
                Outlet::<Route> {}
            }
        }
    }
}
async fn handle_logs(mut rx: UnboundedReceiver<String>, mut logs: Signal<Vec<String>>) {
    let mut buffer = Vec::new();
    let mut tick = interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            Some(msg) = rx.next() => {
                buffer.push(msg);

                if buffer.len() > 1000 {
                    logs.extend(buffer.drain(..));
                }
            }
            _ = tick.tick() => {
                if !buffer.is_empty() {
                    logs.extend(buffer.drain(..));
                }
                if logs.len() > 500 {
                    let remove_len = logs.len() - 500;
                    logs.write().drain(0..remove_len);
                }
            }
        }
    }
}
#[component]
fn Settings() -> Element {
    let mut is_open = use_signal(|| false);
    let mut theme = use_signal(|| "light".to_string());
    let mut language = use_context::<Signal<Language>>();

    let change_theme = move |e: Event<FormData>| {
        theme.set(e.value());
    };

    rsx! {
        button { class: "group cursor-pointer p-1",
            onclick: move |_| is_open.set(true),
            svg {
                class: "stroke-2 group-hover:stroke-3",
                fill: "none",
                height: "24",
                stroke: "currentColor",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                view_box: "0 0 24 24",
                width: "24",
                xmlns: "http://www.w3.org/2000/svg",
                path { d: "M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915" }
                circle { cx: "12", cy: "12", r: "3" }
            }
        }
        dialog {
            class: if is_open() { "modal modal-open" } else { "modal" },

            div { class: "modal-box overflow-visible flex flex-col items-center gap-4",
                h1 { class: "text-4xl font-bold self-start mb-4", r#"{t!("settings")}"# }
                div { class: "flex items-center justify-between w-1/2",
                    label { class: "text-2xl", r#"{t!("theme")} :"# }
                    div { class: "dropdown dropdown-end",
                        div { class: "btn", role: "button", tabindex: "0",
                            "{theme}"
                            svg {
                                class: "lucide lucide-chevron-down-icon lucide-chevron-down",
                                fill: "none",
                                height: "24",
                                stroke: "currentColor",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                view_box: "0 0 24 24",
                                width: "24",
                                xmlns: "http://www.w3.org/2000/svg",
                                path { d: "m6 9 6 6 6-6" }
                            }
                        }
                        ul { class: "dropdown-content menu bg-base-100 rounded-box shadow z-1 p-2",
                            tabindex: "-1",
                            li {
                                input { class: "theme-controller btn btn-sm btn-ghost", r#type: "radio", name: "theme",
                                    onchange: change_theme,
                                    checked: *theme.read() == "light",
                                    aria_label: "Light",value: "light"
                                }
                            }
                            li {
                                input { class: "theme-controller btn btn-sm btn-ghost", r#type: "radio", name: "theme",
                                    onchange: change_theme,
                                    checked: *theme.read() == "dark",
                                    aria_label: "Dark",value: "dark"
                                }
                            }
                            li {
                                input { class: "theme-controller btn btn-sm btn-ghost", r#type: "radio", name: "theme",
                                    onchange: change_theme,
                                    checked: *theme.read() == "cupcake",
                                    aria_label: "Cupcake",value: "cupcake"
                                }
                            }
                            li {
                                input { class: "theme-controller btn btn-sm btn-ghost", r#type: "radio", name: "theme",
                                    onchange: change_theme,
                                    checked: *theme.read() == "lemonade",
                                    aria_label: "Lemonade",value: "lemonade"
                                }
                            }
                        }
                    }
                }
                div { class: "flex items-center justify-between w-1/2",
                    label { class: "text-2xl", r#"{t!("language")} :"# }
                    div { class: "dropdown dropdown-end",
                        div { class: "btn", role: "button", tabindex: "0",
                            "{language}"
                            svg {
                                class: "lucide lucide-chevron-down-icon lucide-chevron-down",
                                fill: "none",
                                height: "24",
                                stroke: "currentColor",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                view_box: "0 0 24 24",
                                width: "24",
                                xmlns: "http://www.w3.org/2000/svg",
                                path { d: "m6 9 6 6 6-6" }
                            }
                        }
                        ul { class: "dropdown-content menu bg-base-100 rounded-box shadow z-1 p-2",
                            tabindex: "-1",
                            li {
                                input { class: "btn btn-sm btn-ghost", r#type: "radio", name: "language",
                                    onchange: move |e| {
                                        rust_i18n::set_locale(&e.value());
                                        language.set(Language::Chinese);
                                    },
                                    checked: *language.read() == Language::Chinese,
                                    aria_label: "{Language::Chinese}", value: "zh",
                                }
                            }
                            li {
                                input { class: "btn btn-sm btn-ghost", r#type: "radio", name: "language",
                                    onchange: move |e| {
                                        rust_i18n::set_locale(&e.value());
                                        language.set(Language::English);
                                    },
                                    checked: *language.read() == Language::English,
                                    aria_label: "{Language::English}", value: "en"
                                }
                            }
                        }
                    }
                }
                button { class: "btn bg-blue-500 hover:bg-blue-600 text-white self-end mt-4",
                    onclick: move |_| is_open.set(false),
                    r#"{t!("close")}"#
                }
            }
        }
    }
}
