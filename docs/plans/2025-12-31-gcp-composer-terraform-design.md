# GCP Composer Terraform Infrastructure Design

## Overview

Terraform configuration to spin up a minimal GCP Composer 3 environment in a new GCP project.

## Requirements

- **Composer Version:** Composer 3 (latest generation)
- **Project:** Create new GCP project via Terraform
- **Region:** europe-west1 (Belgium)
- **Access:** Public webserver

## Architecture

```text
GCP Project (new)
├── Enabled APIs (Composer, Compute, Storage)
├── VPC Network
│   └── Subnetwork (europe-west1)
├── Service Account
│   └── IAM bindings (Composer Worker role)
├── Cloud Composer 3 Environment
│   ├── Small environment config
│   ├── Public webserver access
│   └── GCS bucket (auto-created by Composer)
└── Example DAG (uploaded to DAGs folder)
```

## File Structure

```text
infrastructure/gcp/
├── main.tf              # Provider, project, APIs
├── network.tf           # VPC and subnet
├── composer.tf          # Composer environment
├── iam.tf               # Service account and bindings
├── variables.tf         # Input variables
├── outputs.tf           # Useful outputs (Airflow URL, etc.)
├── terraform.tfvars.example
└── README.md            # Updated with instructions
```

## Project & API Configuration

### Project Creation

```hcl
resource "google_project" "composer" {
  name            = var.project_name
  project_id      = var.project_id
  billing_account = var.billing_account
  org_id          = var.org_id      # Optional, use one or the other
  folder_id       = var.folder_id   # Optional
}
```

### Required APIs

- `composer.googleapis.com` - Cloud Composer
- `compute.googleapis.com` - VPC networking
- `storage.googleapis.com` - GCS for DAGs

### Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `project_name` | `"flowrs-composer"` | Display name |
| `project_id` | `"flowrs-composer-${random}"` | Must be globally unique |
| `region` | `"europe-west1"` | Belgium |
| `billing_account` | (required) | No default |
| `org_id` | `null` | Either org_id or folder_id required |
| `folder_id` | `null` | Either org_id or folder_id required |

## Network Configuration

### VPC Network

```hcl
resource "google_compute_network" "composer" {
  name                    = "composer-network"
  auto_create_subnetworks = false
}
```

### Subnetwork

```hcl
resource "google_compute_subnetwork" "composer" {
  name          = "composer-subnet"
  ip_cidr_range = "10.0.0.0/24"
  region        = var.region
  network       = google_compute_network.composer.id

  secondary_ip_range {
    range_name    = "pods"
    ip_cidr_range = "10.1.0.0/16"
  }

  secondary_ip_range {
    range_name    = "services"
    ip_cidr_range = "10.2.0.0/20"
  }
}
```

No NAT Gateway needed - Composer 3 with public webserver handles egress automatically.

## Composer 3 Environment

```hcl
resource "google_composer_environment" "this" {
  name   = var.environment_name
  region = var.region

  config {
    software_config {
      image_version = "composer-3-airflow-2.10.2"
    }

    workloads_config {
      scheduler {
        cpu        = 0.5
        memory_gb  = 2
        storage_gb = 1
        count      = 1
      }
      triggerer {
        cpu       = 0.5
        memory_gb = 0.5
        count     = 1
      }
      dag_processor {
        cpu        = 0.5
        memory_gb  = 2
        storage_gb = 1
        count      = 1
      }
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
}
```

**Estimated cost:** ~$300-400/month for smallest Composer 3 config.

## IAM Configuration

### Service Account

```hcl
resource "google_service_account" "composer" {
  account_id   = "composer-worker"
  display_name = "Composer Worker Service Account"
}
```

### IAM Binding

```hcl
resource "google_project_iam_member" "composer_worker" {
  project = google_project.composer.project_id
  role    = "roles/composer.worker"
  member  = "serviceAccount:${google_service_account.composer.email}"
}
```

The `roles/composer.worker` role includes GCS access, logging, and monitoring permissions.

## Outputs

```hcl
output "airflow_uri" {
  description = "Airflow web UI URL"
  value       = google_composer_environment.this.config[0].airflow_uri
}

output "gcs_bucket" {
  description = "GCS bucket for DAGs"
  value       = google_composer_environment.this.config[0].dag_gcs_prefix
}

output "project_id" {
  description = "GCP project ID"
  value       = google_project.composer.project_id
}
```

## Usage

1. `terraform init`
2. `terraform apply` (takes ~25-30 min for Composer)
3. Access Airflow UI via output URL
4. Configure flowrs with `managed_services = ["Gcc"]`
