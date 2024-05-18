output "ecr_repository_uri" {
  value = aws_ecr_repository.rust_server.repository_url
}
