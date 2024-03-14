use crate::error::Error;
use crate::run::{EventDetail, EventRecord};
use crate::state::*;
use std::time::{Duration, Instant};

use kanidm_client::KanidmClient;

use async_trait::async_trait;
use strum_macros::EnumCount;

#[derive(EnumCount)]
pub enum TransitionAction {
    Login = 0,
    Logout = 1,
    ReadProperty = 2,
    WriteProperty = 3,
}

impl TryFrom<i32> for TransitionAction {
    type Error = ();
    // TODO: avoid future tech debt with this simple trick: don't write each entry manually
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == TransitionAction::Login as i32 => Ok(TransitionAction::Login),
            x if x == TransitionAction::Logout as i32 => Ok(TransitionAction::Logout),
            x if x == TransitionAction::ReadProperty as i32 => Ok(TransitionAction::ReadProperty),
            x if x == TransitionAction::WriteProperty as i32 => Ok(TransitionAction::WriteProperty),
            _ => Err(()),
        }
    }
}

// Is this the right way? Should transitions/delay be part of the actor model? Should
// they be responsible.
pub struct Transition {
    pub delay: Option<Duration>,
    pub action: TransitionAction,
}

impl Transition {
    #[allow(dead_code)]
    pub fn delay(&self) -> Option<Duration> {
        self.delay
    }
}

pub enum TransitionResult {
    // Success
    Ok,
    // We need to re-authenticate, the session expired.
    // AuthenticationNeeded,
    // An error occurred.
    Error,
}

#[async_trait]
pub trait ActorModel {
    async fn transition(
        &mut self,
        client: &KanidmClient,
        person: &Person,
    ) -> Result<EventRecord, Error>;
}

pub async fn login(
    client: &KanidmClient,
    person: &Person,
) -> Result<(TransitionResult, EventRecord), Error> {
    // Should we measure the time of each call rather than the time with multiple calls?
    let start = Instant::now();
    let result = match &person.credential {
        Credential::Password { plain } => {
            client
                .auth_simple_password(person.username.as_str(), plain.as_str())
                .await
        }
    };
    let end = Instant::now();

    let duration = end.duration_since(start);

    match result {
        Ok(_) => Ok((
            TransitionResult::Ok,
            EventRecord {
                start,
                duration,
                details: EventDetail::Authentication,
            },
        )),
        Err(client_err) => {
            debug!(?client_err);
            Ok((
                TransitionResult::Error,
                EventRecord {
                    start,
                    duration,
                    details: EventDetail::Error,
                },
            ))
        }
    }
}

pub async fn person_get(
    client: &KanidmClient,
    person: &Person,
) -> Result<(TransitionResult, EventRecord), Error> {
    // Should we measure the time of each call rather than the time with multiple calls?
    let start = Instant::now();
    let result = client
        .idm_person_account_get(person.username.as_str())
        .await;
    let end = Instant::now();

    let duration = end.duration_since(start);

    match result {
        Ok(_) => Ok((
            TransitionResult::Ok,
            EventRecord {
                start,
                duration,
                details: EventDetail::PersonGet,
            },
        )),
        Err(client_err) => {
            debug!(?client_err);
            Ok((
                TransitionResult::Error,
                EventRecord {
                    start,
                    duration,
                    details: EventDetail::Error,
                },
            ))
        }
    }
}

pub async fn person_set(
    client: &KanidmClient,
    person: &Person,
) -> Result<(TransitionResult, EventRecord), Error> {
    // Should we measure the time of each call rather than the time with multiple calls?
    let start = Instant::now();
    let person_username = person.username.as_str();
    let result = client
        .idm_person_account_set_attr(person_username, "displayname", &[person_username])
        .await;
    let end = Instant::now();

    let duration = end.duration_since(start);

    match result {
        Ok(_) => Ok((
            TransitionResult::Ok,
            EventRecord {
                start,
                duration,
                details: EventDetail::PersonGet,
            },
        )),
        Err(client_err) => {
            debug!(?client_err);
            Ok((
                TransitionResult::Error,
                EventRecord {
                    start,
                    duration,
                    details: EventDetail::Error,
                },
            ))
        }
    }
}

pub async fn logout(
    client: &KanidmClient,
    _person: &Person,
) -> Result<(TransitionResult, EventRecord), Error> {
    let start = Instant::now();
    let result = client.logout().await;
    let end = Instant::now();

    let duration = end.duration_since(start);

    match result {
        Ok(_) => Ok((
            TransitionResult::Ok,
            EventRecord {
                start,
                duration,
                details: EventDetail::Logout,
            },
        )),
        Err(client_err) => {
            debug!(?client_err);
            Ok((
                TransitionResult::Error,
                EventRecord {
                    start,
                    duration,
                    details: EventDetail::Error,
                },
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::TransitionAction;
    use strum::EnumCount;

    #[test]
    fn transition_action_try_from_test() {
        for i in 0..TransitionAction::COUNT {
            let transition_action = TransitionAction::try_from(i as i32);
            assert!(transition_action.is_ok());
            assert_eq!(transition_action.unwrap() as usize, i);
        }
    }
}
