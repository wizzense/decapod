# CLOUD.md - Cloud Architecture (DENSE)

**Authority:** guidance (cloud infrastructure, deployment patterns, and operational excellence)
**Layer:** Guides
**Binding:** No
**Scope:** cloud platforms, infrastructure patterns, and DevOps practices
**Non-goals:** specific cloud provider tutorials, vendor-specific implementations

---

## 1. Cloud Architecture Principles

### 1.1 Design for Failure
**Everything fails, all the time.**
- Hardware fails
- Networks partition
- Services degrade
- Regions go offline

**Resilience requires:**
- Redundancy at every layer
- Automated recovery
- Graceful degradation
- Circuit breakers and bulkheads

**Architecture Decision Matrix:**

| Failure Mode | Detection | Response | Recovery |
|--------------|-----------|----------|----------|
| Instance crash | Health check + ELB | Auto-replace via ASG | < 60 seconds |
| AZ failure | Multi-AZ health | Traffic failover | DNS TTL dependent |
| Region failure | Health endpoint | DR region activation | RTO dependent |
| Network partition | Circuit breaker | Degrade to cached data | Auto-heal |
| Database failover | Primary/standby sync | Promote standby | 30-120 seconds |
| Cache miss storm | Eviction rate spike | Staggered TTL + mutex | Automatic |

### 1.2 Elasticity
**Scale horizontally, not vertically.**
- Add/remove instances based on demand
- Stateless services enable elasticity
- Auto-scaling based on metrics
- Scale to zero for cost savings (serverless)

**Auto-Scaling Configuration Schema:**

```yaml
# AWS Auto Scaling Group Configuration
AWSTemplateFormatVersion: '2010-09-09'
Description: Auto Scaling Group with scaling policies

Resources:
  AutoScalingGroup:
    Type: AWS::AutoScaling::AutoScalingGroup
    Properties:
      MinSize: 2
      MaxSize: 20
      DesiredCapacity: 2
      HealthCheckType: ELB
      HealthCheckGracePeriod: 300
      VPCZoneIdentifier:
        - !Ref PrivateSubnet1
        - !Ref PrivateSubnet2
      TargetGroupARNs:
        - !Ref TargetGroup
      TerminationPolicies:
        - OldestInstance
        - Default
      Tags:
        - Key: Name
          Value: !Sub '${AWS::StackName}-instance'
          PropagateAtLaunch: true

  ScaleUpPolicy:
    Type: AWS::AutoScaling::ScalingPolicy
    Properties:
      AdjustmentType: PercentChangeInCapacity
      PolicyType: StepScaling
      StepAdjustments:
        - MetricIntervalLowerBound: 0
          MetricIntervalUpperBound: 30
          ScalingAdjustment: 10
        - MetricIntervalLowerBound: 30
          ScalingAdjustment: 20
      TargetTrackingConfiguration:
        TargetValue: 70
        PredefinedMetricSpecification:
          PredefinedMetricType: ASGAverageCPUUtilization

  ScaleDownPolicy:
    Type: AWS::AutoScaling::ScalingPolicy
    Properties:
      AdjustmentType: PercentChangeInCapacity
      PolicyType: StepScaling
      StepAdjustments:
        - MetricIntervalUpperBound: -10
          MetricIntervalLowerBound: -100
          ScalingAdjustment: -5
      TargetTrackingConfiguration:
        TargetValue: 50
        PredefinedMetricSpecification:
          PredefinedMetricType: ASGAverageCPUUtilization
```

### 1.3 Infrastructure as Code (IaC)
**If it's not in code, it doesn't exist.**
- Version-controlled infrastructure
- Reproducible environments
- Peer review for changes
- Automated testing and deployment

**Terraform Module Structure:**

```hcl
# modules/networking/vpc/main.tf
variable "environment" {
  description = "Environment name (prod/staging/dev)"
  type        = string
}

variable "cidr_block" {
  description = " VPC CIDR block"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "List of AZs for subnets"
  type        = list(string)
  default     = ["us-east-1a", "us-east-1b", "us-east-1c"]
}

variable "private_subnet_cidrs" {
  description = "CIDR blocks for private subnets"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
}

variable "public_subnet_cidrs" {
  description = "CIDR blocks for public subnets"
  type        = list(string)
  default     = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]
}

resource "aws_vpc" "main" {
  cidr_block           = var.cidr_block
  enable_dns_hostnames = true
  enable_dns_support   = true
  
  tags = {
    Name        = "${var.environment}-vpc"
    Environment = var.environment
    ManagedBy   = "terraform"
  }
}

resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id

  tags = {
    Name        = "${var.environment}-igw"
    Environment = var.environment
  }
}

resource "aws_subnet" "private" {
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = var.private_subnet_cidrs[count.index]
  availability_zone = var.availability_zones[count.index]

  tags = {
    Name        = "${var.environment}-private-${var.availability_zones[count.index]}"
    Environment = var.environment
    Type        = "private"
  }
}

resource "aws_subnet" "public" {
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = var.public_subnet_cidrs[count.index]
  availability_zone = var.availability_zones[count.index]
  map_public_ip_on_launch = true

  tags = {
    Name        = "${var.environment}-public-${var.availability_zones[count.index]}"
    Environment = var.environment
    Type        = "public"
  }
}

resource "aws_nat_gateway" "main" {
  count         = length(var.availability_zones)
  subnet_id     = aws_subnet.public[count.index].id
  allocation_id = aws_eip.nat[count.index].id

  tags = {
    Name        = "${var.environment}-nat-${var.availability_zones[count.index]}"
    Environment = var.environment
  }
}

resource "aws_eip" "nat" {
  count  = length(var.availability_zones)
  domain = "vpc"

  tags = {
    Name        = "${var.environment}-eip-nat-${var.availability_zones[count.index]}"
    Environment = var.environment
  }
}

resource "aws_route_table" "private" {
  count  = length(var.availability_zones)
  vpc_id = aws_vpc.main.id

  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.main[count.index].id
  }

  tags = {
    Name        = "${var.environment}-rt-private-${var.availability_zones[count.index]}"
    Environment = var.environment
  }
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }

  tags = {
    Name        = "${var.environment}-rt-public"
    Environment = var.environment
  }
}

resource "aws_route_table_association" "private" {
  count          = length(var.availability_zones)
  subnet_id      = aws_subnet.private[count.index].id
  route_table_id = aws_route_table.private[count.index].id
}

resource "aws_route_table_association" "public" {
  count          = length(var.availability_zones)
  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

output "vpc_id" {
  value = aws_vpc.main.id
}

output "private_subnet_ids" {
  value = aws_subnet.private[*].id
}

output "public_subnet_ids" {
  value = aws_subnet.public[*].id
}
```

### 1.4 Cost Awareness
**Cloud costs are architecture decisions.**
- Visibility into spending
- Reserved capacity for steady-state
- Spot instances for fault-tolerant workloads
- Right-sizing resources

