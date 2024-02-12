


struct ConveyorAuth {
    token: String, 
}

impl ConveyorAuth {
    fn new(token: String) -> Self {
        Self {
            token,
        }
    }

    fn refresh_token(&self) -> Self {
        Self {
            token: self.token.clone(),
        }
    }
}