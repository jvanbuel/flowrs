# Flowrs Infrastructure

Infrastructure as Code for deploying Apache Airflow across multiple cloud providers and platforms.

## Overview

This directory contains Terraform and other IaC configurations for deploying managed Apache Airflow environments that can be monitored and managed using flowrs.

## Available Environments

### AWS (Amazon Web Services)
**Status**: Implemented ✓

Deploy Apache Airflow using Amazon MWAA (Managed Workflows for Apache Airflow).

- **Directory**: `aws/`
- **Technology**: Terraform
- **Features**:
  - VPC with private subnets
  - S3 bucket for DAGs
  - IAM roles with least privilege
  - NAT Gateways for outbound connectivity
  - Example DAG included

**Cost**: Starting at ~$420/month for smallest configuration

See [aws/README.md](aws/README.md) for detailed instructions.

### Azure
**Status**: Coming soon

Deploy Apache Airflow using Azure Data Factory Managed Airflow.

- **Directory**: `azure/`
- **Technology**: Terraform or Bicep
- **Planned Features**:
  - Virtual Network configuration
  - Storage Account for DAGs
  - Managed Airflow environment
  - Managed Identity authentication

See [azure/README.md](azure/README.md) for more information.

### Astronomer
**Status**: Coming soon

Deploy Apache Airflow using Astronomer (Cloud or Software).

- **Directory**: `astronomer/`
- **Options**:
  - Astronomer Cloud (managed service)
  - Astronomer Software (Kubernetes-based)
- **Planned Features**:
  - Workspace and deployment configuration
  - Kubernetes cluster setup (for Software)
  - Helm chart configuration

See [astronomer/README.md](astronomer/README.md) for more information.

### GCP (Google Cloud Platform)
**Status**: Implemented ✓

Deploy Apache Airflow using Google Cloud Composer 3.

- **Directory**: `gcp/`
- **Technology**: Terraform
- **Features**:
  - Configures required APIs in an existing GCP project
  - VPC network with Composer 3 subnet configuration
  - Cloud Composer 3 environment (minimal config)
  - Service account with Composer Worker role
  - Example DAG included

**Cost**: Starting at ~$300-400/month for smallest configuration

See [gcp/README.md](gcp/README.md) for detailed instructions.

## Contributing

To add a new platform:

1. Create a directory with the platform name
2. Add Terraform/IaC configuration
3. Include a comprehensive README.md
4. Document connection instructions for flowrs
5. Add cost estimates if applicable

## Support

For issues or questions:
- flowrs issues: https://github.com/jvanbuel/flowrs/issues
- Platform-specific issues: Check respective cloud provider documentation
