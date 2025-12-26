use crate::{
    app_state::{AppState, ReceiverState, SenderState},
    ui::Route,
};
use dioxus::prelude::*;
use futures_util::StreamExt;

#[component]
pub fn DashboardLayout() -> Element {
    let current_route = use_route::<Route>();

    let mut receiver_logs = Signal::new(Vec::new());
    let receiver_tx = use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
        while let Some(log) = rx.next().await {
            receiver_logs.push(log);
        }
    });

    let mut sender_logs = Signal::new(Vec::new());
    let sender_log_tx = use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
        while let Some(log) = rx.next().await {
            sender_logs.push(log);
        }
    });
    let mut send_progress = Signal::new((0.0, "0.00MB/s".to_string()));
    let sender_progress_tx =
        use_coroutine(move |mut rx: UnboundedReceiver<(f64, String)>| async move {
            while let Some((percent, speed)) = rx.next().await {
                send_progress.set((percent, speed));
            }
        });

    use_context_provider(|| AppState {
        receiver: ReceiverState::new(Signal::new(receiver_tx.tx()), receiver_logs),
        sender: SenderState::new(
            Signal::new(sender_log_tx.tx()),
            sender_logs,
            Signal::new(sender_progress_tx.tx()),
            send_progress,
        ),
    });

    rsx! {
        div { class: "flex h-screen gap-4 p-4",
            div { class: "flex-1",
                Outlet::<Route> {}
            }
            div { class: "w-2/12 flex flex-col justify-around p-4 shadow rounded bg-base-200",
                Link {
                    class: if current_route == Route::SenderPage {
                        "btn btn-info"
                    } else {
                        "btn "
                    },
                    to: Route::SenderPage,
                    "发送文件"
                }
                Link {
                    class: if current_route == Route::ReceiverPage {
                        "btn btn-info"
                    } else {
                        "btn"
                    },
                    to: Route::ReceiverPage,
                    "接收文件"
                }
            }
        }
    }
}
