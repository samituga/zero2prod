resource "aws_ecr_repository" "rust_server" {
  name = "rust-server"

  image_scanning_configuration {
    scan_on_push = true
  }

  image_tag_mutability = "MUTABLE"
}
