use crate::{app::DatabaseConfigState, components::utils::*, database, messages::parse_emails};
use dioxus::prelude::*;
pub use dioxus_tailwindcss;
pub use dioxus_tailwindcss::build;
pub use dioxus_tailwindcss::prelude::*;

#[derive(PartialEq, Clone)]
pub struct Email {
    pub message_id: String,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub cc: String,
    pub bcc: String,
    pub html_format: String,
    pub text_format: String,
    pub date_sent: String,
    pub done: bool,
    pub pinned: bool,
    pub reminder_at: String,
}
// share click state and make subject its own thing.
#[inline_props]
pub fn Email(cx: Scope, email: Email, start_expanded: bool) -> Element {
    let database_config = use_shared_state::<DatabaseConfigState>(cx).unwrap();
    let database_config = &database_config.read().0;
    let expanded = use_state(&cx, || start_expanded.clone());
    let from = parse_emails(&email.from)
        .first()
        .map(|f| f.1.clone().unwrap_or_else(|| f.0.clone()))
        .unwrap_or_else(|| email.from.clone());

    let icon = from.chars().nth(0).unwrap_or_default();
    let to = parse_emails(&email.to)
        .iter()
        .map(|f| f.1.clone().unwrap_or_else(|| f.0.clone()))
        .collect::<Vec<_>>()
        .join(", ");
    if *expanded.get() {
        let content = database::get_message_id_content(database_config, &email.message_id)
            .unwrap_or_default();
        // move style tags inside tags
        let inliner = css_inline::CSSInliner::options()
            .load_remote_stylesheets(false)
            .build();

        let inlined = inliner.inline(&content.1).unwrap_or_default();

        // make it "safe"
        let clean = ammonia::Builder::default()
            .url_relative(ammonia::UrlRelative::Deny)
            .add_generic_attributes(&["class", "style"])
            .clean(&inlined)
            .to_string();

        let clean = if clean.len() < 10 { content.0 } else { clean };

        cx.render(rsx! {
            div {
                key: "{email.message_id}",
                class: class!(w_full),
                div {
                    class: class!(flex justify_between px_3 py_2 grow gap_5 ),
                    div{
                        class: class!(flex gap_16 grow),
                        onclick:  move |_| {
                            expanded.set(false);
                        },
                        div{
                            class: class!( h_5 w_5 font_bold text_gray_700 rounded_full bg_sky_600 flex items_center justify_center ),
                            "{icon}"
                        }
                        div{
                            class: "email-expaned-from",
                            "{from}"
                        }

                        div{
                            class: class!(grow text_right),
                            "{date_format(&email.date_sent)}"
                        }
                    }
                    div{
                        class: "email-expaned-actions",
                        "☰"
                    }
                }

                div {
                    class: "email-expaned-content",
                    div{
                        style: "display: flex; column-gap: 10px; ",
                        div{
                            class: "email-expaned-to",
                            "To: {to}"
                        }
                        div{
                            class: "email-expaned-subject",
                            "{email.subject}"
                        }

                    }
                    div {
                        dangerous_inner_html: "{clean}"
                    }
                }
            }
        })
    } else {
        let date = relative_date_format(&email.date_sent);
        let mut from = from;
        from.truncate(30);

        cx.render(rsx! {
            div {
                class: class!(w_full border_t border_t_gray_200 ),
                div {
                    class: "parent_hover",
                    // class: class!() "email-expaned-header flex gap-20",
                    div{
                        class: class!(flex justify_between px_3 py_2 grow gap_5 hover(bg_slate_200)),
                        onclick:  move |_| {
                            expanded.set(true);
                        },
                        div{
                            class: class!( h_5 w_7 font_bold text_gray_700 rounded_full bg_sky_600 flex items_center justify_center ),
                            "{icon}"
                        }
                        div{
                            class: class!(w_3__12 overflow_hidden text_ellipsis whitespace_nowrap),
                            "{from}"
                        }

                        div{
                            class: class!(w_full grow overflow_hidden text_ellipsis whitespace_nowrap),
                            "{email.subject}"
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
        })
    }
}
