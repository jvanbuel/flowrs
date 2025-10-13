# Astronomer Infrastructure

Infrastructure as Code for deploying Apache Airflow using Astronomer.

## Coming Soon

This directory will contain configuration for deploying Airflow via:

- **Astronomer Cloud**: Managed Airflow service
- **Astronomer Software**: Self-hosted Kubernetes-based deployment

## Planned Resources

### For Astronomer Cloud

- Workspace configuration
- Deployment definitions
- Environment variables
- Connection secrets

### For Astronomer Software (Kubernetes)

- Kubernetes cluster (EKS/GKE/AKS)
- Helm chart configuration
- Astronomer platform installation
- Ingress and certificate management
- Monitoring and observability stack

## Prerequisites

### Astronomer Cloud
- Astronomer account
- Astro CLI installed

### Astronomer Software
- Kubernetes cluster (v1.25+)
- kubectl installed and configured
- Helm 3 installed
- Domain name for Airflow UI

## Status

Status: Not yet implemented

## References

- [Astronomer Documentation](https://docs.astronomer.io/)
- [Astro CLI](https://docs.astronomer.io/astro/cli/overview)
- [Astronomer Software Installation](https://docs.astronomer.io/software/install-aws)
