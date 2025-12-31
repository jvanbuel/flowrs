# Cloud Composer 3 Environment
resource "google_composer_environment" "this" {
  name   = var.environment_name
  region = var.region

  config {
    software_config {
      image_version = var.airflow_image_version
    }

    workloads_config {
      # Minimal scheduler configuration
      scheduler {
        cpu        = 0.5
        memory_gb  = 2
        storage_gb = 1
        count      = 1
      }

      # Minimal triggerer configuration
      triggerer {
        cpu       = 0.5
        memory_gb = 2
        count     = 1
      }

      # Minimal DAG processor configuration
      dag_processor {
        cpu        = 0.5
        memory_gb  = 2
        storage_gb = 1
        count      = 1
      }

      # Minimal web server configuration
      web_server {
        cpu        = 1
        memory_gb  = 2
        storage_gb = 1
      }
    }

    environment_size = "ENVIRONMENT_SIZE_SMALL"

    node_config {
      network         = google_compute_network.composer.id
      subnetwork      = google_compute_subnetwork.composer.id
      service_account = google_service_account.composer.email
    }
  }

  depends_on = [
    google_project_service.composer,
    google_project_iam_member.composer_worker,
  ]
}

# Extract bucket name from DAG GCS prefix
locals {
  # dag_gcs_prefix format: gs://bucket-name/dags
  dags_bucket = split("/", google_composer_environment.this.config[0].dag_gcs_prefix)[2]
}

# Upload an example DAG
resource "google_storage_bucket_object" "example_dag" {
  name   = "dags/example_dag.py"
  bucket = local.dags_bucket

  content = <<-EOT
from datetime import datetime
from airflow import DAG
from airflow.operators.python import PythonOperator

def hello_world():
    print("Hello from Cloud Composer!")

with DAG(
    'example_dag',
    start_date=datetime(2024, 1, 1),
    schedule='@daily',
    catchup=False,
) as dag:
    task = PythonOperator(
        task_id='hello_task',
        python_callable=hello_world,
    )
EOT

  content_type = "text/x-python"
}
