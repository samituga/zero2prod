variable "aws_region" {
  description = "The AWS region to deploy the infrastructure"
  default     = "eu-west-1"
}

variable "ecs_cluster_name" {
  description = "The ECS cluster name"
  default     = "rust-server-cluster"
}

variable "ecs_service_name" {
  description = "The ECS service name"
  default     = "rust-server-service"
}

variable "image_repo_name" {
  description = "image repository name"
  type        = string
  default     = "rust-server"
}

variable "github_token" {
  description = "The GitHub OAuth token"
  type        = string
}

variable "github_repository" {
  description = "The GitHub repository name"
  type        = string
}

variable "github_username" {
  description = "The GitHub username"
  type        = string
}

variable "db_name" {
  description = "The database name"
  type        = string
}

variable "db_username" {
  description = "The database credential username"
  type        = string
}

variable "db_password" {
  description = "The database credential password"
  type        = string
}
