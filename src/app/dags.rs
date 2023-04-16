use crate::app::auth::AirflowConfig;

pub struct DagList {
    pub dags: Vec<Dag>,
}

pub struct Dag {
    name: String,
}

pub fn list_dags(creds: &AirflowConfig) -> DagList {
    DagList {
        dags: vec![Dag {
            name: "test".to_string(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use crate::app::auth::get_config;
    use crate::app::dags::list_dags;

    #[test]
    fn test_list_dags() {
        let daglist = list_dags(&get_config().servers[0]);
        assert_eq!(daglist.dags[0].name, "test");
    }
}
