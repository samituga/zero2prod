resource "aws_codedeploy_app" "ecs_app" {
  name             = "ecs-codedeploy-app"
  compute_platform = "ECS"
}

resource "aws_codedeploy_deployment_group" "ecs_deployment_group" {
  app_name              = aws_codedeploy_app.ecs_app.name
  deployment_group_name = "ecs-deployment-group"
  service_role_arn      = aws_iam_role.codedeploy.arn

  deployment_config_name = "CodeDeployDefault.ECSAllAtOnce"

  deployment_style {
    deployment_type   = "BLUE_GREEN"
    deployment_option = "WITH_TRAFFIC_CONTROL"
  }

  auto_rollback_configuration {
    enabled = true
    events  = ["DEPLOYMENT_FAILURE"]
  }

  ecs_service {
    cluster_name = aws_ecs_cluster.main.name
    service_name = aws_ecs_service.rust_server.name
  }

  blue_green_deployment_config {
    terminate_blue_instances_on_deployment_success {
      action                           = "TERMINATE"
      termination_wait_time_in_minutes = 1
    }

    deployment_ready_option {
      action_on_timeout    = "CONTINUE_DEPLOYMENT"
      wait_time_in_minutes = 0
    }
  }

  load_balancer_info {
    target_group_pair_info {
      prod_traffic_route {
        listener_arns = [aws_lb_listener.l_80.arn]
      }

      target_group {
        name = aws_lb_target_group.tg[0].name
      }

      target_group {
        name = aws_lb_target_group.tg[1].name
      }
    }
  }
}

resource "aws_iam_role" "codedeploy" {
  name = "codedeploy-role"

  assume_role_policy = jsonencode({
    Version   = "2012-10-17",
    Statement = [
      {
        Effect = "Allow",
        Principal = {
          Service = "codedeploy.amazonaws.com"
        },
        Action = "sts:AssumeRole"
      }
    ]
  })

  inline_policy {
    name   = "codedeploy-policy"
    policy = jsonencode({
      Version   = "2012-10-17",
      Statement = [
        {
          Effect = "Allow",
          Action = [
            "ecs:UpdateService",
            "ecs:CreateDeployment",
            "ecs:DescribeServices",
            "ecs:DescribeTasks",
            "ecs:DescribeTaskDefinition",
            "elasticloadbalancing:DescribeTargetGroups",
            "elasticloadbalancing:DescribeListeners",
            "elasticloadbalancing:ModifyListener",
            "elasticloadbalancing:RegisterTargets",
            "elasticloadbalancing:DeregisterTargets",
            "ec2:DescribeInstances",
            "ec2:DescribeSecurityGroups",
            "ec2:DescribeSubnets",
            "ec2:DescribeNetworkInterfaces",
            "iam:PassRole",
            "s3:GetObject",
            "s3:GetObjectVersion"
          ],
          Resource = "*"
        }
      ]
    })
  }
}

resource "aws_iam_role_policy_attachment" "codedeploy_ecs_access" {
  role       = aws_iam_role.codedeploy.name
  policy_arn = "arn:aws:iam::aws:policy/AWSCodeDeployRoleForECS"
}
