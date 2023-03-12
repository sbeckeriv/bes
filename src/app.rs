use crate::{
    components::{email::Email, utils::*},
    config::AccountConfig,
    database,
    messages::parse_emails,
};

use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;
use itertools::Itertools;
use std::{cell::Cell, collections::HashSet, ops::BitXorAssign};
use tokio::runtime::Handle;
// filter ideas
// filter img src urls https://github.com/rust-ammonia/ammonia/issues/175 ?

pub fn App(cx: Scope<AppProps>) -> Element {
    let config: &UseState<Option<Config>> = {
        let initial = cx.props.config.take();
        use_state(&cx, || initial)
    };

    let filter: &UseState<Option<ViewFilter>> = {
        let initial = cx.props.view_filter.take();
        use_state(&cx, || initial)
    };

    // state! why a state? login/setup/config process. inspired by twitvault
    let view = match (filter.get(), config.get()) {
        (Some(filter), Some(config)) => cx.render(rsx! { EmailContent {
            config: config.clone(), filter: filter.clone()
        }}),
        (_, _) => {
            cx.render(rsx! {
                span {
                    "none state"
                    // "Done"
                }
            })
        }
    };

    cx.render(rsx! {
    section { class: "header",
        Header{  pinned: false},
    }
    section { class: "content",
        view
        }
    })
}

pub struct AppProps {
    pub config: Cell<Option<Config>>,
    pub view_filter: Cell<Option<ViewFilter>>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ViewFilter {
    pub query: Option<String>,
    pub pinned: bool,
    pub expanded_email: Option<HashSet<String>>,
    pub expanded_thread: Option<HashSet<String>>,
    pub folder: Option<String>,
    pub account: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Config {
    pub config_data: Vec<AccountConfig>,
    pub page_loade: usize,
}

#[derive(PartialEq, Props)]
struct HeaderProps {
    #[props(optional)]
    search: Option<String>,
    pinned: bool,
}

fn Header(cx: Scope<HeaderProps>) -> Element {
    cx.render(rsx! {
        div {
            "Header",
        }
    })
}

#[inline_props]
async fn EmailContent(cx: Scope, config: Config, filter: ViewFilter) -> Element {
    //todo: dont load email body till clicked.
    let groups = database::list_threads().unwrap_or_default();

    let mut list = vec![];
    for (key, group) in &groups
        .into_iter()
        .group_by(|e| date_group(&e.sent_at.clone().unwrap_or_default()))
    {
        let threads = group
            .into_iter()
            .map(|e| EmailThread {
                subject: e.subject.clone().unwrap_or_default(),
                children: vec![Email {
                    message_id: e.message_id,
                    subject: e.subject.unwrap_or_default(),
                    from: e.message_from.unwrap_or_default(),
                    to: e.message_to.unwrap_or_default(),
                    cc: e.message_cc.unwrap_or_default(),
                    bcc: e.message_bcc.unwrap_or_default(),
                    html_format: e.html_format.unwrap_or_default(),
                    text_format: e.text_format.unwrap_or_default(),
                    date_sent: e.sent_at.unwrap_or_default(),
                    done: e.done_at.map(|_| true).unwrap_or(false),
                    pinned: e.pinned_at.map(|_| true).unwrap_or(false),
                    reminder_at: e.reminder_at.unwrap_or_default(),
                }],
            })
            .collect::<Vec<_>>();
        list.push(EmailGroup {
            name: key,
            children: threads,
        });
    }
    let handle = Handle::current();
    let views = list
        .into_iter()
        .map(|group| {
            cx.render(rsx! {
                EmailGroup{group: group}
            })
        })
        .collect::<Vec<_>>();

    cx.render(rsx!(
        div {
            for view in views.iter(){
                div{
                    class: "email-group",
                    view
                }
            }
    }))
}

#[derive(PartialEq, Clone)]
struct EmailGroup {
    pub name: String,
    pub children: Vec<EmailThread>,
}

#[derive(PartialEq, Clone)]
struct EmailThread {
    pub subject: String,
    pub children: Vec<Email>,
}

#[inline_props]
fn EmailGroup(cx: Scope, group: EmailGroup) -> Element {
    let expanded = use_state(&cx, || true);
    cx.render(rsx! {
        div {
            class: "email-thread-item",
            onclick:  move |_| {
                expanded.set(expanded.get()^true);
            },
            div{
               class: "email-thread-name",
               "{group.name}"
            }
        if *expanded.get(){
        rsx!(div {
            class: "email-thread-content",
            style: "display: grid; grid-template-columns: 1fr; gap: .1em;",
            for email_thread in group.children.to_owned().into_iter(){
                div{
                    class: "email-thread",
                    EmailThread{ thread: email_thread}
                }
            }
        })}


        }
    })
}

#[inline_props]
fn EmailThread(cx: Scope, thread: EmailThread) -> Element {
    let icon = thread
        .children
        .first()
        .and_then(|c| c.from.chars().nth(0))
        .unwrap_or_default();
    cx.render(rsx! {
        if thread.children.len()>1{
            rsx!(
            div {
                class: "email-thread-detail",
                div{
                    class: "email-thread-icon circle",
                    "{icon}"
                }

                div {
                    for email in thread.children.to_owned().into_iter(){
                        div{
                            class: "email-list",
                            Email{ email: email }
                        }
                    }
                }
        })
        }else{
            rsx!(div{
                class: "email-list",
                Email{ email: thread.children.first().unwrap().to_owned()}
            })
        }

    })
}
