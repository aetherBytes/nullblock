resource "aws_ecs_task_definition" "erebus" {
  family                = "erebus"
  network_mode          = "bridge"
  requires_compatibilities = ["EC2"]
  execution_role_arn    = aws_iam_role.ecs_task_execution.arn
  task_role_arn         = aws_iam_role.ecs_task.arn
  cpu                   = "256"
  memory                = "512"

  container_definitions = jsonencode([{
    name      = "erebus"
    image     = "ghcr.io/aetherbytes/nullblock/erebus:latest"
    essential = true
    portMappings = [{ containerPort = 3000, hostPort = 3000, protocol = "tcp" }]
    repositoryCredentials = {
      credentialsParameter = data.aws_secretsmanager_secret.ghcr_token.arn
    }
    healthCheck = {
      command     = ["CMD-SHELL", "curl -f http://localhost:3000/health || exit 1"]
      interval    = 30
      timeout     = 5
      retries     = 3
      startPeriod = 60
    }
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = "/ecs/nullblock/erebus"
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "ecs"
      }
    }
    environment = [
      { name = "RUST_LOG", value = "info" },
      { name = "HOST", value = "0.0.0.0" },
      { name = "PORT", value = "3000" },
      { name = "AGENTS_SERVICE_URL", value = "http://172.17.0.1:9003" },
      { name = "ENGRAMS_SERVICE_URL", value = "http://172.17.0.1:9004" },
      { name = "PROTOCOLS_SERVICE_URL", value = "http://172.17.0.1:8001" },
      { name = "ARBFARM_SERVICE_URL", value = "http://172.17.0.1:9007" },
    ]
    secrets = [
      { name = "DATABASE_URL", valueFrom = "${data.aws_secretsmanager_secret.prod.arn}:DB_URL_EREBUS::" },
      { name = "ENCRYPTION_MASTER_KEY", valueFrom = "${data.aws_secretsmanager_secret.prod.arn}:ENCRYPTION_MASTER_KEY::" },
    ]
  }])
}

resource "aws_ecs_service" "erebus" {
  name            = "erebus"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.erebus.arn
  desired_count   = 1

  capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.ec2.name
    weight            = 1
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.erebus.arn
    container_name   = "erebus"
    container_port   = 3000
  }

  depends_on = [aws_lb_listener.https]

  lifecycle {
    ignore_changes = [task_definition]
  }
}

resource "aws_ecs_task_definition" "agents" {
  family                = "nullblock-agents"
  network_mode          = "bridge"
  requires_compatibilities = ["EC2"]
  execution_role_arn    = aws_iam_role.ecs_task_execution.arn
  task_role_arn         = aws_iam_role.ecs_task.arn
  cpu                   = "256"
  memory                = "512"

  container_definitions = jsonencode([{
    name      = "nullblock-agents"
    image     = "ghcr.io/aetherbytes/nullblock/nullblock-agents:latest"
    essential = true
    portMappings = [{ containerPort = 9003, hostPort = 9003, protocol = "tcp" }]
    repositoryCredentials = {
      credentialsParameter = data.aws_secretsmanager_secret.ghcr_token.arn
    }
    healthCheck = {
      command     = ["CMD-SHELL", "curl -f http://localhost:9003/health || exit 1"]
      interval    = 30
      timeout     = 5
      retries     = 3
      startPeriod = 60
    }
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = "/ecs/nullblock/agents"
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "ecs"
      }
    }
    environment = [
      { name = "RUST_LOG", value = "info" },
      { name = "HOST", value = "0.0.0.0" },
      { name = "PORT", value = "9003" },
      { name = "ENGRAMS_SERVICE_URL", value = "http://172.17.0.1:9004" },
    ]
    secrets = [
      { name = "DATABASE_URL", valueFrom = "${data.aws_secretsmanager_secret.prod.arn}:DB_URL_AGENTS::" },
      { name = "OPENROUTER_KEY_HECATE", valueFrom = "${data.aws_secretsmanager_secret.prod.arn}:OPENROUTER_KEY_HECATE::" },
      { name = "OPENROUTER_KEY_MOROS", valueFrom = "${data.aws_secretsmanager_secret.prod.arn}:OPENROUTER_KEY_MOROS::" },
      { name = "OPENROUTER_KEY_CLAWROS", valueFrom = "${data.aws_secretsmanager_secret.prod.arn}:OPENROUTER_KEY_CLAWROS::" },
      { name = "ENCRYPTION_MASTER_KEY", valueFrom = "${data.aws_secretsmanager_secret.prod.arn}:ENCRYPTION_MASTER_KEY::" },
    ]
  }])
}

