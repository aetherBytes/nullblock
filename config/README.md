# Nullblock Configuration Management

This directory contains a **language-agnostic configuration management system** for Nullblock services using Terraform. This system eliminates hardcoded port references and provides centralized configuration management across all environments.

## 🎯 Overview

The configuration system provides:
- **Single source of truth** for all service ports and hosts
- **Environment-specific configurations** (dev, staging, prod)
- **Language-agnostic** - works with Python, TypeScript, JavaScript, and any other language
- **Automated generation** of configuration files
- **Port conflict validation** and prevention
- **Easy environment switching** and overrides

## 📁 Structure

```
config/
├── README.md                    # This file
├── generate-config.sh           # Main configuration generator script
├── terraform/
│   ├── main.tf                  # Main Terraform configuration
│   ├── dev.tfvars               # Development environment overrides
│   ├── staging.tfvars           # Staging environment overrides
│   ├── prod.tfvars              # Production environment overrides
│   └── templates/
│       ├── env.tftpl            # Environment file template
│       ├── python_config.tftpl  # Python configuration template
│       ├── typescript_config.tftpl # TypeScript configuration template
│       └── simple_health_monitor.tftpl # Health monitoring script template
└── generated/                   # Generated configuration files
    ├── .env.dev                 # Environment variables for dev
    ├── config.py                # Python configuration
    ├── config.ts                # TypeScript configuration
    └── monitor-health.sh        # Health monitoring script
```

## 🚀 Quick Start

### Prerequisites

1. **Install Terraform** (if not already installed):
   ```bash
   brew install terraform
   ```

2. **Navigate to the config directory**:
   ```bash
   cd config
   ```

### Generate Configuration

#### For Development Environment
```bash
./generate-config.sh dev
```

#### For All Environments
```bash
./generate-config.sh all
```

#### For Specific Environment with Overrides
```bash
./generate-config.sh prod custom-overrides.tfvars
```

## 📋 Generated Files

The system generates the following files:

### 1. Environment File (`.env.dev`)
Contains all environment variables for the specified environment:
```bash
# Infrastructure Services
POSTGRESQL_HOST=localhost
POSTGRESQL_PORT=5432
REDIS_HOST=localhost
REDIS_PORT=6379

# Backend Services
MCP_SERVER_HOST=0.0.0.0
MCP_SERVER_PORT=8001
ORCHESTRATION_HOST=0.0.0.0
ORCHESTRATION_PORT=8002

# Agent Services
AGENTS_HOST=0.0.0.0
AGENTS_PORT=9001
HECATE_HOST=0.0.0.0
HECATE_PORT=9002

# Frontend API URLs
VITE_MCP_API_URL=http://0.0.0.0:8001
VITE_HECATE_API_URL=http://0.0.0.0:9002
```

### 2. Python Configuration (`config.py`)
Generated Python configuration with type hints:
```python
from ..config import get_hecate_config, config

# Use the generated configuration
hecate_config = get_hecate_config()
print(f"Hecate running on: {hecate_config.host}:{hecate_config.port}")
```

### 3. TypeScript Configuration (`config.ts`)
Generated TypeScript configuration with interfaces:
```typescript
import { getHecateConfig, config } from './generated/config';

// Use the generated configuration
const hecateConfig = getHecateConfig();
console.log(`Hecate running on: ${hecateConfig.host}:${hecateConfig.port}`);
```

### 4. Health Monitoring Script (`monitor-health.sh`)
Automated health monitoring with status tables:
```bash
# Single health check
./generated/monitor-health.sh once

# Continuous monitoring
./generated/monitor-health.sh
```

## 🔧 Service Port Assignments

| Service | Port | Description | Environment |
|---------|------|-------------|-------------|
| **PostgreSQL** | 5432 | Primary database | All |
| **Redis** | 6379 | Cache and session store | All |
| **IPFS** | 5001 | InterPlanetary File System | All |
| **MCP Server** | 8001 | Model Context Protocol server | All |
| **Orchestration** | 8002 | Workflow orchestration engine | All |
| **Erebus** | 3000 | Rust backend server | All |
| **General Agents** | 9001 | Agent services and management | All |
| **Hecate Agent** | 9002 | Chat interface and orchestration | All |
| **Frontend** | 5173 | Vite development server | Dev |
| **LM Studio** | 1234 | Local LLM server | All |

