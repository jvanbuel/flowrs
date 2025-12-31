use serde::{Deserialize, Serialize};

/// Composer authentication data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComposerAuth {
    pub project_id: String,
    pub environment_name: String,
}
