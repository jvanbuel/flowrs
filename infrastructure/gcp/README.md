# GCP Cloud Composer Infrastructure

Infrastructure as Code for deploying Apache Airflow using Google Cloud Composer.

## Coming Soon

This directory will contain Terraform configuration for deploying:

- Cloud Composer environment (managed Airflow)
- VPC network configuration
- Cloud Storage buckets for DAGs and data
- IAM roles and service accounts

## Planned Resources

- **VPC Network**: Dedicated network for Composer
- **Cloud Composer Environment**: Managed Airflow environment
  - Airflow version configuration
  - Node configuration (machine types, node count)
  - Networking (IP allocation, private IP)
- **Cloud Storage**: GCS buckets for DAGs, plugins, and data
- **Service Account**: Identity with appropriate permissions
- **IAM Bindings**: Role assignments for the service account

## Prerequisites

- Google Cloud SDK installed and configured
- Terraform >= 1.0
- GCP project with appropriate APIs enabled:
  - Cloud Composer API
  - Compute Engine API
  - Cloud Storage API
- Appropriate IAM permissions

## Deployment Options

### Standard Environment
- Smaller scale deployments
- Lower cost
- Suitable for development and small production workloads

### Cloud Composer 2
- Autoscaling workers
- Better performance
- Enhanced security features

### Cloud Composer 3
- Latest generation
- Improved autoscaling
- Better resource utilization

## Status

Status: Not yet implemented

## References

- [Cloud Composer Documentation](https://cloud.google.com/composer/docs)
- [Cloud Composer Terraform Module](https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/composer_environment)
- [Cloud Composer Pricing](https://cloud.google.com/composer/pricing)