resource "aws_ecs_service" "agents" {
  name            = "nullblock-agents"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.agents.arn
  desired_count   = 1

  capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.ec2.name
    weight            = 1
  }

  lifecycle {
    ignore_changes = [task_definition]
  }
}

resource "aws_ecs_task_definition" "engrams" {
  family                = "nullblock-engrams"
  network_mode          = "bridge"
  requires_compatibilities = ["EC2"]
  execution_role_arn    = aws_iam_role.ecs_task_execution.arn
  task_role_arn         = aws_iam_role.ecs_task.arn
  cpu                   = "128"
  memory                = "256"

  container_definitions = jsonencode([{
    name      = "nullblock-engrams"
    image     = "ghcr.io/aetherbytes/nullblock/nullblock-engrams:latest"
    essential = true
    portMappings = [{ containerPort = 9004, hostPort = 9004, protocol = "tcp" }]
    repositoryCredentials = {
      credentialsParameter = data.aws_secretsmanager_secret.ghcr_token.arn
    }
    healthCheck = {
      command     = ["CMD-SHELL", "curl -f http://localhost:9004/health || exit 1"]
      interval    = 30
      timeout     = 5
      retries     = 3
      startPeriod = 60
    }
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = "/ecs/nullblock/engrams"
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "ecs"
      }
    }
    environment = [
      { name = "RUST_LOG", value = "info" },
      { name = "HOST", value = "0.0.0.0" },
      { name = "PORT", value = "9004" },
    ]
    secrets = [
      { name = "DATABASE_URL", valueFrom = "${data.aws_secretsmanager_secret.prod.arn}:DB_URL_EREBUS::" },
    ]
  }])
}

resource "aws_ecs_service" "engrams" {
  name            = "nullblock-engrams"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.engrams.arn
  desired_count   = 1

  capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.ec2.name
    weight            = 1
  }

  lifecycle {
    ignore_changes = [task_definition]
  }
}

resource "aws_ecs_task_definition" "protocols" {
  family                = "nullblock-protocols"
  network_mode          = "bridge"
  requires_compatibilities = ["EC2"]
  execution_role_arn    = aws_iam_role.ecs_task_execution.arn
  task_role_arn         = aws_iam_role.ecs_task.arn
  cpu                   = "128"
  memory                = "256"

  container_definitions = jsonencode([{
    name      = "nullblock-protocols"
    image     = "ghcr.io/aetherbytes/nullblock/nullblock-protocols:latest"
    essential = true
    portMappings = [{ containerPort = 8001, hostPort = 8001, protocol = "tcp" }]
    repositoryCredentials = {
      credentialsParameter = data.aws_secretsmanager_secret.ghcr_token.arn
    }
    healthCheck = {
      command     = ["CMD-SHELL", "curl -f http://localhost:8001/health || exit 1"]
      interval    = 30
      timeout     = 5
      retries     = 3
      startPeriod = 60
    }
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = "/ecs/nullblock/protocols"
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "ecs"
      }
    }
    environment = [
      { name = "RUST_LOG", value = "info" },
      { name = "HOST", value = "0.0.0.0" },
      { name = "PORT", value = "8001" },
      { name = "AGENTS_SERVICE_URL", value = "http://172.17.0.1:9003" },
      { name = "ENGRAMS_SERVICE_URL", value = "http://172.17.0.1:9004" },
      { name = "ARBFARM_SERVICE_URL", value = "http://172.17.0.1:9007" },
    ]
    secrets = []
  }])
}

