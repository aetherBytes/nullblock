"""
Centralized configuration for Nullblock Agents

This module provides a single source of truth for all service configurations,
port assignments, and environment-specific settings.
"""

import os
from typing import Dict, Any, Optional
from dataclasses import dataclass

@dataclass
class ServiceConfig:
    """Configuration for a single service"""
    host: str
    port: int
    name: str
    description: str
    health_endpoint: Optional[str] = None
    docs_endpoint: Optional[str] = None

class NullblockConfig:
    """
    Centralized configuration manager for Nullblock services
    
    All configuration values can be overridden via environment variables.
    Default values are provided for development environments.
    """
    
    def __init__(self):
        # Infrastructure Services
        self.postgresql = ServiceConfig(
            host=os.getenv('POSTGRESQL_HOST', 'localhost'),
            port=int(os.getenv('POSTGRESQL_PORT', '5432')),
            name='PostgreSQL',
            description='Primary database'
        )
        
        self.redis = ServiceConfig(
            host=os.getenv('REDIS_HOST', 'localhost'),
            port=int(os.getenv('REDIS_PORT', '6379')),
            name='Redis',
            description='Cache and session store'
        )
        
        # Backend Services
        self.mcp_server = ServiceConfig(
            host=os.getenv('MCP_SERVER_HOST', '0.0.0.0'),
            port=int(os.getenv('MCP_SERVER_PORT', '8001')),
            name='MCP Server',
            description='Model Context Protocol server',
            health_endpoint='/health',
            docs_endpoint='/docs'
        )
        
        self.orchestration = ServiceConfig(
            host=os.getenv('ORCHESTRATION_HOST', '0.0.0.0'),
            port=int(os.getenv('ORCHESTRATION_PORT', '8002')),
            name='Orchestration',
            description='Workflow orchestration engine',
            health_endpoint='/health',
            docs_endpoint='/docs'
        )
        
        self.erebus = ServiceConfig(
            host=os.getenv('EREBUS_HOST', '127.0.0.1'),
            port=int(os.getenv('EREBUS_PORT', '3000')),
            name='Erebus',
            description='Rust backend server',
            health_endpoint='/health',
            docs_endpoint='/docs'
        )
        
        # Agent Services
        self.general_agents = ServiceConfig(
            host=os.getenv('AGENTS_HOST', '0.0.0.0'),
            port=int(os.getenv('AGENTS_PORT', '9001')),
            name='General Agents',
            description='Agent services and management',
            health_endpoint='/health',
            docs_endpoint='/docs'
        )
        
        self.hecate_agent = ServiceConfig(
            host=os.getenv('HECATE_HOST', '0.0.0.0'),
            port=int(os.getenv('HECATE_PORT', '9002')),
            name='Hecate Agent',
            description='Chat interface and orchestration',
            health_endpoint='/health',
            docs_endpoint='/docs'
        )
        
        # Frontend Services
        self.frontend = ServiceConfig(
            host=os.getenv('FRONTEND_HOST', 'localhost'),
            port=int(os.getenv('FRONTEND_PORT', '5173')),
            name='Frontend',
            description='Vite development server'
        )
        
        # LLM Services
        self.lm_studio = ServiceConfig(
            host=os.getenv('LM_STUDIO_HOST', 'localhost'),
            port=int(os.getenv('LM_STUDIO_PORT', '1234')),
            name='LM Studio',
            description='Local LLM server'
        )
        
        # IPFS Configuration
        self.ipfs = ServiceConfig(
            host=os.getenv('IPFS_HOST', '127.0.0.1'),
            port=int(os.getenv('IPFS_PORT', '5001')),
            name='IPFS Daemon',
            description='InterPlanetary File System'
        )
    
    def get_service_url(self, service: ServiceConfig, endpoint: str = '') -> str:
        """Generate a full URL for a service endpoint"""
        return f"http://{service.host}:{service.port}{endpoint}"
    
    def get_health_url(self, service: ServiceConfig) -> str:
        """Get the health check URL for a service"""
        if service.health_endpoint:
            return self.get_service_url(service, service.health_endpoint)
        return self.get_service_url(service)
    
    def get_docs_url(self, service: ServiceConfig) -> str:
        """Get the documentation URL for a service"""
        if service.docs_endpoint:
            return self.get_service_url(service, service.docs_endpoint)
        return self.get_service_url(service)
    
    def get_all_services(self) -> Dict[str, ServiceConfig]:
        """Get all configured services"""
        return {
            'postgresql': self.postgresql,
            'redis': self.redis,
            'mcp_server': self.mcp_server,
            'orchestration': self.orchestration,
            'erebus': self.erebus,
            'general_agents': self.general_agents,
            'hecate_agent': self.hecate_agent,
            'frontend': self.frontend,
            'lm_studio': self.lm_studio,
            'ipfs': self.ipfs
        }
    
    def get_service_by_port(self, port: int) -> Optional[ServiceConfig]:
        """Find a service by its port number"""
        for service in self.get_all_services().values():
            if service.port == port:
                return service
        return None
    
    def validate_ports(self) -> Dict[str, Any]:
        """Validate that all ports are unique"""
        ports = {}
        conflicts = []
        
        for name, service in self.get_all_services().items():
            if service.port in ports:
                conflicts.append({
                    'port': service.port,
                    'services': [ports[service.port], name]
                })
            else:
                ports[service.port] = name
        
        return {
            'valid': len(conflicts) == 0,
            'conflicts': conflicts,
            'port_assignments': ports
        }
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert configuration to dictionary for logging/debugging"""
        return {
            name: {
                'host': service.host,
                'port': service.port,
                'name': service.name,
                'description': service.description,
                'url': self.get_service_url(service),
                'health_url': self.get_health_url(service) if service.health_endpoint else None,
                'docs_url': self.get_docs_url(service) if service.docs_endpoint else None
            }
            for name, service in self.get_all_services().items()
        }
    
    def print_configuration(self):
        """Print the current configuration for debugging"""
        print("🔧 NULLBLOCK CONFIGURATION")
        print("=" * 50)
        
        validation = self.validate_ports()
        if not validation['valid']:
            print("❌ PORT CONFLICTS DETECTED:")
            for conflict in validation['conflicts']:
                print(f"   Port {conflict['port']}: {', '.join(conflict['services'])}")
            print()
        
        for name, service in self.get_all_services().items():
            print(f"📡 {service.name} ({name})")
            print(f"   Host: {service.host}")
            print(f"   Port: {service.port}")
            print(f"   URL: {self.get_service_url(service)}")
            if service.health_endpoint:
                print(f"   Health: {self.get_health_url(service)}")
            if service.docs_endpoint:
                print(f"   Docs: {self.get_docs_url(service)}")
            print()

# Global configuration instance
config = NullblockConfig()

# Convenience functions for common configurations
def get_hecate_config() -> ServiceConfig:
    """Get Hecate agent configuration"""
    return config.hecate_agent

def get_agents_config() -> ServiceConfig:
    """Get general agents configuration"""
    return config.general_agents

def get_mcp_config() -> ServiceConfig:
    """Get MCP server configuration"""
    return config.mcp_server

def get_frontend_config() -> ServiceConfig:
    """Get frontend configuration"""
    return config.frontend

def get_all_service_urls() -> Dict[str, str]:
    """Get all service URLs for frontend configuration"""
    return {
        'VITE_MCP_API_URL': config.get_service_url(config.mcp_server),
        'VITE_EREBUS_API_URL': config.get_service_url(config.erebus),
        'VITE_ORCHESTRATION_API_URL': config.get_service_url(config.orchestration),
        'VITE_AGENTS_API_URL': config.get_service_url(config.general_agents),
        'VITE_HECATE_API_URL': config.get_service_url(config.hecate_agent),
    }
