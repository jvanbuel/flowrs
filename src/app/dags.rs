use super::client::AirFlowClient;

pub struct DagList {
    pub dags: Vec<Dag>,
}

pub struct Dag {
    name: String,
}

impl AirFlowClient<'_> {
    pub fn list_dags(&self) -> DagList {
        DagList {
            dags: vec![Dag {
                name: "test".to_string(),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::app::auth::get_config;
    use crate::app::client::AirFlowClient;

    #[test]
    fn test_list_dags() {
        let binding = get_config(Some(&Path::new(".flowrs")));
        let client = AirFlowClient::new(&binding.servers[1]);

        let daglist = client.list_dags();
        assert_eq!(daglist.dags[0].name, "test");
    }
}
