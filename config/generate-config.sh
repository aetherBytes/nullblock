#!/bin/bash

# Nullblock Configuration Generator
# This script uses Terraform to generate all configuration files from a centralized config

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🔧 Nullblock Configuration Generator"
echo "=================================="

# Check if Terraform is installed
if ! command -v terraform &> /dev/null; then
    echo "❌ Terraform is not installed. Please install Terraform first."
    echo "   Visit: https://www.terraform.io/downloads.html"
    exit 1
fi

# Create generated directory
mkdir -p "$SCRIPT_DIR/generated"

# Function to generate config for a specific environment
generate_config() {
    local environment="$1"
    local overrides_file="$2"
    
    echo "📋 Generating configuration for environment: $environment"
    
    # Build terraform command
    local tf_cmd="terraform -chdir=$SCRIPT_DIR/terraform apply -auto-approve"
    
    # Add environment variable
    tf_cmd="$tf_cmd -var='environment=$environment'"
    
    # Add overrides if provided
    if [ -n "$overrides_file" ] && [ -f "$overrides_file" ]; then
        echo "📝 Using overrides from: $overrides_file"
        tf_cmd="$tf_cmd -var-file='$overrides_file'"
    fi
    
    # Run terraform
    echo "🚀 Running: $tf_cmd"
    eval $tf_cmd
    
    echo "✅ Configuration generated for $environment"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  generate [ENVIRONMENT] [OVERRIDES_FILE]  Generate configuration for environment"
    echo "  dev                                     Generate development configuration"
    echo "  staging                                 Generate staging configuration"
    echo "  prod                                    Generate production configuration"
    echo "  all                                     Generate all environment configurations"
    echo "  clean                                   Clean generated files"
    echo "  validate                                Validate Terraform configuration"
    echo "  help                                    Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 dev                                  # Generate dev config"
    echo "  $0 prod overrides.tfvars               # Generate prod config with overrides"
    echo "  $0 all                                 # Generate all environments"
    echo ""
    echo "Environment-specific override files:"
    echo "  - dev.tfvars                           # Development overrides"
    echo "  - staging.tfvars                       # Staging overrides"
    echo "  - prod.tfvars                          # Production overrides"
}

# Function to clean generated files
clean_generated() {
    echo "🧹 Cleaning generated files..."
    rm -rf "$SCRIPT_DIR/generated"
    echo "✅ Generated files cleaned"
}

# Function to validate Terraform configuration
validate_config() {
    echo "🔍 Validating Terraform configuration..."
    terraform -chdir="$SCRIPT_DIR/terraform" init
    terraform -chdir="$SCRIPT_DIR/terraform" validate
    echo "✅ Configuration is valid"
}

# Main script logic
case "${1:-help}" in
    "generate")
        environment="${2:-dev}"
        overrides_file="$3"
        generate_config "$environment" "$overrides_file"
        ;;
    "dev")
        generate_config "dev" "$SCRIPT_DIR/terraform/dev.tfvars"
        ;;
    "staging")
        generate_config "staging" "$SCRIPT_DIR/terraform/staging.tfvars"
        ;;
    "prod")
        generate_config "prod" "$SCRIPT_DIR/terraform/prod.tfvars"
        ;;
    "all")
        echo "🔄 Generating all environment configurations..."
        generate_config "dev" "$SCRIPT_DIR/terraform/dev.tfvars"
        generate_config "staging" "$SCRIPT_DIR/terraform/staging.tfvars"
        generate_config "prod" "$SCRIPT_DIR/terraform/prod.tfvars"
        echo "✅ All configurations generated"
        ;;
    "clean")
        clean_generated
        ;;
    "validate")
        validate_config
        ;;
    "help"|*)
        show_usage
        ;;
esac

echo ""
echo "📁 Generated files location: $SCRIPT_DIR/generated"
echo "🔗 To use generated configs:"
echo "   - Copy .env files to your service directories"
echo "   - Import generated Python/TypeScript configs into your code"
echo "   - Use generated tmuxinator config for development"
