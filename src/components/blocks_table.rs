use crate::{context::Cluster, utils::ClockworkWasmClient};
use dioxus::prelude::*;
use solana_client_wasm::WasmClient;
use solana_extra_wasm::transaction_status::UiConfirmedBlock;

pub fn BlocksTable(cx: Scope) -> Element {
    let block = use_state::<Option<UiConfirmedBlock>>(&cx, || None);
    let cluster_context = use_shared_state::<Cluster>(cx).unwrap();

    use_future(&cx, (), |_| {
        let block = block.clone();
        let cluster_context = cluster_context.clone();
        async move {
            let client = WasmClient::new_with_config(cluster_context.read().to_owned());
            loop {
                if let Some(recent_block) = client.get_block().await {
                    block.set(Some(recent_block));
                }
                gloo_timers::future::TimeoutFuture::new(1000).await;
            }
        }
    });

    cx.render(rsx! {
        div {
            h1 {
                class: "text-2xl font-semibold mb-6",
                "Blocks"
            }
            table {
                class: "w-full divide-y divide-slate-800",
                Header {}
                if let Some(block) = block.get() {
                    rsx! {
                        tr {
                            class: "px-3 text-base hover:bg-slate-100 hover:text-slate-900 hover:cursor-pointer focus:bg-slate-900",
                            td {
                                class: "whitespace-nowrap px-3 py-4",
                                "{block.blockhash}"
                            }
                            td {
                                class: "whitespace-nowrap px-3 py-4",
                                "{block.signatures.as_ref().unwrap_or(&vec![]).len()}"
                            }
                        }
                    }
                }
            }
        }
    })
}

fn Header(cx: Scope) -> Element {
    cx.render(rsx! {
        thead {
            tr {
                class: "text-left text-sm text-slate-500",
                th {
                    class: "py-3 px-3 font-medium",
                    scope: "col",
                    "Blockhash"
                }
                th {
                    class: "py-3 px-3 font-medium",
                    scope: "col",
                    "Transactions"
                }
            }
        }
    })
}
