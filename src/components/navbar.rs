use dioxus::prelude::*;
use dioxus_router::Link;

use crate::components::{ConnectButton, SearchButton};

pub fn Navbar(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            class: "flex flex-row justify-between w-screen p-8",
            Logo {}
            div {
                class: "flex items-center space-x-4",
                Link {
                    to: "/cron",
                    "Cron"            
                }
                SearchButton {}
                ConnectButton {}
            }
        }
    })
}

pub fn Logo(cx: Scope) -> Element {
    cx.render(rsx! {
        Link {
            to: "/",
            class: "flex items-center w-40",
            img {
                class: "w-12 h-12",
                src: "/img/CLOCKWORK.svg",
            }
        }
    })
}