## 🌍 Environment Management

### Development Environment
- Uses localhost for infrastructure services
- Agent services bind to `0.0.0.0` for external access
- Frontend runs on port 5173 (Vite default)

### Production Environment
- Uses external hostnames for infrastructure
- Frontend runs on port 80 (standard HTTP)
- All services configured for production deployment

### Environment Overrides
Create custom override files to modify specific services:

```hcl
# custom-overrides.tfvars
service_overrides = {
  frontend = {
    port = 3000  # Override frontend port
  }
  postgresql = {
    host = "db.example.com"  # Override database host
  }
}
```

## 🔄 Integration with Existing Code

### Update Python Services
Replace hardcoded configurations with generated ones:

```python
# Before (hardcoded)
HECATE_PORT = 8001  # ❌ Hardcoded

# After (generated)
from ..config import get_hecate_config
hecate_config = get_hecate_config()
HECATE_PORT = hecate_config.port  # ✅ Generated
```

### Update TypeScript/JavaScript Services
```typescript
// Before (hardcoded)
const HECATE_PORT = 8001;  // ❌ Hardcoded

// After (generated)
import { getHecateConfig } from './generated/config';
const hecateConfig = getHecateConfig();
const HECATE_PORT = hecateConfig.port;  // ✅ Generated
```

### Update tmuxinator Configuration
Copy the generated environment file to your tmuxinator configuration:
```bash
cp config/generated/.env.dev ~/.env.nullblock-dev
```

## 🛠️ Advanced Usage

### Custom Service Configuration
Add new services to `config/terraform/main.tf`:

```hcl
locals {
  service_ports = {
    # ... existing services ...
    new_service = 8080  # Add new service
  }
  
  service_hosts = {
    # ... existing services ...
    new_service = "0.0.0.0"  # Add new service
  }
  
  service_descriptions = {
    # ... existing services ...
    new_service = "New service description"  # Add new service
  }
}
```

### Environment-Specific Overrides
Modify `config/terraform/prod.tfvars` for production:

```hcl
service_overrides = {
  postgresql = {
    host = "db.nullblock.com"
  }
  redis = {
    host = "cache.nullblock.com"
  }
  frontend = {
    port = 80
  }
}
```

### Validation and Testing
```bash
# Validate Terraform configuration
./generate-config.sh validate

# Clean generated files
./generate-config.sh clean

# Generate and test configuration
./generate-config.sh dev
./generated/monitor-health.sh once
```

## 🔍 Troubleshooting

### Port Conflicts
If you encounter port conflicts:
1. Check what's using the port: `lsof -i :PORT`
2. Kill conflicting processes: `kill -9 PID`
3. Regenerate configuration: `./generate-config.sh dev`

### Terraform Errors
If Terraform fails:
1. Validate configuration: `./generate-config.sh validate`
2. Check template syntax in `templates/` directory
3. Ensure all required variables are defined

### Service Not Starting
If services fail to start:
1. Check generated configuration: `cat generated/.env.dev`
2. Verify port assignments are correct
3. Check service logs for specific errors

## 📚 Best Practices

1. **Always use generated configurations** - never hardcode ports or hosts
2. **Version control override files** - keep environment-specific changes in `.tfvars` files
3. **Test configurations** - use the health monitor to verify all services
4. **Document customizations** - update this README when adding new services
5. **Use environment variables** - load generated `.env` files in your applications

## 🤝 Contributing

When adding new services or modifying configurations:

1. Update `config/terraform/main.tf` with new service definitions
2. Add corresponding templates in `config/terraform/templates/`
3. Update this README with new service information
4. Test configuration generation for all environments
5. Update existing services to use generated configurations

## 📞 Support

For issues with the configuration system:
1. Check the troubleshooting section above
2. Review generated configuration files
3. Validate Terraform configuration
4. Check service logs for specific errors

---

**🎉 Congratulations!** You now have a robust, language-agnostic configuration management system that eliminates hardcoded references and provides centralized control over all Nullblock services.
