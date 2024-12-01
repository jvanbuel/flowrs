output "mwaa_webserver_url" {
  value = aws_mwaa_environment.this.webserver_url
}

output "mwaa_dag_bucket" {
  value = aws_s3_bucket.dags.bucket
}