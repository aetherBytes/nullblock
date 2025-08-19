# Production Environment Overrides
# These values override the defaults for the production environment

environment = "prod"

# Production-specific overrides
service_overrides = {
  # Production services typically run on different hosts
  postgresql = {
    host = "db.nullblock.com"
  }
  
  redis = {
    host = "cache.nullblock.com"
  }
  
  # Production services might use different ports
  frontend = {
    port = 80
  }
  
  # Example: Production might use load balancer hosts
  # mcp_server = {
  #   host = "api.nullblock.com"
  # }
}
