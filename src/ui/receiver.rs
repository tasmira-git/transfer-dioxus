use std::rc::Rc;

use dioxus::{html::geometry::PixelsVector2D, prelude::*};

use crate::{app_state::AppState, receiver::handle_receive};

#[component]
pub fn ReceiverPage() -> Element {
    let app_state = use_context::<AppState>();
    let mut port = app_state.receiver.port;
    let mut dir = app_state.receiver.dir;
    let mut logs = app_state.receiver.logs;
    let log_tx = app_state.receiver.log_tx;
    let mut is_running = app_state.receiver.is_running;

    let mut log_container = use_signal(|| None::<Rc<MountedData>>);

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
        div { class: "flex flex-col h-full gap-4",
            div { class: "flex-1 min-h-0 flex flex-col items-center shadow rounded-lg bg-base-200",
                fieldset { class: "fieldset w-2/5",
                    legend { class: "fieldset-legend", "选择保存文件的路径" }
                    input { class: "file-input file-input-info",
                        r#type: "file",
                        directory: true,
                        onchange: move |e| {
                            for file in e.files() {
                                dir.set(file.path());
                            }

                        }
                    }
                    p { class: "label break-all whitespace-normal", "保存路径：{dir:?}"}
                }
                fieldset { class: "fieldset",
                    legend { class: "fieldset-legend", "监听端口" }
                    input { class: "input input-info",
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
                    class: if is_running.read().load(std::sync::atomic::Ordering::Relaxed) {
                        "btn btn-error px-20 mt-6"
                    } else {
                        "btn btn-info px-20 mt-6"
                    },
                    onclick: move |_| {
                        if is_running.read().load(std::sync::atomic::Ordering::Relaxed) {
                            is_running.write().store(false, std::sync::atomic::Ordering::Relaxed);
                        } else {
                            is_running.write().store(true, std::sync::atomic::Ordering::Relaxed);
                            let running = is_running();
                            let dir = dir();
                            let tx = log_tx();
                            let addr = format!("0.0.0.0:{port}");

                            std::thread::spawn(move || {
                                handle_receive(addr, dir, tx, running);
                            });
                        }
                    },
                    if is_running.read().load(std::sync::atomic::Ordering::Relaxed) {
                        "停止接收"
                    } else {
                        "开始接收"
                    }
                }
            }
            fieldset { class: "fieldset bg-base-100 border border-base-300 rounded-box h-1/3 px-4 flex",
                legend { class: "fieldset-legend",
                    "日志输出"
                    div { class: "tooltip ", "data-tip": "清空日志",
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

                div { class: "overflow-y-auto flex-1",
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
