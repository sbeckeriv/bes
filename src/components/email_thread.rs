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
    cx.render(rsx! {
        if thread.children.len()>1{
            let expanded = use_state(&cx, || false);
            let expanded_list = use_state(&cx, || false);
            let raw_from = thread.children.first().as_ref().map(|e| e.from.clone()).unwrap_or_default();
            let from = parse_emails(&raw_from)
                .first()
                .map(|f| f.1.clone().unwrap_or_else(|| f.0.clone()))
                .unwrap_or_else(|| raw_from.clone());

            let icon = from.chars().nth(0).unwrap_or_default();

            if !*expanded.get(){
                let date = relative_date_format(&thread.children.first().unwrap().date_sent);
                let mut from = from;
                from.truncate(30);

                rsx! {
                    div {
                        class: class!(w_full border_t border_t_gray_200 ),
                        key: "thread-{thread.children.first().unwrap().message_id}",
                        div {
                            class: "parent_hover",
                            div{
                                class: class!(flex justify_between px_3 py_2 grow gap_5 hover(bg_slate_200)),
                                onclick:  move |_| {
                                    expanded.set(true);
                                },
                                div{
                                    class: class!( h_5 w_5 font_bold text_gray_700 rounded_full bg_sky_600 flex items_center justify_center ),
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
                                    class: class!("hide" w_2__12 text_right overflow_hidden whitespace_nowrap),
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
                        div{

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

                        },
                        // its own line bumped in
                        if *expanded_list.get() || thread.children.len()==2{
                        rsx!(div {
                            class: "email-expaned-list",
                                class: class!(ml_2 w_full border_t border_t_gray_200 bg_white),
                                for (i,email) in thread.children.to_owned().into_iter().rev().enumerate(){

                                    div{
                                        class: "email-list",
                                        Email{ email: email, start_expanded: i==end-1}
                                    }
                                }
                            })
                            }
                        else{

                            rsx!(div {
                                class: "email-thread-short-list",
                                class: class!(ml_2 w_full border_t border_t_gray_200 bg_white),
                                    div{
                                        class: "email-list",
                                        Email{ email: thread.children.last().cloned().unwrap(), start_expanded: false}


                                    }

                                    div{
                                        class: class!(border_b border_b_gray_200 border_t border_t_gray_200 bg_white),

                                        onclick:  move |_| {
                                            expanded_list.set(expanded_list.get()^true);
                                        },
                                        div{
                                            class: class!( h_7 w_7 font_bold text_gray_700 rounded_full bg_sky_600 flex items_center justify_center ),
                                            "{thread.children.len()-2}"
                                        }
                                    }

                                    div{
                                        class: "email-list",
                                        Email{ email: thread.children.first().cloned().unwrap(), start_expanded: false}
                                    }
                            })
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
