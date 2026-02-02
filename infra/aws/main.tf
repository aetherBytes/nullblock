provider "aws" {
  region = "us-east-1"
}

data "aws_caller_identity" "current" {}

resource "aws_route53_zone" "nullblock" {
  name = "nullblock.io"
}

resource "aws_acm_certificate" "nullblock" {
  domain_name       = "nullblock.io"
  subject_alternative_names = ["*.nullblock.io"]
  validation_method = "DNS"
}

resource "aws_eip" "nullblock_static_ip" {
  domain = "vpc"
}

resource "aws_elb" "nullblock_alb" {
  name               = "nullblock-prod-alb"
  security_groups    = [aws_security_group.alb.id]
  subnets            = ["subnet-0a1b2c3d", "subnet-0e4f5a6b"] # <-- Replace with your public subnets
  idle_timeout       = 60
  connection_draining = true

  listener {
    load_balancer_port = 443
    load_balancer_protocol = "https"
    instance_port      = 3000
    instance_protocol  = "http"
    ssl_certificate_id = aws_acm_certificate.nullblock.arn
  }
}

resource "aws_security_group" "alb" {
  name        = "nullblock-alb-sg"
  description = "Allow HTTPS to ALB"
  vpc_id      = "vpc-0f1e2d3c" # <-- Replace with your VPC ID

  ingress {
    description = "HTTPS"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_route53_record" "nullblock_root" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "nullblock.io"
  type    = "A"
  ttl     = "300"
  records = [aws_eip.nullblock_static_ip.public_ip]
}

resource "aws_route53_record" "api_subdomain" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "api.nullblock.io"
  type    = "A"
  ttl     = "300"
  records = [aws_eip.nullblock_static_ip.public_ip]
}

resource "aws_route53_record" "web_subdomain" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "web.nullblock.io"
  type    = "A"
  ttl     = "300"
  records = [aws_eip.nullblock_static_ip.public_ip]
}

resource "aws_route53_record" "events_subdomain" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "events.nullblock.io"
  type    = "A"
  ttl     = "300"
  records = [aws_eip.nullblock_static_ip.public_ip]
}