**Cost Allocation JSON Schema:**

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CostAllocation",
  "description": "Cost allocation tags for cloud resources",
  "type": "object",
  "required": ["service", "team", "environment", "component"],
  "properties": {
    "service": {
      "type": "string",
      "description": "Primary service this resource belongs to",
      "examples": ["api-gateway", "data-processing", "user-service"]
    },
    "team": {
      "type": "string",
      "description": "Team responsible for this resource",
      "examples": ["platform", "backend", "data"]
    },
    "environment": {
      "type": "string",
      "enum": ["production", "staging", "development", "testing"]
    },
    "component": {
      "type": "string",
      "description": "Specific component within the service",
      "examples": ["database", "cache", "queue", "compute"]
    },
    "cost-center": {
      "type": "string",
      "description": "Financial cost center code"
    },
    "owner": {
      "type": "string",
      "description": "Email or identifier of resource owner"
    }
  },
  "additionalProperties": true
}
```

### 1.5 Production Mindset
Cloud infrastructure decisions have direct business consequences. Apply the same rigor to infrastructure as to application code:

- **Unit economics are the architecture test:** If the cost to serve one customer exceeds the revenue they generate, the architecture is broken regardless of how elegantly it scales. Every architectural decision has a cost per unit; make it visible.
- **Portability is leverage, not ideology:** Full vendor lock-in is a negotiating failure. Using managed services accelerates delivery — that's the right trade — but core domain logic must remain portable enough to migrate within a reasonable window if vendor economics turn predatory.
- **Click-ops in production is a defect:** Infrastructure that was configured through a web console cannot be reviewed, versioned, tested, or recovered reliably. Every production state change must be expressed in code and promoted through the same review process as application changes.
- **Cost is an engineering signal, not a finance problem:** If an engineer cannot explain the cost impact of a PR, it cannot ship. Cloud spend is a direct output of architectural decisions; teams own that number.
- **Stateless compute is the default contract:** Any compute that accumulates local state breaks auto-scaling and complicates recovery. If an instance cannot be terminated safely at any moment, the system is brittle by design.
- **FaaS has a shape constraint:** Serverless functions are excellent for event-driven, bursty workloads. They are poor fits for consistent, high-throughput, latency-sensitive APIs where cold starts are visible and predictable resource allocation matters.
- **Least privilege is non-negotiable:** IAM roles must be scoped per service, per action, per resource. Wildcard permissions in production are a critical security defect. A compromised service must not be a pivot to adjacent systems.

---

## 2. Multi-Cloud Architecture

### 2.1 Multi-Cloud Strategy Matrix

| Strategy | Use Case | Complexity | Risk Reduction | Cost Impact |
|----------|----------|------------|----------------|-------------|
| Active-Active Multi-Region | DR, geo-distribution | High | Full failover | 2-3x compute |
| Active-Passive | DR only | Medium | Region failover | 1.2-1.5x compute |
| Multi-Provider | Vendor negotiation | Very High | Provider independence | 1.5-2x compute |
| Hybrid Cloud | Burst capacity | Medium | Variable | Pay-per-use |

### 2.2 EKS (AWS Kubernetes) Complete Configuration

```yaml
# eks-cluster.yaml - Complete EKS Cluster Definition
apiVersion: eksctl.io/v1alpha5
kind: ClusterConfig

metadata:
  name: production-cluster
  region: us-east-1
  version: "1.29"

iam:
  withOIDC: true
  serviceAccounts:
    - metadata:
        name: aws-load-balancer-controller
        namespace: kube-system
      wellKnownPolicies:
        awsLoadBalancerController: true
      roleName: ${CLUSTER_NAME}-aws-lb-controller
    - metadata:
        name: external-dns
        namespace: kube-system
      wellKnownPolicies:
        externalDNS: true
    - metadata:
        name: cluster-autoscaler
        namespace: kube-system
      wellKnownPolicies:
        autoScaler: true

addons:
  - name: vpc-cni
    version: latest
    configurationValues: |-
      enableNetworkPolicy = "true"
  - name: coredns
    version: latest
  - name: kube-proxy
    version: latest
  - name: aws-ebs-csi-driver
    serviceAccount:
      create: true
      name: aws-ebs-csi-driver
      wellKnownPolicies:
        ebsDriver: true

managedNodeGroups:
  - name: general-purpose
    instanceType: m6i.xlarge
    desiredCapacity: 3
    minSize: 2
    maxSize: 10
    volumeSize: 100
    volumeType: gp3
    privateNetworking: true
    securityGroups:
      attachIDs:
        - sg-mongodb
        - sg-redis
    labels:
      workload: general
    tags:
      Team: platform
      CostCenter: "12345"
    iam:
      withAddonPolicies:
        cloudWatch: true
        ebs: true
        fsx: true
        efs: true

  - name: memory-optimized
    instanceType: r6i.2xlarge
    desiredCapacity: 2
    minSize: 1
    maxSize: 5
    privateNetworking: true
    labels:
      workload: memory-intensive
      node-type: memory-optimized
    taints:
      - key: workload
        value: memory-intensive
        effect: NoSchedule

  - name: gpu-compute
    instanceType: g5.xlarge
    desiredCapacity: 0
    minSize: 0
    maxSize: 4
    privateNetworking: true
    labels:
      workload: ml-inference
      accelerator: nvidia
    taints:
      - key: nvidia.com/gpu
        value: present
        effect: NoSchedule

vpc:
  cidr: 10.50.0.0/16
  clusterEndpoints:
    publicAccess: true
    privateAccess: true
  nat:
    gateway: HighlyAvailable

cloudWatch:
  clusterLogging:
    enableTypes:
      - api
      - audit
      - authenticator
      - controllerManager
      - scheduler
```

### 2.3 GKE (Google Kubernetes Engine) Complete Configuration

```yaml
# gke-cluster.yaml - Complete GKE Cluster Definition
apiVersion: container.cnrm.cloud.google.com/v1beta1
kind: ContainerCluster
metadata:
  name: production-cluster
  namespace: config-control
  annotations:
    config.kubernetes.io/depends-on: compute.cnrm.cloud.google.com/projects/p producer-networking/v1beta1.Subnetwork/producer-us-central1
spec:
  projectRef:
    external: producer-project-123
  location: us-central1
  initialClusterVersion: "1.29"
  
  # Network Policy
  networkPolicy:
    provider: CALICO
    enabled: true
  
  # Network Config
  networkingMode: VPC_NATIVE
  networkRef:
    name: producer-vpc
  subnetworkRef:
    name: producer-us-central1
  
  # IP Allocation
  ipAllocationPolicy:
    clusterIpv4CidrBlock: /16
    servicesIpv4CidrBlock: /22
    clusterSecondaryRangeName: pods
    servicesSecondaryRangeName: services
  
  # Master Configuration
  masterAuth:
    clusterCaCertificate: |
      -----BEGIN CERTIFICATE-----
      ... (base64 encoded cert)
      -----END CERTIFICATE-----
    endpoint: "*.*.*.*"
  
  # Private Cluster
  privateClusterConfig:
    enablePrivateNodes: true
    enablePrivateEndpoint: true
    masterIpv4CidrBlock: 172.16.0.0/28
  
  # Node Pools
  nodeConfig:
    machineType: n2-standard-4
    diskSizeGb: 100
    diskType: pd-ssd
    imageType: COS_CONTAINERD
    serviceAccountRef:
      name: gke-node-sa
    
    shieldedInstanceConfig:
      enableSecureBoot: true
      enableIntegrityMonitoring: true
    
    gcfsConfig:
      enabled: true
    
    gvnic:
      enabled: true
    
    metadata:
      disable-legacy-endpoints: "true"
    
    oauthScopes:
      - https://www.googleapis.com/auth/cloud-platform
    
    labels:
      environment: production
  
  nodePools:
    - name: general
      config:
        machineType: n2-standard-4
        nodeCount: 3
        upgradeSettings:
          maxSurge: 1
          maxUnavailable: 0
      management:
        autoUpgrade: true
        autoRepair: true
      locations:
        - us-central1-a
        - us-central1-b
        - us-central1-c
    
    - name: memory-optimized
      config:
        machineType: n2-highmem-8
        nodeCount: 2
        upgradeSettings:
          maxSurge: 1
          maxUnavailable: 0
      management:
        autoUpgrade: true
        autoRepair: true
      locations:
        - us-central1-a
        - us-central1-b
  
  # Addons
  addonsConfig:
    horizontalPodAutoscaling:
      disabled: false
    networkPolicyConfig:
      disabled: false
    cloudDNSConfig:
      disabled: false
  
  # Workload Identity
  workloadIdentityConfig:
    workloadPool: producer-project-123.svc.id.goog
  
  # Binary Authorization
  binaryAuthorization:
    enabled: true
    evaluationMode: PROJECT_SINGLETON_POLICY_ENFORCE
  
  # Maintenance Window
  maintenanceWindow:
    dailyWindowStartTime: 03:00
    weeklyMaintenanceWindows:
      - day: SUNDAY
        startTime:
          hours: 3
          minutes: 0
