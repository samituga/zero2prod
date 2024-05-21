resource "aws_iam_policy" "assume_ssm_role_policy" {
  name        = "AssumeSSMSessionRolePolicy"
  description = "Policy to allow assuming the SSM session role"
  policy      = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Effect = "Allow",
        Action = "sts:AssumeRole",
        Resource = aws_iam_role.ssm_session_role.arn
      }
    ]
  })
}

resource "aws_iam_user_policy_attachment" "attach_assume_ssm_role_policy" {
  user       = var.aws_user_name
  policy_arn = aws_iam_policy.assume_ssm_role_policy.arn
}

resource "aws_iam_role" "ssm_session_role" {
  name = "SSMSessionRole"

  assume_role_policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Effect = "Allow",
        Principal = {
          AWS = "arn:aws:iam::${var.aws_account_id}:user/${var.aws_user_name}"
        },
        Action = "sts:AssumeRole"
      }
    ]
  })
}

resource "aws_iam_policy" "ssm_session_policy" {
  name        = "SSMSessionPolicy"
  description = "Policy to allow starting SSM sessions"
  policy      = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Effect = "Allow",
        Action = [
          "ssm:StartSession",
          "ssm:DescribeSessions",
          "ssm:GetConnectionStatus",
          "ssm:TerminateSession"
        ],
        Resource = "*"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "ssm_session_policy_attachment" {
  role       = aws_iam_role.ssm_session_role.name
  policy_arn = aws_iam_policy.ssm_session_policy.arn
}
