variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-west-2"
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

