pub type Result<T> = std::result::Result<T, FlowrsError>;

#[derive(Debug)]
pub enum FlowrsError {
    ConfigError(ConfigError),
}

#[derive(Debug)]
pub enum ConfigError {
    Serde(ConfigSerdeError),
    Input(inquire::InquireError),
    IO(std::io::Error),
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
        }
    }
}

impl std::fmt::Display for FlowrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FlowrsError::ConfigError(e) => write!(f, "ConfigError: {}", e),
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
