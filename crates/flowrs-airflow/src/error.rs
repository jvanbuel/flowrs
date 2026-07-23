//! The error type returned by every fallible operation in this crate.

/// Result alias for this crate's operations.
pub type Result<T, E = AirflowError> = std::result::Result<T, E>;

/// Everything that can go wrong while talking to an Airflow deployment.
///
/// The variants are the categories a caller can meaningfully act on: a bad
/// endpoint is a config problem, a `Status` is the server rejecting the call,
/// and a `Decode` means the deployment speaks a schema we do not understand.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AirflowError {
    /// The configured endpoint is not a usable base URL.
    #[error("invalid endpoint URL '{url}': {source}")]
    InvalidUrl {
        url: String,
        #[source]
        source: url::ParseError,
    },

    /// The request could not be completed: DNS, TLS, connect, or timeout.
    #[error("HTTP request failed: {source}")]
    Http {
        #[from]
        source: reqwest::Error,
    },

    /// The server answered with a non-success status.
    ///
    /// `detail` holds the response body, which is where Airflow puts the reason a
    /// call was rejected. `Response::error_for_status` throws that body away, so we
    /// read it ourselves before turning the status into an error.
    #[error("{method} {path} failed with HTTP {status}: {detail}")]
    Status {
        method: String,
        path: String,
        status: u16,
        detail: String,
    },

    /// The response body did not match the schema this client expects.
    #[error("failed to parse {context}: {source} (body starts with: {snippet})")]
    Decode {
        context: String,
        snippet: String,
        #[source]
        source: serde_json::Error,
    },

    /// Obtaining credentials failed.
    #[error("{provider} authentication failed: {message}")]
    Auth {
        provider: &'static str,
        message: String,
    },

    /// Discovering servers from a managed service failed.
    #[error("{service} discovery failed: {message}")]
    Discovery {
        service: &'static str,
        message: String,
    },

    /// The configured auth or service needs a Cargo feature that was not compiled in.
    #[error("{service} support is not compiled in; rebuild with the '{feature}' feature")]
    FeatureNotEnabled {
        service: &'static str,
        feature: &'static str,
    },
}

/// How much of a response body to keep in an error message.
const SNIPPET_LEN: usize = 1000;

impl AirflowError {
    pub(crate) fn invalid_url(url: impl Into<String>, source: url::ParseError) -> Self {
        Self::InvalidUrl {
            url: url.into(),
            source,
        }
    }

    pub(crate) fn status(
        method: &reqwest::Method,
        url: &url::Url,
        status: reqwest::StatusCode,
        body: &str,
    ) -> Self {
        Self::Status {
            method: method.to_string(),
            // Keep the query string so pagination and filter parameters stay
            // visible in the error message.
            path: match url.query() {
                Some(query) => format!("{}?{query}", url.path()),
                None => url.path().to_string(),
            },
            status: status.as_u16(),
            // An empty body would render as a message ending in a bare colon, so fall
            // back to the status' canonical reason.
            detail: if body.trim().is_empty() {
                status
                    .canonical_reason()
                    .unwrap_or("no response body")
                    .to_string()
            } else {
                truncate(body)
            },
        }
    }

    pub(crate) fn decode(
        context: impl Into<String>,
        body: &str,
        source: serde_json::Error,
    ) -> Self {
        Self::Decode {
            context: context.into(),
            snippet: truncate(body),
            source,
        }
    }

    /// Flattens an `anyhow` chain into `message`.
    ///
    /// The managed-service integrations use `anyhow` internally for their layered
    /// `.context()` messages. Alternate formatting joins the whole chain into one
    /// line, so no context is lost when only `Display` is rendered.
    pub(crate) fn auth(provider: &'static str, source: &anyhow::Error) -> Self {
        Self::Auth {
            provider,
            message: format!("{source:#}"),
        }
    }

    /// See [`AirflowError::auth`] for why the chain is flattened into a string.
    #[cfg(any(feature = "conveyor", feature = "mwaa"))]
    pub(crate) fn discovery(service: &'static str, source: &anyhow::Error) -> Self {
        Self::Discovery {
            service,
            message: format!("{source:#}"),
        }
    }
}

fn truncate(body: &str) -> String {
    body.chars().take(SNIPPET_LEN).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url() -> url::Url {
        "http://localhost:8080/api/v2/dags/x/dagRuns"
            .parse()
            .expect("test URL")
    }

    #[test]
    fn status_error_preserves_the_query_string() {
        let url: url::Url = "http://localhost:8080/api/v2/dags?limit=100&offset=200"
            .parse()
            .expect("test URL");
        let error = AirflowError::status(
            &reqwest::Method::GET,
            &url,
            reqwest::StatusCode::BAD_REQUEST,
            "bad request",
        );
        assert!(
            error.to_string().contains("?limit=100&offset=200"),
            "got: {error}"
        );
    }

    #[test]
    fn status_error_surfaces_the_response_body() {
        let error = AirflowError::status(
            &reqwest::Method::POST,
            &url(),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
            r#"{"detail":"logical_date is in the past"}"#,
        );
        assert!(
            error.to_string().contains("logical_date is in the past"),
            "got: {error}"
        );
    }

    #[test]
    fn status_error_without_a_body_falls_back_to_the_status_reason() {
        let error = AirflowError::status(
            &reqwest::Method::GET,
            &url(),
            reqwest::StatusCode::NOT_FOUND,
            "   ",
        );
        assert!(error.to_string().ends_with("Not Found"), "got: {error}");
    }
}
