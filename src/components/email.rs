use crate::{components::utils::*, database, messages::parse_emails};
use dioxus::prelude::*;

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

#[inline_props]
pub fn Email(cx: Scope, email: Email) -> Element {
    let expanded = use_state(&cx, || false);
    let hovered = use_state(&cx, || false);
    let background_hover = if *hovered.get() {
        "background-color: #ddd;"
    } else {
        ""
    };

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
        let content = database::get_message_id_content(&email.message_id).unwrap_or_default();
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
                class: "email-expanded", style: "width: 100%;  ",
                div {
                    class: "email-expaned-header",
                    style: "display: flex;column-gap: 20px; background-color: #999;",

                    div{
                        style: "display: flex; column-gap: 15px; flex-grow: 1; column-count: 3;",
                        onclick:  move |_| {
                            expanded.set(false);
                            //you clicked it you should be hovering over it!
                            hovered.set(true);
                        },
                        div{
                            class: "email-expanded-user-icon circle",
                            "{icon}"
                        }
                        div{
                            class: "email-expaned-from",
                            "{from}"
                        }

                        div{
                            class: "email-expaned-date", style: "flex-grow: 1; text-align: right;",
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
        let date = if *hovered.get() {
            relative_date_format(&email.date_sent)
        } else {
            "".into()
        };
        let mut from = from;
        from.truncate(30);

        cx.render(rsx! {
            div {
                class: "email-expanded", style: "width: 100%; padding-bottom:10px",
                div {
                    class: "email-expaned-header",
                    style: "display: flex; min-height: 30px; column-gap: 20px; {background_hover}",
                    onmouseenter: move |_| {hovered.set(true)},
                    onmouseleave: move |_| {hovered.set(false)},
                    div{
                        style: "display: flex; column-gap: 15px ;flex-grow: 1; column-count: 4;",
                        onclick:  move |_| {
                            hovered.set(false);
                            expanded.set(true);
                        },
                        div{
                            class: "email-expanded-user-icon circle",
                            "{icon}"
                        }
                        div{
                            class: "email-expaned-from",
                            "{from}"
                        }

                        div{
                            class: "email-expaned-subject", style: "overflow: hidden; text-overflow: ellipsis; ",
                            "{email.subject}"
                        }

                        div{
                            class: "email-expaned-date", style: "flex-grow: 1; text-align: right;",
                            "{date}"
                        }
                    }
                    div{
                        class: "email-expaned-actions",
                        "☰"
                    }
                }
            }
        })
    }
}
