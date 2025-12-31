# GCP Cloud Composer Infrastructure

Terraform configuration for deploying a minimal Cloud Composer 3 environment in an existing GCP project.

## Features

- Configures required APIs in an existing GCP project
- VPC network with proper subnet configuration for Composer 3
- Cloud Composer 3 environment with minimal resource configuration
- Service account with Composer Worker role
- Example DAG included

## Prerequisites

- [Terraform](https://www.terraform.io/downloads) >= 1.0
- [Google Cloud SDK](https://cloud.google.com/sdk/docs/install) installed and authenticated
- An existing GCP project with billing enabled
- IAM permissions to enable APIs and create resources in the project

## Quick Start

1. **Authenticate with GCP:**
   ```bash
   gcloud auth application-default login
   ```

2. **Copy and configure variables:**
   ```bash
   cp terraform.tfvars.example terraform.tfvars
   # Edit terraform.tfvars with your values
   ```

3. **Initialize and apply:**
   ```bash
   terraform init
   terraform plan
   terraform apply
   ```

   Note: Composer environment creation takes approximately 25-30 minutes.

4. **Get the Airflow URL:**
   ```bash
   terraform output airflow_uri
   ```

## Configuration

### Required Variables

| Variable | Description |
|----------|-------------|
| `project_id` | Existing GCP project ID |

### Optional Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `region` | `europe-west1` | GCP region |
| `environment_name` | `flowrs-composer` | Composer environment name |
| `airflow_image_version` | `composer-3-airflow-3.1.0` | Composer image version |

## Outputs

| Output | Description |
|--------|-------------|
| `project_id` | GCP project ID |
| `airflow_uri` | Airflow web UI URL |
| `gcs_bucket` | GCS bucket path for DAGs |
| `composer_environment_name` | Composer environment name |
| `service_account_email` | Service account used by Composer |

## Using with flowrs

After the environment is created, configure flowrs to auto-discover Composer environments:

```toml
# In ~/.config/flowrs/config.toml
managed_services = ["Gcc"]
```

Flowrs will automatically discover all accessible Composer environments using your Application Default Credentials.

## Uploading DAGs

Upload DAGs to the GCS bucket:

```bash
# Get the DAGs bucket path
DAGS_BUCKET=$(terraform output -raw gcs_bucket)

# Upload a DAG
gsutil cp my_dag.py $DAGS_BUCKET/
```

## Cost Estimate

The minimal Composer 3 configuration costs approximately **$300-400/month**, which includes:
- Composer environment (small)
- GKE Autopilot cluster (managed)
- Cloud Storage for DAGs
- Cloud Logging and Monitoring

## Cleanup

To destroy all resources:

```bash
terraform destroy
```

Note: This will delete all Composer resources within the project (but not the project itself).

## Troubleshooting

### API Enablement

If you see API-related errors, ensure the Composer API is enabled:

```bash
gcloud services enable composer.googleapis.com --project=$(terraform output -raw project_id)
```

### Permission Errors

Ensure your account has the required permissions on the organization/folder and billing account.

### Composer Creation Timeout

Composer environment creation can take 25-30 minutes. If Terraform times out, run `terraform apply` again to continue.

## References

- [Cloud Composer Documentation](https://cloud.google.com/composer/docs)
- [Cloud Composer 3 Overview](https://cloud.google.com/composer/docs/composer-3/composer-3-overview)
- [Terraform Google Provider - Composer](https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/composer_environment)
