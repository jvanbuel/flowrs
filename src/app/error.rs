pub type Result<T> = std::result::Result<T, FlowrsError>;

#[derive(Debug)]
pub enum FlowrsError {
    ConfigError(ConfigError),
    APIError(reqwest::Error),
}

#[derive(Debug)]
pub enum ConfigError {
    Serde(ConfigSerdeError),
    Input(inquire::InquireError),
    IO(std::io::Error),
    TokenCmdParse(std::string::FromUtf8Error),
    URLParse(url::ParseError),
    LogError(log::SetLoggerError),
}

#[derive(Debug)]
pub enum ConfigSerdeError {
    ConfigSerializeError(toml::ser::Error),
    ConfigDeserializeError(toml::de::Error),
}

impl std::fmt::Display for ConfigSerdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigSerdeError::ConfigSerializeError(e) => write!(f, "ConfigSerializeError: {}", e),
            ConfigSerdeError::ConfigDeserializeError(e) => {
                write!(f, "ConfigDeserializeError: {}", e)
            }
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::Serde(e) => write!(f, "SerdeError: {}", e),
            ConfigError::Input(e) => write!(f, "InputError: {}", e),
            ConfigError::IO(e) => write!(f, "IOError: {}", e),
            ConfigError::TokenCmdParse(e) => write!(f, "TokenCmdParseError: {}", e),
            ConfigError::URLParse(e) => write!(f, "URLParseError: {}", e),
            ConfigError::LogError(e) => write!(f, "LogError: {}", e),
        }
    }
}

impl std::fmt::Display for FlowrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FlowrsError::ConfigError(e) => write!(f, "ConfigError: {}", e),
            FlowrsError::APIError(e) => write!(f, "APIError: {}", e),
        }
    }
}

impl From<toml::de::Error> for FlowrsError {
    fn from(error: toml::de::Error) -> Self {
        FlowrsError::ConfigError(ConfigError::Serde(
            ConfigSerdeError::ConfigDeserializeError(error),
        ))
    }
}

impl From<toml::ser::Error> for FlowrsError {
    fn from(error: toml::ser::Error) -> Self {
        FlowrsError::ConfigError(ConfigError::Serde(ConfigSerdeError::ConfigSerializeError(
            error,
        )))
    }
}

impl From<inquire::InquireError> for FlowrsError {
    fn from(error: inquire::InquireError) -> Self {
        FlowrsError::ConfigError(ConfigError::Input(error))
    }
}

impl From<std::io::Error> for FlowrsError {
    fn from(error: std::io::Error) -> Self {
        FlowrsError::ConfigError(ConfigError::IO(error))
    }
}

impl From<std::string::FromUtf8Error> for FlowrsError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        FlowrsError::ConfigError(ConfigError::TokenCmdParse(error))
    }
}

impl From<reqwest::Error> for FlowrsError {
    fn from(error: reqwest::Error) -> Self {
        FlowrsError::APIError(error)
    }
}

impl From<url::ParseError> for FlowrsError {
    fn from(error: url::ParseError) -> Self {
        FlowrsError::ConfigError(ConfigError::URLParse(error))
    }
}

impl From<log::SetLoggerError> for FlowrsError {
    fn from(error: log::SetLoggerError) -> Self {
        FlowrsError::ConfigError(ConfigError::LogError(error))
    }
}