resource "aws_ecs_service" "protocols" {
  name            = "nullblock-protocols"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.protocols.arn
  desired_count   = 1

  capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.ec2.name
    weight            = 1
  }

  lifecycle {
    ignore_changes = [task_definition]
  }
}

resource "aws_ecs_task_definition" "arb_farm" {
  family                = "arb-farm"
  network_mode          = "bridge"
  requires_compatibilities = ["EC2"]
  execution_role_arn    = aws_iam_role.ecs_task_execution.arn
  task_role_arn         = aws_iam_role.ecs_task.arn
  cpu                   = "512"
  memory                = "1024"

  container_definitions = jsonencode([{
    name      = "arb-farm"
    image     = "ghcr.io/aetherbytes/nullblock/arb-farm:latest"
    essential = true
    portMappings = [{ containerPort = 9007, hostPort = 9007, protocol = "tcp" }]
    repositoryCredentials = {
      credentialsParameter = data.aws_secretsmanager_secret.ghcr_token.arn
    }
    healthCheck = {
      command     = ["CMD-SHELL", "curl -f http://localhost:9007/health || exit 1"]
      interval    = 30
      timeout     = 5
      retries     = 3
      startPeriod = 60
    }
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = "/ecs/nullblock/arb-farm"
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "ecs"
      }
    }
    environment = [
      { name = "RUST_LOG", value = "info" },
      { name = "HOST", value = "0.0.0.0" },
      { name = "PORT", value = "9007" },
      { name = "ENGRAMS_SERVICE_URL", value = "http://172.17.0.1:9004" },
    ]
    secrets = [
      { name = "DATABASE_URL", valueFrom = "${data.aws_secretsmanager_secret.prod.arn}:DB_URL_AGENTS::" },
      { name = "OPENROUTER_KEY_HECATE", valueFrom = "${data.aws_secretsmanager_secret.prod.arn}:OPENROUTER_KEY_HECATE::" },
      { name = "ENCRYPTION_MASTER_KEY", valueFrom = "${data.aws_secretsmanager_secret.prod.arn}:ENCRYPTION_MASTER_KEY::" },
    ]
  }])
}

resource "aws_ecs_service" "arb_farm" {
  name            = "arb-farm"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.arb_farm.arn
  desired_count   = 1

  capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.ec2.name
    weight            = 1
  }

  lifecycle {
    ignore_changes = [task_definition]
  }
}

resource "aws_ecs_task_definition" "hecate" {
  family                = "hecate"
  network_mode          = "bridge"
  requires_compatibilities = ["EC2"]
  execution_role_arn    = aws_iam_role.ecs_task_execution.arn
  task_role_arn         = aws_iam_role.ecs_task.arn
  cpu                   = "256"
  memory                = "512"

  container_definitions = jsonencode([{
    name      = "hecate"
    image     = "ghcr.io/aetherbytes/nullblock/hecate:latest"
    essential = true
    portMappings = [{ containerPort = 3000, hostPort = 5173, protocol = "tcp" }]
    repositoryCredentials = {
      credentialsParameter = data.aws_secretsmanager_secret.ghcr_token.arn
    }
    healthCheck = {
      command     = ["CMD-SHELL", "curl -f http://localhost:3000/ || exit 1"]
      interval    = 30
      timeout     = 5
      retries     = 3
      startPeriod = 60
    }
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = "/ecs/nullblock/hecate"
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "ecs"
      }
    }
    environment = [
      { name = "VITE_API_URL", value = "https://nullblock.io" },
      { name = "NODE_ENV", value = "production" },
      { name = "PORT", value = "5173" },
    ]
    secrets = []
  }])
}

resource "aws_ecs_service" "hecate" {
  name            = "hecate"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.hecate.arn
  desired_count   = 1

  capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.ec2.name
    weight            = 1
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.hecate.arn
    container_name   = "hecate"
    container_port   = 3000
  }

  depends_on = [aws_lb_listener.https]

  lifecycle {
    ignore_changes = [task_definition]
  }
}
