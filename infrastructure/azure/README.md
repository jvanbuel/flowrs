# Azure Managed Airflow Infrastructure

Infrastructure as Code for deploying Apache Airflow on Azure.

## Coming Soon

This directory will contain Terraform/Bicep configuration for deploying:

- Azure Data Factory Managed Airflow
- Virtual Network configuration
- Storage Account for DAGs
- Identity and Access Management

## Planned Resources

- **Resource Group**: Dedicated resource group for Airflow resources
- **Virtual Network**: VNet with subnets for Airflow components
- **Storage Account**: Blob storage for DAGs and logs
- **Managed Airflow**: Azure Data Factory Managed Airflow environment
- **Identity**: Managed identity with appropriate permissions

## Prerequisites

- Azure CLI installed and configured
- Terraform >= 1.0 (or Azure Bicep)
- Azure subscription with appropriate permissions

## Status

Status: Not yet implemented

## References

- [Azure Data Factory Managed Airflow Documentation](https://learn.microsoft.com/en-us/azure/data-factory/concept-managed-airflow)
