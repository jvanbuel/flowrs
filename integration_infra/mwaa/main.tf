resource "aws_mwaa_environment" "this" {
  dag_s3_path        = "dags/"
  execution_role_arn = aws_iam_role.this.arn
  name           = var.environment_name
  webserver_access_mode = "PUBLIC_ONLY" 

  network_configuration {
    security_group_ids = [aws_security_group.this.id]
    subnet_ids         = aws_subnet.private[*].id
  }

  source_bucket_arn = aws_s3_bucket.dags.arn
}


resource "aws_s3_bucket" "dags" {
  bucket = "flowrs-mwaa-${random_string.suffix.result}"
}

resource "random_string" "suffix" {
  length = 6
  special = false
  upper = false
}