```

### 2.4 AKS (Azure Kubernetes Service) Complete Configuration

```yaml
# aks-cluster.json - Complete AKS Cluster Definition
{
  "$schema": "https://azuremgmt.azureedge.net/schemas/2023-04-01/Microsoft.ContainerService.json",
  "type": "Microsoft.ContainerService/managedClusters",
  "apiVersion": "2023-04-01",
  "name": "production-aks",
  "location": "eastus",
  "dependsOn": [
    "Microsoft.Network/virtualNetworks/producer-vnet",
    "Microsoft.ManagedIdentity/userAssignedIdentities/producer-aks-identity"
  ],
  "identity": {
    "type": "UserAssigned",
    "userAssignedIdentities": {
      "/subscriptions/resourceGroups/producer-rg/providers/Microsoft.ManagedIdentity/userAssignedIdentities/producer-aks-identity": {}
    }
  },
  "properties": {
    "kubernetesVersion": "1.29",
    "dnsPrefix": "producer-aks",
    
    "agentPoolProfiles": [
      {
        "name": "general",
        "count": 3,
        "vmSize": "Standard_D4s_v3",
        "osDiskSizeGB": 100,
        "osDiskType": "Managed",
        "type": "VirtualMachineScaleSets",
        "availabilityZones": ["1", "2", "3"],
        "minCount": 2,
        "maxCount": 10,
        "scaleSetPriority": "Regular",
        "enableAutoScaling": true,
        "networkProfile": {
          "networkPlugin": "azure",
          "loadBalancerSku": "standard",
          "networkMode": "transparent"
        },
        "vnetSubnetId": "/subscriptions/resourceGroups/producer-rg/providers/Microsoft.Network/virtualNetworks/producer-vnet/subnets/aks-subnet",
        "osType": "Linux",
        "enableFIPS": true
      },
      {
        "name": "memoryoptimized",
        "count": 2,
        "vmSize": "Standard_E8s_v3",
        "osDiskSizeGB": 100,
        "type": "VirtualMachineScaleSets",
        "availabilityZones": ["1", "2"],
        "minCount": 1,
        "maxCount": 4,
        "enableAutoScaling": true,
        "nodeLabels": {
          "workload": "memory-intensive"
        },
        "nodeTaints": ["workload=memory-intensive:NoSchedule"],
        "vnetSubnetId": "/subscriptions/resourceGroups/producer-rg/providers/Microsoft.Network/virtualNetworks/producer-vnet/subnets/aks-subnet",
        "osType": "Linux"
      }
    ],
    
    "linuxProfile": {
      "adminUsername": "azureuser",
      "ssh": {
        "publicKeys": [
          {
            "keyData": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQD..."
          }
        ]
      }
    },
    
    "servicePrincipalProfile": {
      "clientId": "<sp-client-id>",
      "secret": "<sp-secret>"
    },
    
    "networkProfile": {
      "networkPlugin": "azure",
      "networkMode": "transparent",
      "loadBalancerSku": "standard",
      "loadBalancerProfile": {
        "managedOutboundIPs": {
          "count": 2
        },
        "effectiveOutboundIPs": [
          {
            "id": "/subscriptions/resourceGroups/producer-rg/providers/Microsoft.Network/publicIPAddresses/producer-lb-ip-1"
          },
          {
            "id": "/subscriptions/resourceGroups/producer-rg/providers/Microsoft.Network/publicIPAddresses/producer-lb-ip-2"
          }
        ]
      },
      "natGatewayProfile": {
        "mode": "Managed",
        "effectiveOutboundIPs": [
          {
            "id": "/subscriptions/resourceGroups/producer-rg/providers/Microsoft.Network/publicIPAddresses/producer-nat-ip"
          }
        ],
        "idleTimeoutInMinutes": 4
      },
      "podCidr": "10.244.0.0/16",
      "serviceCidr": "10.96.0.0/12",
      "dnsServiceIP": "10.96.0.10",
      "dockerBridgeCidr": "172.17.0.1/16"
    },
    
    "apiServerAccessProfile": {
      "enablePrivateCluster": true,
      "privateDNSZone": "/subscriptions/resourceGroups/producer-rg/providers/Microsoft.Network/privateDnsZones/privatelink.eastus.azmk8s.io",
      "enableRbac": true
    },
    
    "addonProfiles": {
      "omsAgent": {
        "enabled": true,
        "config": {
          "logAnalyticsWorkspaceResourceID": "/subscriptions/resourceGroups/producer-rg/providers/Microsoft.OperationalInsights/workspaces/producer-la"
        }
      },
      "azurePolicy": {
        "enabled": true
      },
      "azureKeyvaultSecretsProvider": {
        "enabled": true
      }
    },
    
    "sku": {
      "name": "Basic",
      "tier": "Paid"
    },
    
    "enableRBAC": true,
    
    "securityProfile": {
      "azureKeyVaultKms": {
        "enabled": true,
        "keyId": "/subscriptions/resourceGroups/producer-rg/providers/Microsoft.KeyVault/vaults/producer-kv/keys/aks-key"
      },
      "defender": {
        "logAnalyticsWorkspaceResourceId": "/subscriptions/resourceGroups/producer-rg/providers/Microsoft.OperationalInsights/workspaces/producer-la",
        "securityMonitoring": "enabled"
      }
    },
    
    "maintenanceConfiguration": {
      "timeInWeek": [
        {
          "day": "Sunday",
          "hoursSlots": [3, 4]
        }
      ]
    }
  }
}
```

---

## 3. Compute Options

### 3.1 Virtual Machines (IaaS) Decision Matrix

| Consideration | EC2 | GCE | Azure VMs |
|---------------|-----|-----|-----------|
| Instance types | 750+ | 100+ | 500+ |
| Pricing models | On-demand, Reserved, Spot, Savings Plans | On-demand, Committed Use, Spot | On-demand, Reserved, Spot |
| Bare metal | Nitro, Metal | Compute Engine Bare Metal | Bare Metal |
| Max vCPUs | 448 (c6i.48xlarge) | 416 (n2-highmem-416) | 416 (Standard192v3) |
| Max memory | 24 TB (x1e.48xlarge) | 8 TB (n1-ultramem-416) | 12 TB (m192sv2) |
| Local storage | NVMe, Instance Store | Local SSD | Temp storage, Lsv3 |
| Network performance | Up to 100 Gbps (EFA) | Up to 100 Gbps (T2A) | Up to 100 Gbps (HBv3) |

**When to use:**
- Legacy applications with specific kernel/OS requirements
- Long-running compute with consistent resource needs
- Applications that cannot be containerized
- Specific licensing requirements (Windows, Oracle, SAP)

### 3.2 Containers (CaaS) Specification

```yaml
# Container Deployment Specification (Kubernetes)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: production-api
  namespace: production
  labels:
    app: production-api
    version: v2.4.1
    environment: production
spec:
  replicas: 5
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 2
      maxUnavailable: 0
  selector:
    matchLabels:
      app: production-api
  template:
    metadata:
      labels:
        app: production-api
        version: v2.4.1
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: production-api-sa
      
      securityContext:
        runAsNonRoot: true
        runAsUser: 10000
        runAsGroup: 10000
        fsGroup: 10000
        seccompProfile:
          type: RuntimeDefault
      
      containers:
      - name: api
        image: registry.example.com/production/api:v2.4.1
        imagePullPolicy: Always
        
        ports:
        - name: http
          containerPort: 8080
          protocol: TCP
        - name: grpc
          containerPort: 9090
          protocol: TCP
        
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: api-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            configMapKeyRef:
              name: api-config
              key: redis-url
        - name: LOG_LEVEL
          value: "info"
        - name: OTEL_EXPORTER_OTLP_ENDPOINT
          value: "http://otel-collector:4317"
        
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "2000m"
            memory: "2Gi"
            ephemeral-storage: "1Gi"
        
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 15
          timeoutSeconds: 5
          failureThreshold: 3
        
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
          timeoutSeconds: 3
          failureThreshold: 3
          successThreshold: 1
        
        startupProbe:
          httpGet:
            path: /health/startup
            port: 8080
          initialDelaySeconds: 0
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 30
        
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
              - ALL
          volumeMounts:
          - name: tmp
            mountPath: /tmp
          - name: cache
            mountPath: /app/cache
        
        lifecycle:
          preStop:
            exec:
              command: ["/bin/sh", "-c", "sleep 10"]
      
      volumes:
      - name: tmp
        emptyDir:
          medium: Memory
          sizeLimit: 100Mi
      - name: cache
        emptyDir:
          medium: Memory
          sizeLimit: 512Mi
      
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchLabels:
                  app: production-api
              topologyKey: topology.kubernetes.io/zone
      
      topologySpreadConstraints:
      - maxSkew: 1
        topologyKey: topology.kubernetes.io/zone
        whenUnsatisfiable: DoNotSchedule
        labelSelector:
          matchLabels:
            app: production-api
