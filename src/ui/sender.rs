use crate::{
    app_state::{Language, SenderState},
    sender::handle_send,
};
use dioxus::{
    html::{geometry::PixelsVector2D, HasFileData},
    prelude::*,
};
use rust_i18n::t;
use std::{rc::Rc, sync::atomic::Ordering::Relaxed};

#[component]
pub fn SenderPage() -> Element {
    let mut is_hovered = use_signal(|| false);

    let sender_state = use_context::<SenderState>();
    let mut enable_directory = sender_state.enable_directory;
    let mut ip_field = sender_state.ip_field;
    let mut port_field = sender_state.port_field;
    let mut file = sender_state.file;
    let mut is_running = sender_state.is_running;
    let log_tx = sender_state.log_tx;
    let mut logs = sender_state.logs;
    let progress_tx = sender_state.progress_tx;
    let mut progress = sender_state.progress;

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
        div { class: "flex flex-col gap-8 h-full",
            div { class: "flex-1 flex gap-4 min-h-0 shadow rounded-box bg-base-100",
                div { class: "flex-1 flex flex-col justify-center gap-2 p-4",
                    div {
                        class: if *is_hovered.read() { "rounded-box flex justify-center items-center border border-dashed border-info h-16 bg-blue-300" } else { "rounded-box flex justify-center items-center border border-dashed border-info h-16" },
                        ondragover: move |e| {
                            e.prevent_default();
                            is_hovered.set(true)
                        },
                        ondragleave: move |_| is_hovered.set(false),
                        ondrop: move |e| {
                            e.prevent_default();
                            is_hovered.set(false);
                            for f in e.files() {
                                file.set(f.path());
                            }
                        },
                        p { class: "text-gray-500", r#"{t!("drag_drop")}"# }
                    }
                    div { class: "divider", "OR" }
                    fieldset { class: "fieldset ",
                        input {
                            class: "file-input",
                            r#type: "file",
                            directory: enable_directory,
                            onchange: move |e| {
                                for f in e.files() {
                                    file.set(f.path());
                                }
                            },
                        }
                        div { class: "flex items-center gap-2 ",
                            label {
                                class: "text-lg text-gray-600",
                                r#for: "directory-upload",
                                r#"{t!("enable_dir")}"#
                            }
                            input {
                                class: "checkbox checkbox-info checkbox-md",
                                r#type: "checkbox",
                                id: "directory-upload",
                                checked: enable_directory,
                                onchange: move |evt| enable_directory.set(evt.checked()),
                            }
                        }
                    }
                    p { class: "text-gray-500 break-all whitespace-normal",
                        r#"{t!("selected_file")} : {file:?}"#
                    }
                }
                div { class: "flex-1 flex flex-col p-4 items-center justify-center",
                    fieldset { class: "fieldset ",
                        legend { class: "fieldset-legend text-gray-500", "IP" }
                        input {
                            class: "input input-lg",
                            r#type: "text",
                            placeholder: "192.168.1.1",
                            value: "{ip_field.raw_value}",
                            oninput: ip_field.oninput,
                            onmounted: move |e| ip_field.mounted.set(Some(e)),
                        }
                        p { class: "text-error", {ip_field.error} }
                    }
                    fieldset { class: "fieldset",
                        legend { class: "fieldset-legend text-gray-500", r#"{t!("port")}"# }
                        input {
                            class: "input input-lg",
                            r#type: "number",
                            placeholder: "8000",
                            value: "{port_field.raw_value}",
                            oninput: port_field.oninput,
                            onmounted: move |e| port_field.mounted.set(Some(e)),
                        }
                        p { class: "text-error", {port_field.error} }
                    }
                    button {
                        class: "btn btn-info mt-8 px-20",
                        disabled: is_running.read().load(Relaxed),
                        onclick: move |_| async move {
                            if ip_field.error.read().is_some() {
                                ip_field.focus().await;
                                return;
                            }
                            if port_field.error.read().is_some() {
                                port_field.focus().await;
                                return;
                            }

                            is_running.write().store(true, Relaxed);
                            progress.set((0.0, "0.00MB/s".to_string()));
                            let running = is_running();
                            let addr = format!("{}:{}", ip_field.value, port_field.value);
                            let file = file();
                            let log_tx = log_tx();
                            let progress_tx = progress_tx();
                            std::thread::spawn(move || {
                                match handle_send(addr, file, log_tx.clone(), progress_tx) {
                                    Ok(()) => {
                                        running.store(false, Relaxed);
                                        _ = log_tx.unbounded_send(t!("send_over").to_string());
                                    }
                                    Err(e) => {
                                        running.store(false, Relaxed);
                                        _ = log_tx.unbounded_send(format!("{} : {}", t!("send_fail"), e));
                                    }
                                }
                            });
                        },
                        if is_running.read().load(Relaxed) {
                            r#"{t!("sending")}"#
                            span { class: "loading loading-dots" }
                        } else {
                            r#"{t!("send")}"#
                        }
                    }
                }
            }
            div { class: "h-1/3 flex gap-4",
                div { class: "w-1/3 flex flex-col items-center justify-center gap-2 shadow rounded-box bg-base-100",
                    div {
                        class: "radial-progress bg-blue-500 text-white border-blue-500 border-4",
                        role: "progressbar",
                        aria_valuenow: "{progress.read().0:.0}",
                        style: "--value:{progress.read().0:.0};",
                        "{progress.read().0:.0}%"
                    }
                    p { class: "text-gray-500", r#"{t!("speed")}: {progress.read().1}"# }
                }
                div { class: "flex-1 fieldset shadow rounded-box bg-base-100 px-4 flex relative",
                    div { class: "absolute -top-3 left-4 flex items-center gap-2",
                        p { class: "font-bold text-gray-500", r#"{t!("logs")}"# }
                        div {
                            class: "tooltip ",
                            "data-tip": r#"{t!("clear_logs")}"#,
                            button {
                                class: "btn btn-xs btn-error btn-outline btn-square",
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
                    div {
                        class: "overflow-y-auto flex-1 mt-3",
                        onmounted: move |e| log_container.set(Some(e.data())),
                        for log in logs.iter() {
                            p { class: "break-all whitespace-pre-wrap", "{log}" }
                        }
                    }
                }
            }
        }
    }
}
