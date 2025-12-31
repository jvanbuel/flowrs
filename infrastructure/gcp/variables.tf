variable "project_id" {
  description = "Existing GCP project ID"
  type        = string
}

variable "region" {
  description = "GCP region for Composer environment"
  type        = string
  default     = "europe-west1"
}

variable "environment_name" {
  description = "Name of the Composer environment"
  type        = string
  default     = "flowrs-composer"
}

variable "airflow_image_version" {
  description = "Composer image version (includes Airflow version)"
  type        = string
  # default     = "composer-3-airflow-2.10.5"
  default     = "composer-3-airflow-3.1.0"
}
