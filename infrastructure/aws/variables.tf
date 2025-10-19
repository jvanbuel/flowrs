variable "aws_region" {
  description = "AWS region for MWAA environment"
  type        = string
  default     = "eu-west-1"
}

variable "environment_name" {
  description = "Name of the MWAA environment"
  type        = string
  default     = "flowrs-mwaa"
}

variable "airflow_version" {
  description = "Airflow version for MWAA"
  type        = string
  default     = "2.10.3"
}

variable "environment_class" {
  description = "Environment class for MWAA (mw1.small, mw1.medium, mw1.large, mw1.xlarge, mw1.2xlarge)"
  type        = string
  default     = "mw1.small"
}

variable "max_workers" {
  description = "Maximum number of workers"
  type        = number
  default     = 2
}

variable "min_workers" {
  description = "Minimum number of workers"
  type        = number
  default     = 1
}

variable "webserver_access_mode" {
  description = "Access mode for the Airflow webserver (PUBLIC_ONLY or PRIVATE_ONLY)"
  type        = string
  default     = "PUBLIC_ONLY"
}
