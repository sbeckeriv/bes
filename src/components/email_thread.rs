use crate::components::utils::relative_date_format;
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
    let hovered = use_state(&cx, || false);
    cx.render(rsx! {
        if thread.children.len()>1{
            let expanded = use_state(&cx, || false);
            let raw_from = thread.children.first().as_ref().map(|e| e.from.clone()).unwrap_or_default();
            let from = parse_emails(&raw_from)
                .first()
                .map(|f| f.1.clone().unwrap_or_else(|| f.0.clone()))
                .unwrap_or_else(|| raw_from.clone());

            let icon = from.chars().nth(0).unwrap_or_default();

            if !*expanded.get(){
                let date = if *hovered.get() {
                    relative_date_format(&thread.children.first().unwrap().date_sent)
                } else {
                    "".into()
                };
                let mut from = from;
                from.truncate(30);

                rsx! {
                    div {
                        class: class!(w_full border_t border_t_gray_200 ),
                        key: "thread-{thread.children.first().unwrap().message_id}",
                        div {
                            // class: class!() "email-expaned-header flex gap-20",
                            onmouseenter: move |_| {hovered.set(true)},
                            onmouseleave: move |_| {hovered.set(false)},
                            div{
                                class: class!(flex justify_between px_3 py_2 grow gap_5 hover(bg_slate_200)),
                                onclick:  move |_| {
                                    hovered.set(false);
                                    expanded.set(true);
                                },
                                div{
                                    style: "width: 32px;",
                                    "{icon}"
                                }
                                div{
                                    class: class!(w_3__12 overflow_hidden text_ellipsis whitespace_nowrap),
                                    "{from} ({thread.children.len()})"
                                }

                                div{
                                    class: "email-expaned-subject",
                                    class: class!(w_full grow overflow_hidden text_ellipsis whitespace_nowrap),
                                    "{thread.children.first().unwrap().subject}"
                                }

                                div{
                                    class: class!(w_2__12 text_right overflow_hidden whitespace_nowrap),
                                    "{date}"
                                }

                                div {
                                    class: class!(shrink),
                                    "☰"
                                }
                            }
                        }
                    }
                }
            }else{
                let end = thread.children.len();
                rsx!(
                    div {
                        class: class!(w_full border_t border_t_gray_200 bg_blue_100 ),
                        onclick:  move |_| {
                            expanded.set(expanded.get()^true);
                        },

                        div{
                            class: class!(w_3__12 overflow_hidden text_ellipsis whitespace_nowrap),
                            "{from} ({thread.children.len()})"
                        }

                        div{
                            class: "email-expaned-subject",
                            class: class!(w_full grow overflow_hidden text_ellipsis whitespace_nowrap),
                            "{thread.children.first().unwrap().subject}"
                        }
                        // its own line bumped in
                        div {
                            class: "email-expaned-list",
                            class: class!(ml_2 w_full border_t border_t_gray_200 bg_white),
                            for (i,email) in thread.children.to_owned().into_iter().rev().enumerate(){

                                div{
                                    class: "email-list",
                                    Email{ email: email, start_expanded: i==end-1}
                                }
                            }
                        }
                    }
                )
            }
        }else{
            rsx!(div{
                class: "email-list",
                Email{ email: thread.children.first().unwrap().to_owned(), start_expanded: false}
            })
        }
    })
}
