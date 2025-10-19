terraform {
  required_version = ">= 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# VPC for MWAA
resource "aws_vpc" "mwaa" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name = "${var.environment_name}-vpc"
  }
}

# Internet Gateway for NAT
resource "aws_internet_gateway" "mwaa" {
  vpc_id = aws_vpc.mwaa.id

  tags = {
    Name = "${var.environment_name}-igw"
  }
}

# Public subnets for NAT Gateway
resource "aws_subnet" "public" {
  count             = 2
  vpc_id            = aws_vpc.mwaa.id
  cidr_block        = "10.0.${count.index}.0/24"
  availability_zone = data.aws_availability_zones.available.names[count.index]

  tags = {
    Name = "${var.environment_name}-public-subnet-${count.index + 1}"
  }
}

# Private subnets for MWAA
resource "aws_subnet" "private" {
  count             = 2
  vpc_id            = aws_vpc.mwaa.id
  cidr_block        = "10.0.${count.index + 10}.0/24"
  availability_zone = data.aws_availability_zones.available.names[count.index]

  tags = {
    Name = "${var.environment_name}-private-subnet-${count.index + 1}"
  }
}

# Elastic IP for NAT Gateway
resource "aws_eip" "nat" {
  domain = "vpc"

  tags = {
    Name = "${var.environment_name}-nat-eip"
  }
}

# NAT Gateway
resource "aws_nat_gateway" "mwaa" {
  allocation_id = aws_eip.nat.id
  subnet_id     = aws_subnet.public[0].id

  tags = {
    Name = "${var.environment_name}-nat"
  }

  depends_on = [aws_internet_gateway.mwaa]
}

# Route table for public subnets
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.mwaa.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.mwaa.id
  }

  tags = {
    Name = "${var.environment_name}-public-rt"
  }
}

# Route table associations for public subnets
resource "aws_route_table_association" "public" {
  count          = 2
  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

# Route table for private subnets
resource "aws_route_table" "private" {
  vpc_id = aws_vpc.mwaa.id

  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.mwaa.id
  }

  tags = {
    Name = "${var.environment_name}-private-rt"
  }
}

# Route table associations for private subnets
resource "aws_route_table_association" "private" {
  count          = 2
  subnet_id      = aws_subnet.private[count.index].id
  route_table_id = aws_route_table.private.id
}

# Security group for MWAA
resource "aws_security_group" "mwaa" {
  name        = "${var.environment_name}-mwaa-sg"
  description = "Security group for MWAA environment"
  vpc_id      = aws_vpc.mwaa.id

  ingress {
    from_port = 0
    to_port   = 0
    protocol  = "-1"
    self      = true
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "${var.environment_name}-mwaa-sg"
  }
}

# S3 bucket for DAGs
resource "aws_s3_bucket" "mwaa" {
  bucket = "${var.environment_name}-dags"

  tags = {
    Name = "${var.environment_name}-dags"
  }
}

# Versioning has to be enabled for MWAA S3 bucket
# See https://docs.aws.amazon.com/mwaa/latest/userguide/working-dags.html
resource "aws_s3_bucket_versioning" "mwaa" {
  bucket = aws_s3_bucket.mwaa.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_lifecycle_configuration" "mwaa" {
  bucket = aws_s3_bucket.mwaa.id

  rule {
    id     = "expire-old-versions"
    status = "Enabled"

    noncurrent_version_expiration {
      noncurrent_days = 90
    }
  }
}

