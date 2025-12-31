output "project_id" {
  description = "GCP project ID"
  value       = var.project_id
}

output "airflow_uri" {
  description = "Airflow web UI URL"
  value       = google_composer_environment.this.config[0].airflow_uri
}

output "gcs_bucket" {
  description = "GCS bucket for DAGs"
  value       = google_composer_environment.this.config[0].dag_gcs_prefix
}

output "composer_environment_name" {
  description = "Composer environment name"
  value       = google_composer_environment.this.name
}

output "service_account_email" {
  description = "Service account email used by Composer"
  value       = google_service_account.composer.email
}
