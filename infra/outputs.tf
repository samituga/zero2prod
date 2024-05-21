output "ecr_repository_uri" {
  value = aws_ecr_repository.rust_server.repository_url
}

output "ecs_cluster_id" {
  description = "The ID of the ECS cluster"
  value       = aws_ecs_cluster.main.id
}

output "ecs_cluster_name" {
  description = "The name of the ECS cluster"
  value       = aws_ecs_cluster.main.name
}

output "ecs_task_definition_id" {
  description = "The ID of the ECS task definition"
  value       = aws_ecs_task_definition.rust_server.id
}

output "ecs_task_definition_arn" {
  description = "The ARN of the ECS task definition"
  value       = aws_ecs_task_definition.rust_server.arn
}

output "ecs_service_name" {
  description = "The name of the ECS service"
  value       = aws_ecs_service.rust_server.name
}

output "ecs_service_arn" {
  description = "The ID of the ECS service"
  value       = aws_ecs_service.rust_server.id
}

output "cloudwatch_log_group_name" {
  description = "The name of the CloudWatch log group"
  value       = aws_cloudwatch_log_group.ecs_log_group.name
}

output "load_balancer_dns_name" {
  description = "The DNS name of the load balancer"
  value       = aws_lb.ecs.dns_name
}

output "rds_endpoint" {
  description = "The endpoint of the RDS instance"
  value       = aws_db_instance.postgres.endpoint
}

output "rds_port" {
  description = "The port of the RDS instance"
  value       = aws_db_instance.postgres.port
}

output "rds_username" {
  description = "The username for the RDS instance"
  value       = aws_db_instance.postgres.username
}

output "vpc_id" {
  description = "The ID of the VPC"
  value       = aws_vpc.main.id
}

output "subnet_ids" {
  description = "The IDs of the subnets"
  value       = aws_subnet.public[*].id
}

output "ssm_session_role_arn" {
  description = "The ARN of the SSM session role"
  value       = aws_iam_role.ssm_session_role.arn
}

output "ecs_task_arns" {
  description = "The ARNs of the ECS tasks"
  value       = aws_ecs_task_definition.rust_server.arn
}

output "ecs_task_role_arn" {
  description = "The ARN of the ECS task role"
  value       = aws_iam_role.ecs_task_execution_new.arn
}
