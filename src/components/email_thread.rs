use crate::messages::parse_emails;
use dioxus::prelude::*;
pub use dioxus_tailwindcss;
pub use dioxus_tailwindcss::build;
pub use dioxus_tailwindcss::prelude::*;

use super::email::Email;
#[derive(PartialEq, Clone)]
pub struct EmailThread {
    pub subject: String,
    pub children: Vec<Email>,
}
#[inline_props]
pub fn EmailThread(cx: Scope, thread: EmailThread) -> Element {
    let icon = thread
        .children
        .first()
        .and_then(|c| c.from.chars().nth(0))
        .unwrap_or_default();
    cx.render(rsx! {
        if thread.children.len()>1{
            let expanded = use_state(&cx, || false);
            let raw_from = thread.children.first().as_ref().map(|e| e.from.clone()).unwrap_or_default();
            let from = parse_emails(&raw_from);
            let from = from.first().map(|f| f.0.clone()).unwrap_or(raw_from);
            if *expanded.get(){
                rsx!(
                div {
                        class: "email-thread-detail",
                        onclick:  move |_| {
                            expanded.set(expanded.get()^true);
                        },
                        div{
                            class: "email-thread-icon circle",
                            "{from}"
                        }
                    }
                )
            }else{
                rsx!(
                    div {
                        class: "email-thread-detail",

                        onclick:  move |_| {
                            expanded.set(expanded.get()^true);
                        },

                        div{
                            class: "email-thread-icon circle",
                            "{from} ({thread.children.len()})"
                        }

                        div {
                            for email in thread.children.to_owned().into_iter(){
                                div{
                                    class: "email-list",
                                    Email{ email: email }
                                }
                            }
                        }
                    }
                )
            }
        }else{
            rsx!(div{
                class: "email-list",
                Email{ email: thread.children.first().unwrap().to_owned()}
            })
        }
    })
}
