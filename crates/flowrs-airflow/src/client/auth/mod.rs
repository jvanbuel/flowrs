mod basic;
mod command;
mod static_token;

use anyhow::Result;

pub use basic::BasicAuthProvider;
pub use command::CommandTokenProvider;
pub use static_token::StaticTokenProvider;

use async_trait::async_trait;
use reqwest::RequestBuilder;

use crate::auth::{AirflowAuth, BasicAuth, TokenSource};
#[cfg(feature = "astronomer")]
use crate::managed_services::astronomer::AstronomerAuthProvider;
#[cfg(feature = "composer")]
use crate::managed_services::composer::ComposerAuthProvider;
#[cfg(feature = "conveyor")]
use crate::managed_services::conveyor::ConveyorAuthProvider;
#[cfg(feature = "mwaa")]
use crate::managed_services::mwaa::MwaaAuthProvider;

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
        #[cfg(feature = "conveyor")]
        AirflowAuth::Conveyor => Ok(Box::new(ConveyorAuthProvider)),
        #[cfg(not(feature = "conveyor"))]
        AirflowAuth::Conveyor => {
            anyhow::bail!("Conveyor support not compiled. Enable the 'conveyor' feature.")
        }
        #[cfg(feature = "mwaa")]
        AirflowAuth::Mwaa(mwaa_auth) => Ok(Box::new(MwaaAuthProvider::from(mwaa_auth))),
        #[cfg(not(feature = "mwaa"))]
        AirflowAuth::Mwaa(_) => {
            anyhow::bail!("MWAA support not compiled. Enable the 'mwaa' feature.")
        }
        #[cfg(feature = "astronomer")]
        AirflowAuth::Astronomer(astro_auth) => {
            Ok(Box::new(AstronomerAuthProvider::from(astro_auth)))
        }
        #[cfg(not(feature = "astronomer"))]
        AirflowAuth::Astronomer(_) => {
            anyhow::bail!("Astronomer support not compiled. Enable the 'astronomer' feature.")
        }
        #[cfg(feature = "composer")]
        AirflowAuth::Composer(composer_auth) => {
            Ok(Box::new(ComposerAuthProvider::new(composer_auth)?))
        }
        #[cfg(not(feature = "composer"))]
        AirflowAuth::Composer(_) => {
            anyhow::bail!(
                "Google Cloud Composer support not compiled. Enable the 'composer' feature."
            )
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
