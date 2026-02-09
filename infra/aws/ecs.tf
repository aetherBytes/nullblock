resource "aws_ecs_cluster" "main" {
  name = "DevCluster"

  setting {
    name  = "containerInsights"
    value = "enabled"
  }
}

resource "aws_iam_role" "ecs_instance" {
  name = "nullblock-ecs-instance-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "ec2.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "ecs_instance" {
  role       = aws_iam_role.ecs_instance.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonEC2ContainerServiceforEC2Role"
}

resource "aws_iam_role_policy_attachment" "ecs_instance_ssm" {
  role       = aws_iam_role.ecs_instance.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

resource "aws_iam_instance_profile" "ecs_instance" {
  name = "nullblock-ecs-instance-profile"
  role = aws_iam_role.ecs_instance.name
}

resource "aws_launch_template" "ecs" {
  name_prefix   = "nullblock-ecs-"
  image_id      = data.aws_ssm_parameter.ecs_ami.value
  instance_type = "t3.xlarge"

  iam_instance_profile {
    arn = aws_iam_instance_profile.ecs_instance.arn
  }

  network_interfaces {
    associate_public_ip_address = true
    security_groups             = [aws_security_group.ecs_instances.id]
  }

  user_data = base64encode(<<-EOF
    #!/bin/bash
    echo "ECS_CLUSTER=DevCluster" >> /etc/ecs/ecs.config
    echo "ECS_ENABLE_CONTAINER_METADATA=true" >> /etc/ecs/ecs.config

    yum install -y aws-cli jq || true

    # Install docker compose v2 plugin
    mkdir -p /usr/local/lib/docker/cli-plugins
    curl -SL https://github.com/docker/compose/releases/download/v2.24.5/docker-compose-linux-x86_64 \
      -o /usr/local/lib/docker/cli-plugins/docker-compose
    chmod +x /usr/local/lib/docker/cli-plugins/docker-compose

    mkdir -p /data/postgres-erebus /data/postgres-agents /data/redis

    DB_PASSWORD=$(aws secretsmanager get-secret-value \
      --secret-id nullblock/prod \
      --region ${var.aws_region} \
      --query 'SecretString' --output text | jq -r '.DB_PASSWORD')

    cat > /opt/docker-compose.prod.yml << 'COMPOSE'
${file("${path.module}/../../infra/docker-compose.prod.yml")}
COMPOSE

    cd /opt && DB_PASSWORD=$DB_PASSWORD docker compose -f docker-compose.prod.yml up -d
    EOF
  )

  tag_specifications {
    resource_type = "instance"
    tags = {
      Name = "nullblock-ecs-instance"
    }
  }
}

resource "aws_autoscaling_group" "ecs" {
  name                = "nullblock-ecs-asg"
  min_size            = 1
  max_size            = 1
  desired_capacity    = 1
  vpc_zone_identifier = [aws_subnet.public_a.id, aws_subnet.public_b.id]

  launch_template {
    id      = aws_launch_template.ecs.id
    version = "$Latest"
  }

  tag {
    key                 = "AmazonECSManaged"
    value               = "true"
    propagate_at_launch = true
  }
}

resource "aws_ecs_capacity_provider" "ec2" {
  name = "nullblock-ec2-provider"

  auto_scaling_group_provider {
    auto_scaling_group_arn = aws_autoscaling_group.ecs.arn

    managed_scaling {
      status          = "ENABLED"
      target_capacity = 100
    }
  }
}

resource "aws_ecs_cluster_capacity_providers" "main" {
  cluster_name       = aws_ecs_cluster.main.name
  capacity_providers = [aws_ecs_capacity_provider.ec2.name]

  default_capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.ec2.name
    weight            = 1
  }
}

resource "aws_iam_role_policy" "ecs_instance_secrets" {
  name = "nullblock-ecs-instance-secrets"
  role = aws_iam_role.ecs_instance.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = [
        "secretsmanager:GetSecretValue"
      ]
      Resource = [
        data.aws_secretsmanager_secret.prod.arn
      ]
    }]
  })
}

resource "aws_cloudwatch_log_group" "ecs" {
  for_each = toset([
    "/ecs/nullblock/erebus",
    "/ecs/nullblock/agents",
    "/ecs/nullblock/engrams",
    "/ecs/nullblock/protocols",
    "/ecs/nullblock/arb-farm",
    "/ecs/nullblock/hecate",
  ])

  name              = each.key
  retention_in_days = 30
}
