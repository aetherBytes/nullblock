variable "vpc_id" {
  description = "VPC ID where EKS and ALB will be deployed"
  type        = string
}

variable "public_subnet_ids" {
  description = "List of public subnets for ALB"
  type        = list(string)
}

variable "aws_region" {
  description = "AWS region (must be us-east-1)"
  type        = string
  default     = "us-east-1"
}

variable "domain_name" {
  description = "Primary domain (e.g., nullblock.io)"
  type        = string
  default     = "nullblock.io"
}