resource "aws_s3_bucket_public_access_block" "mwaa" {
  bucket = aws_s3_bucket.mwaa.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# Upload an example DAG
resource "aws_s3_object" "example_dag" {
  bucket = aws_s3_bucket.mwaa.id
  key    = "dags/example_dag.py"
  content = <<-EOT
    from datetime import datetime
    from airflow import DAG
    from airflow.operators.python import PythonOperator

    def hello_world():
        print("Hello from MWAA!")

    with DAG(
        'example_dag',
        start_date=datetime(2024, 1, 1),
        schedule_interval='@daily',
        catchup=False,
    ) as dag:
        task = PythonOperator(
            task_id='hello_task',
            python_callable=hello_world,
        )
  EOT

  content_type = "text/x-python"
}

# IAM role for MWAA
resource "aws_iam_role" "mwaa" {
  name = "${var.environment_name}-mwaa-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = {
          Service = [
            "airflow.amazonaws.com",
            "airflow-env.amazonaws.com"
          ]
        }
        Action = "sts:AssumeRole"
      }
    ]
  })
}

# IAM policy for MWAA
resource "aws_iam_role_policy" "mwaa" {
  name = "${var.environment_name}-mwaa-execution-policy"
  role = aws_iam_role.mwaa.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "airflow:PublishMetrics"
        ]
        Resource = "arn:aws:airflow:${var.aws_region}:${data.aws_caller_identity.current.account_id}:environment/${var.environment_name}"
      },
      {
        Effect = "Allow"
        Action = [
          "s3:ListAllMyBuckets"
        ]
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject*",
          "s3:GetBucket*",
          "s3:List*"
        ]
        Resource = [
          aws_s3_bucket.mwaa.arn,
          "${aws_s3_bucket.mwaa.arn}/*"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogStream",
          "logs:CreateLogGroup",
          "logs:PutLogEvents",
          "logs:GetLogEvents",
          "logs:GetLogRecord",
          "logs:GetLogGroupFields",
          "logs:GetQueryResults"
        ]
        Resource = "arn:aws:logs:${var.aws_region}:${data.aws_caller_identity.current.account_id}:log-group:airflow-${var.environment_name}-*"
      },
      {
        Effect = "Allow"
        Action = [
          "logs:DescribeLogGroups"
        ]
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = "cloudwatch:PutMetricData"
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = [
          "sqs:ChangeMessageVisibility",
          "sqs:DeleteMessage",
          "sqs:GetQueueAttributes",
          "sqs:GetQueueUrl",
          "sqs:ReceiveMessage",
          "sqs:SendMessage"
        ]
        Resource = "arn:aws:sqs:${var.aws_region}:*:airflow-celery-*"
      },
      {
        Effect = "Allow"
        Action = [
          "kms:Decrypt",
          "kms:DescribeKey",
          "kms:GenerateDataKey*",
          "kms:Encrypt"
        ]
        NotResource = "arn:aws:kms:*:${data.aws_caller_identity.current.account_id}:key/*"
        Condition = {
          StringLike = {
            "kms:ViaService" = [
              "sqs.${var.aws_region}.amazonaws.com"
            ]
          }
        }
      }
    ]
  })
}

# MWAA Environment
resource "aws_mwaa_environment" "this" {
  name              = var.environment_name
  airflow_version   = var.airflow_version
  execution_role_arn = aws_iam_role.mwaa.arn

  source_bucket_arn = aws_s3_bucket.mwaa.arn
  dag_s3_path       = "dags"

  network_configuration {
    security_group_ids = [aws_security_group.mwaa.id]
    subnet_ids         = aws_subnet.private[*].id
  }

  logging_configuration {
    dag_processing_logs {
      enabled   = true
      log_level = "INFO"
    }

    scheduler_logs {
      enabled   = true
      log_level = "INFO"
    }

    task_logs {
      enabled   = true
      log_level = "INFO"
    }

    webserver_logs {
      enabled   = true
      log_level = "INFO"
    }

    worker_logs {
      enabled   = true
      log_level = "INFO"
    }
  }

  environment_class = var.environment_class
  max_workers       = var.max_workers
  min_workers       = var.min_workers

  webserver_access_mode = var.webserver_access_mode
}

# Data sources
data "aws_availability_zones" "available" {
  state = "available"
}

data "aws_caller_identity" "current" {}
