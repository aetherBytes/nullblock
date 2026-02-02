variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "domain_name" {
  description = "Primary domain"
  type        = string
  default     = "nullblock.io"
}

variable "admin_cidr" {
  description = "CIDR block for SSH access"
  type        = string
}

variable "db_password" {
  description = "Database password for production PostgreSQL"
  type        = string
  sensitive   = true
}
