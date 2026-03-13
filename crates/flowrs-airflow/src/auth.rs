use serde::{Deserialize, Serialize};

// --- Core auth types ---

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AirflowAuth {
    Basic(BasicAuth),
    Token(TokenSource),
    Conveyor,
    Mwaa(MwaaAuth),
    Astronomer(AstronomerAuth),
    Composer(ComposerAuth),
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

impl std::fmt::Debug for BasicAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicAuth")
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .finish()
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum TokenSource {
    Command { cmd: String },
    Static { token: String },
}

impl std::fmt::Debug for TokenSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenSource::Command { cmd } => f.debug_struct("Command").field("cmd", cmd).finish(),
            TokenSource::Static { .. } => f
                .debug_struct("Static")
                .field("token", &"<redacted>")
                .finish(),
        }
    }
}

// --- Managed service auth data ---

/// MWAA authentication token type
#[derive(Clone, Serialize, Deserialize)]
pub enum MwaaTokenType {
    /// Session cookie for Airflow 2.x (uses Cookie header)
    SessionCookie(String),
    /// JWT token for Airflow 3.x (uses Bearer auth)
    JwtToken(String),
}

impl std::fmt::Debug for MwaaTokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MwaaTokenType::SessionCookie(_) => write!(f, "SessionCookie(<redacted>)"),
            MwaaTokenType::JwtToken(_) => write!(f, "JwtToken(<redacted>)"),
        }
    }
}

/// MWAA authentication data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MwaaAuth {
    pub token: MwaaTokenType,
    pub environment_name: String,
}

/// Astronomer authentication data including API token
#[derive(Clone, Serialize, Deserialize)]
pub struct AstronomerAuth {
    pub api_token: String,
}

impl std::fmt::Debug for AstronomerAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AstronomerAuth")
            .field("api_token", &"***redacted***")
            .finish()
    }
}

/// Composer authentication data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComposerAuth {
    pub project_id: String,
    pub environment_name: String,
}
