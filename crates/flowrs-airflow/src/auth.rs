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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum TokenSource {
    Command { cmd: String },
    Static { token: String },
}

// --- Managed service auth data ---

/// MWAA authentication token type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MwaaTokenType {
    /// Session cookie for Airflow 2.x (uses Cookie header)
    SessionCookie(String),
    /// JWT token for Airflow 3.x (uses Bearer auth)
    JwtToken(String),
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
