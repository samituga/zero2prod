resource "aws_ecs_cluster" "main" {
  name = var.ecs_cluster_name
}

resource "aws_iam_role" "ecs_task_execution_new" {
  name = "ecsTaskExecutionRoleNew"

  assume_role_policy = jsonencode({
    Version   = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "ecs_task_execution" {
  role       = aws_iam_role.ecs_task_execution_new.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_iam_role_policy_attachment" "ecs_task_execution_ecr" {
  role       = aws_iam_role.ecs_task_execution_new.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonEC2ContainerRegistryReadOnly"
}

resource "aws_cloudwatch_log_group" "ecs_log_group" {
  name              = "/ecs/rust-server"
  retention_in_days = 7

  tags = {
    Name = "ecs-log-group"
  }
}

resource "aws_ecs_task_definition" "rust_server" {
  family                   = "rust-server-task"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = "256"
  memory                   = "512"
  execution_role_arn       = aws_iam_role.ecs_task_execution_new.arn
  container_definitions    = jsonencode([
    {
      name         = "rust-server"
      image        = "${aws_ecr_repository.rust_server.repository_url}:latest"
      essential    = true
      portMappings = [
        {
          containerPort = 8080
          hostPort      = 8080
          protocol      = "tcp"
        }
      ]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          awslogs-group         = aws_cloudwatch_log_group.ecs_log_group.name
          awslogs-region        = var.aws_region
          awslogs-stream-prefix = "ecs"
        }
      }
    }
  ])
}

resource "aws_security_group" "ecs_task" {
  vpc_id = aws_vpc.main.id

  ingress {
    from_port       = 8080
    to_port         = 8080
    protocol        = "tcp"
    security_groups = [aws_security_group.lb.id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_ecs_service" "rust_server" {
  name            = var.ecs_service_name
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.rust_server.arn
  launch_type     = "FARGATE"
  desired_count   = 1

  load_balancer {
    target_group_arn = aws_lb_target_group.ecs.arn
    container_name   = "rust-server"
    container_port   = 8080
  }

  network_configuration {
    subnets          = aws_subnet.public[*].id
    assign_public_ip = true
    security_groups  = [aws_security_group.ecs_task.id]
  }

  deployment_controller {
    type = "ECS"
  }

  depends_on = [aws_lb_listener.http]
}
