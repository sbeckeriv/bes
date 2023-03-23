use crate::{
    components::{email::Email, email_thread::EmailThread, utils::*},
    config::{AccountConfig, DatabaseConfig},
    database,
    log::debug_log,
};
use dioxus::prelude::*;
use dioxus_daisyui::prelude::*;
use itertools::Itertools;
use std::{cell::Cell, collections::HashSet};
use tokio::runtime::Handle;
// filter ideas
// filter img src urls https://github.com/rust-ammonia/ammonia/issues/175 ?

pub fn App(cx: Scope<AppProps>) -> Element {
    let account_config: &UseState<Option<_>> = {
        let initial = cx.props.account_config.take();
        use_state(&cx, || initial)
    };

    let database_config: &UseState<Option<_>> = {
        let initial = cx.props.database_config.take();
        use_state(&cx, || initial)
    };

    let filter: &UseState<Option<_>> = {
        let initial = cx.props.view_filter.take();
        use_state(&cx, || initial)
    };

    // state! why a state? login/setup/config process. inspired by twitvault
    let view = match (filter.get(), account_config.get(), database_config.get()) {
        (Some(filter), Some(account_config), Some(database_config)) => {
            use_shared_state_provider(cx, || ViewFilterState(filter.clone()));
            cx.render(rsx! { EmailContent {
            account_config: account_config.clone(), filter: filter.clone(), database_config: database_config.clone()
        }})
        }
        (_, _, _) => {
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
    pub account_config: Cell<Option<AccountConfig>>,
    pub database_config: Cell<Option<DatabaseConfig>>,
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
    pinned: bool,
}

fn Header(cx: Scope<HeaderProps>) -> Element {
    let mut view_filter_state = use_shared_state::<ViewFilterState>(cx).unwrap();
    let view_filter = &view_filter_state.read().0;
    let search_text = use_state(&cx, || view_filter.query.clone());
    let pinned = use_state(&cx, || view_filter.pinned.clone());
    cx.render(rsx! {
        div{
        input {
            r#type: "text",
            class: class!(w_80 h_12 text_2xl m_4 px_2 input input_primary input_bordered),
            placeholder: "Type to search",
            oninput: move | evt | {
                let text =  evt.value.clone();
                if text.len()>0{
                    view_filter_state.write().0.query = Some(text.clone());
                    search_text.set(Some(text));
                }else{
                    view_filter_state.write().0.query = None;
                    search_text.set(None);
                }
            }
        }
    }
    })
}
pub struct AccountConfigState(pub AccountConfig);
pub struct DatabaseConfigState(pub DatabaseConfig);
pub struct ViewFilterState(pub ViewFilter);
#[inline_props]
async fn EmailContent(
    cx: Scope,
    account_config: AccountConfig,
    filter: ViewFilter,
    database_config: DatabaseConfig,
) -> Element {
    use_shared_state_provider(cx, || AccountConfigState(account_config.clone()));
    use_shared_state_provider(cx, || DatabaseConfigState(database_config.clone()));

    let view_filter_state = use_shared_state::<ViewFilterState>(cx).unwrap();
    let view_filter = &view_filter_state.read().0;
    let groups = database::list_threads(&database_config).unwrap_or_default();
    debug_log(groups.len());
    let mut list = vec![];
    for (key, group) in &groups
        .into_iter()
        .group_by(|e| date_group(&e.first().unwrap().sent_at.clone().unwrap_or_default()))
    {
        debug_log(&key);
        let threads = group
            .into_iter()
            .map(|e| EmailThread {
                subject: e.first().unwrap().subject.clone().unwrap_or_default(),
                children: e
                    .into_iter()
                    .map(|message| Email {
                        message_id: message.message_id,
                        subject: message.subject.unwrap_or_default(),
                        from: message.message_from.unwrap_or_default(),
                        to: message.message_to.unwrap_or_default(),
                        cc: message.message_cc.unwrap_or_default(),
                        bcc: message.message_bcc.unwrap_or_default(),
                        html_format: message.html_format.unwrap_or_default(),
                        text_format: message.text_format.unwrap_or_default(),
                        date_sent: message.sent_at.unwrap_or_default(),
                        done: message.done_at.map(|_| true).unwrap_or(false),
                        pinned: message.pinned_at.map(|_| true).unwrap_or(false),
                        reminder_at: message.reminder_at.unwrap_or_default(),
                    })
                    .collect::<Vec<_>>(),
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
            class: class!(text_slate_600),
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

#[inline_props]
fn EmailGroup(cx: Scope, group: EmailGroup) -> Element {
    let expanded = use_state(&cx, || true);
    cx.render(rsx! {
        div {
            class: class!(mb_5),
            div {
                class: class!(uppercase px_3 py_2 w_full text_sm text_sky_600 border_b border_b_slate_200),
                onclick:  move |_| {
                    expanded.set(expanded.get()^true);
                },
                div{
                    class: "email-thread-name",
                    "{group.name}"
                }
            }
            if *expanded.get(){
                rsx!(div {
                    class: class!(flex flex_col),
                    for email_thread in group.children.to_owned().into_iter(){
                        div{
                            class: "email-thread",
                            EmailThread{ thread: email_thread}
                        }
                    }
                })
            }
        }
    })
}
