use crate::app_state::{Language, ReceiverState};
use crate::receiver::handle_receive;
use dioxus::{html::geometry::PixelsVector2D, prelude::*};
use rust_i18n::t;
use std::rc::Rc;
use std::sync::atomic::Ordering::Relaxed;

#[component]
pub fn ReceiverPage() -> Element {
    let receiver_state = use_context::<ReceiverState>();
    let mut port = receiver_state.port;
    let mut dir = receiver_state.dir;
    let mut logs = receiver_state.logs;
    let log_tx = receiver_state.log_tx;
    let mut is_running = receiver_state.is_running;

    let mut log_container = use_signal(|| None::<Rc<MountedData>>);

    let language = use_context::<Signal<Language>>();
    _ = language.read();

    use_effect(move || {
        logs.read();

        spawn(async move {
            if let Some(container) = log_container() {
                let scroll_size = container.get_scroll_size().await;
                if let Ok(size) = scroll_size {
                    _ = container
                        .scroll(
                            PixelsVector2D::new(0.0, size.height),
                            ScrollBehavior::Smooth,
                        )
                        .await;
                }
            }
        });
    });

    rsx! {
        div { class: "flex flex-col h-full gap-8",
            div { class: "flex-1 min-h-0 flex flex-col items-center justify-center shadow rounded-lg bg-base-100",
                fieldset { class: "fieldset w-2/5",
                    input { class: "file-input",
                        r#type: "file",
                        directory: true,
                        onchange: move |e| {
                            for file in e.files() {
                                dir.set(file.path());
                            }

                        }
                    }
                    p { class: "label break-all whitespace-normal", r#"{t!("save_path")} : {dir:?}"# }
                }
                fieldset { class: "fieldset",
                    legend { class: "fieldset-legend text-gray-500", r#"{t!("port")}"# }
                    input { class: "input",
                        r#type: "number",
                        placeholder: "8000",
                        value: "{port}",
                        oninput: move |e| {
                            if let Ok(p) = e.value().parse() {
                                port.set(p);
                            }
                        }
                    }
                }
                button {
                    class: if is_running.read().load(Relaxed) {
                        "btn btn-error text-white px-20 mt-6"
                    } else {
                        "btn bg-blue-500 hover:bg-blue-600 text-white px-20 mt-6"
                    },
                    onclick: move |_| {
                        if is_running.read().load(Relaxed) {
                            is_running.write().store(false, Relaxed);
                        } else {
                            is_running.write().store(true, Relaxed);
                            let is_running = is_running();
                            let dir = dir();
                            let log_tx = log_tx();
                            let addr = format!("0.0.0.0:{port}");

                            std::thread::spawn(move || {
                                match handle_receive(addr, dir, log_tx.clone(), is_running.clone()) {
                                    Ok(()) => {
                                        is_running.store(false, Relaxed);
                                        _ = log_tx.unbounded_send(t!("stop_server").to_string());
                                    }
                                    Err(e) => {
                                        is_running.store(false, Relaxed);
                                        _ = log_tx.unbounded_send(format!("{} : {}",t!("start_server_fail"), e));
                                    }
                                }
                            });
                        }
                    },
                    if is_running.read().load(std::sync::atomic::Ordering::Relaxed) {
                        r#"{t!("stop_server")}"#
                    } else {
                        r#"{t!("start_server")}"#
                    }
                }
            }
            div { class: "h-1/3 fieldset shadow rounded-box bg-base-100 px-4 flex relative",
                div { class: "absolute -top-3 left-4 flex items-center gap-2",
                    p { class: "font-bold text-gray-500", r#"{t!("logs")}"# }
                    div { class: "tooltip ", "data-tip": r#"{t!("clear_logs")}"#,
                        button { class: "btn btn-xs btn-error btn-outline btn-square",
                            onclick: move |_| logs.clear(),
                            svg {
                                class: "size-4",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.5",
                                view_box: "0 0 24 24",
                                path {
                                    d: "m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                }
                            }
                        }
                    }
                }
                div { class: "overflow-y-auto flex-1 mt-3",
                    onmounted: move |e| log_container.set(Some(e.data())),
                    for log in logs.iter() {
                        p { class: "break-all whitespace-pre-wrap",
                            "{log}"
                        }
                    }
                }
            }
        }
    }
}
