# INFRASTRUCTURE.md - Infrastructure as Code and Cluster Provisioning

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

## Table of Contents

1. [Terraform Patterns](#1-terraform-patterns)
2. [Pulumi Patterns](#2-pulumi-patterns)
3. [Helm Charts](#3-helm-charts)
4. [Ansible Playbooks](#4-ansible-playbooks)
5. [Crossplane Compositions](#5-crossplane-compositions)
6. [Cluster Provisioning](#6-cluster-provisioning)
7. [GitOps Workflows](#7-gitops-workflows)
8. [Security and Compliance](#8-security-and-compliance)
9. [Disaster Recovery](#9-disaster-recovery)
10. [References](#10-references)

---

## 1. Terraform Patterns

### 1.1 Terraform Directory Structure

```
infrastructure/
├── environments/
│   ├── dev/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── terraform.tfvars
│   ├── staging/
│   └── production/
├── modules/
│   ├── networking/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── versions.tf
│   ├── kubernetes/
│   ├── database/
│   └── monitoring/
├── shared/
│   └── modules/
└── templates/
```

### 1.2 Terraform Module Examples

```hcl
# modules/networking/vpc/main.tf

terraform {
  required_version = ">= 1.5.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  
  backend "s3" {
    bucket         = "terraform-state-bucket"
    key            = "networking/vpc"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "terraform-locks"
  }
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
}

variable "cidr_block" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "List of AZs for subnets"
  type        = list(string)
  default     = ["us-east-1a", "us-east-1b", "us-east-1c"]
}

variable "public_subnet_cidrs" {
  description = "CIDR blocks for public subnets"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
}

variable "private_subnet_cidrs" {
  description = "CIDR blocks for private subnets"
  type        = list(string)
  default     = ["10.0.11.0/24", "10.0.12.0/24", "10.0.13.0/24"]
}

variable "enable_nat_gateway" {
  description = "Enable NAT Gateway for private subnets"
  type        = bool
  default     = true
}

variable "tags" {
  description = "Common tags to apply to resources"
  type        = map(string)
  default     = {}
}

locals {
  name_prefix = "${var.environment}-vpc"
  
  common_tags = merge(
    var.tags,
    {
      Environment = var.environment
      ManagedBy   = "terraform"
      Project     = "decapod"
    }
  )
}

resource "aws_vpc" "main" {
  cidr_block           = var.cidr_block
  enable_dns_hostnames = true
  enable_dns_support   = true
  
  tags = merge(
    local.common_tags,
    {
      Name = "${local.name_prefix}-vpc"
    }
  )
}

resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id
  
  tags = merge(
    local.common_tags,
    {
      Name = "${local.name_prefix}-igw"
    }
  )
}

resource "aws_subnet" "public" {
  count             = length(var.public_subnet_cidrs)
  vpc_id            = aws_vpc.main.id
  cidr_block        = var.public_subnet_cidrs[count.index]
  availability_zone = var.availability_zones[count.index]
  
  map_public_ip_on_launch = true
  
  tags = merge(
    local.common_tags,
    {
      Name = "${local.name_prefix}-public-${count.index + 1}"
      Type = "public"
    }
  )
}

resource "aws_subnet" "private" {
  count             = length(var.private_subnet_cidrs)
  vpc_id            = aws_vpc.main.id
  cidr_block        = var.private_subnet_cidrs[count.index]
  availability_zone = var.availability_zones[count.index]
  
  tags = merge(
    local.common_tags,
    {
      Name = "${local.name_prefix}-private-${count.index + 1}"
      Type = "private"
    }
  )
}

resource "aws_eip" "nat" {
  count  = var.enable_nat_gateway ? length(var.availability_zones) : 0
  domain = "vpc"
  
  tags = merge(
    local.common_tags,
    {
      Name = "${local.name_prefix}-nat-eip-${count.index + 1}"
    }
  )
  
  depends_on = [aws_internet_gateway.main]
}

resource "aws_nat_gateway" "main" {
  count         = var.enable_nat_gateway ? length(var.availability_zones) : 0
  allocation_id = aws_eip.nat[count.index].id
  subnet_id     = aws_subnet.public[count.index].id
  
  tags = merge(
    local.common_tags,
    {
      Name = "${local.name_prefix}-nat-${count.index + 1}"
    }
  )
  
  depends_on = [aws_internet_gateway.main]
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id
  
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }
  
  tags = merge(
    local.common_tags,
    {
      Name = "${local.name_prefix}-public-rt"
    }
  )
}

resource "aws_route_table" "private" {
  count  = var.enable_nat_gateway ? length(var.availability_zones) : 0
  vpc_id = aws_vpc.main.id
  
  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.main[count.index].id
  }
  
  tags = merge(
    local.common_tags,
    {
      Name = "${local.name_prefix}-private-rt-${count.index + 1}"
    }
  )
}

resource "aws_route_table_association" "public" {
  count          = length(var.public_subnet_cidrs)
  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

resource "aws_route_table_association" "private" {
  count          = length(var.private_subnet_cidrs)
  subnet_id      = aws_subnet.private[count.index].id
  route_table_id = aws_route_table.private[count.index % length(var.availability_zones)].id
}

# VPC Endpoints for private connectivity to AWS services
resource "aws_vpc_endpoint" "s3" {
  vpc_id       = aws_vpc.main.id
  service_name = "com.amazonaws.${var.availability_zones[0].split("-")[0]}-${var.availability_zones[0].split("-")[1]}.s3"
  
  route_table_ids = concat(
    [aws_route_table.public.id],
    aws_route_table.private[*].id
  )
  
  tags = merge(
    local.common_tags,
    {
      Name = "${local.name_prefix}-s3-endpoint"
    }
  )
}

resource "aws_vpc_endpoint" "ecr_api" {
  vpc_id       = aws_vpc.main.id
  service_name = "com.amazonaws.${var.availability_zones[0].split("-")[0]}-${var.availability_zones[0].split("-")[1]}.ecr.api"
  
  vpc_endpoint_type = "Interface"
  
  security_groups = [aws_security_group.vpc_endpoints.id]
  
  private_dns_enabled = true
  
  subnet_ids = aws_subnet.private[*].id
  
  tags = merge(
    local.common_tags,
    {
      Name = "${local.name_prefix}-ecr-api-endpoint"
    }
  )
}

resource "aws_security_group" "vpc_endpoints" {
  name        = "${local.name_prefix}-vpc-endpoints"
  description = "Security group for VPC endpoints"
  vpc_id      = aws_vpc.main.id
  
  tags = merge(
    local.common_tags,
    {
      Name = "${local.name_prefix}-vpc-endpoints-sg"
    }
  )
}

resource "aws_security_group_rule" "vpc_endpoints_ingress" {
  type              = "ingress"
  from_port         = 443
  to_port           = 443
  protocol          = "tcp"
  cidr_blocks       = [var.cidr_block]
  security_group_id = aws_security_group.vpc_endpoints.id
  description       = "Allow HTTPS from VPC"
}

output "vpc_id" {
  description = "ID of the created VPC"
  value       = aws_vpc.main.id
}

output "vpc_cidr" {
  description = "CIDR block of the VPC"
  value       = aws_vpc.main.cidr_block
}

output "public_subnet_ids" {
  description = "IDs of public subnets"
  value       = aws_subnet.public[*].id
}

output "private_subnet_ids" {
  description = "IDs of private subnets"
  value       = aws_subnet.private[*].id
}

output "nat_gateway_ips" {
  description = "IP addresses of NAT Gateways"
  value       = var.enable_nat_gateway ? aws_eip.nat[*].public_ip : []
}
```

### 1.3 Terraform Kubernetes Provider Configuration

```hcl
# modules/kubernetes/eks/main.tf

terraform {
  required_version = ">= 1.5.0"
  
  required_providers {
    aws        = { source  = "hashicorp/aws", version = "~> 5.0" }
    kubernetes = { source  = "hashicorp/kubernetes", version = "~> 2.23" }
    helm       = { source  = "hashicorp/helm", version = "~> 2.11" }
  }
}

variable "cluster_name" {
  description = "Name of the EKS cluster"
  type        = string
}

variable "environment" {
  description = "Environment name"
  type        = string
}

variable "vpc_id" {
  description = "VPC ID for the cluster"
  type        = string
}

variable "private_subnet_ids" {
  description = "Private subnet IDs for the cluster"
  type        = list(string)
}

variable "cluster_version" {
  description = "Kubernetes version"
  type        = string
  default     = "1.28"
}

variable "cluster_addons" {
  description = "EKS cluster addons configuration"
  type = object({
    vpc_cni     = object({ version = string, enabled = bool })
    coredns     = object({ version = string, enabled = bool })
    kube_proxy  = object({ version = string, enabled = bool })
    aws_ebs_csi = object({ version = string, enabled = bool })
  })
  default = {
    vpc_cni     = { version = "v1.15.3-eksbuild.1", enabled = true }
    coredns     = { version = "v1.10.1-eksbuild.1", enabled = true }
    kube_proxy  = { version = "v1.28.1-eksbuild.1", enabled = true }
    aws_ebs_csi = { version = "v1.24.0-eksbuild.1", enabled = true }
  }
}

locals {
  cluster_identity = {
    oidc = {
      issuer_url = aws_eks_cluster.main.identity[0].oidc[0].issuer
      iam_role   = aws_iam_role.cluster_oidc.arn
    }
  }
}

# EKS Cluster
resource "aws_eks_cluster" "main" {
  name     = var.cluster_name
  version  = var.cluster_version
  role_arn = aws_iam_role.cluster.arn
  vpc_config {
    subnet_ids              = var.private_subnet_ids
    vpc_id                  = var.vpc_id
    endpoint_private_access = true
    endpoint_public_access  = true
    public_access_cidrs     = ["0.0.0.0/0"]
  }
  
  kubernetes_network_config {
    ip_family         = "ipv4"
    service_ipv6_cidr = null
    service_cidr      = "10.96.0.0/12"
  }
  
  eks_addons {
    for_each = toset([
      for name, config in var.cluster_addons : name
      if config.enabled
    ])
    name    = each.value
    version = var.cluster_addons[each.value].version
  }
  
  depends_on = [
    aws_iam_role_policy_attachment.cluster_policy,
    aws_iam_role_policy_attachment.service_policy,
  ]
  
  tags = {
    Environment = var.environment
    ManagedBy  = "terraform"
  }
}

# Node Group IAM Role
resource "aws_iam_role" "nodes" {
  name = "${var.cluster_name}-nodes"
  
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "ec2.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "nodes_base" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSWorkerNodePolicy"
  role       = aws_iam_role.nodes.name
}

resource "aws_iam_role_policy_attachment" "nodes_cni" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKS_CNI_Policy"
  role       = aws_iam_role.nodes.name
}

resource "aws_iam_role_policy_attachment" "nodes_registry" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEC2ContainerRegistryReadOnly"
  role       = aws_iam_role.nodes.name
}

# Managed Node Group
resource "aws_eks_node_group" "main" {
  cluster_name    = aws_eks_cluster.main.name
  node_group_name = "${var.cluster_name}-workers"
  node_role_arn   = aws_iam_role.nodes.arn
  subnet_ids      = var.private_subnet_ids
  
  scaling_config {
    desired_size = 3
    min_size     = 2
    max_size     = 10
  }
  
  instance_types = ["m6i.xlarge"]
  
  disk_size = 100
  
  labels = {
    role = "general"
  }
  
  taints = []
  
  update_config {
    max_unavailable = 1
  }
  
  depends_on = [
    aws_iam_role_policy_attachment.nodes_base,
    aws_iam_role_policy_attachment.nodes_cni,
    aws_iam_role_policy_attachment.nodes_registry,
  ]
  
  tags = {
    Environment = var.environment
    ManagedBy   = "terraform"
  }
}

# Kubernetes Provider
provider "kubernetes" {
  host                   = aws_eks_cluster.main.endpoint
  cluster_ca_certificate = base64decode(aws_eks_cluster.main.certificate_authority[0].data)
  token                  = data.aws_eks_cluster_auth.main.token
  
  exec {
    api_version = "client.authentication.k8s.io/v1beta1"
    command     = "aws"
    args        = ["eks", "get-token", "--cluster-name", aws_eks_cluster.main.name]
  }
}

data "aws_eks_cluster_auth" "main" {
  name = aws_eks_cluster.main.name
}

# Helm Provider
provider "helm" {
  kubernetes {
    host                   = aws_eks_cluster.main.endpoint
    cluster_ca_certificate = base64decode(aws_eks_cluster.main.certificate_authority[0].data)
    token                  = data.aws_eks_cluster_auth.main.token
    
    exec {
      api_version = "client.authentication.k8s.io/v1beta1"
      command     = "aws"
      args        = ["eks", "get-token", "--cluster-name", aws_eks_cluster.main.name]
    }
  }
}
```

---

## 2. Pulumi Patterns

### 2.1 Pulumi Project Structure

```yaml
# Pulumi.yaml
name: decapod-infrastructure
runtime: yaml
description: Infrastructure as Code for Decapod platform
backend:
  url: s3://pulumi-state-bucket/
  encryptionsalt: <encryption-salt>

# Pulumi.<stack>.yaml files for each environment
```

### 2.2 Pulumi Python Infrastructure Code

```python
# __main__.py - Pulumi entry point
import pulumi
import pulumi_aws as aws
import pulumi_eks as eks
import pulumi_kubernetes as k8s
from pulumi import Config, StackReference, Output

# Configuration
config = Config()
stack_name = pulumi.get_stack()
project_name = pulumi.get_project()

# Shared configuration across environments
shared_tags = {
    "Project": "decapod",
    "Environment": stack_name,
    "ManagedBy": "pulumi",
}

# Reference shared networking module
networking_stack = StackReference(f"decapod/networking/{stack_name}")
vpc_id = networking_stack.require_output("vpc_id")
private_subnet_ids = networking_stack.require_output("private_subnet_ids")
public_subnet_ids = networking_stack.require_output("public_subnet_ids")

# EKS Cluster
cluster = eks.Cluster(
    f"decapod-eks-{stack_name}",
    name=f"decapod-{stack_name}",
    version="1.28",
    vpc_id=vpc_id,
    private_subnet_ids=private_subnet_ids,
    public_subnet_ids=public_subnet_ids,
    instance_type="m6i.xlarge",
    desired_capacity=3,
    min_size=2,
    max_size=10,
    storage_classes={
        "gp3": eks.ClusterStorageClassArgs(
            type="gp3",
            magnetic_storage_name="standard",
        ),
        "io2": eks.ClusterStorageClassArgs(
            type="io2",
            magnetic_storage_name="io2",
            provisioner="kubernetes.io/aws-ebs",
            parameters={
                "type": "io2",
                "iops": "20000",
                "fsType": "ext4",
            },
        ),
    },
    node_root_volume_size=100,
    tags=shared_tags,
)

# Export cluster config
pulumi.export("cluster_name", cluster.name)
pulumi.export("cluster_endpoint", cluster.endpoint)
pulumi.export("kubeconfig", cluster.kubeconfig)

# Create Kubernetes provider
k8s_provider = k8s.Provider(
    f"decapod-k8s-{stack_name}",
    kubeconfig=cluster.kubeconfig,
)

# Deploy cluster addons using Helm
metrics_server = k8s.helm.v3.Chart(
    "metrics-server",
    k8s.helm.v3.ChartOpts(
        chart="metrics-server",
        version="3.11.0",
        fetch_opts=k8s.helm.v3.FetchOpts(
            repo="https://kubernetes-sigs.github.io/metrics-server",
        ),
        namespace="kube-system",
        values={
            "args": ["--kubelet-insecure-tls"]
        },
    ),
    opts=pulumi.ResourceOptions(provider=k8s_provider),
)

# AWS Load Balancer Controller
lb_controller_values = {
    "clusterName": cluster.name,
    "region": aws.get_region().name,
    "serviceAccount": {
        "annotations": {
            "eks.amazonaws.com/role-arn": create_lb_controller_iam_role(cluster)
        }
    },
    "controller": {
        "replicas": 2,
        "resources": {
            "limits": {"cpu": "200m", "memory": "256Mi"},
            "requests": {"cpu": "100m", "memory": "128Mi"},
        }
    }
}

aws_load_balancer_controller = k8s.helm.v3.Chart(
    "aws-load-balancer-controller",
    k8s.helm.v3.ChartOpts(
        chart="aws-load-balancer-controller",
        version="1.6.2",
        fetch_opts=k8s.helm.v3.FetchOpts(
            repo="https://aws.github.io/eks-charts",
        ),
        namespace="kube-system",
        values=lb_controller_values,
    ),
    opts=pulumi.ResourceOptions(
        provider=k8s_provider,
        depends_on=[cluster],
    ),
)

def create_lb_controller_iam_role(cluster: eks.Cluster) -> str:
    """Create IAM role for AWS Load Balancer Controller"""
    # Create OIDC provider
    oidc_provider = aws.iam.OpenIdConnectProvider(
        f"decapod-oidc-{stack_name}",
        url=cluster.identities[0].oidcs[0].url,
        client_id_lists=["sts.amazonaws.com"],
        thumbprint_lists=["9e5a7e70c7bbae25"],
    )
    
    # IAM Role for LB Controller
    lb_controller_role = aws.iam.Role(
        f"decapod-lb-controller-{stack_name}",
        assume_role_policy=Output.all(
            oidc_provider.url,
            oidc_provider.arn,
        ).apply(lambda args: f"""{{
            "Version": "2012-10-17",
            "Statement": [{{
                "Effect": "Allow",
                "Principal": {{
                    "Federated": "{args[1]}"
                }},
                "Action": "sts:AssumeRoleWithWebIdentity",
                "Condition": {{
                    "StringEquals": {{
                        "{args[0]}:sub": "system:serviceaccount:kube-system:aws-load-balancer-controller"
                    }}
                }}
            }}]
        }}"""),
    )
    
    # Attach AWSLoadBalancerController policy
    aws.iam.RolePolicyAttachment(
        f"decapod-lb-controller-policy-{stack_name}",
        role=lb_controller_role.name,
        policy_arn="arn:aws:iam::aws:policy/AWSLoadBalancerControllerPolicy",
    )
    
    return lb_controller_role.arn.apply(lambda arn: arn)
```

---

## 3. Helm Charts

### 3.1 Helm Chart Structure

```
charts/
├── my-service/
│   ├── Chart.yaml
│   ├── values.schema.json
│   ├── values.yaml
│   ├── templates/
│   │   ├── _helpers.tpl
│   │   ├── NOTES.txt
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   ├── serviceaccount.yaml
│   │   ├── hpa.yaml
│   │   ├── pdb.yaml
│   │   ├── ingress.yaml
│   │   ├── configmap.yaml
│   │   └── secret.yaml
│   └── .helmignore
```

### 3.2 Complete Helm Chart Example

```yaml
# Chart.yaml
apiVersion: v2
name: order-service
description: A Helm chart for the Order Service microservice
type: application
version: 1.2.3
appVersion: "1.2.3"
kubeVersion: ">= 1.28-0"
keywords:
  - order
  - e-commerce
  - microservices
home: https://github.com/example/order-service
sources:
  - https://github.com/example/order-service
maintainers:
  - name: Platform Team
    email: platform@example.com
dependencies:
  - name: common
    version: "1.x.x"
    repository: "https://charts.bitnami.com/bitnami"
  - name: postgresql
    version: "12.x.x"
    repository: "https://charts.bitnami.com/bitnami"
    condition: postgresql.enabled
    tags:
      - database

# values.schema.json
{
  "$schema": "https://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "image": {
      "type": "object",
      "properties": {
        "repository": {"type": "string"},
        "tag": {"type": "string"},
        "pullPolicy": {"type": "string", "enum": ["IfNotPresent", "Always", "Never"]},
        "pullSecrets": {"type": "array"}
      },
      "required": ["repository", "tag"]
    },
    "replicaCount": {"type": "integer", "minimum": 1},
    "resources": {
      "type": "object",
      "properties": {
        "limits": {
          "type": "object",
          "properties": {
            "cpu": {"type": "string"},
            "memory": {"type": "string"}
          }
        },
        "requests": {
          "type": "object",
          "properties": {
            "cpu": {"type": "string"},
            "memory": {"type": "string"}
          }
        }
      }
    },
    "service": {
      "type": "object",
      "properties": {
        "type": {"type": "string", "enum": ["ClusterIP", "NodePort", "LoadBalancer"]},
        "port": {"type": "integer", "minimum": 1, "maximum": 65535}
      }
    }
  },
  "required": ["image", "replicaCount"]
}
```

```yaml
# values.yaml
# Default values for order-service.

replicaCount: 3

image:
  repository: ghcr.io/example/order-service
  tag: "1.2.3"
  pullPolicy: IfNotPresent
  pullSecrets: []
  securityContext:
    enabled: true
    runAsNonRoot: true
    runAsUser: 1000
    fsGroup: 1000

service:
  type: ClusterIP
  port: 8080
  grpcPort: 9090
  adminPort: 8081
  metricsPort: 9090
  
  annotations: {}
  labels: {}

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "60"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "60"
  hosts:
    - host: orders.example.com
      paths:
        - path: /
          pathType: Prefix
          service: http
          port: 8080
  tls:
    - secretName: orders-tls
      hosts:
        - orders.example.com

serviceAccount:
  create: true
  name: order-service
  annotations:
    eks.amazonaws.com/role-arn: arn:aws:iam::123456789012:role/order-service-role

podAnnotations:
  prometheus.io/scrape: "true"
  prometheus.io/port: "9090"
  prometheus.io/path: "/metrics"
  linkerd.io/inject: "enabled"

podSecurityContext:
  enabled: true
  fsGroup: 1000
  runAsNonRoot: true
  runAsUser: 1000

securityContext:
  enabled: true
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  capabilities:
    drop:
      - ALL

resources:
  limits:
    cpu: 2000m
    memory: 2Gi
  requests:
    cpu: 500m
    memory: 512Mi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 50
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80
  hpa:
    behavior:
      scaleDown:
        stabilizationWindowSeconds: 300
        policies:
          - type: Percent
            value: 10
            periodSeconds: 60
      scaleUp:
        stabilizationWindowSeconds: 0
        policies:
          - type: Percent
            value: 100
            periodSeconds: 15

podDisruptionBudget:
  enabled: true
  minAvailable: 2
  maxUnavailable: null

nodeSelector: {}

tolerations: []

affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
      - weight: 100
        podAffinityTerm:
          labelSelector:
            matchLabels:
              app.kubernetes.io/name: order-service
          topologyKey: kubernetes.io/hostname
  topologySpreadConstraints:
    - maxSkew: 1
      topologyKey: topology.kubernetes.io/zone
      whenUnsatisfiable: ScheduleAnyway
      labelSelector:
        matchLabels:
          app.kubernetes.io/name: order-service

livenessProbe:
  enabled: true
  httpGet:
    path: /health/live
    port: admin
  initialDelaySeconds: 10
  periodSeconds: 15
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  enabled: true
  httpGet:
    path: /health/ready
    port: admin
  initialDelaySeconds: 5
  periodSeconds: 10
  timeoutSeconds: 3
  failureThreshold: 3

startupProbe:
  enabled: true
  httpGet:
    path: /health/started
    port: admin
  initialDelaySeconds: 0
  periodSeconds: 5
  failureThreshold: 30

config:
  database:
    host: postgres.database.svc.cluster.local
    port: 5432
    name: orders
    username: orders
    pool:
      min: 5
      max: 50
      idle_timeout: 30s
      max_lifetime: 1h
    ssl:
      enabled: true
      mode: require
  
  redis:
    host: redis.cache.svc.cluster.local
    port: 6379
    password:
      value: ""
      valueFrom:
        secretKeyRef:
          name: redis-credentials
          key: password
    database: 0
    pool:
      max_active: 50
      max_idle: 10
      min_idle: 5
  
  kafka:
    brokers:
      - kafka-0.kafka.svc.cluster.local:9092
      - kafka-1.kafka.svc.cluster.local:9092
      - kafka-2.kafka.svc.cluster.local:9092
    topic_prefix: orders
    consumer_group: order-service
    ssl:
      enabled: true
  
  observability:
    tracing:
      enabled: true
      endpoint: http://jaeger-collector.observability.svc.cluster.local:4317
      sampling_rate: 0.1
    metrics:
      enabled: true
      path: /metrics
    logging:
      level: info
      format: json
  
  rate_limiting:
    enabled: true
    requests_per_second: 1000
    burst: 100

env:
  - name: GOMAXPROCS
    value: "4"
  - name: GOMEMLIMIT
    value: "2GiB"
  - name: GRACEFUL_SHUTDOWN_TIMEOUT
    value: "30s"
  - name: API_RATE_LIMIT
    value: "1000"

secret:
  enabled: true
  name: order-service-secrets
  type: Opaque
  data: {}

postgresql:
  enabled: true
  auth:
    database: orders
    username: orders
    password: ""
    existingSecret: postgres-credentials
  primary:
    persistence:
      enabled: true
      size: 10Gi
      storageClass: gp3
    resources:
      limits:
        cpu: 1000m
        memory: 1Gi
      requests:
        cpu: 100m
        memory: 256Mi
```

```yaml
# templates/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "order-service.fullname" . }}
  namespace: {{ .Release.Namespace }}
  labels:
    {{- include "order-service.labels" . | nindent 4 }}
    app.kubernetes.io/component: application
  annotations:
    {{- toYaml .Values.podAnnotations | nindent 4 }}
spec:
  replicas: {{ .Values.replicaCount }}
  revisionHistoryLimit: 5
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      {{- include "order-service.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "order-service.labels" . | nindent 8 }}
        app.kubernetes.io/component: application
      annotations:
        {{- toYaml .Values.podAnnotations | nindent 8 }}
    spec:
      serviceAccountName: {{ include "order-service.serviceAccountName" . }}
      {{- with .Values.podSecurityContext }}
      securityContext:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.affinity }}
      affinity:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.topologySpreadConstraints }}
      topologySpreadConstraints:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.tolerations }}
      tolerations:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.nodeSelector }}
      nodeSelector:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      terminationGracePeriodSeconds: 60
      dnsPolicy: ClusterFirst
      restartPolicy: Always
      containers:
        - name: {{ .Chart.Name }}
          image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          ports:
            - name: http
              containerPort: {{ .Values.service.port }}
              protocol: TCP
            - name: grpc
              containerPort: {{ .Values.service.grpcPort }}
              protocol: TCP
            - name: admin
              containerPort: {{ .Values.service.adminPort }}
              protocol: TCP
            - name: metrics
              containerPort: {{ .Values.service.metricsPort }}
              protocol: TCP
          env:
            {{- toYaml .Values.env | nindent 12 }}
            - name: POD_NAME
              valueFrom:
                fieldRef:
                  fieldPath: metadata.name
            - name: POD_NAMESPACE
              valueFrom:
                fieldRef:
                  fieldPath: metadata.namespace
          {{- with .Values.resources }}
          resources:
            {{- toYaml . | nindent 12 }}
          {{- end }}
          {{- with .Values.securityContext }}
          securityContext:
            {{- toYaml . | nindent 10 }}
          {{- end }}
          {{- if .Values.livenessProbe.enabled }}
          livenessProbe:
            {{- omit .Values.livenessProbe "enabled" | toYaml | nindent 12 }}
          {{- end }}
          {{- if .Values.readinessProbe.enabled }}
          readinessProbe:
            {{- omit .Values.readinessProbe "enabled" | toYaml | nindent 12 }}
          {{- end }}
          {{- if .Values.startupProbe.enabled }}
          startupProbe:
            {{- omit .Values.startupProbe "enabled" | toYaml | nindent 12 }}
          {{- end }}
          volumeMounts:
            - name: tmp
              mountPath: /tmp
            - name: cache
              mountPath: /app/cache
          {{- range .Values.extraConfigMapMounts }}
            - name: {{ .name }}
              mountPath: {{ .mountPath }}
              readOnly: {{ .readOnly }}
              subPath: {{ .subPath }}
          {{- end }}
      initContainers:
        {{- if .Values.postgresql.enabled }}
        - name: schema-migration
          image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          command: ["/app/bin/migrate"]
          args: ["up", "--timeout=60s"]
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: {{ include "order-service.fullname" . }}-db-url
                  key: url
          resources:
            limits:
              cpu: 500m
              memory: 256Mi
            requests:
              cpu: 100m
              memory: 64Mi
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            capabilities:
              drop:
                - ALL
        {{- end }}
      volumes:
        - name: tmp
          emptyDir:
            medium: Memory
            sizeLimit: 256Mi
        - name: cache
          emptyDir:
            medium: Memory
            sizeLimit: 512Mi
        {{- range .Values.extraConfigMapMounts }}
        - name: {{ .name }}
          configMap:
            name: {{ .configMap }}
        {{- end }}
```

---

## 4. Ansible Playbooks

### 4.1 Kubernetes Node Configuration Playbook

```yaml
# ansible/playbooks/kubernetes-nodes.yml
---
- name: Configure Kubernetes Nodes
  hosts: k8s_nodes
  become: true
  gather_facts: true
  vars:
    k8s_version: "1.28.0"
    container_runtime: containerd
    pod_cidr: "10.244.0.0/16"
    service_cidr: "10.96.0.0/12"
    
  pre_tasks:
    - name: Update apt cache
      ansible.builtin.apt:
        update_cache: yes
        cache_valid_time: 3600
      when: ansible_os_family == "Debian"
      
    - name: Create kubernetes repo directory
      ansible.builtin.file:
        path: /etc/apt/keyrings
        state: directory
        mode: '0755'

  tasks:
    - name: Install prerequisites
      ansible.builtin.apt:
        name:
          - apt-transport-https
          - ca-certificates
          - curl
          - gnupg
          - lsb-release
          - software-properties-common
        state: present
        update_cache: yes
        
    - name: Add Kubernetes signing key
      ansible.builtin.apt_key:
        url: https://pkgs.k8s.io/core:/stable:/v1.28/deb/Release.key
        state: present
        
    - name: Add Kubernetes repository
      ansible.builtin.apt_repository:
        repo: "deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v1.28/deb/ /"
        state: present
        
    - name: Install containerd
      ansible.builtin.apt:
        name:
          - containerd
        state: present
        update_cache: yes
        
    - name: Generate containerd config
      ansible.builtin.command:
        cmd: containerd config default
      register: containerd_config
      
    - name: Save containerd config
      ansible.builtin.copy:
        content: "{{ containerd_config.stdout }}"
        dest: /etc/containerd/config.toml
        mode: '0644'
        
    - name: Configure containerd systemd
      ansible.builtin.lineinfile:
        path: /etc/containerd/config.toml
        regexp: '^\s*SystemdCgroup\s*='
        line: '            SystemdCgroup = true'
        
    - name: Restart containerd
      ansible.builtin.service:
        name: containerd
        state: restarted
        enabled: yes
        
    - name: Install Kubernetes components
      ansible.builtin.apt:
        name:
          - kubelet
          - kubeadm
          - kubectl
        state: present
        default_release: v1.28
        
    - name: Hold Kubernetes packages
      community.general.debconf:
        name: "{{ item }}"
        question: "{{ item }}/hold"
        value: "true"
        vtype: boolean
      loop:
        - kubelet
        - kubeadm
        - kubectl
        
    - name: Configure kernel modules
      community.general.modprobe:
        name: "{{ item }}"
        state: present
      loop:
        - overlay
        - br_netfilter
        
    - name: Configure sysctl
      ansible.posix.sysctl:
        name: "{{ item.name }}"
        value: "{{ item.value }}"
        sysctl_file: /etc/sysctl.d/k8s.conf
        state: present
        reload: yes
      loop:
        - { name: net.bridge.bridge-nf-call-iptables, value: 1 }
        - { name: net.bridge.bridge-nf-call-ip6tables, value: 1 }
        - { name: net.ipv4.ip_forward, value: 1 }
        - { name: ip_tables, value: 1 }
        - { name: i6_tables, value: 1 }
        - { name: ip_vs, value: 1 }
        - { name: ip_vs_rr, value: 1 }
        - { name: ip_vs_wrr, value: 1 }
        - { name: ip_vs_sh, value: 1 }
        - { name: nf_conntrack, value: 1 }
        
    - name: Disable swap
      ansible.builtin.shell: |
        swapoff -a && sed -i '/swap/d' /etc/fstab
      when: ansible_swaptotal_mb > 0
      
    - name: Ensure kubelet is running
      ansible.builtin.service:
        name: kubelet
        state: started
        enabled: yes

  handlers:
    - name: Reload systemd
      ansible.builtin.systemd_service:
        daemon_reload: yes
        
    - name: Restart kubelet
      ansible.builtin.service:
        name: kubelet
        state: restarted
```

---

## 5. Crossplane Compositions

### 5.1 Crossplane XRD (Composite Resource Definition)

```yaml
# crossplane/definition.yaml
apiVersion: apiextensions.crossplane.io/v1
kind: CompositeResourceDefinition
metadata:
  name: compositepostgresqlinstances.database.example.com
  labels:
    crossplane.io/composite: compositepostgresqlinstance
spec:
  group: database.example.com
  names:
    kind: CompositePostgreSQLInstance
    plural: compositepostgresqlinstances
  claimNames:
    kind: PostgreSQLInstance
    plural: postgresqlinstances
  connectionSecretKeys:
    - username
    - password
    - endpoint
    - port
    - database
  versions:
    - name: v1alpha1
      served: true
      referenceable: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              properties:
                parameters:
                  type: object
                  properties:
                    storageGB:
                      type: integer
                      default: 20
                    instanceClass:
                      type: string
                      default: db.t3.medium
                    engineVersion:
                      type: string
                      default: "14"
                    multiAZ:
                      type: boolean
                      default: true
                    backupRetentionDays:
                      type: integer
                      default: 7
                    encrypted:
                      type: boolean
                      default: true
                  required:
                    - storageGB
              required:
                - parameters
            status:
              type: object
              properties:
                conditions:
                  type: array
                connectionDetails:
                  type: object
```

### 5.2 Crossplane Composition

```yaml
# crossplane/composition.yaml
apiVersion: apiextensions.crossplane.io/v1
kind: Composition
metadata:
  name: compositepostgresqlinstances-aws
  labels:
    provider: aws
    guide: example
spec:
  writeConnectionSecretsToNamespace: crossplane-system
  compositeResourceDefinition:
    name: compositepostgresqlinstances.database.example.com
    
  mode: Pipeline
  
  pipeline:
    - step: create-vpc
      functionRef:
        name: function-patch-values
      input:
        apiVersion: patchvalues.fn.crossplane.io/v1beta1
        kind: PatchValues
        patchSets:
          - name: common
            patches:
              - type: FromCompositeFieldPath
                fromFieldPath: metadata.labels
                toFieldPath: metadata.labels
              - type: FromCompositeFieldPath
                fromFieldPath: metadata.annotations
                toFieldPath: metadata.annotations
        resources:
          - name: rds-instance
            base:
              apiVersion: rds.aws.crossplane.io/v1alpha1
              kind: Instance
              spec:
                forProvider:
                  region: us-east-1
                  engine: postgres
                  dbInstanceClass: db.t3.medium
                  allocatedStorage: 20
                  engineVersion: "14"
                  masterUsername: postgres
                  publiclyAccessible: false
                  backupRetentionPeriod: 7
                  storageEncrypted: true
                  skipFinalSnapshotBeforeDeletion: true
                  finalDBSnapshotIdentifierPrefix: final-snapshot
                writeConnectionSecretToRef:
                  namespace: crossplane-system
                providerConfigRef:
                  name: default
            patches:
              - type: PatchAndTransform
                patch:
                  fromFieldPath: spec.parameters.storageGB
                  toFieldPath: spec.forProvider.allocatedStorage
                transform:
                  type: convert
                  convert:
                    toType: int64
              - type: PatchAndTransform
                patch:
                  fromFieldPath: spec.parameters.instanceClass
                  toFieldPath: spec.forProvider.dbInstanceClass
              - type: PatchAndTransform
                patch:
                  fromFieldPath: spec.parameters.engineVersion
                  toFieldPath: spec.forProvider.engineVersion
              - type: PatchAndTransform
                patch:
                  fromFieldPath: spec.parameters.multiAZ
                  toFieldPath: spec.forProvider.multiAZ
              - type: PatchAndTransform
                patch:
                  fromFieldPath: spec.parameters.backupRetentionDays
                  toFieldPath: spec.forProvider.backupRetentionPeriod
              - type: PatchAndTransform
                patch:
                  fromFieldPath: spec.parameters.encrypted
                  toFieldPath: spec.forProvider.storageEncrypted
              - type: PatchAndTransform
                patch:
                  fromFieldPath: metadata.labels[crossplane.io/claim-name]
                  toFieldPath: spec.forProvider.dbName
                  transform:
                    type: string
                    string:
                      format: "%s-db"
                      
          - name: security-group
            base:
              apiVersion: ec2.aws.crossplane.io/v1alpha1
              kind: SecurityGroup
              spec:
                forProvider:
                  region: us-east-1
                  groupName: postgres-sg
                  description: Security group for PostgreSQL
                  ingress:
                    - fromPort: 5432
                      toPort: 5432
                      ipProtocol: tcp
                      ipRanges:
                        - cidrIp: "10.0.0.0/16"
                          description: VPC internal
                  egress:
                    - ipProtocol: "-1"
                      ipRanges:
                        - cidrIp: "0.0.0.0/0"
                  vpcId: ""  # Will be patched
                providerConfigRef:
                  name: default
            patches:
              - type: FromCompositeFieldPath
                fromFieldPath: spec.parameters.vpcId
                toFieldPath: spec.forProvider.vpcId
                
          - name: rds-instance-to-sg
            base:
              apiVersion: ec2.aws.crossplane.io/v1alpha1
              kind: SecurityGroupRule
              spec:
                forProvider:
                  region: us-east-1
                  type: ingress
                  fromPort: 5432
                  toPort: 5432
                  ipProtocol: tcp
                providerConfigRef:
                  name: default
            patches:
              - type: FromCompositeFieldPath
                fromFieldPath: status.securityGroupId
                toFieldPath: spec.forProvider.groupId
              - type: FromCompositeFieldPath
                fromFieldPath: status.rdsInstance.status.atProvider.address
                toFieldPath: status.atProvider.cidrIP
```

---

## 6. Cluster Provisioning

### 6.1 EKS Cluster Provisioning

```yaml
# Terraform EKS cluster provisioning
# environments/production/eks.tf

terraform {
  required_version = ">= 1.5.0"
  
  required_providers {
    aws        = { source  = "hashicorp/aws", version = "~> 5.0" }
    kubernetes = { source  = "hashicorp/kubernetes", version = "~> 2.23" }
    helm       = { source  = "hashicorp/helm", version = "~> 2.11" }
  }
  
  backend "s3" {
    bucket = "terraform-state-bucket"
    key    = "production/eks/cluster.tfstate"
    region = "us-east-1"
    encrypt = true
  }
}

variable "cluster_name" {
  default = "decapod-production"
}

variable "cluster_version" {
  default = "1.28"
}

variable "vpc_id" {
  default = "vpc-0123456789abcdef0"
}

variable "private_subnet_ids" {
  type = list(string)
  default = [
    "subnet-0123456789abcdef1",
    "subnet-0123456789abcdef2",
    "subnet-0123456789abcdef3",
  ]
}

# EKS Cluster
resource "aws_eks_cluster" "main" {
  name     = var.cluster_name
  version  = var.cluster_version
  role_arn = aws_iam_role.cluster.arn
  
  vpc_config {
    subnet_ids                      = var.private_subnet_ids
    vpc_id                          = var.vpc_id
    endpoint_private_access         = true
    endpoint_public_access          = true
    public_access_cidrs             = ["10.0.0.0/8"]
    control_plane_subnet_ids         = var.private_subnet_ids
  }
  
  kubernetes_network_config {
    ip_family         = "ipv4"
    service_cidr      = "10.96.0.0/12"
    pod_cidr          = "10.244.0.0/16"
  }
  
  encryption_config {
    provider {
      key_arn = aws_kms_key.eks.arn
    }
    resources = ["secrets"]
  }
  
  enabled_cluster_log_types = [
    "api",
    "audit",
    "authenticator",
    "controllerManager",
    "scheduler"
  ]
  
  timeouts {
    create = "60m"
    update = "120m"
    delete = "60m"
  }
  
  tags = {
    Environment = "production"
    ManagedBy   = "terraform"
  }
}

# Cluster KMS Key
resource "aws_kms_key" "eks" {
  description             = "EKS cluster encryption key"
  deletion_window_in_days  = 10
  enable_key_rotation     = true
  
  tags = {
    Environment = "production"
    ManagedBy   = "terraform"
  }
}

resource "aws_kms_alias" "eks" {
  name          = "alias/eks-cluster-key"
  target_key_id = aws_kms_key.eks.key_id
}

# Cluster IAM Role
resource "aws_iam_role" "cluster" {
  name = "${var.cluster_name}-cluster"
  
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect = "Allow"
      Action = "sts:AssumeRole"
      Principal = {
        Service = "eks.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "cluster_policy" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSClusterPolicy"
  role       = aws_iam_role.cluster.name
}

resource "aws_iam_role_policy_attachment" "cluster_service_policy" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSServicePolicy"
  role       = aws_iam_role.cluster.name
}

# Node Group
resource "aws_eks_node_group" "main" {
  cluster_name    = aws_eks_cluster.main.name
  node_group_name = "${var.cluster_name}-nodes"
  node_role_arn   = aws_iam_role.nodes.arn
  subnet_ids      = var.private_subnet_ids
  instance_types  = ["m6i.xlarge"]
  
  scaling_config {
    desired_size = 3
    min_size     = 2
    max_size     = 10
  }
  
  disk_size = 100
  
  remote_access {
    ec2_ssh_key = "production-key"
    source_security_group_ids = []
  }
  
  update_config {
    max_unavailable            = 1
    max_unavailable_percentage = null
  }
  
  labels = {
    node-group = "general"
  }
  
  taints = []
  
  timeouts {
    create = "30m"
    update = "30m"
    delete = "30m"
  }
  
  depends_on = [
    aws_iam_role_policy_attachment.nodes_base,
    aws_iam_role_policy_attachment.nodes_cni,
    aws_iam_role_policy_attachment.nodes_registry,
  ]
}

# Output kubeconfig
output "kubeconfig" {
  value = <<-EOT
    apiVersion: v1
    kind: Config
    clusters:
    - cluster:
        server: ${aws_eks_cluster.main.endpoint}
        certificate-authority-data: ${aws_eks_cluster.main.certificate_authority[0].data}
      name: ${aws_eks_cluster.main.name}
    contexts:
    - context:
        cluster: ${aws_eks_cluster.main.name}
        user: ${aws_eks_cluster.main.name}
      name: ${aws_eks_cluster.main.name}
    current-context: ${aws_eks_cluster.main.name}
    users:
    - name: ${aws_eks_cluster.main.name}
      user:
        exec:
          apiVersion: client.authentication.k8s.io/v1beta1
          command: aws
          args:
          - eks
          - get-token
          - --cluster-name
          - ${aws_eks_cluster.main.name}
  EOT
  sensitive = false
}
```

---

## 7. GitOps Workflows

### 7.1 ArgoCD Application

```yaml
# gitops/argocd/application.yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: order-service
  namespace: argocd
  labels:
    app: order-service
    tier: backend
  annotations:
    argocd.argoproj.io/sync-options: PruneLast=true
    argocd.argoproj.io/sync-wave: "1"
spec:
  project: platform
  source:
    repoURL: https://github.com/example/helm-charts
    targetRevision: main
    path: charts/order-service
    helm:
      valueFiles:
        - values.yaml
        - values-prod.yaml
      parameters:
        - name: image.tag
          value: latest
        - name: replicaCount
          value: "5"
        - name: autoscaling.minReplicas
          value: "5"
        - name: autoscaling.maxReplicas
          value: "50"
  destination:
    server: https://kubernetes.default.svc
    namespace: platform
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
      allowEmpty: false
    syncOptions:
      - CreateNamespace=true
      - PruneLast=true
      - PrunePropagation=foreground
      - Replace=false
      - ServerSideApply=true
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
    ignoredDifferences:
      - group: apps
        kind: Deployment
        jsonPointers:
          - /spec/replicas
      - group: ""
        kind: Pod
        jsonPointers:
          - /spec/initContainers
  ignoreDifferences:
    - group: apps
      kind: Deployment
      jsonPointers:
        - /spec/replicas
        - /metadata/annotations
    - group: ""
      kind: Secret
      jsonPointers:
        - /data
```

---

## 8. Security and Compliance

### 8.1 Terraform Security

```hcl
# Security module for infrastructure

# S3 bucket with encryption and versioning
resource "aws_s3_bucket" "state" {
  bucket = "terraform-state-${var.environment}"
  
  versioning {
    enabled = true
  }
  
  server_side_encryption_configuration {
    rule {
      apply_server_side_encryption_by_default {
        sse_algorithm     = "AES256"
        kms_master_key_id = aws_kms_key.terraform.arn
      }
    }
  }
  
  lifecycle_rule {
    enabled = true
    noncurrent_version_transition {
      days          = 30
      storage_class = "GLACIER"
    }
    noncurrent_version_expiration {
      days = 90
    }
  }
  
  tags = var.common_tags
}

# DynamoDB table for state locking
resource "aws_dynamodb_table" "state_locks" {
  name           = "terraform-locks"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "LockID"
  
  attribute {
    name = "LockID"
    type = "S"
  }
  
  point_in_time_recovery {
    enabled = true
  }
  
  server_side_encryption {
    enabled = true
  }
  
  tags = var.common_tags
}
```

---

## 9. Disaster Recovery

### 9.1 Backup Configuration

```yaml
# Backup configuration for Kubernetes resources
backup:
  velero:
    enabled: true
    namespace: velero
    image: velero/velero:v1.12.0
    
    backup_storage_locations:
      - name: primary
        provider: aws
        bucket: backup-bucket
        region: us-east-1
        prefix: velero
        config:
          s3ForcePathStyle: "false"
          s3Url: ""
          kmsKeyId: arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012
        
    default_volumes_to_fs_backup: false
    
    schedule:
      daily:
        schedule: "0 2 * * *"
        ttl: 720h  # 30 days
        included_namespaces:
          - platform
          - monitoring
        excluded_resources:
          - events
          - events.events.k8s.io
          
      weekly:
        schedule: "0 3 * * 0"
        ttl: 2160h  # 90 days
        included_namespaces:
          - "*"
        storage_location: primary
        
      databases:
        schedule: "0 4 * * *"
        ttl: 8760h  # 1 year
        included_namespaces:
          - database
        snapshot_volumes: true
        include_cluster_resources: true
```

---

## 10. References

### Terraform

- [Terraform Documentation](https://www.terraform.io/docs)
- [AWS Provider Documentation](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)
- [Terraform Module Registry](https://registry.terraform.io)
- [Terraform Best Practices](https://www.terraform-best-practices.com/)

### Pulumi

- [Pulumi Documentation](https://www.pulumi.com/docs/)
- [Pulumi GitHub](https://github.com/pulumi/pulumi)
- [Pulumi EKS](https://github.com/pulumi/pulumi-eks)

### Helm

- [Helm Documentation](https://helm.sh/docs/)
- [Helm Charts Best Practices](https://helm.sh/docs/chart_best_practices/)
- [Bitnami Charts](https://github.com/bitnami/charts)

### Crossplane

- [Crossplane Documentation](https://docs.crossplane.io/)
- [Crossplane GitHub](https://github.com/crossplane/crossplane)
- [Upbound Registry](https://marketplace.upbound.io/)

### Kubernetes

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [AWS EKS Best Practices](https://aws.github.io/aws-eks-best-practices/)
- [Production Kubernetes](https://www.oreilly.com/library/view/production-kubernetes/9781492055536/)