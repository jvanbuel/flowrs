mod common;

use common::{create_test_client_v3, should_run_for_api_version};

#[tokio::test]
async fn test_v2_list_dags() {
    if !should_run_for_api_version("v2") {
        println!("Skipping V2 test - TEST_API_VERSION is not 'v2'");
        return;
    }

    let client = create_test_client_v3()
        .await
        .expect("Failed to create test client");
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
async fn test_v2_dag_has_required_fields() {
    if !should_run_for_api_version("v2") {
        return;
    }

    let client = create_test_client_v3()
        .await
        .expect("Failed to create test client");
    let dag_list = client.list_dags().await.expect("Failed to list DAGs");

    if let Some(dag) = dag_list.dags.first() {
        // Verify DAG has required fields populated
        assert!(!dag.dag_id.is_empty(), "DAG ID should not be empty");
    }
}

#[tokio::test]
async fn test_v2_get_dag_code() {
    if !should_run_for_api_version("v2") {
        return;
    }

    let client = create_test_client_v3()
        .await
        .expect("Failed to create test client");
    let dag_list = client.list_dags().await.expect("Failed to list DAGs");

    if let Some(dag) = dag_list.dags.first() {
        let code = client
            .get_dag_code(dag)
            .await
            .expect("Failed to get DAG code");
        assert!(
            code.contains(&*dag.dag_id),
            "DAG code should contain DAG ID"
        );
    }
}

#[tokio::test]
async fn test_v2_list_dagruns() {
    if !should_run_for_api_version("v2") {
        return;
    }

    let client = create_test_client_v3()
        .await
        .expect("Failed to create test client");
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
async fn test_v2_dag_stats() {
    if !should_run_for_api_version("v2") {
        return;
    }

    let client = create_test_client_v3()
        .await
        .expect("Failed to create test client");
    let dag_list = client.list_dags().await.expect("Failed to list DAGs");

    if !dag_list.dags.is_empty() {
        let dag_ids: Vec<&str> = dag_list
            .dags
            .iter()
            .take(2)
            .map(|d| d.dag_id.as_ref())
            .collect();
        let result = client.get_dag_stats(dag_ids.clone()).await;
        assert!(
            result.is_ok(),
            "Failed to get DAG stats: {:?}",
            result.err()
        );
        // Note: stats may be empty if no DAG runs have occurred yet
        // The important thing is that the API call succeeds
    }
}

#[tokio::test]
async fn test_v2_list_task_instances() {
    if !should_run_for_api_version("v2") {
        return;
    }

    let client = create_test_client_v3()
        .await
        .expect("Failed to create test client");
    let dag_list = client.list_dags().await.expect("Failed to list DAGs");

    if let Some(dag) = dag_list.dags.first() {
        let dagruns = client
            .list_dagruns(&dag.dag_id)
            .await
            .expect("Failed to list DAG runs");

        if let Some(dagrun) = dagruns.dag_runs.first() {
            let result = client
                .list_task_instances(&dag.dag_id, &dagrun.dag_run_id)
                .await;
            assert!(
                result.is_ok(),
                "Failed to list task instances: {:?}",
                result.err()
            );
            // Note: task instances may be empty depending on DAG configuration
        }
    }
}