```

### 3.3 Serverless (FaaS) Specification

```json
{
  "FunctionResourceSchema": {
    "type": "object",
    "required": ["runtime", "handler", "events"],
    "properties": {
      "name": {
        "type": "string",
        "description": "Function name (must be unique per region)",
        "pattern": "^[a-zA-Z][a-zA-Z0-9-_]{1,64}$"
      },
      "runtime": {
        "type": "string",
        "enum": ["nodejs20.x", "nodejs18.x", "python3.12", "python3.11", "java21", "java17", "go1.21", "go1.20", "ruby3.3", "ruby3.2", "dotnet8", "dotnet6", "provided.al2023", "provided.al2"],
        "description": "Execution runtime"
      },
      "handler": {
        "type": "string",
        "description": "Function entry point in format 'file.method' for NodeJS/Python, 'package.function' for Java/Go"
      },
      "memorySize": {
        "type": "integer",
        "minimum": 128,
        "maximum": 10240,
        "multipleOf": 64,
        "default": 128,
        "description": "Memory allocation in MB"
      },
      "timeout": {
        "type": "integer",
        "minimum": 1,
        "maximum": 900,
        "default": 3,
        "description": "Maximum execution time in seconds"
      },
      "ephemeralStorage": {
        "type": "integer",
        "minimum": 512,
        "maximum": 10240,
        "default": 512,
        "description": "Ephemeral storage in MB"
      },
      "environment": {
        "type": "object",
        "description": "Environment variables",
        "additionalProperties": {
          "type": "string"
        }
      },
      "events": {
        "type": "object",
        "description": "Event source mappings",
        "properties": {
          "http": {
            "type": "object",
            "properties": {
              "path": {
                "type": "string",
                "default": "/{proxy+}"
              },
              "method": {
                "type": "array",
                "items": {
                  "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"]
                }
              },
              "cors": {
                "type": "boolean",
                "default": false
              },
              "authorizer": {
                "type": "object",
                "properties": {
                  "type": {
                    "enum": ["request", "token", "cognito_user_pools"]
                  },
                  "arn": {
                    "type": "string"
                  },
                  "identitySource": {
                    "type": "array",
                    "items": {
                      "type": "string"
                    }
                  }
                }
              }
            }
          },
          "sqs": {
            "type": "object",
            "properties": {
              "queueArn": {
                "type": "string"
              },
              "batchSize": {
                "type": "integer",
                "minimum": 1,
                "maximum": 10,
                "default": 1
              },
              "scalingConfig": {
                "type": "object",
                "properties": {
                  "maximumConcurrency": {
                    "type": "integer",
                    "minimum": 2,
                    "maximum": 1000
                  }
                }
              }
            }
          },
          "s3": {
            "type": "object",
            "properties": {
              "bucket": {
                "type": "string"
              },
              "events": {
                "type": "array",
                "items": {
                  "enum": ["s3:ObjectCreated:*", "s3:ObjectRemoved:*", "s3:ObjectRestore:*"]
                }
              },
              "filter": {
                "type": "object",
                "properties": {
                  "key": {
                    "type": "object",
                    "properties": {
                      "filterRules": {
                        "type": "array",
                        "items": {
                          "type": "object",
                          "properties": {
                            "name": {
                              "enum": ["prefix", "suffix"]
                            },
                            "value": {
                              "type": "string"
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      },
      "vpcConfig": {
        "type": "object",
        "properties": {
          "subnetIds": {
            "type": "array",
            "items": {
              "type": "string"
            }
          },
          "securityGroupIds": {
            "type": "array",
            "items": {
              "type": "string"
            }
          }
        }
      },
      "reservedConcurrency": {
        "type": "integer",
        "minimum": 0,
        "description": "0 = reserved but not scaling; null = unreserved"
      },
      "provisionedConcurrency": {
        "type": "object",
        "properties": {
          "quantity": {
            "type": "integer",
            "minimum": 1
          },
          "statement": {
            "type": "string",
            "enum": ["ALL", "COUNT"]
          },
          "value": {
            "type": "integer"
          }
        }
      },
      "layers": {
        "type": "array",
        "items": {
          "type": "string"
        }
      },
      "deadLetterConfig": {
        "type": "object",
        "properties": {
          "targetArn": {
            "type": "string"
          }
        }
      },
      "tracing": {
        "type": "object",
        "properties": {
          "mode": {
            "enum": ["Active", "PassThrough"]
          }
        }
      },
      "policies": {
        "type": "array",
        "description": "IAM policies (inline or ARN references)",
        "items": {
          "type": "string"
        }
      }
    }
  }
}
```

**Cold Start Latency by Runtime:**

| Runtime | Cold Start (P50) | Cold Start (P99) | Notes |
|---------|------------------|------------------|-------|
| Node.js 20.x | ~50ms | ~200ms | With bundling |
| Python 3.12 | ~80ms | ~300ms | Standard |
| Java 21 | ~500ms | ~2000ms | JIT compilation |
| Go 1.21 | ~5ms | ~30ms | Compiled |
| .NET 8 | ~200ms | ~800ms | ReadyToRun helps |
| Ruby 3.3 | ~100ms | ~400ms | Rails lazy-load |

---

## 4. Deployment Patterns

### 4.1 Blue-Green Deployment Configuration

```yaml
# Blue-Green Deployment with AWS CodeDeploy
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-blue-green
  namespace: production
spec:
  replicas: 10
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 10
      maxUnavailable: 0

---
# Infrastructure: Blue-Green Setup
AWSTemplateFormatVersion: '2010-09-09'
Description: Blue-Green Deployment Infrastructure

Resources:
  # Blue Environment (Current)
  BlueTargetGroup:
    Type: AWS::ElasticLoadBalancingV2::TargetGroup
    Properties:
      Name: api-blue-tg
      Port: 8080
      Protocol: HTTP
      VpcId: !Ref VPC
      HealthCheckIntervalSeconds: 10
      HealthCheckPath: /health
      HealthCheckPort: 8080
      HealthCheckProtocol: HTTP
      HealthyThresholdCount: 2
      UnhealthyThresholdCount: 3
      TargetGroupAttributes:
        - Key: deregistration_delay.timeout_seconds
          Value: 30
        - Key: slow_start.duration_seconds
          Value: 60
      Targets:
        - Id: !Ref BlueASG
          Port: 8080

  # Green Environment (New)
  GreenTargetGroup:
    Type: AWS::ElasticLoadBalancingV2::TargetGroup
    Properties:
      Name: api-green-tg
      Port: 8080
      Protocol: HTTP
      VpcId: !Ref VPC
      HealthCheckIntervalSeconds: 10
      HealthCheckPath: /health
      HealthCheckPort: 8080
      TargetGroupAttributes:
        - Key: deregistration_delay.timeout_seconds
          Value: 30

  # Load Balancer Listener
  LoadBalancerListener:
    Type: AWS::ElasticLoadBalancingV2::Listener
    Properties:
      LoadBalancerArn: !Ref ApplicationLoadBalancer
      Port: 443
      Protocol: HTTPS
      Certificates:
        - CertificateArn: !Ref SSLCertificate
      DefaultActions:
        - Type: forward
          TargetGroupArn: !Ref BlueTargetGroup

  # Listener Rule for Testing
  GreenListenerRule:
    Type: AWS::ElasticLoadBalancingV2::ListenerRule
    Properties:
      ListenerArn: !Ref LoadBalancerListener
      Priority: 1
      Conditions:
        - Field: path-pattern
          Values:
            - /test-green/*
      Actions:
        - Type: forward
          TargetGroupArn: !Ref GreenTargetGroup

  # Auto Scaling Group Blue
  BlueASG:
    Type: AWS::AutoScaling::AutoScalingGroup
    Properties:
      LaunchTemplate:
        LaunchTemplateId: !Ref BlueLaunchTemplate
        Version: !GetAtt BlueLaunchTemplate.LatestVersionNumber
      MinSize: 4
      MaxSize: 20
      DesiredCapacity: 10
      VPCZoneIdentifier:
        - !Ref PrivateSubnet1
        - !Ref PrivateSubnet2
      TargetGroupARNs:
        - !Ref BlueTargetGroup
      HealthCheckType: ELB
      HealthCheckGracePeriod: 60
      TagSpecifications:
        - ResourceType: instance
          PropagateAtLaunch: true
          Tags:
            - Key: Name
              Value: api-blue

  # CodeDeploy Deployment Group
  DeploymentGroup:
    Type: AWS::CodeDeploy::DeploymentGroup
    Properties:
      ApplicationName: !Ref CodeDeployApplication
      DeploymentConfigName: CodeDeployDefault.HalfAtATime
      DeploymentGroupName: production-blue-green
      LoadBalancerInfo:
        TargetGroupInfoList:
          - !Ref GreenTargetGroup
      AutoScalingGroups:
        - !Ref GreenASG
      DeploymentStyle:
        DeploymentType: BLUE_GREEN
        DeploymentOption: WITH_TRAFFIC_DIFFERENTIAL
      Hooks:
        BeforeAllowTraffic:
          - HookName: !Ref BeforeAllowTrafficHook
            Revision: !Ref LambdaHookVersion
        AfterAllowTraffic:
          - HookName: !Ref AfterAllowTrafficHook
            Revision: !Ref LambdaHookVersion
```

### 4.2 Canary Deployment Specification

```yaml
# Canary Deployment Strategy (Argo Rollouts)
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: api-canary
  namespace: production
spec:
  replicas: 10
  selector:
    matchLabels:
      app: api
  strategy:
    canary:
      # Stable reference for rollback
      stableService: api-stable
      # Canary receives live traffic
      canaryService: api-canary
      
      # Traffic routing
      trafficRouting:
        plugins:
          argoproj-labs/istio:
            virtualService:
              name: api-vsvc
              routes:
                - primary
                - canary
      
      # Step-based progression
      steps:
        - setWeight: 5
        - pause: {duration: 10m}
        - analysis:
            templates:
              - templateName: success-rate
            args:
              - name: service-name
                value: api-canary
        - setWeight: 20
        - pause: {duration: 30m}
        - analysis:
            templates:
              - templateName: success-rate
              - templateName: latency
            args:
              - name: service-name
                value: api-canary
        - setWeight: 50
        - pause: {}
      
      # Analysis template
      analysis:
        successfulRunHistoryLimit: 3
        unsuccessfulRunHistoryLimit: 3
        template: success-rate
      
      # Pod template
      template:
        spec:
          containers:
          - name: api
            image: registry.example.com/api:v2.4.1
            ports:
            - containerPort: 8080
            resources:
              requests:
                cpu: "500m"
                memory: "512Mi"

  # Analysis templates referenced above
---
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: success-rate
spec:
  args:
    - name: service-name
  metrics:
    - name: success-rate
      interval: 2m
      successCondition: result[0] >= 0.95
      failureLimit: 3
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            sum(rate(http_requests_total{service="{{args.service-name}}",status!~"5.."}[5m]))
            /
            sum(rate(http_requests_total{service="{{args.service-name}}"}[5m]))
    
    - name: error-budget
      interval: 2m
      failureLimit: 1
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            1 - (
              sum(rate(http_requests_total{service="{{args.service-name}}",status=~"5.."}[5m]))
              /
              sum(rate(http_requests_total{service="{{args.service-name}}"}[5m]))
            )

---
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: latency
spec:
  args:
    - name: service-name
  metrics:
    - name: p99-latency
      interval: 2m
      successCondition: result[0] <= 500
      failureLimit: 3
      provider:
        prometheus:
          address: http://prometheus:9090
          query: |
            histogram_quantile(0.99,
              sum(rate(http_request_duration_ms_bucket{service="{{args.service-name}}"}[5m])) by (le)
            )
```

### 4.3 Feature Flags Specification

```json
{
  "FeatureFlagSchema": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "FeatureFlag",
    "type": "object",
    "required": ["key", "flagType", "enabled", "rules"],
    "properties": {
      "key": {
        "type": "string",
        "pattern": "^[a-z][a-z0-9_]{1,62}[a-z0-9]$",
        "examples": ["new_checkout_flow", "ml_recommendations", "a_b_test_variant"]
      },
      "name": {
        "type": "string",
        "description": "Human-readable name"
      },
      "description": {
        "type": "string",
        "description": "Detailed description of what this flag controls"
      },
      "flagType": {
        "type": "string",
        "enum": ["boolean", "string", "number", "json"],
        "description": "Type of the flag value"
      },
      "enabled": {
        "type": "boolean",
        "description": "Master switch for this feature"
      },
      "defaultValue": {
        "type": ["boolean", "string", "number", "null"],
        "description": "Value returned when no rules match"
      },
      "rules": {
        "type": "array",
        "items": {
          "type": "object",
          "required": ["name", "conditions", "value"],
          "properties": {
            "name": {
              "type": "string",
              "description": "Rule name for logging/audit"
            },
            "priority": {
              "type": "integer",
              "minimum": 0,
              "description": "Lower = evaluated first"
            },
            "conditions": {
              "type": "array",
              "items": {
                "type": "object",
                "required": ["attribute", "operator", "value"],
                "properties": {
                  "attribute": {
                    "type": "string",
                    "enum": [
                      "user_id", "email", "country", "region", 
                      "device_type", "browser", "app_version",
                      "percentage", "timestamp", "environment"
                    ]
                  },
                  "operator": {
                    "type": "string",
                    "enum": [
                      "eq", "neq", "in", "not_in", 
                      "contains", "starts_with", "ends_with",
                      "gte", "lte", "gt", "lt",
                      "matches_regex", "semver_eq", "semver_gte"
                    ]
                  },
                  "value": {
                    "type": ["string", "number", "boolean", "array"]
                  },
                  "attributeType": {
                    "type": "string",
                    "enum": ["string", "number", "boolean", "date"]
                  }
                }
              }
            },
            "value": {
              "type": ["boolean", "string", "number", "null"]
            },
            "rolloutPercentage": {
              "type": "number",
              "minimum": 0,
              "maximum": 100,
              "description": "Percentage of users matching conditions who get this value"
            },
            "seed": {
              "type": "integer",
              "description": "Deterministic randomization seed for percentage rollout"
            }
          }
        }
      },
      "metadata": {
        "type": "object",
        "properties": {
          "owner": {
            "type": "string",
            "description": "Team or person responsible"
          },
          "tickets": {
            "type": "array",
            "items": {
              "type": "string"
            }
          },
          "requiresApproval": {
            "type": "boolean",
            "default": false
          },
          "expirationDate": {
            "type": "string",
            "format": "date-time"
          },
          "tags": {
            "type": "array",
            "items": {
              "type": "string"
            }
          }
        }
      }
    }
  }
}
```

**Feature Flag Evaluation Example:**

```json
{
  "key": "new_pricing_engine",
  "flagType": "boolean",
  "enabled": true,
  "defaultValue": false,
  "rules": [
    {
      "name": "Internal users (bypass)",
      "priority": 0,
      "conditions": [
        {
          "attribute": "email",
          "operator": "matches_regex",
          "value": ".*@example\\.com$"
        }
      ],
      "value": true
    },
    {
      "name": "Beta users opt-in",
      "priority": 1,
      "conditions": [
        {
          "attribute": "user_id",
          "operator": "in",
          "value": ["user_123", "user_456", "user_789"]
        }
      ],
      "value": true
    },
    {
      "name": "10% rollout to US users",
      "priority": 2,
      "conditions": [
        {
          "attribute": "country",
          "operator": "eq",
          "value": "US"
        }
      ],
      "value": true,
      "rolloutPercentage": 10,
      "seed": 42
    },
    {
      "name": "5% global rollout",
      "priority": 3,
      "conditions": [],
      "value": true,
      "rolloutPercentage": 5,
      "seed": 42
    }
  ],
  "metadata": {
    "owner": "pricing-team",
    "tickets": ["PRICING-123", "PRICING-456"],
    "expirationDate": "2026-06-01T00:00:00Z",
    "tags": ["pricing", "experiment", "high-risk"]
  }
}
```

---

## 5. High Availability Patterns

### 5.1 Multi-AZ Architecture

```
                    ┌─────────────────────────────────────────────┐
                    │              Auto Scaling Group              │
                    │  ┌───────────┐  ┌───────────┐  ┌───────────┐│
                    │  │  AZ-1     │  │  AZ-2     │  │  AZ-3     ││
                    │  │ ┌───────┐ │  │ ┌───────┐ │  │ ┌───────┐ ││
                    │  │ │Instance│ │  │ │Instance│ │  │ │Instance│ ││
                    │  │ │  1-a   │ │  │ │  2-b   │ │  │ │  3-c   │ ││
                    │  │ └───────┘ │  │ └───────┘ │  │ └───────┘ ││
                    │  └───────────┘  └───────────┘  └───────────┘│
                    └─────────────────────────────────────────────┘
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    │                     │                     │
              ┌─────┴─────┐         ┌─────┴─────┐         ┌─────┴─────┐
              │   NAT     │         │   NAT     │         │   NAT     │
              │  Gateway  │         │  Gateway  │         │  Gateway  │
              │   AZ-1    │         │   AZ-2    │         │   AZ-3    │
              └─────┬─────┘         └─────┬─────┘         └─────┬─────┘
                    │                     │                     │
              ┌─────┴─────────────────────┴─────────────────────┴─────┐
              │                    Internet Gateway                    │
              └───────────────────────────────────────────────────────┘
```

### 5.2 Health Check Specification

```yaml
# Health Check Configuration Schema
HealthCheckConfiguration:
  type: object
  required:
    - liveness
    - readiness
    - startup
  properties:
    liveness:
      type: object
      description: Is the process running and not deadlocked?
      required:
        - path
        - port
      properties:
        path:
          type: string
          description: HTTP path for liveness probe
          examples: ["/health/live", "/api/healthz"]
        port:
          type: integer
          minimum: 1
          maximum: 65535
        initialDelaySeconds:
          type: integer
          minimum: 0
          default: 0
          description: Initial delay before first check
        periodSeconds:
          type: integer
          minimum: 1
          default: 10
        timeoutSeconds:
          type: integer
          minimum: 1
          default: 5
        failureThreshold:
          type: integer
          minimum: 1
          default: 3
          description: Consecutive failures before restart
        successThreshold:
          type: integer
          minimum: 1
          default: 1
        httpHeaders:
          type: array
          items:
            type: object
            properties:
              name:
                type: string
              value:
                type: string
    
    readiness:
      type: object
      description: Is it ready to serve traffic?
      required:
        - path
        - port
      properties:
        path:
          type: string
          default: "/health/ready"
        port:
          type: integer
          default: 8080
        initialDelaySeconds:
          type: integer
          default: 5
        periodSeconds:
          type: integer
          default: 10
        failureThreshold:
          type: integer
          default: 3
        httpHeaders:
          type: array
        # Exclude from load balancer for initial delay
        initialDelaySeconds: 10
    
    startup:
      type: object
      description: Is initialization complete?
      properties:
        path:
          type: string
          default: "/health/startup"
        port:
          type: integer
          default: 8080
        initialDelaySeconds:
          type: integer
          default: 0
        periodSeconds:
          type: integer
          default: 5
        failureThreshold:
          type: integer
          description: "Total wait = failureThreshold × periodSeconds"
          default: 30
          examples: [30]  # 30 × 5s = 150s max startup time
```

---

## 6. Storage Patterns

### 6.1 S3 Storage Configuration

```yaml
# S3 Bucket Configuration
AWSTemplateFormatVersion: '2010-09-09'
Description: S3 Bucket with proper configuration

Resources:
  # Data Bucket
  DataBucket:
    Type: AWS::S3::Bucket
    DeletionPolicy: Retain
    Properties:
      BucketName: !Sub '${AWS::StackName}-data-${AWS::Region}'
      
      # Ownership controls
      OwnershipControls:
        Rules:
          - ObjectOwnership: BucketOwnerPreferred
      
      # Public Access Block
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        IgnorePublicAcls: true
        BlockPublicPolicy: true
        RestrictPublicBuckets: true
      
      # Versioning
      VersioningConfiguration:
        Status: Enabled
      
      # Encryption
      BucketEncryption:
        ServerSideEncryptionConfiguration:
          - ServerSideEncryptionByDefault:
              SSEAlgorithm: AES256
            BucketKeyEnabled: true
      
      # Lifecycle Rules
      LifecycleConfiguration:
        Rules:
          - ID: Move-to-IA-after-30
            Status: Enabled
            Filter:
              Prefix: "processed/"
            Transitions:
              - Days: 30
                StorageClass: STANDARD_IA
              - Days: 90
                StorageClass: GLACIER
              - Days: 365
                StorageClass: DEEP_ARCHIVE
            ExpirationInDays: 2555  # 7 years
          
          - ID: Delete-incomplete-uploads
            Status: Enabled
            AbortIncompleteMultipartUpload:
              DaysAfterInitiation: 7
          
          - ID: Current-version-rules
            Status: Enabled
            NoncurrentVersionTransitions:
              - NoncurrentDays: 30
                StorageClass: STANDARD_IA
              - NoncurrentDays: 90
                StorageClass: GLACIER
            NoncurrentVersionExpirationInDays: 365
      
      # CORS
      CorsConfiguration:
        CorsRules:
          - AllowedHeaders:
              - "*"
            AllowedMethods:
              - GET
              - HEAD
            AllowedOrigins:
              - "https://app.example.com"
              - "https://admin.example.com"
            ExposedHeaders:
              - x-amz-request-id
              - x-amz-id-2
            MaxAge: 3600
      
      # Logging
      LoggingConfiguration:
        DestinationBucketName: !Ref AccessLogBucket
        LogFilePrefix: !Sub '${AWS::StackName}/data-bucket/'
      
      # Tags
      Tags:
        - Key: Environment
          Value: production
        - Key: DataClassification
          Value: internal
        - Key: Owner
          Value: platform-team
      
  # Access Log Bucket (separate from data)
  AccessLogBucket:
    Type: AWS::S3::Bucket
    DeletionPolicy: Retain
    Properties:
      BucketName: !Sub '${AWS::StackName}-access-logs-${AWS::Region}'
      PublicAccessBlockConfiguration:
        BlockPublicAcls: true
        IgnorePublicAcls: true
        BlockPublicPolicy: true
        RestrictPublicBuckets: true
      
      # Short retention for access logs
      LifecycleConfiguration:
        Rules:
          - ID: Delete-logs-after-90
            Status: Enabled
            ExpirationInDays: 90
      
  # Bucket Policy - Restrict access
  DataBucketPolicy:
    Type: AWS::S3::BucketPolicy
    Properties:
      Bucket: !Ref DataBucket
      PolicyDocument:
        Version: "2012-10-17"
        Statement:
          - Sid: EnforceTLS
            Effect: Deny
            Principal:
              AWS: "*"
            Action:
              - "s3:*"
            Resource:
              - !Sub '${DataBucket.Arn}/*'
              - !Sub '${DataBucket.Arn}'
            Condition:
              Bool:
                aws:SecureTransport: false
          
          - Sid: RestrictToVPCEndpoints
            Effect: Deny
            Principal: "*"
            Action:
              - "s3:GetObject"
              - "s3:PutObject"
            Resource:
              - !Sub '${DataBucket.Arn}/*'
            Condition:
              StringNotEquals:
                aws:SourceVpce: !Ref VPCEndpoint
          
          - Sid: RequireOwnerTag
            Effect: Deny
            Principal: "*"
            Action:
              - "s3:DeleteObject"
              - "s3:DeleteObjectVersion"
            Resource:
              - !Sub '${DataBucket.Arn}/*'
            Condition:
              StringNotEquals:
                s3:RequestObjectTagOwner: !Ref AWS::StackName
```

### 6.2 Block Storage Types Comparison

| Type | AWS | GCP | Azure | Use Case | IOPS/GB | Throughput |
|------|-----|-----|-------|----------|---------|------------|
| GP3 (SSD) | EBS gp3 | Balanced PD | Standard SSD | General purpose | 3,000 min | 125 MB/s min |
| GP3 (SSD) | EBS gp3 | Balanced PD | Standard SSD | Scale up to | 16,000 max | 1,000 MB/s max |
| GP2 (SSD) | EBS gp2 | Balanced PD | Standard SSD | Legacy workloads | 3,000 | 250 MB/s |
| IO2 (SSD) | EBS io2 | SSD PD | Premium SSD | High IOPS | 500 per GB | 1,000 MB/s |
| IO2 (SSD) | EBS io2 | SSD PD | Premium SSD | Max per volume | 64,000 | 1,000 MB/s |
| IO2 Block Express | EBS io2 | - | - | Ultra-low latency | 256,000 | 4,000 MB/s |
| ST1 (HDD) | EBS st1 | - | Standard HDD | Throughput heavy | 500 | 500 MB/s |
| SC1 (HDD) | EBS sc1 | - | - | Cold storage | 250 | 250 MB/s |
| Local NVMe | Instance Store | Local SSD | Ephemeral | Cache, temp | Per-instance | Per-instance |

---

## 7. Networking

### 7.1 VPC Architecture

```
┌────────────────────────────────────────────────────────────────────────────┐
│                              VPC: 10.50.0.0/16                             │
│                                                                            │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        Public Subnets (10.50.0.0/24)                  │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                    │  │
│  │  │   AZ-1     │  │   AZ-2     │  │   AZ-3     │                    │  │
│  │  │ 10.50.1.0/24│  │ 10.50.2.0/24│  │ 10.50.3.0/24│                    │  │
│  │  │             │  │             │  │             │                    │  │
│  │  │ ┌─────────┐ │  │ ┌─────────┐ │  │ ┌─────────┐ │                    │  │
│  │  │ │  NAT    │ │  │ │  NAT    │ │  │ │  NAT    │ │                    │  │
│  │  │ │ Gateway │ │  │ │ Gateway │ │  │ │ Gateway │ │                    │  │
│  │  │ └─────────┘ │  │ └─────────┘ │  │ └─────────┘ │                    │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                    │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                            │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                     Private Subnets (10.50.101.0/24)                  │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                    │  │
│  │  │   AZ-1     │  │   AZ-2     │  │   AZ-3     │                    │  │
│  │  │10.50.101.0/│  │10.50.102.0/│  │10.50.103.0/│                    │  │
│  │  │    24      │  │    24      │  │    24      │                    │  │
│  │  │            │  │            │  │            │                    │  │
│  │  │ ┌────────┐ │  │ ┌────────┐ │  │ ┌────────┐ │                    │  │
│  │  │ │ EKS    │ │  │ │ EKS    │ │  │ │ EKS    │ │                    │  │
│  │  │ │ Nodes  │ │  │ │ Nodes  │ │  │ │ Nodes  │ │                    │  │
│  │  │ └────────┘ │  │ └────────┘ │  │ └────────┘ │                    │  │
│  │  │ ┌────────┐ │  │ ┌────────┐ │  │ ┌────────┐ │                    │  │
│  │  │ │  RDS   │ │  │ │  RDS   │ │  │ │  RDS   │ │                    │  │
│  │  │ │Primary │ │  │ │Replica │ │  │ │Replica │ │                    │  │
│  │  │ └────────┘ │  │ └────────┘ │  │ └────────┘ │                    │  │
│  │  │ ┌────────┐ │  │ ┌────────┐ │  │ ┌────────┐ │                    │  │
│  │  │ │ Redis  │ │  │ │ Redis  │ │  │ │ Redis  │ │                    │  │
│  │  │ │Cluster │ │  │ │Cluster │ │  │ │Cluster │ │                    │  │
│  │  │ └────────┘ │  │ └────────┘ │  │ └────────┘ │                    │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                    │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                            │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                     Isolated Subnets (10.50.201.0/24)                 │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                    │  │
│  │  │   AZ-1     │  │   AZ-2     │  │   AZ-3     │                    │  │
│  │  │10.50.201.0/│  │10.50.202.0/│  │10.50.203.0/│                    │  │
│  │  │    24      │  │    24      │  │    24      │                    │  │
│  │  │            │  │            │  │            │                    │  │
│  │  │ ┌────────┐ │  │ ┌────────┐ │  │ ┌────────┐ │                    │  │
│  │  │ │Elasti   │ │  │ │Elasti  │ │  │ │Elasti  │ │                    │  │
│  │  │ │Cache    │ │  │ │Cache   │ │  │ │Cache   │ │                    │  │
│  │  │ └────────┘ │  │ └────────┘ │  │ └────────┘ │                    │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                    │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                            │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                         VPC Endpoints (S3, DynamoDB)                    │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Security Group Specification

```json
{
  "SecurityGroupSchema": {
    "type": "object",
    "required": ["name", "vpcId", "rules"],
    "properties": {
      "name": {
        "type": "string",
        "pattern": "^[a-zA-Z0-9._-]{1,255}$"
      },
      "vpcId": {
        "type": "string",
        "description": "VPC ID this security group belongs to"
      },
      "description": {
        "type": "string",
        "maxLength": 255
      },
      "tags": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "key": {
              "type": "string"
            },
            "value": {
              "type": "string"
            }
          }
        }
      },
      "rules": {
        "type": "object",
        "properties": {
          "ingress": {
            "type": "array",
            "items": {
              "type": "object",
              "required": ["protocol", "fromPort", "toPort"],
              "properties": {
                "description": {
                  "type": "string"
                },
                "protocol": {
                  "type": "string",
                  "enum": ["tcp", "udp", "icmp", "icmpv6", "-1", "all"]
                },
                "fromPort": {
                  "type": "integer",
                  "minimum": -1,
                  "maximum": 65535
                },
                "toPort": {
                  "type": "integer",
                  "minimum": -1,
                  "maximum": 65535
                },
                "sourceSecurityGroupId": {
                  "type": "string"
                },
                "sourcePrefixListId": {
                  "type": "string"
                },
                "cidrIpv4": {
                  "type": "string",
                  "pattern": "^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)/(3[0-2]|[12]?[0-9])$"
                },
                "cidrIpv6": {
                  "type": "string"
                }
              }
            }
          },
          "egress": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "description": {
                  "type": "string"
                },
                "protocol": {
                  "type": "string"
                },
                "fromPort": {
                  "type": "integer"
                },
                "toPort": {
                  "type": "integer"
                },
                "destinationSecurityGroupId": {
                  "type": "string"
                },
                "destinationPrefixListId": {
                  "type": "string"
                },
                "cidrIpv4": {
                  "type": "string"
                },
                "cidrIpv6": {
                  "type": "string"
                }
              }
            }
          }
        }
      }
    }
  }
}
```

**Example Security Groups (JSON):**

```json
{
  "securityGroups": [
    {
      "name": "eks-nodes-sg",
      "description": "Security group for EKS worker nodes",
      "vpcId": "vpc-0123456789abcdef0",
      "rules": {
        "ingress": [
          {
            "description": "Worker to worker communication",
            "protocol": "all",
            "sourceSecurityGroupId": "sg-0123456789abcdef0"
          },
          {
            "description": "Allow traffic from Control Plane",
            "protocol": "tcp",
            "fromPort": 10250,
            "toPort": 10250,
            "sourceSecurityGroupId": "sg-control-plane"
          },
          {
            "description": "Kubelet API",
            "protocol": "tcp",
            "fromPort": 10250,
            "toPort": 10250,
            "sourceSecurityGroupId": "sg-control-plane"
          },
          {
            "description": "NodePort services",
            "protocol": "tcp",
            "fromPort": 30000,
            "toPort": 32767,
            "cidrIpv4": "10.50.0.0/16"
          }
        ],
        "egress": [
          {
            "description": "Allow all outbound",
            "protocol": "-1",
            "cidrIpv4": "0.0.0.0/0"
          }
        ]
      }
    },
    {
      "name": "api-service-sg",
      "description": "Security group for API service",
      "vpcId": "vpc-0123456789abcdef0",
      "rules": {
        "ingress": [
          {
            "description": "HTTP from ALB",
            "protocol": "tcp",
            "fromPort": 8080,
            "toPort": 8080,
            "sourceSecurityGroupId": "sg-alb"
          },
          {
            "description": "HTTPS from ALB",
            "protocol": "tcp",
            "fromPort": 8443,
            "toPort": 8443,
            "sourceSecurityGroupId": "sg-alb"
          }
        ],
        "egress": [
          {
            "description": "To PostgreSQL",
            "protocol": "tcp",
            "fromPort": 5432,
            "toPort": 5432,
            "sourceSecurityGroupId": "sg-postgres"
          },
          {
            "description": "To Redis",
            "protocol": "tcp",
            "fromPort": 6379,
            "toPort": 6379,
            "sourceSecurityGroupId": "sg-redis"
          },
          {
            "description": "To S3 via VPC Endpoint",
            "protocol": "tcp",
            "fromPort": 443,
            "toPort": 443,
            "destinationPrefixListId": "pl-0123456789abcdef0"
          }
        ]
      }
    }
  ]
}
```

---

## 8. Service Mesh Configuration

### 8.1 Istio Service Mesh Configuration

```yaml
# istio-config.yaml - Comprehensive Istio Service Mesh Configuration
apiVersion: install.istio.io/v1alpha1
kind: IstioOperator
metadata:
  name: production-istio
  namespace: istio-system
spec:
  profile: default
  
  # Component configuration
  components:
    pilot:
      k8s:
        resources:
          requests:
            cpu: "500m"
            memory: "2Gi"
          limits:
            cpu: "4000m"
            memory: "16Gi"
        replicaCount: 3
    
    ingressGateways:
      - name: istio-ingressgateway
        enabled: true
        k8s:
          resources:
            requests:
              cpu: "200m"
              memory: "256Mi"
            limits:
              cpu: "2000m"
              memory: "4Gi"
          service:
            ports:
              - name: http
                port: 80
                targetPort: 8080
              - name: https
                port: 443
                targetPort: 8443
              - name: istio-mtls
                port: 15443
          hpaSpec:
            minReplicas: 2
            maxReplicas: 10
            metrics:
              - type: Resource
                resource:
                  name: cpu
                  targetAverageUtilization: 70
    
    egressGateways:
      - name: istio-egressgateway
        enabled: true
        k8s:
          resources:
            requests:
              cpu: "100m"
              memory: "128Mi"
            limits:
              cpu: "1000m"
              memory: "1Gi"
          hpaSpec:
            minReplicas: 2
            maxReplicas: 5
  
  # Network configuration
  meshConfig:
    # Enable auto mTLS
    enableAutoMtls: true
    
    # Default protocol detection
    defaultConfig:
      proxyMetadata:
        ISTIO_META_DNS_CAPTURE: "true"
        ISTIO_META_DNS_AUTO_ALLOCATE: "true"
      
      # Tracing configuration
      tracing:
        sampling: 1.0
        zipkin:
          address: jaeger-collector.observability:9411
      
      # Access log configuration
      accessLogFile: /dev/stdout
      accessLogFormat: |
        "[%START_TIME%] %REQ(:METHOD)% %REQ(:AUTHORITY)% %RESPONSE_FLAGS% %DURATION% %RESP(status)%"
      
      # Mixer policy
      disableMixerHttpReports: true
    
    # Outbound traffic policy
    outboundTrafficPolicy:
      mode: ALLOW_ANY
    
    # Locality load balancing
    localityLbSetting:
      enabled: true
      failover:
        - from: us-east-1a
          to: us-east-1b
        - from: us-east-1b
          to: us-east-1c
        - from: us-east-1c
          to: us-east-1a
  
  # Security configuration
  security:
    enabled: true
    createRootSecret: true

---
# Authorization Policy - Zero Trust Network
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: api-service-authz
  namespace: production
spec:
  selector:
    matchLabels:
      app: api-service
  action: ALLOW
  rules:
    # Allow only from ingress gateway and services in same namespace
    - from:
        - source:
            principals:
              - "cluster.local/ns/production/sa/api-service"
        - source:
            namespaces: ["production"]
      to:
        - operation:
            methods: ["GET", "POST"]
            paths: ["/api/*"]
    
    # Allow health checks
    - from:
        - source:
            principals: []
      to:
        - operation:
            paths: ["/health", "/healthz", "/ready", "/live"]

---
# PeerAuthentication - Enforce mTLS
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: production
spec:
  mtls:
    mode: STRICT

---
# DestinationRules - Circuit Breaker & Retry Policy
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: api-service
  namespace: production
spec:
  host: api-service.production.svc.cluster.local
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
        connectTimeout: 10s
      http:
        h2UpgradePolicy: UPGRADE
        http1MaxPendingRequests: 100
        http2MaxRequests: 1000
        maxRequestsPerConnection: 100
        maxRetries: 3
    
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
      minHealthPercent: 30
    
    loadBalancer:
      simple: LEAST_REQUEST
      localityLbSetting:
        enabled: true
        distribute:
          - from: us-east-1a/*
            to:
              "us-east-1a/*": 80
              "us-east-1b/*": 20
    
    tls:
      mode: ISTIO_MUTUAL
      clientCertificate: /etc/certs/cert-chain.pem
      privateKey: /etc/certs/key.pem
      caCertificates: /etc/certs/root-cert.pem
```

---

## 9. Anti-Patterns and Failure Modes

### 9.1 Critical Anti-Patterns

| Anti-Pattern | Failure Mode | Detection | Prevention |
|--------------|--------------|-----------|------------|
| **Lift and shift without optimization** | No cloud benefits, same cost as on-prem | Cost analysis | Re-architect for cloud-native |
| **Giant VM (vertical scaling)** | Instance limits, single point of failure | Max instance size reached | Design for horizontal scaling |
| **No automation (click-ops)** | Configuration drift, unrecoverable failures | Manual changes in audit logs | All changes via IaC only |
| **Hardcoded credentials** | Security breach, credential rotation impossible | Secrets scanner in CI | Use secrets manager |
| **Public S3 bucket** | Data breach, compliance violation | AWS Config rule | Block public access enabled |
| **No monitoring** | Outages without detection, SLA violation | Customer complaints | CloudWatch/Datadog mandatory |
| **Single AZ deployment** | AZ failure = total outage | None | Multi-AZ minimum |
| **Over-provisioning** | Wasted spend, poor resource utilization | Cost anomaly | Right-size with metrics |
| **No IaC** | Snowflake servers, unreproducible | Manual changes detected | Terraform/GitOps mandatory |
| **Ignoring costs** | Budget overruns, surprise bills | Monthly bill shock | Cost alerts, budget tracking |

### 9.2 Specific Failure Mode Analysis

**Scenario: Cache Stampede**
```
Problem: Cache expires → 1000 requests hit database simultaneously
Impact: Database overload, P99 latency spikes, potential cascade failure
Root Cause: No protection against thundering herd on shared cache key

Prevention Patterns:
1. Per-item TTL jitter (±10% random offset)
2. Probabilistic early expiration (1% chance before actual TTL)
3. Lock-based recomputation (single rebuild, others wait)
4. Background refresh (stale-while-revalidate)
```

**Scenario: AZ Failure Without Redundancy**
```
Problem: Single-AZ deployment loses one availability zone
Impact: Service-wide outage, zero fault tolerance
Root Cause: Cost savings by deploying to single AZ

Prevention:
1. Multi-AZ deployment mandatory for production
2. Health checks detect and remove unhealthy instances
3. ASG spans all configured AZs
4. Database configured for multi-AZ (automatic failover)
```

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/KUBERNETES.md` - Kubernetes patterns
- `architecture/SECURITY.md` - Security architecture
- `architecture/OBSERVABILITY.md` - Monitoring and observability
- `architecture/CONCURRENCY.md` - Distributed systems patterns

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition
- `specs/SECURITY.md` - Security doctrine

### Interface Contracts
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/CONTROL_PLANE.md` - Agent sequencing
- `interfaces/STORE_MODEL.md` - Store semantics

### Methodology
- `methodology/ARCHITECTURE.md` - Architecture methodology
- `methodology/CLOUD_ADOPT.md` - Cloud adoption patterns