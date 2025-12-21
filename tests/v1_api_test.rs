mod common;

use common::{create_test_client, should_run_for_api_version};

#[tokio::test]
async fn test_v1_list_dags() {
    if !should_run_for_api_version("v1") {
        println!("Skipping V1 test - TEST_API_VERSION is not 'v1'");
        return;
    }

    let client = create_test_client().expect("Failed to create test client");
    let result = client.list_dags().await;

    assert!(result.is_ok(), "Failed to list DAGs: {:?}", result.err());

    let dag_list = result.unwrap();
    // Airflow with LOAD_EXAMPLES=true should have example DAGs
    assert!(
        !dag_list.dags.is_empty(),
        "Expected at least one DAG, got none"
    );
}

#[tokio::test]
async fn test_v1_dag_has_required_fields() {
    if !should_run_for_api_version("v1") {
        return;
    }

    let client = create_test_client().expect("Failed to create test client");
    let dag_list = client.list_dags().await.expect("Failed to list DAGs");

    if let Some(dag) = dag_list.dags.first() {
        // Verify DAG has required fields populated
        assert!(!dag.dag_id.is_empty(), "DAG ID should not be empty");
    }
}

#[tokio::test]
async fn test_v1_get_dag_code() {
    if !should_run_for_api_version("v1") {
        return;
    }

    let client = create_test_client().expect("Failed to create test client");
    let dag_list = client.list_dags().await.expect("Failed to list DAGs");

    if let Some(dag) = dag_list.dags.first() {
        let code = client
            .get_dag_code(dag)
            .await
            .expect("Failed to get DAG code");
        assert!(
            code.contains("DAG") || code.contains("dag"),
            "DAG code should contain DAG definition"
        );
    }
}

#[tokio::test]
async fn test_v1_list_dagruns() {
    if !should_run_for_api_version("v1") {
        return;
    }

    let client = create_test_client().expect("Failed to create test client");
    let dag_list = client.list_dags().await.expect("Failed to list DAGs");

    if let Some(dag) = dag_list.dags.first() {
        let result = client.list_dagruns(&dag.dag_id).await;
        assert!(
            result.is_ok(),
            "Failed to list DAG runs: {:?}",
            result.err()
        );
        // Note: dag_runs may be empty if no runs have been triggered
    }
}

#[tokio::test]
async fn test_v1_list_tasks() {
    if !should_run_for_api_version("v1") {
        return;
    }

    let client = create_test_client().expect("Failed to create test client");
    let dag_list = client.list_dags().await.expect("Failed to list DAGs");

    if let Some(dag) = dag_list.dags.first() {
        let result = client.list_tasks(&dag.dag_id).await;
        assert!(result.is_ok(), "Failed to list tasks: {:?}", result.err());

        let _task_list = result.unwrap();
        // Most DAGs should have at least one task
        // (empty DAGs are unusual but valid)
    }
}
