# AWS MWAA Infrastructure

This directory contains Terraform configuration for deploying a simple Amazon Managed Workflows for Apache Airflow (MWAA) environment.

## Architecture

This deployment creates:

- **VPC**: A dedicated VPC (10.0.0.0/16) with DNS support
- **Subnets**:
  - 2 public subnets for NAT Gateways
  - 2 private subnets for MWAA (required for MWAA deployment)
- **Internet Gateway**: For outbound connectivity
- **NAT Gateways**: 2 NAT Gateways (one per AZ) for private subnet internet access
- **Security Group**: MWAA security group allowing internal communication
- **S3 Bucket**: Versioned S3 bucket for DAGs storage with an example DAG
- **IAM Role**: Execution role with necessary permissions for MWAA
- **MWAA Environment**: Apache Airflow managed environment

## Prerequisites

- AWS CLI configured with appropriate credentials
- Terraform >= 1.0
- AWS account with permissions to create VPC, MWAA, S3, and IAM resources

## Configuration

You can customize the deployment by modifying variables in `terraform.tfvars` or passing them via command line:

| Variable                | Description                  | Default       |
| ----------------------- | ---------------------------- | ------------- |
| `aws_region`            | AWS region for deployment    | `eu-west-1`   |
| `environment_name`      | Name of the MWAA environment | `flowrs-mwaa` |
| `airflow_version`       | Airflow version              | `2.10.3`      |
| `environment_class`     | MWAA environment class       | `mw1.small`   |
| `max_workers`           | Maximum number of workers    | `2`           |
| `min_workers`           | Minimum number of workers    | `1`           |
| `webserver_access_mode` | Webserver access mode        | `PUBLIC_ONLY` |

## Deployment

### 1. Initialize Terraform

```bash
terraform init
```

### 2. Review the plan

```bash
terraform plan
```

### 3. Apply the configuration

```bash
terraform apply
```

The deployment typically takes 20-30 minutes as MWAA environment creation is time-consuming.

### 4. Get the webserver URL

After successful deployment:

```bash
terraform output mwaa_webserver_url
```

## Customization

### Using a custom terraform.tfvars file

Create a `terraform.tfvars` file:

```hcl
aws_region       = "eu-west-1"
environment_name = "my-airflow"
airflow_version  = "2.10.3"
environment_class = "mw1.medium"
```

### Changing Airflow version

Available versions can be found in the [AWS MWAA documentation](https://docs.aws.amazon.com/mwaa/latest/userguide/airflow-versions.html).

### Adding custom requirements

To add Python dependencies:

1. Create a `requirements.txt` file
2. Upload it to your S3 bucket: `s3://<bucket-name>/requirements.txt`
3. Add to MWAA configuration in `main.tf`:

```hcl
resource "aws_mwaa_environment" "this" {
  # ... other configuration ...

  requirements_s3_path = "requirements.txt"
}
```

### Adding plugins

Similar to requirements, upload plugins to `s3://<bucket-name>/plugins.zip` and reference:

```hcl
resource "aws_mwaa_environment" "this" {
  # ... other configuration ...

  plugins_s3_path = "plugins.zip"
}
```

## Adding DAGs

Upload your DAG files to the S3 bucket under the `dags/` prefix:

```bash
aws s3 cp my_dag.py s3://$(terraform output -raw s3_bucket_name)/dags/
```

MWAA will automatically detect and load new DAGs (may take a few minutes).

## Connecting with flowrs

After deployment, add the MWAA environment to your flowrs configuration via

```bash
flowrs config enable -m mwaa
```

Ensure your AWS credentials are set up correctly for flowrs to access the MWAA environment.

## Accessing the Airflow UI

MWAA uses AWS authentication. To access:

1. Navigate to the AWS MWAA console
2. Find your environment
3. Click "Open Airflow UI"

Or use the AWS CLI to get a web login token:

```bash
aws mwaa create-web-login-token \
  --name $(terraform output -raw mwaa_environment_name) \
  --region us-east-1
```

## Cost Estimation

The minimum cost for this setup includes:

- MWAA environment (mw1.small): ~$0.49/hour (~$350/month)
- NAT Gateways (2): ~$0.045/hour each (~$65/month total)
- S3 storage: Minimal (typically < $1/month)
- Data transfer: Variable

**Total estimated cost**: ~$420/month for the smallest configuration

To reduce costs:

- Use `mw1.small` environment class
- Consider removing one NAT Gateway (reduces redundancy)
- Use PRIVATE_ONLY webserver access with VPN/bastion

## Cleanup

To destroy all resources:

```bash
terraform destroy
```

**Note**: Ensure the S3 bucket is empty or Terraform will fail. You may need to:

```bash
aws s3 rm s3://$(terraform output -raw s3_bucket_name) --recursive
terraform destroy
```

## Additional Resources

- [AWS MWAA Documentation](https://docs.aws.amazon.com/mwaa/)
- [Apache Airflow Documentation](https://airflow.apache.org/)
- [flowrs Documentation](https://github.com/jvanbuel/flowrs)
