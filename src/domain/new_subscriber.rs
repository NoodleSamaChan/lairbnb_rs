use crate::domain::{subscriber_name::SubscriberName, subscriber_email::SubscriberEmail, subscriber_password::SubscriberPassword};

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
    pub password: SubscriberPassword,
}