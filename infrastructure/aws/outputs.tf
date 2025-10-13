output "mwaa_environment_name" {
  description = "Name of the MWAA environment"
  value       = aws_mwaa_environment.this.name
}

output "mwaa_webserver_url" {
  description = "URL of the MWAA webserver"
  value       = aws_mwaa_environment.this.webserver_url
}

output "mwaa_arn" {
  description = "ARN of the MWAA environment"
  value       = aws_mwaa_environment.this.arn
}

output "s3_bucket_name" {
  description = "Name of the S3 bucket for DAGs"
  value       = aws_s3_bucket.mwaa.id
}

output "mwaa_execution_role_arn" {
  description = "ARN of the MWAA execution role"
  value       = aws_iam_role.mwaa.arn
}

output "vpc_id" {
  description = "ID of the VPC"
  value       = aws_vpc.mwaa.id
}

output "private_subnet_ids" {
  description = "IDs of the private subnets"
  value       = aws_subnet.private[*].id
}

output "security_group_id" {
  description = "ID of the MWAA security group"
  value       = aws_security_group.mwaa.id
}
