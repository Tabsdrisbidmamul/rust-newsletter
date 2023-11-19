use reqwest::Client;

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    base_url: String,
    sender: SubscriberEmail,
    http_client: Client,
}

impl EmailClient {
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }

    ///
    /// Create a new instance of EmailClient
    ///
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
        }
    }
}
