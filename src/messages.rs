use himalaya_lib::{BackendBuilder, Email, Emails};
use mailparse::{addrparse, body::Body, MailAddr, MailParseError, ParsedMail};

use crate::config::AccountConfig;
#[derive(Debug, Default)]
pub struct MessageFilter {
    pub since: Option<u32>,
    pub before: Option<u32>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub folder: String,
}
pub fn get_messages(account: &AccountConfig, filter: MessageFilter) -> Emails {
    let (account_config, backend_config) = account.backend_config();
    dbg!(&account_config, &backend_config);
    let backend = BackendBuilder::new()
        .build(&account_config, &backend_config)
        .unwrap();

    let x = backend
        .list_envelopes(
            &filter.folder.as_str(),
            filter.limit.unwrap_or(10) as usize,
            filter.page.unwrap_or_default() as usize,
        )
        .unwrap();
    let ids: Vec<_> = x.iter().map(|e| e.id.as_ref()).collect();
    dbg!(&ids);

    let emails = backend
        .get_emails(filter.folder.as_str(), ids)
        .expect("email");
    emails
}
pub mod email {
    use super::*;
    // main lib tui
    pub fn parsed_mail_body(email: &ParsedMail) -> Result<String, MailParseError> {
        match email.get_body_encoded() {
            Body::Base64(body) | Body::QuotedPrintable(body) => {
                Ok(body.get_decoded_as_string().unwrap().to_string())
            }
            Body::SevenBit(body) | Body::EightBit(body) => {
                Ok(body.get_as_string().unwrap().to_string())
            }
            Body::Binary(body) => Ok(format!("{:?}", body.get_raw())),
        }
    }
}
pub fn parse_emails(emails: &str) -> Vec<(String, Option<String>)> {
    let list = addrparse(emails)
        .ok()
        .map(|l| l.into_inner())
        .unwrap_or_default();
    list.iter()
        .flat_map(|mail_addr| match mail_addr {
            MailAddr::Group(group) => group
                .addrs
                .iter()
                .map(|single| (single.addr.clone(), single.display_name.clone()))
                .collect::<Vec<_>>(),
            MailAddr::Single(single) => vec![(single.addr.clone(), single.display_name.clone())],
        })
        .collect::<Vec<_>>()
}
