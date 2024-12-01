data "aws_availability_zones" "available" {
  filter {
    name   = "group-name"
    values = [var.region]
  }
}

locals {
  az_names = sort(data.aws_availability_zones.available.names)
}

resource "aws_vpc" "this" {
  cidr_block = "10.1.0.0/16"
}

resource "aws_subnet" "public" {
  count                   = 2
  cidr_block              = cidrsubnet(aws_vpc.this.cidr_block, 8, count.index)
  vpc_id                  = aws_vpc.this.id
  map_public_ip_on_launch = true
  availability_zone       = count.index % 2 == 0 ? local.az_names[0] : local.az_names[1]
  tags = merge({
    Name = "mwaa-${var.environment_name}-public-subnet-${count.index}"
  }, )
}

resource "aws_subnet" "private" {
  count                   = 2
  cidr_block              = cidrsubnet(aws_vpc.this.cidr_block, 8, count.index + 2)
  vpc_id                  = aws_vpc.this.id
  map_public_ip_on_launch = false
  availability_zone       = count.index % 2 == 0 ? local.az_names[0] : local.az_names[1]
  tags = merge({
    Name = "mwaa-${var.environment_name}-private-subnet-${count.index}"
  }, )
}

resource "aws_eip" "this" {
  count  = length(aws_subnet.public)
  domain = "vpc"
  tags = merge({
    Name = "mwaa-${var.environment_name}-eip-${count.index}"
  }, )
}

resource "aws_nat_gateway" "this" {
  count         = length(aws_subnet.public)
  allocation_id = aws_eip.this[count.index].id
  subnet_id     = aws_subnet.public[count.index].id
  tags = merge({
    Name = "mwaa-${var.environment_name}-nat-gateway-${count.index}"
  }, )
}

resource "aws_internet_gateway" "this" {
  vpc_id = aws_vpc.this.id
  tags = merge({
    Name = "mwaa-${var.environment_name}-internet-gateway"
  })
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.this.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.this.id
  }
  tags = merge({
    Name = "mwaa-${var.environment_name}-public-routes"
  })
}

resource "aws_route_table_association" "public" {
  count          = length(aws_subnet.public)
  route_table_id = aws_route_table.public.id
  subnet_id      = aws_subnet.public[count.index].id
}

resource "aws_route_table" "private" {
  count  = length(aws_nat_gateway.this)
  vpc_id = aws_vpc.this.id
  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.this[count.index].id
  }
  tags = merge({
    Name = "mwaa-${var.environment_name}-private-routes-a"
  },)
}

resource "aws_route_table_association" "private" {
  count          = length(aws_subnet.private) 
  route_table_id = aws_route_table.private[count.index].id
  subnet_id      = aws_subnet.private[count.index].id
}

resource "aws_security_group" "this" {
  vpc_id = aws_vpc.this.id
  name   = "mwaa-${var.environment_name}-no-ingress-sg"
  tags = merge({
    Name = "mwaa-${var.environment_name}-no-ingress-sg"
  },)
}

resource "aws_security_group_rule" "ingress_from_self" {
  from_port         = 0
  protocol          = "-1"
  security_group_id = aws_security_group.this.id
  to_port           = 0
  type              = "ingress"
  self              = true
}

resource "aws_security_group_rule" "egress_all_ipv4" {
  from_port         = 0
  protocol          = "-1"
  security_group_id = aws_security_group.this.id
  to_port           = 0
  type              = "egress"
  cidr_blocks       = ["0.0.0.0/0"]
}
