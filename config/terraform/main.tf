terraform {
  required_version = ">= 1.0"
  required_providers {
    local = {
      source  = "hashicorp/local"
      version = "~> 2.4"
    }
  }
}

# Local variables for service configuration
locals {
  # Environment configuration
  environment = var.environment
  
  # Service port assignments
  service_ports = {
    # Infrastructure Services
    postgresql = 5432
    redis      = 6379
    ipfs       = 5001
    
    # Backend Services
    mcp_server     = 8001
    orchestration  = 8002
    erebus         = 3000
    
    # Agent Services
    general_agents = 9001
    hecate_agent   = 9002
    
    # Frontend Services
    frontend = 5173
    
    # LLM Services
    lm_studio = 1234
  }
  
  # Service hosts (can be overridden per environment)
  service_hosts = {
    postgresql     = "localhost"
    redis          = "localhost"
    ipfs           = "127.0.0.1"
    mcp_server     = "0.0.0.0"
    orchestration  = "0.0.0.0"
    erebus         = "127.0.0.1"
    general_agents = "0.0.0.0"
    hecate_agent   = "0.0.0.0"
    frontend       = "localhost"
    lm_studio      = "localhost"
  }
  
  # Service descriptions
  service_descriptions = {
    postgresql     = "Primary database"
    redis          = "Cache and session store"
    ipfs           = "InterPlanetary File System"
    mcp_server     = "Model Context Protocol server"
    orchestration  = "Workflow orchestration engine"
    erebus         = "Rust backend server"
    general_agents = "Agent services and management"
    hecate_agent   = "Chat interface and orchestration"
    frontend       = "Vite development server"
    lm_studio      = "Local LLM server"
  }
}

# Variables
variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "dev"
}

variable "service_overrides" {
  description = "Override default service configurations"
  type = map(object({
    host = optional(string)
    port = optional(number)
  }))
  default = {}
}

# Generate environment-specific configuration files
resource "local_file" "env_file" {
  filename = "${path.module}/../generated/.env.${local.environment}"
  content  = templatefile("${path.module}/templates/env.tftpl", {
    environment = local.environment
    services    = local.service_ports
    hosts       = local.service_hosts
    overrides   = var.service_overrides
  })
}

# Generate Python configuration
resource "local_file" "python_config" {
  filename = "${path.module}/../generated/config.py"
  content  = templatefile("${path.module}/templates/python_config.tftpl", {
    environment = local.environment
    services    = local.service_ports
    hosts       = local.service_hosts
    descriptions = local.service_descriptions
    overrides   = var.service_overrides
  })
}

# Generate JavaScript/TypeScript configuration
resource "local_file" "typescript_config" {
  filename = "${path.module}/../generated/config.ts"
  content  = templatefile("${path.module}/templates/typescript_config.tftpl", {
    environment = local.environment
    services    = local.service_ports
    hosts       = local.service_hosts
    descriptions = local.service_descriptions
    overrides   = var.service_overrides
  })
}

# Generate simple health monitoring script
resource "local_file" "health_monitor" {
  filename = "${path.module}/../generated/monitor-health.sh"
  content  = templatefile("${path.module}/templates/simple_health_monitor.tftpl", {
    environment = local.environment
    services    = local.service_ports
    hosts       = local.service_hosts
    descriptions = local.service_descriptions
    overrides   = var.service_overrides
  })
}

# Outputs
output "service_urls" {
  description = "All service URLs"
  value = {
    for service, port in local.service_ports : service => "http://${local.service_hosts[service]}:${port}"
  }
}

output "configuration_files" {
  description = "Generated configuration files"
  value = [
    local_file.env_file.filename,
    local_file.python_config.filename,
    local_file.typescript_config.filename,
    local_file.health_monitor.filename
  ]
}
