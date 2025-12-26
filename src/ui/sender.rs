use crate::{app_state::AppState, sender::handle_send};
use dioxus::{html::geometry::PixelsVector2D, prelude::*};
use std::{rc::Rc, sync::atomic::Ordering::Relaxed};

#[component]
pub fn SenderPage() -> Element {
    let app_state = use_context::<AppState>();
    let mut enable_directory = app_state.sender.enable_directory;
    let mut ip = app_state.sender.ip;
    let mut port = app_state.sender.port;
    let mut file = app_state.sender.file;
    let mut is_running = app_state.sender.is_running;
    let log_tx = app_state.sender.log_tx;
    let mut logs = app_state.sender.logs;
    let progress_tx = app_state.sender.progress_tx;
    let progress = app_state.sender.progress;

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
        div { class: "flex flex-col gap-4 h-full",
            div { class: "flex-1 flex gap-4 min-h-0 shadow rounded-lg bg-base-200",
                div { class: "flex-1 flex flex-col justify-center p-4",
                    fieldset { class: "fieldset ",
                        legend { class: "fieldset-legend", "选择需要发送的文件" }
                        input { class: "file-input file-input-info",
                            r#type: "file",
                            directory: enable_directory,
                            onchange: move |e| {
                               for f in e.files() {
                                   file.set(f.path());
                                }
                            }
                        }
                        p { class: "label break-all whitespace-normal", "当前文件：{file:?}" }
                        div { class: "flex items-center gap-2 text-lg",
                            label { r#for: "directory-upload", "启用目录上传" }
                            input { class: "checkbox checkbox-info",
                                r#type: "checkbox",
                                id: "directory-upload",
                                checked: enable_directory,
                                oninput: move |evt| enable_directory.set(evt.checked()),
                            }
                        }
                    }
                }
                div { class: "flex-1 flex flex-col p-4",
                    fieldset { class: "fieldset ",
                        legend { class: "fieldset-legend", "目标IP" }
                        input { class: "input input-info",
                            r#type: "text",
                            placeholder: "192.168.1.1",
                            value: "{ip}",
                            oninput: move |evt| ip.set(evt.value()),
                        }
                    }
                    fieldset { class: "fieldset",
                        legend { class: "fieldset-legend", "端口" }
                        input { class: "input input-info",
                            r#type: "number",
                            placeholder: "8000",
                            value: "{port}",
                            oninput: move |evt| {
                                if let Ok(p) = evt.value().parse() {
                                    port.set(p);
                                }
                            }
                        }
                    }
                    button { class: "btn btn-info px-20 mt-8",
                        disabled: is_running.read().load(Relaxed),
                        onclick: move |_| {
                            is_running.write().store(true, Relaxed);
                            let running = is_running();
                            let addr = format!("{ip}:{port}");
                            let file = file();
                            let log_tx = log_tx();
                            let progress_tx = progress_tx();

                            std::thread::spawn(move || {
                                 match handle_send(addr, file, log_tx.clone(), progress_tx) {
                                     Ok(()) => {
                                         running.store(false, Relaxed);
                                         _ = log_tx.unbounded_send(format!("发送任务成功"));
                                     }
                                     Err(e) => {
                                         running.store(false, Relaxed);
                                         _ = log_tx.unbounded_send(format!("发送任务失败: {}", e));
                                     }
                                 }
                            });
                        },
                        if is_running.read().load(Relaxed) {
                            "发送中"
                            span { class: "loading loading-dots" }
                        } else {
                            "发送"
                        }
                    }
                }
            }
            div { class: "h-2/5 flex gap-4 shadow rounded-lg bg-base-200",
                div { class: "w-1/3 flex flex-col items-center justify-center gap-2",
                    div {
                        class: "radial-progress bg-info text-primary-content border-info border-4",
                        role: "progressbar",
                        aria_valuenow: "{progress.read().0:.1}",
                        style: "--value:{progress.read().0:.1};",
                        "{progress.read().0:.1}%"
                    }
                    p { "上传速度: {progress.read().1}" }
                }
                div { class: "flex-1 pr-2 pb-2",
                    fieldset { class: "fieldset bg-base-100 border border-base-300 rounded-box h-full px-4 flex",
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
    }
}
