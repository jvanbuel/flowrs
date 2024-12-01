variable "region" {
  description = "The AWS region to deploy into" 
  default = "eu-west-1"
}

variable "account_id" {
  description = "The AWS account ID to deploy into" 
}

variable "environment_name" {
  description = "The name of the environment" 
  default = "mwaa-flowrs"
}