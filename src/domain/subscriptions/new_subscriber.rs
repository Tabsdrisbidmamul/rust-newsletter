use crate::domain::subscriptions::SubscriberName;

use super::SubscriberEmail;

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
