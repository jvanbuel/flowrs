terraform {
  required_providers {
    aws = {
      source = "hashicorp/aws"
      version = "5.78.0"
    }
  }
}

provider "aws" {
  allowed_account_ids = [var.account_id]
}