mod basic;
mod command;
mod static_token;

use anyhow::Result;

pub use basic::BasicAuthProvider;
pub use command::CommandTokenProvider;
pub use static_token::StaticTokenProvider;

use async_trait::async_trait;
use reqwest::RequestBuilder;

use crate::airflow::config::{AirflowAuth, BasicAuth, TokenSource};
use crate::airflow::managed_services::astronomer::AstronomerAuthProvider;
use crate::airflow::managed_services::composer::ComposerAuthProvider;
use crate::airflow::managed_services::conveyor::ConveyorAuthProvider;
use crate::airflow::managed_services::mwaa::MwaaAuthProvider;

/// Authentication provider trait for Airflow API requests.
///
/// Each implementation decorates a `RequestBuilder` with the appropriate
/// authentication headers/cookies for a specific auth method.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder>;
}

/// Create an auth provider from an `AirflowAuth` config enum variant.
pub fn create_auth_provider(auth: &AirflowAuth) -> Result<Box<dyn AuthProvider>> {
    match auth {
        AirflowAuth::Basic(BasicAuth { username, password }) => Ok(Box::new(BasicAuthProvider {
            username: username.clone(),
            password: password.clone(),
        })),
        AirflowAuth::Token(TokenSource::Static { token }) => Ok(Box::new(StaticTokenProvider {
            token: token.clone(),
        })),
        AirflowAuth::Token(TokenSource::Command { cmd }) => {
            Ok(Box::new(CommandTokenProvider { cmd: cmd.clone() }))
        }
        AirflowAuth::Conveyor => Ok(Box::new(ConveyorAuthProvider)),
        AirflowAuth::Mwaa(mwaa_auth) => Ok(Box::new(MwaaAuthProvider::from(mwaa_auth))),
        AirflowAuth::Astronomer(astro_auth) => {
            Ok(Box::new(AstronomerAuthProvider::from(astro_auth)))
        }
        AirflowAuth::Composer(composer_auth) => {
            Ok(Box::new(ComposerAuthProvider::new(composer_auth)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_auth_provider_basic() {
        let auth = AirflowAuth::Basic(BasicAuth {
            username: "user".to_string(),
            password: "pass".to_string(),
        });
        assert!(create_auth_provider(&auth).is_ok());
    }

    #[test]
    fn test_create_auth_provider_static_token() {
        let auth = AirflowAuth::Token(TokenSource::Static {
            token: "tok".to_string(),
        });
        assert!(create_auth_provider(&auth).is_ok());
    }

    #[test]
    fn test_create_auth_provider_command_token() {
        let auth = AirflowAuth::Token(TokenSource::Command {
            cmd: "echo hi".to_string(),
        });
        assert!(create_auth_provider(&auth).is_ok());
    }
}
