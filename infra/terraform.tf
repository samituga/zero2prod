terraform {
  cloud {
    organization = "avada_demo"
    workspaces {
      name = "zero2prod"
    }
  }

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.47.0"
    }
  }

  required_version = "~> 1.2"
}