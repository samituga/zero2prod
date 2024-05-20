resource "aws_ecr_repository" "rust_server" {
  name = var.image_repo_name

  image_tag_mutability = "MUTABLE"
}

resource "aws_ecr_lifecycle_policy" "rust_server_lifecycle_policy" {
  repository = aws_ecr_repository.rust_server.name
  policy     = jsonencode(
    {
      "rules" : [
        {
          "rulePriority" : 1,
          "description" : "Delete untagged images older than 1 day",
          "selection" : {
            "tagStatus" : "untagged",
            "countType" : "sinceImagePushed",
            "countUnit" : "days",
            "countNumber" : 1
          },
          "action" : {
            "type" : "expire"
          }
        }
      ]
    }
  )
}
