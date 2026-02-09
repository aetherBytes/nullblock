output "alb_dns_name" {
  description = "DNS name of the Application Load Balancer"
  value       = aws_lb.main.dns_name
}

output "certificate_arn" {
  description = "ACM certificate ARN"
  value       = aws_acm_certificate.nullblock.arn
}

output "route53_zone_id" {
  description = "Route53 hosted zone ID"
  value       = aws_route53_zone.nullblock.zone_id
}

output "vpc_id" {
  description = "VPC ID"
  value       = aws_vpc.main.id
}

output "ecs_cluster_name" {
  description = "ECS cluster name"
  value       = aws_ecs_cluster.main.name
}

output "route53_nameservers" {
  description = "Route53 nameservers — set these in GoDaddy DNS"
  value       = aws_route53_zone.nullblock.name_servers
}

output "task_execution_role_arn" {
  description = "ECS task execution role ARN"
  value       = aws_iam_role.ecs_task_execution.arn
}
