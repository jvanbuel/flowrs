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
**Status**: Coming soon

Deploy Apache Airflow using Google Cloud Composer.

- **Directory**: `gcp/`
- **Technology**: Terraform
- **Planned Features**:
  - VPC network configuration
  - Cloud Composer environment
  - GCS buckets for DAGs
  - Service account with IAM bindings

See [gcp/README.md](gcp/README.md) for more information.

## Quick Start

### AWS Deployment

```bash
cd aws
terraform init
terraform plan
terraform apply
```

See the AWS README for detailed configuration options and connecting to flowrs.

### Other Platforms

Other platforms are not yet implemented. Check the respective README files for status updates and planned features.

## Connecting to flowrs

After deploying an Airflow environment, you can add it to flowrs:

```bash
flowrs config add \
  --name my-airflow \
  --url <airflow-webserver-url> \
  --auth-type <basic|token> \
  [additional auth options]
```

For detailed connection instructions for each platform, see the platform-specific README files.

## Architecture Considerations

When choosing a platform, consider:

- **Cost**: MWAA and Cloud Composer have base costs even when idle
- **Scalability**: Cloud-native solutions auto-scale better
- **Control**: Self-hosted (Astronomer Software) gives more control
- **Maintenance**: Managed services reduce operational overhead
- **Integration**: Choose based on existing cloud infrastructure

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
