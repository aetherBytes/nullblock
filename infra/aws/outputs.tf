output "alb_dns_name" {
  description = "DNS name of the Application Load Balancer"
  value       = aws_elb.nullblock_alb.dns_name
}

output "static_ip" {
  description = "Elastic IP assigned to ALB"
  value       = aws_eip.nullblock_static_ip.public_ip
}

output "certificate_arn" {
  description = "ACM certificate ARN for TLS"
  value       = aws_acm_certificate.nullblock.arn
}

output "route53_zone_id" {
  description = "Route53 hosted zone ID for nullblock.io"
  value       = aws_route53_zone.nullblock.zone_id
}