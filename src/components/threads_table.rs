use std::{str::FromStr, cmp::Ordering};
use anchor_lang::prelude::Clock;
use chrono::{DateTime, NaiveDateTime, Utc};
use clockwork_thread_program_v2::state::{Trigger, TriggerContext, VersionedThread};
use clockwork_utils::pubkey::Abbreviated;
use dioxus::prelude::*;
use dioxus_router::Link;
use solana_client_wasm::{solana_sdk::account::Account, WasmClient};

use crate::{
    context::{User, Cluster},   
    hooks::use_pagination,
    utils::{format_balance, format_timestamp}, components::page_control::PageControl, ClockworkWasmClient
};

pub fn ThreadsTable(cx: Scope) -> Element {
    let paginated_threads = use_pagination::<(VersionedThread, Account)>(cx, 15, || vec![]);
    let clock = use_state::<Option<Clock>>(cx, || None);
    let cluster_context = use_shared_state::<Cluster>(cx).unwrap();
    let user_context = use_shared_state::<User>(cx).unwrap();
    let filter = use_state(cx, || false);
    let filter_dropdown_open = use_state(cx, || false);
    let is_loading = use_state(cx, || false);

    use_future!(cx, |(filter,)| {
        let cluster_context = cluster_context.clone();
        let user_context = user_context.clone();
        let is_loading = is_loading.clone();
        let clock = clock.clone();
        let paginated_threads = paginated_threads.clone();
        async move {
            is_loading.set(true);
            let client = WasmClient::new_with_config(cluster_context.read().to_owned());
            if let Ok(mut sorted_threads) = client.get_threads().await {
                let current_clock = client.get_clock().await.unwrap(); 
                sorted_threads.sort_by(|a, b| {
                     if let Some(exec_context_a) = a.0.exec_context() {
                         if let Some(exec_context_b) = b.0.exec_context() {
                             exec_context_b.last_exec_at.partial_cmp(&exec_context_a.last_exec_at).unwrap_or(Ordering::Equal)
                         } else {
                             Ordering::Less
                         }
                     } else {
                         Ordering::Greater
                     }
                });
                if *filter.get() {
                    let user_pubkey = user_context.read().pubkey.unwrap();
                    let filtered_threads = sorted_threads.clone();
                        let ft = filtered_threads
                            .into_iter()
                            .filter(|(vt, _a)| vt.authority().eq(&user_pubkey))
                            .collect::<Vec<(VersionedThread, Account)>>();
                    paginated_threads.set(ft); 
                } else {
                    paginated_threads.set(sorted_threads);
                }
                clock.set(Some(current_clock));
            };
            is_loading.set(false);
        } 
    });

    cx.render(rsx! {
        if *is_loading.get() {
            rsx! {
                div {
                    "loading..."
                }
            }
        } else {           
            rsx! {
                if let Some(threads) = paginated_threads.get() {
                    if let Some(clock) = clock.get() {
                        rsx! {
                            div {
                                class: "flex flex-row w-full justify-end",
                                button {
                                    class: "py-2 px-2 text-slate-100 hover:bg-slate-800 active:bg-slate-100 active:text-slate-900 active:ring-0 active:focus-0 transition text-sm font-medium rounded",
                                    onclick: move |_| { filter_dropdown_open.set(!filter_dropdown_open.get()) },
                                    svg {
                                        xmlns: "http://www.w3.org/2000/svg",
                                        fill: "none",
                                        view_box: "0 0 24 24", 
                                        stroke_width: "1.5", 
                                        stroke: "currentColor", 
                                        class: "w-5 h-5",
                                        path {
                                            d: "M12 3c2.755 0 5.455.232 8.083.678.533.09.917.556.917 1.096v1.044a2.25 2.25 0 01-.659 1.591l-5.432 5.432a2.25 2.25 0 00-.659 1.591v2.927a2.25 2.25 0 01-1.244 2.013L9.75 21v-6.568a2.25 2.25 0 00-.659-1.591L3.659 7.409A2.25 2.25 0 013 5.818V4.774c0-.54.384-1.006.917-1.096A48.32 48.32 0 0112 3z",
                                            "stroke-linecap": "round",
                                            "stroke-linejoin": "round"
                                        }
                                    }
                                }
                                if *filter_dropdown_open.get() {
                                    rsx! {
                                        div {
                                            class: "absolute mt-10 mr-2 w-56 h-24 bg-slate-700 rounded-lg",
                                            div {
                                                class: "flex flex-col w-full p-4 justify-start space-y-2",
                                                p {
                                                    class: "text-sm text-slate-400",
                                                    "Filter by:"
                                                }
                                                div {
                                                    class: "w-full h-0.5 bg-gray-400 rounded-xl border-0"
                                                }    
                                                div {
                                                    class: "flex flex-row py-2 space-x-2 items-center", 
                                                    input {
                                                        class: "h-4 w-4 rounded border-gray-300",
                                                        r#type: "checkbox",
                                                        value: "filter",
                                                        checked: "{filter.get()}",
                                                        id: "authority",
                                                        onchange: move |_| { 
                                                            let val = *filter.get();
                                                            filter.set(!val); 
                                                        }
                                                    }
                                                    p {
                                                        class: "text-sm",
                                                        "Authority"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } 
                            table {
                                class: "w-full",
                                Header {}
                                div {
                                    class: "table-row-group",
                                    for (i, thread) in threads.clone().iter().enumerate() {
                                        div {}
                                        Row {
                                            cluster_context: cluster_context.clone(),
                                            thread: thread.0.clone(),
                                            account: thread.1.clone(),
                                            elem_id: format!("list-item-{}", i),
                                            clock: clock.clone()
                                        }
                                    }
                                }
                            }
                            PageControl {
                                paginated_data: paginated_threads.clone(),
                            }
                        }
                    } 
                    else {
                        rsx! {
                            div {
                                "no clock"
                            }
                        }
                    }
                } else {
                    rsx! { 
                        div {
                            "No Results"
                        }
                    }
                }
            }
        }
    })
}    

fn Header(cx: Scope) -> Element {
    let cell_class = "table-cell font-medium py-2 px-5 first:pl-3 first:w-full first:truncate last:pr-3";
    cx.render(rsx! {
        thead {
            class: "table-header-group",
            div {
                class: "table-row text-left text-sm text-slate-500",
                th {
                    class: cell_class,
                    scope: "col",
                    "ID"
                }
                // th {
                //     class: cell_class,
                //     scope: "col",
                //     "Thread"
                // }
                th {
                    class: cell_class,
                    scope: "col",
                    "Authority"
                }
                th {
                    class: cell_class,
                    scope: "col",
                    "Balance"
                }
                // th {
                //     class: cell_class,
                //     scope: "col",
                //     "Created at"
                // }
                th {
                    class: cell_class,
                    scope: "col",
                    "Last exec"
                }
                // th {
                //     class: cell_class,
                //     scope: "col",
                //     "Paused"
                // }
                th {
                    class: cell_class,
                    scope: "col",
                    "Status"
                }
                th {
                    class: cell_class,
                    scope: "col",
                    "Trigger"
                }
            }
        }
    })
}

#[derive(Clone, Props)]
struct RowProps {
    cluster_context: UseSharedState<Cluster>,
    thread: VersionedThread,
    account: Account,
    elem_id: String,
    clock: Clock
}

impl PartialEq for RowProps {
    fn eq(&self, other: &Self) -> bool {
        self.thread.id().eq(&other.thread.id())
    }
}

fn Row(cx: Scope<RowProps>) -> Element {
    let thread = cx.props.thread.clone();
    let account = cx.props.account.clone();
    let clock = cx.props.clock.clone();
    let address = thread.pubkey();
    let address_state = use_state(cx, || address);
    // let address_abbr = address.abbreviated();
    let authority = thread.authority().abbreviated();
    let balance = format_balance(cx.props.account.lamports, true);
    // let created_at = format_timestamp(thread.created_at().unix_timestamp);
    let id = String::from_utf8(thread.id()).unwrap();
    let cluster_context = cx.props.cluster_context.clone();

    let last_exec = match thread.exec_context() {
        None => String::from("–"),
        Some(exec_context) => {
            let slots_ago = clock.slot - exec_context.last_exec_at;
            let last_exec_est_secs = (slots_ago * 400) / 1000;
            if last_exec_est_secs > 86_400 {
                format!("{} day ago", last_exec_est_secs / 86_400)
            } else if last_exec_est_secs > 3600 {
                format!("{} hr ago", last_exec_est_secs / 3600)
            } else if last_exec_est_secs > 60 {
                format!("{} min ago", last_exec_est_secs / 60)
            } else {
                format!("{} sec ago", last_exec_est_secs)
            }
        },
    };

    let trigger = match thread.trigger() {
        Trigger::Account {
            address,
            offset: _,
            size: _,
        } => address.abbreviated(),
        Trigger::Cron {
            schedule,
            skippable: _,
        } => {
            let reference_timestamp = match thread.exec_context().clone() {
                None => thread.created_at().unix_timestamp,
                Some(exec_context) => match exec_context.trigger_context {
                    TriggerContext::Cron { started_at } => started_at,
                    _ => 0,
                },
            };
            next_timestamp(reference_timestamp, schedule)
                .map_or("–".to_string(), |v| format_timestamp(v))
        }
        Trigger::Now => "–".to_string(),
        Trigger::Slot { slot } => slot.to_string(),
        Trigger::Epoch { epoch } => epoch.to_string(),
        Trigger::Timestamp { unix_ts } => unix_ts.to_string(),
        Trigger::Pyth {
            price_feed,
            equality: _,
            limit: _,
        } => price_feed.to_string(),
    };

    enum Status {
        Done,
        Healthy,
        Unhealthy,
        Unknown,
    }

    // let health = if thread.next_instruction().is_some() {
    //     if let Some(exec_context) = thread.exec_context() {
    //         if exec_context.last_exec_at.lt(&(clock.slot + 10)) {
    //             Status::Unhealthy
    //         } else {
    //             Status::Healthy
    //         }
    //     } else {
    //         Status::Healthy
    //     }
    // } else {
    //     match thread.trigger() {
    //         Trigger::Account {
    //             address: taddress,
    //             offset,
    //             size,
    //         } => {
    //             // Begin computing the data hash of this account.
    //             match use_future(cx, (), |_| {
    //                 let cluster_context = cluster_context.clone();
    //                 async move {
    //                     let cluster = cluster_context.read();
    //                     let client = WasmClient::new_with_config(cluster.to_owned());
    //                     client.get_account(&taddress).await 
    //                 }
    //             }).value() {
    //                 Some(res) => {
    //                     match res {
    //                         Ok(maybe_taccount) => {
    //                             if let Some(taccount) = maybe_taccount {
    //                                 let mut hasher = DefaultHasher::new();
    //                                 let data = &taccount.data;
    //                                 let offset = offset as usize;
    //                                 let range_end = offset.checked_add(size as usize).unwrap() as usize;
    //                                 if data.len().gt(&range_end) {
    //                                     data[offset..range_end].hash(&mut hasher);
    //                                 } else {
    //                                     data[offset..].hash(&mut hasher)
    //                                 }
    //                                 let data_hash = hasher.finish();
    //                                 if let Some(exec_context) = thread.exec_context() {
    //                                     match exec_context.trigger_context {
    //                                         TriggerContext::Account { data_hash: prior_data_hash } => {
    //                                             log::info!("Data: {:?}", data);
    //                                             log::info!("Data hash: {:?} {:?}", data_hash, prior_data_hash);
    //                                             if data_hash.eq(&prior_data_hash) {
    //                                                 Status::Healthy
    //                                             } else {
    //                                                 Status::Unhealthy
    //                                             }
    //                                         }
    //                                         _ => Status::Unknown
    //                                     }
    //                                 } else {
    //                                     Status::Healthy
    //                                 }
    //                             } else {
    //                                 Status::Healthy
    //                             }
    //                         }
    //                         Err(_err) => {
    //                             Status::Healthy
    //                         }
    //                     }
    //                 }
    //                 None => {
    //                     // TODO
    //                     Status::Healthy
    //                 }
    //             }
    //             // log::info!("Account: {:?}", trigger_account.value());
    //             // Verify the data hash is different than the prior data hash.
    //             // if let Some(exec_context) = thread.exec_context {
    //             //     match exec_context.trigger_context {
    //             //         TriggerContext::Account {
    //             //             data_hash: prior_data_hash,
    //             //         } => {
    //             //             require!(
    //             //                 data_hash.ne(&prior_data_hash),
    //             //                 ClockworkError::TriggerConditionFailed
    //             //             )
    //             //         }
    //             //         _ => return Err(ClockworkError::InvalidThreadState.into()),
    //             //     }
    //             // }

                // Status::Unknown
    //         },
    //         Trigger::Cron {
    //             schedule,
    //             skippable: _,
    //         } => {
    //             let reference_timestamp = match thread.exec_context().clone() {
    //                 None => thread.created_at().unix_timestamp,
    //                 Some(exec_context) => match exec_context.trigger_context {
    //                     TriggerContext::Cron { started_at } => started_at,
    //                     _ => panic!("Invalid thread state"),
    //                 },
    //             };
    //             if let Some(target_ts) = next_timestamp(reference_timestamp, schedule) {
    //                 // TODO Compare the target timestamp to the current timestamp. If this thread should have fired a while ago, it is "unhealthy".
    //                 if (target_ts + 10).gt(&clock.unix_timestamp) {
    //                     Status::Healthy
    //                 } else {
    //                     Status::Unhealthy
    //                 }
    //             } else {
    //                 Status::Done
    //             }
    //         }
    //         Trigger::Now => Status::Unhealthy,
    //         Trigger::Slot { slot: _ } => Status::Unknown,
    //         Trigger::Epoch { epoch: _ } => Status::Unknown,
    //         Trigger::Timestamp { unix_ts: _ } => Status::Unknown,
    //     }
    // };
    let status_class = match Status::Healthy {
        Status::Done => "w-3 h-3 my-auto bg-green-500 outline outline-slate-100 outline-1 outline-offset-2 rounded-full ml-4",
        Status::Healthy => "w-3 h-3 my-auto bg-green-500 rounded-full ml-4",
        Status::Unhealthy => "w-3 h-3 my-auto bg-red-500 rounded-full ml-4",
        Status::Unknown =>"w-3 h-3 my-auto bg-slate-500 rounded-full ml-4",
    };

    let cell_class = "table-cell whitespace-nowrap font-medium py-2 px-5 first:pl-3 first:truncate last:pr-3 first:rounded-tl first:rounded-bl last:rounded-tr last:rounded-br";
    cx.render(rsx! {
        Link {
            class: "table-row font-mono text-sm items-start transition hover:cursor-pointer hover:bg-slate-800 active:bg-slate-100 active:text-slate-900",
            to: "/threads/{address}",
            id: cx.props.elem_id.as_str(),
            div {
                class: cell_class,
                "{id}"
            }
            // div {
            //     class: cell_class,
            //     "{address_abbr}"
            // }
            div {
                class: cell_class,
                "{authority}"
            }
            div {
                class: cell_class,
                "{balance}"
            }
            // div {
            //     class: cell_class,
            //     "{created_at}"
            // }
            div {
                class: cell_class,
                "{last_exec}"
            }
            // div {
            //     class: cell_class,
            //     "{paused}"
            // }
            div {
                class: cell_class,
                div {
                    class: status_class, 
                }
            }
            div {
                class: cell_class,
                "{trigger}"
            }
        }
    })
}

fn next_timestamp(after: i64, schedule: String) -> Option<i64> {
    match clockwork_cron::Schedule::from_str(&schedule) {
       Ok(schedule) => schedule.next_after(&DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt(after, 0).unwrap(),
            Utc,
        ))
        .take()
        .map(|datetime| datetime.timestamp()),
        Err(_err) => None
    }
}
