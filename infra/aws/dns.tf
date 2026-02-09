resource "aws_route53_zone" "nullblock" {
  name = var.domain_name
}

resource "aws_route53_record" "root" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = var.domain_name
  type    = "A"

  alias {
    name                   = aws_lb.main.dns_name
    zone_id                = aws_lb.main.zone_id
    evaluate_target_health = true
  }
}

resource "aws_route53_record" "api" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "api.${var.domain_name}"
  type    = "A"

  alias {
    name                   = aws_lb.main.dns_name
    zone_id                = aws_lb.main.zone_id
    evaluate_target_health = true
  }
}

resource "aws_route53_record" "web" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "web.${var.domain_name}"
  type    = "A"

  alias {
    name                   = aws_lb.main.dns_name
    zone_id                = aws_lb.main.zone_id
    evaluate_target_health = true
  }
}

resource "aws_route53_record" "events" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "events.${var.domain_name}"
  type    = "A"

  alias {
    name                   = aws_lb.main.dns_name
    zone_id                = aws_lb.main.zone_id
    evaluate_target_health = true
  }
}

resource "aws_route53_record" "autodiscover" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "autodiscover.${var.domain_name}"
  type    = "CNAME"
  ttl     = 3600
  records = ["autodiscover.outlook.com"]
}

resource "aws_route53_record" "lyncdiscover" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "lyncdiscover.${var.domain_name}"
  type    = "CNAME"
  ttl     = 3600
  records = ["webdir.online.lync.com"]
}

resource "aws_route53_record" "msoid" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "msoid.${var.domain_name}"
  type    = "CNAME"
  ttl     = 3600
  records = ["clientconfig.microsoftonline-p.net"]
}

resource "aws_route53_record" "sip" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "sip.${var.domain_name}"
  type    = "CNAME"
  ttl     = 3600
  records = ["sipdir.online.lync.com"]
}

resource "aws_route53_record" "www" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "www.${var.domain_name}"
  type    = "CNAME"
  ttl     = 3600
  records = ["nullblock.io"]
}

resource "aws_route53_record" "email" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "email.${var.domain_name}"
  type    = "CNAME"
  ttl     = 3600
  records = ["email.secureserver.net"]
}

resource "aws_route53_record" "mx" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = var.domain_name
  type    = "MX"
  ttl     = 3600
  records = ["0 nullblock-io.mail.protection.outlook.com"]
}

resource "aws_route53_record" "txt_spf" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = var.domain_name
  type    = "TXT"
  ttl     = 3600
  records = [
    "NETORGFT15123362.onmicrosoft.com",
    "v=spf1 include:secureserver.net -all",
  ]
}

resource "aws_route53_record" "srv_sip_tls" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "_sip._tls.${var.domain_name}"
  type    = "SRV"
  ttl     = 3600
  records = ["100 1 443 sipdir.online.lync.com"]
}

resource "aws_route53_record" "srv_sipfederation" {
  zone_id = aws_route53_zone.nullblock.zone_id
  name    = "_sipfederationtls._tcp.${var.domain_name}"
  type    = "SRV"
  ttl     = 3600
  records = ["100 1 5061 sipfed.online.lync.com"]
}
