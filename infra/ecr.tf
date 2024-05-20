resource "aws_ecr_repository" "rust_server" {
  name = var.image_repo_name

  image_tag_mutability = "MUTABLE"
}
