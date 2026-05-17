# DR.md - Disaster Recovery Architecture Standards

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

---

## 1. Backup Strategies

### 1.1 Database Backup Implementation

```yaml
# kubernetes/database-backup.yaml - Complete backup configuration

apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
  namespace: database
spec:
  schedule: "0 2 * * *"  # 2 AM daily
  successfulJobsHistoryLimit: 7
  failedJobsHistoryLimit: 3
  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      backoffLimit: 3
      template:
        spec:
          serviceAccountName: backup-service
          containers:
            - name: backup
              image: postgres:15-alpine
              command:
                - sh
                - -c
                - |
                  set -e
                  
                  # Configuration
                  TIMESTAMP=$(date +%Y%m%d_%H%M%S)
                  BACKUP_DIR="/backups"
                  RETENTION_DAYS=30
                  
                  # Database connection
                  export PGHOST=${DB_HOST}
                  export PGPORT=${DB_PORT}
                  export PGUSER=${DB_USER}
                  export PGPASSWORD=${DB_PASSWORD}
                  export PGDATABASE=${DB_NAME}
                  
                  # Create backup directory
                  mkdir -p ${BACKUP_DIR}
                  
                  # Perform backup with compression
                  echo "Starting backup at $(date)"
                  
                  # Full database backup
                  pg_dump -Fc -f ${BACKUP_DIR}/full_backup_${TIMESTAMP}.dump
                  
                  # Schema only backup
                  pg_dump --schema-only -f ${BACKUP_DIR}/schema_${TIMESTAMP}.sql
                  
                  # Calculate checksum
                  sha256sum ${BACKUP_DIR}/full_backup_${TIMESTAMP}.dump > ${BACKUP_DIR}/full_backup_${TIMESTAMP}.dump.sha256
                  
                  # Upload to object storage
                  aws s3 cp ${BACKUP_DIR}/full_backup_${TIMESTAMP}.dump s3://${BACKUP_BUCKET}/postgres/
                  aws s3 cp ${BACKUP_DIR}/schema_${TIMESTAMP}.sql s3://${BACKUP_BUCKET}/postgres/schema/
                  aws s3 cp ${BACKUP_DIR}/full_backup_${TIMESTAMP}.dump.sha256 s3://${BACKUP_BUCKET}/postgres/checksums/
                  
                  # Cleanup old local backups
                  find ${BACKUP_DIR} -type f -mtime +${RETENTION_DAYS} -delete
                  
                  # Cleanup old S3 backups
                  aws s3api list-objects \
                    --bucket ${BACKUP_BUCKET} \
                    --prefix postgres/ \
                    --query 'Contents[?LastModified<`'$(date -d "-${RETENTION_DAYS} days" -I)'`]' \
                    --output text \
                    | xargs -r aws s3 rm
                  
                  echo "Backup completed at $(date)"
                  
              env:
                - name: DB_HOST
                  valueFrom:
                    secretKeyRef:
                      name: postgres-secrets
                      key: host
                - name: DB_PORT
                  value: "5432"
                - name: DB_USER
                  valueFrom:
                    secretKeyRef:
                      name: postgres-secrets
                      key: username
                - name: DB_PASSWORD
                  valueFrom:
                    secretKeyRef:
                      name: postgres-secrets
                      key: password
                - name: DB_NAME
                  value: "app"
                - name: BACKUP_BUCKET
                  valueFrom:
                    configMapKeyRef:
                      name: backup-config
                      key: bucket
              resources:
                requests:
                  cpu: "500m"
                  memory: "256Mi"
                limits:
                  cpu: "2"
                  memory: "1Gi"
              volumeMounts:
                - name: backup-volume
                  mountPath: /backups
          volumes:
            - name: backup-volume
              emptyDir:
                sizeLimit: 10Gi
          restartPolicy: OnFailure
          affinity:
            nodeAffinity:
              preferredDuringSchedulingIgnoredDuringExecution:
                - weight: 100
                  preference:
                    matchExpressions:
                      - key: node-role
                        operator: In
                        values:
                          - backup
          tolerations:
            - key: "dedicated"
              operator: "Equal"
              value: "backup"
              effect: "NoSchedule"

---
# Point-in-time recovery configuration
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-wal-archive
  namespace: database
spec:
  schedule: "*/5 * * * *"  # Every 5 minutes
  successfulJobsHistoryLimit: 1
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: backup-service
          containers:
            - name: wal-archive
              image: postgres:15-alpine
              command:
                - sh
                - -c
                - |
                  set -e
                  
                  # WAL archiving to S3
                  aws s3 sync /wal-archive/ s3://${BACKUP_BUCKET}/wal-archive/
                  
                  # Clean up archived WALs older than 7 days
                  find /wal-archive -type f -mtime +7 -delete
                  
              env:
                - name: BACKUP_BUCKET
                  valueFrom:
                    configMapKeyRef:
                      name: backup-config
                      key: wal-bucket
              volumeMounts:
                - name: wal-archive
                  mountPath: /wal-archive
          volumes:
            - name: wal-archive
              persistentVolumeClaim:
                claimName: wal-archive-pvc
```

### 1.2 File-Level Backup Implementation

```bash
#!/bin/bash
# backup/files-backup.sh - Complete file backup script

set -euo pipefail

# Configuration
BACKUP_DATE=$(date +%Y%m%d_%H%M%S)
S3_BUCKET="s3://company-backups/files"
RETENTION_DAYS=90
BACKUP_PATHS=(
    "/data/uploads"
    "/data/documents"
    "/etc/app/config"
)
ENCRYPTION_KEY_FILE="/secrets/backup-gpg-key"

# Logging
LOG_FILE="/var/log/backup/backup-${BACKUP_DATE}.log"
exec > >(tee -a "${LOG_FILE}") 2>&1

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

log "Starting backup process"

# GPG encryption function
encrypt_file() {
    local input=$1
    local output=$2
    
    gpg --batch --yes --encrypt \
        --recipient backup@company.com \
        --output "${output}" \
        "${input}"
}

# Upload with multipart for large files
upload_to_s3() {
    local source=$1
    local dest=$2
    
    # Use multipart upload for files > 100MB
    local file_size=$(stat -f%z "${source}" 2>/dev/null || stat -c%s "${source}")
    
    if [ "${file_size}" -gt 104857600 ]; then
        log "Uploading ${source} using multipart (${file_size} bytes)"
        aws s3 cp --storage-class STANDARD_IA \
            "${source}" \
            "${dest}"
    else
        log "Uploading ${source} (${file_size} bytes)"
        aws s3 cp \
            "${source}" \
            "${dest}"
    fi
}

# Incremental backup using rsync
perform_incremental_backup() {
    local source=$1
    local dest=$2
    local snapshot_dir="/backup_snapshots/$(basename ${source})"
    
    # Create snapshot directory
    mkdir -p "${snapshot_dir}"
    
    # Sync with hard links (creates incremental backup)
    rsync -avh --delete \
        --link-dest="${snapshot_dir}/latest" \
        "${source}/" \
        "${snapshot_dir}/backup_${BACKUP_DATE}/"
    
    # Update symlink to latest
    rm -f "${snapshot_dir}/latest"
    ln -s "backup_${BACKUP_DATE}" "${snapshot_dir}/latest"
}

# Process each backup path
for backup_path in "${BACKUP_PATHS[@]}"; do
    if [ ! -d "${backup_path}" ]; then
        log "WARNING: Path ${backup_path} does not exist, skipping"
        continue
    fi
    
    log "Processing ${backup_path}"
    
    backup_name=$(basename "${backup_path}")
    local_backup_dir="/tmp/backups/${backup_name}"
    mkdir -p "${local_backup_dir}"
    
    # Create archive
    archive_name="${backup_name}_${BACKUP_DATE}.tar.gz"
    archive_path="${local_backup_dir}/${archive_name}"
    
    tar -czf "${archive_path}" -C "$(dirname ${backup_path})" "$(basename ${backup_path})"
    
    # Calculate checksum
    sha256sum "${archive_path}" > "${archive_path}.sha256"
    
    # Encrypt if key available
    if [ -f "${ENCRYPTION_KEY_FILE}" ]; then
        log "Encrypting backup"
        encrypt_file "${archive_path}" "${archive_path}.gpg"
        mv "${archive_path}.gpg" "${archive_path}"
    fi
    
    # Upload to S3
    upload_to_s3 "${archive_path}" "${S3_BUCKET}/${backup_name}/${archive_name}"
    upload_to_s3 "${archive_path}.sha256" "${S3_BUCKET}/${backup_name}/checksums/${archive_name}.sha256"
    
    # Cleanup local
    rm -rf "${local_backup_dir}"
    
    log "Completed ${backup_path}"
done

# Cleanup old S3 backups
log "Cleaning up backups older than ${RETENTION_DAYS} days"
aws s3 ls "${S3_BUCKET}/" | while read -r prefix; do
    aws s3api list-objects \
        --bucket company-backups \
        --prefix "files/${prefix}" \
        --query "Contents[?LastModified<='$(date -d "-${RETENTION_DAYS} days" -I)']" \
        --output text \
        | awk '{print $2}' \
        | xargs -r -I {} aws s3 rm "s3://company-backups/{}"
done

log "Backup process completed successfully"
```

### 1.3 Application-Level Backup

```typescript
// backup/application-backup.ts - Application data backup service

interface BackupConfig {
  target: BackupTarget;
  schedule: string;
  retention: RetentionPolicy;
  encryption: EncryptionConfig;
  compression: CompressionConfig;
  verification: VerificationConfig;
}

interface BackupTarget {
  type: 'S3' | 'GCS' | 'AZURE_BLOB' | 'LOCAL';
  connectionString: string;
  bucket?: string;
  path: string;
}

interface RetentionPolicy {
  local: {
    enabled: boolean;
    maxAge: number;  // days
    maxBackups: number;
  };
  remote: {
    enabled: boolean;
    maxAge: number;  // days
    maxBackups: number;
  };
}

interface EncryptionConfig {
  enabled: boolean;
  keyId: string;
  algorithm: 'AES-256-GCM' | 'AES-256-CBC';
}

interface VerificationConfig {
  enabled: boolean;
  checksumAlgorithm: 'SHA256' | 'SHA512' | 'MD5';
  restoreTestEnabled: boolean;
  restoreTestInterval: number;  // days
}

class BackupService {
  constructor(
    private config: BackupConfig,
    private storageClient: StorageClient,
    private encryptionService: EncryptionService,
    private notificationService: NotificationService,
    private auditLogger: AuditLogger
  ) {}
  
  async performBackup(): Promise<BackupResult> {
    const backupId = generateUUID();
    const startTime = new Date();
    
    try {
      // 1. Create backup manifest
      const manifest = await this.createManifest(backupId);
      
      // 2. Collect data
      const dataPaths = await this.collectData();
      
      // 3. Create archive
      const archivePath = await this.createArchive(backupId, dataPaths);
      
      // 4. Calculate checksum
      const checksum = await this.calculateChecksum(archivePath);
      
      // 5. Compress if enabled
      const finalPath = await this.compress(archivePath);
      
      // 6. Encrypt if enabled
      const encryptedPath = await this.encrypt(finalPath);
      
      // 7. Upload
      const remotePath = await this.upload(encryptedPath);
      
      // 8. Verify
      if (this.config.verification.enabled) {
        await this.verifyBackup(remotePath, checksum);
      }
      
      // 9. Cleanup old backups
      await this.cleanupOldBackups();
      
      const endTime = new Date();
      const result: BackupResult = {
        backupId,
        status: 'SUCCESS',
        startTime,
        endTime,
        duration: endTime.getTime() - startTime.getTime(),
        size: await this.getFileSize(encryptedPath),
        checksum,
        remotePath,
      };
      
      await this.auditLogger.logBackupCompleted(result);
      await this.notificationService.sendBackupNotification(result);
      
      return result;
      
    } catch (error) {
      const result: BackupResult = {
        backupId,
        status: 'FAILED',
        startTime,
        endTime: new Date(),
        error: (error as Error).message,
      };
      
      await this.auditLogger.logBackupFailed(result);
      await this.notificationService.sendBackupFailureAlert(result);
      
      throw error;
    }
  }
  
  private async createManifest(backupId: string): Promise<BackupManifest> {
    return {
      id: backupId,
      createdAt: new Date(),
      version: '1.0',
      hostname: os.hostname(),
      application: process.env.APP_NAME || 'unknown',
      applicationVersion: process.env.APP_VERSION || 'unknown',
      dataSources: [
        { type: 'postgresql', name: 'primary' },
        { type: 'redis', name: 'cache' },
        { type: 'file', name: 'uploads' },
      ],
    };
  }
  
  private async collectData(): Promise<string[]> {
    const paths: string[] = [];
    
    // Database dump
    const dbDump = await this.backupDatabase();
    paths.push(dbDump);
    
    // Redis data
    const redisDump = await this.backupRedis();
    paths.push(redisDump);
    
    // Files
    const filesArchive = await this.backupFiles();
    paths.push(filesArchive);
    
    return paths;
  }
  
  private async createArchive(backupId: string, dataPaths: string[]): Promise<string> {
    const archivePath = `/tmp/backup_${backupId}.tar`;
    
    await exec(`tar -cf ${archivePath} ${dataPaths.join(' ')}`);
    
    return archivePath;
  }
  
  private async upload(localPath: string): Promise<string> {
    const remotePath = `${this.config.target.path}/backup_${Date.now()}.tar.gz.enc`;
    
    await this.storageClient.upload(localPath, remotePath);
    
    return remotePath;
  }
  
  private async verifyBackup(remotePath: string, expectedChecksum: string): Promise<void> {
    // Download and verify checksum
    const localPath = `/tmp/verify_${Date.now()}`;
    await this.storageClient.download(remotePath, localPath);
    
    const actualChecksum = await this.calculateChecksum(localPath);
    
    if (actualChecksum !== expectedChecksum) {
      throw new Error(`Backup verification failed: checksum mismatch`);
    }
    
    // Optional restore test
    if (this.config.verification.restoreTestEnabled) {
      await this.performRestoreTest(localPath);
    }
    
    // Cleanup verification file
    await fs.unlink(localPath);
  }
  
  private async cleanupOldBackups(): Promise<void> {
    if (this.config.retention.remote.enabled) {
      await this.cleanupRemote();
    }
    
    if (this.config.retention.local.enabled) {
      await this.cleanupLocal();
    }
  }
}

interface BackupManifest {
  id: string;
  createdAt: Date;
  version: string;
  hostname: string;
  application: string;
  applicationVersion: string;
  dataSources: Array<{ type: string; name: string }>;
}

interface BackupResult {
  backupId: string;
  status: 'SUCCESS' | 'FAILED';
  startTime: Date;
  endTime: Date;
  duration?: number;
  size?: number;
  checksum?: string;
  remotePath?: string;
  error?: string;
}
```

## 2. RPO/RTO Definitions

### 2.1 Recovery Objective Matrix

```markdown
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                            Recovery Objective Matrix                                    │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Tier  │ Service Level         │ RPO        │ RTO        │ Examples                  │
├───────┼───────────────────────┼────────────┼────────────┼────────────────────────────┤
│ Tier1 │ Mission Critical      │ 0-15 min   │ 0-15 min   │ Payment processing        │
├───────┼───────────────────────┼────────────┼────────────┼────────────────────────────┤
│ Tier2 │ Business Critical     │ 1 hour     │ 1-4 hours  │ User management, orders   │
├───────┼───────────────────────┼────────────┼────────────┼────────────────────────────┤
│ Tier3 │ Standard              │ 4 hours    │ 8-12 hours │ Reporting, analytics      │
├───────┼───────────────────────┼────────────┼────────────┼────────────────────────────┤
│ Tier4 │ Low Priority          │ 24 hours   │ 24-48 hours│ Logs, archives            │
└───────┴───────────────────────┴────────────┴────────────┴────────────────────────────┘

Recovery Point Objective (RPO): Maximum acceptable data loss measured in time
Recovery Time Objective (RTO): Maximum acceptable downtime measured in time

Key Decisions:
- RPO determines backup frequency
- RTO determines architecture complexity
- Cost increases exponentially as RTO/RPO decreases
```

### 2.2 Recovery Strategy Selection

```typescript
// dr/strategy-selector.ts

interface RecoveryStrategy {
  name: string;
  rpo: number;  // minutes
  rto: number;  // minutes
  cost: 'LOW' | 'MEDIUM' | 'HIGH' | 'VERY_HIGH';
  complexity: 'LOW' | 'MEDIUM' | 'HIGH';
  implementations: string[];
}

const RECOVERY_STRATEGIES: RecoveryStrategy[] = [
  {
    name: 'No DR (Single Site)',
    rpo: 0,
    rto: Infinity,
    cost: 'LOW',
    complexity: 'LOW',
    implementations: ['Single region deployment'],
  },
  {
    name: 'Backup & Restore',
    rpo: 1440,  // 24 hours
    rto: 480,   // 8 hours
    cost: 'LOW',
    complexity: 'LOW',
    implementations: [
      'Nightly backups to S3',
      'Manual restore process',
      'Documented runbook',
    ],
  },
  {
    name: 'Pilot Light',
    rpo: 60,    // 1 hour
    rto: 120,   // 2 hours
    cost: 'MEDIUM',
    complexity: 'MEDIUM',
    implementations: [
      'Hot standby database',
      'Lambda-based scaling',
      'Automated DNS failover',
    ],
  },
  {
    name: 'Warm Standby',
    rpo: 15,    // 15 minutes
    rto: 30,    // 30 minutes
    cost: 'HIGH',
    complexity: 'HIGH',
    implementations: [
      'Multi-AZ deployment',
      'Synchronous data replication',
      'Load balancer with health checks',
    ],
  },
  {
    name: 'Hot Standby (Multi-Region)',
    rpo: 0,     // Real-time
    rto: 15,    // 15 minutes
    cost: 'VERY_HIGH',
    complexity: 'HIGH',
    implementations: [
      'Active-active multi-region',
      'Synchronous replication',
      'Automatic failover',
    ],
  },
];

class RecoveryStrategySelector {
  select(businessRequirements: {
    maxDataLossMinutes: number;
    maxDowntimeMinutes: number;
    budget: 'LOW' | 'MEDIUM' | 'HIGH' | 'VERY_HIGH';
  }): RecoveryStrategy {
    // Filter by requirements
    const viable = RECOVERY_STRATEGIES.filter(s => {
      if (s.rpo > businessRequirements.maxDataLossMinutes) return false;
      if (s.rto > businessRequirements.maxDowntimeMinutes) return false;
      if (this.costToNumber(s.cost) > this.costToNumber(businessRequirements.budget)) return false;
      return true;
    });
    
    if (viable.length === 0) {
      // Return best effort
      return RECOVERY_STRATEGIES[RECOVERY_STRATEGIES.length - 1];
    }
    
    // Sort by cost (prefer cheaper options that meet requirements)
    viable.sort((a, b) => 
      this.costToNumber(a.cost) - this.costToNumber(b.cost)
    );
    
    return viable[0];
  }
  
  private costToNumber(cost: string): number {
    const map = { 'LOW': 1, 'MEDIUM': 2, 'HIGH': 3, 'VERY_HIGH': 4 };
    return map[cost];
  }
}
```

## 3. Failover Patterns

### 3.1 Database Failover Implementation

```typescript
// dr/database-failover.ts

class DatabaseFailoverManager {
  private primary: DatabaseConnection;
  private replicas: DatabaseConnection[];
  private healthCheckInterval: number = 30000;
  private promotionTimeout: number = 60000;
  
  constructor(
    private config: FailoverConfig,
    private eventBus: EventBus,
    private alertService: AlertService,
    private auditLogger: AuditLogger
  ) {
    this.primary = new DatabaseConnection(config.primary);
    this.replicas = config.replicas.map(r => new DatabaseConnection(r));
    
    this.startHealthChecks();
    this.setupFailoverHandlers();
  }
  
  private startHealthChecks(): void {
    setInterval(async () => {
      await this.checkPrimaryHealth();
      await this.checkReplicaHealth();
    }, this.healthCheckInterval);
  }
  
  private async checkPrimaryHealth(): Promise<void> {
    try {
      const isHealthy = await this.primary.healthCheck();
      
      if (!isHealthy && !this.isFailoverInProgress()) {
        console.error('Primary database unhealthy, initiating failover');
        await this.initiateFailover();
      }
    } catch (error) {
      console.error('Error checking primary health:', error);
    }
  }
  
  private async checkReplicaHealth(): Promise<void> {
    for (const replica of this.replicas) {
      try {
        const isHealthy = await replica.healthCheck();
        replica.setHealthy(isHealthy);
      } catch (error) {
        replica.setHealthy(false);
      }
    }
  }
  
  private async initiateFailover(): Promise<void> {
    if (this.isFailoverInProgress()) {
      return;
    }
    
    const failoverId = generateUUID();
    const startTime = new Date();
    
    try {
      // 1. Stop writes to primary
      await this.stopWrites();
      
      // 2. Find best replica
      const bestReplica = await this.selectBestReplica();
      
      if (!bestReplica) {
        throw new Error('No healthy replica available for promotion');
      }
      
      // 3. Wait for replication to catch up
      await this.waitForReplicationCatchup(bestReplica);
      
      // 4. Promote replica
      await this.promoteReplica(bestReplica);
      
      // 5. Update connection strings
      await this.updateConnections(bestReplica);
      
      // 6. Verify new primary
      await this.verifyNewPrimary();
      
      // 7. Resume writes
      await this.resumeWrites();
      
      // 8. Recreate replica pool
      await this.rebuildReplicaPool(bestReplica);
      
      const duration = Date.now() - startTime.getTime();
      
      await this.auditLogger.logFailover({
        failoverId,
        duration,
        promotedReplica: bestReplica.getId(),
        success: true,
      });
      
      await this.notificationService.sendFailoverComplete({
        failoverId,
        duration,
      });
      
    } catch (error) {
      await this.auditLogger.logFailover({
        failoverId,
        duration: Date.now() - startTime.getTime(),
        success: false,
        error: (error as Error).message,
      });
      
      await this.alertService.sendFailoverFailedAlert({
        error: (error as Error).message,
      });
      
      throw error;
    }
  }
  
  private async selectBestReplica(): Promise<DatabaseConnection | null> {
    const healthyReplicas = this.replicas.filter(r => r.isHealthy());
    
    if (healthyReplicas.length === 0) {
      return null;
    }
    
    // Select replica with lowest lag
    const replicasWithLag = await Promise.all(
      healthyReplicas.map(async replica => ({
        replica,
        lag: await replica.getReplicationLag(),
      }))
    );
    
    replicasWithLag.sort((a, b) => a.lag - b.lag);
    
    return replicasWithLag[0].replica;
  }
  
  private async promoteReplica(replica: DatabaseConnection): Promise<void> {
    await replica.promote({
      timeout: this.promotionTimeout,
    });
  }
  
  private async updateConnections(newPrimary: DatabaseConnection): Promise<void> {
    // Update DNS or connection string
    await this.dnsManager.updateRecord({
      name: this.config.dnsRecordName,
      value: newPrimary.getHost(),
      ttl: 60,
    });
  }
}
```

### 3.2 Application Failover Pattern

```yaml
# kubernetes/app-failover.yaml - Application failover configuration

apiVersion: v1
kind: ConfigMap
metadata:
  name: failover-config
  namespace: production
data:
  failover-enabled: "true"
  health-check-path: /health
  health-check-interval: "10s"
  health-check-timeout: "5s"
  health-check-threshold: "3"
  graceful-shutdown-timeout: "30s"
  pre-stop-wait: "10s"

---
# Service with failover
apiVersion: v1
kind: Service
metadata:
  name: api-service
  namespace: production
  annotations:
    # Enable service mesh failover
    service.kubernetes.io/topology-mode: "Auto"
    service.kubernetes.io/local-svc-lb-weight: "100"
spec:
  type: ClusterIP
  ports:
    - name: http
      port: 80
      targetPort: 8080
  selector:
    app: api
  sessionAffinity: ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 10800

---
# Pod disruption budget for controlled failover
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: api-pdb
  namespace: production
spec:
  maxUnavailable: 1
  selector:
    matchLabels:
      app: api

---
# HPA with failover awareness
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-deployment
  minReplicas: 3
  maxReplicas: 50
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Pods
          value: 1
          periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
        - type: Pods
          value: 4
          periodSeconds: 15
```

## 4. Multi-Region Deployment

### 4.1 Multi-Region Architecture

```yaml
# terraform/multi-region/main.tf - Multi-region deployment

terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

# Primary region
provider "aws" {
  alias  = "primary"
  region = "us-east-1"
}

# Secondary region (DR)
provider "aws" {
  alias  = "secondary"
  region = "us-west-2"
}

# Database in primary region
module "primary_database" {
  source  = "./modules/postgres"
  providers = {
    aws = aws.primary
  }
  
  identifier     = "app-primary-db"
  instance_class = "db.r6g.xlarge"
  allocated_storage = 100
  storage_encrypted = true
  
  backup_retention_period = 30
  backup_window           = "03:00-04:00"
  maintenance_window      = "mon:04:00-mon:05:00"
  
  multi_az               = true
  availability_zone      = "us-east-1a"
  secondary_availability_zone = "us-east-1b"
}

# Read replica in secondary region
module "secondary_database" {
  source  = "./modules/postgres-replica"
  providers = {
    aws = aws.secondary
  }
  
  identifier     = "app-dr-db"
  instance_class = "db.r6g.large"
  source_db      = module.primary_database.arn
  
  backup_retention_period = 7
}

# S3 cross-region replication
resource "aws_s3_bucket" "primary_bucket" {
  provider = aws.primary
  bucket   = "app-data-primary"
  
  versioning {
    enabled = true
  }
  
  replication_configuration {
    role = aws_iam_role.replication.arn
    
    rules {
      id     = "replicate-all"
      status = "Enabled"
      
      destination {
        bucket        = aws_s3_bucket.replica_bucket.arn
        storage_class = "STANDARD_IA"
        encryption_configuration {
          replica_kms_key_id = aws_kms_key.replica_key.arn
        }
      }
      
      filter {
        prefix = ""
      }
    }
  }
}

# EFS for shared storage
module "efs_primary" {
  source  = "./modules/efs"
  providers = {
    aws = aws.primary
  }
  
  name                    = "app-shared-storage"
  encrypted               = true
  throughput_mode         = "provisioned"
  provisioned_throughput_mibps = 512
  
  lifecycle_policy {
    transition_to_ia = "AFTER_30_DAYS"
  }
}

# Route53 health check and failover
resource "aws_route53_health_check" "primary" {
  provider               = aws.primary
  fqdn                   = "api-primary.example.com"
  port                   = 443
  type                   = "HTTPS"
  resource_path          = "/health"
  failure_threshold      = 3
  request_interval       = 10
  
  tags = {
    Name = "primary-health-check"
  }
}

resource "aws_route53_record" "api" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "api.example.com"
  type    = "A"
  
  failover_routing_policy {
    type = "PRIMARY"
  }
  
  set_identifier  = "primary"
  health_check_id = aws_route53_health_check.primary.id
  
  alias {
    name                   = module.alb_primary.dns_name
    zone_id                = module.alb_primary.zone_id
    evaluate_target_health = true
  }
}

resource "aws_route53_record" "api_dr" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "api-dr.example.com"
  type    = "A"
  
  failover_routing_policy {
    type = "SECONDARY"
  }
  
  set_identifier = "secondary"
  
  alias {
    name                   = module.alb_secondary.dns_name
    zone_id                = module.alb_secondary.zone_id
    evaluate_target_health = true
  }
}
```

### 4.2 Cross-Region Replication Configuration

```typescript
// dr/cross-region-replication.ts

interface CrossRegionReplicationConfig {
  sourceRegion: string;
  targetRegion: string;
  replicationType: 'SYNC' | 'ASYNC' | 'LOG_SHIPPING';
  conflictResolution: 'SOURCE_WINS' | 'TARGET_WINS' | 'LATEST_WINS' | 'MANUAL';
  filters: ReplicationFilter[];
}

interface ReplicationFilter {
  type: 'TABLE' | 'SCHEMA' | 'CUSTOM';
  pattern: string;
}

class CrossRegionReplicationManager {
  constructor(
    private sourceConnection: DatabaseConnection,
    private targetConnection: DatabaseConnection,
    private config: CrossRegionReplicationConfig
  ) {}
  
  async setupReplication(): Promise<void> {
    switch (this.config.replicationType) {
      case 'SYNC':
        await this.setupSyncReplication();
        break;
      case 'ASYNC':
        await this.setupAsyncReplication();
        break;
      case 'LOG_SHIPPING':
        await this.setupLogShipping();
        break;
    }
  }
  
  private async setupSyncReplication(): Promise<void> {
    // Enable sync replication for critical tables
    for (const filter of this.config.filters) {
      if (filter.type === 'TABLE') {
        await this.sourceConnection.query(`
          ALTER TABLE ${filter.pattern} 
          REPLICA IDENTITY FULL
        `);
      }
    }
    
    // Create replication slot
    await this.sourceConnection.query(`
      SELECT * FROM pg_create_logical_replication_slot(
        'sync_replication',
        'pgoutput'
      )
    `);
    
    // Create subscription
    await this.targetConnection.query(`
      CREATE SUBSCRIPTION sync_sub
      CONNECTION 'host=${this.sourceConnection.getHost()} 
                   port=${this.sourceConnection.getPort()} 
                   dbname=${this.sourceConnection.getDatabase()}'
      PUBLICATION sync_pub
      WITH (copy_data = true, synchronous_commit = on)
    `);
  }
  
  async performFailover(): Promise<FailoverResult> {
    const startTime = Date.now();
    
    try {
      // 1. Stop writes to source
      await this.stopWrites();
      
      // 2. Wait for target to catch up
      await this.waitForCatchup();
      
      // 3. Verify data integrity
      await this.verifyDataIntegrity();
      
      // 4. Promote target
      await this.promoteTarget();
      
      // 5. Update connection strings
      await this.updateConnections();
      
      return {
        success: true,
        duration: Date.now() - startTime,
        dataLoss: await this.calculateDataLoss(),
      };
      
    } catch (error) {
      return {
        success: false,
        duration: Date.now() - startTime,
        error: (error as Error).message,
      };
    }
  }
}
```

## 5. Disaster Recovery Plans

### 5.1 Complete DR Runbook

```markdown
# Disaster Recovery Runbook

## Recovery Time: 4 hours
## Recovery Point: 1 hour

## Pre-conditions
- [ ] DR site infrastructure is operational
- [ ] Network connectivity between sites verified
- [ ] Latest backup verified
- [ ] DR team contacted

## Step 1: Declare Disaster (T+0)
1. [ ] Open incident ticket
2. [ ] Notify DR team lead
3. [ ] Assess situation and confirm DR is required
4. [ ] Document initial findings

## Step 2: Data Recovery (T+0 to T+30min)
1. [ ] Identify latest good backup
2. [ ] Restore database from backup
3. [ ] Verify database integrity
4. [ ] Restore point-in-time if possible

## Step 3: Application Recovery (T+30min to T+2hr)
1. [ ] Deploy applications to DR site
2. [ ] Update DNS records
3. [ ] Verify application connectivity
4. [ ] Test critical paths

## Step 4: Validation (T+2hr to T+3hr)
1. [ ] Run integration tests
2. [ ] Verify data integrity
3. [ ] Check monitoring/alerting
4. [ ] Validate backup procedures

## Step 5: Return to Normal (T+3hr to T+4hr)
1. [ ] Confirm all services operational
2. [ ] Update status page
3. [ ] Notify stakeholders
4. [ ] Begin root cause analysis
```

### 5.2 Automated DR Testing

```yaml
# dr/chaos-testing/backup-restores.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: dr-backup-test
  namespace: dr-testing
spec:
  schedule: "0 3 * * 0"  # Weekly at 3 AM Sunday
  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: dr-test-service
          containers:
            - name: dr-test
              image: dr-test:latest
              command:
                - node
                - /app/dr-test.js
              env:
                - name: DR_TEST_MODE
                  value: "BACKUP_RESTORE"
                - name: NOTIFICATION_WEBHOOK
                  valueFrom:
                    secretKeyRef:
                      name: notification-secrets
                      key: webhook
              resources:
                requests:
                  cpu: "500m"
                  memory: "512Mi"
                limits:
                  cpu: "2"
                  memory: "2Gi"
              volumeMounts:
                - name: test-workspace
                  mountPath: /workspace
          volumes:
            - name: test-workspace
              emptyDir:
                sizeLimit: 10Gi
```

```typescript
// dr/chaos-testing/dr-test.ts

class DRTestRunner {
  constructor(
    private backupService: BackupService,
    private restoreService: RestoreService,
    private databasePool: DatabasePool,
    private notificationService: NotificationService,
    private testResultsStore: TestResultsStore
  ) {}
  
  async runBackupRestoreTest(): Promise<TestResult> {
    const testId = generateUUID();
    const startTime = new Date();
    
    const result: TestResult = {
      testId,
      testType: 'BACKUP_RESTORE',
      startTime,
      status: 'IN_PROGRESS',
    };
    
    try {
      // 1. Create test database
      const testDbName = `dr_test_${Date.now()}`;
      await this.databasePool.createDatabase(testDbName);
      
      // 2. Insert test data
      await this.insertTestData(testDbName);
      
      // 3. Create backup
      const backupResult = await this.backupService.performBackup({
        database: testDbName,
        type: 'FULL',
      });
      
      // 4. Insert more data after backup
      await this.insertMoreTestData(testDbName, 'after_backup');
      const pointInTime = new Date();
      
      // 5. Verify backup exists
      if (!backupResult.success) {
        throw new Error('Backup creation failed');
      }
      
      // 6. Drop test database
      await this.databasePool.dropDatabase(testDbName);
      
      // 7. Restore backup
      const restoreResult = await this.restoreService.restore({
        backupId: backupResult.backupId,
        targetDatabase: testDbName,
      });
      
      // 8. Verify restored data
      const dataVerification = await this.verifyTestData(testDbName);
      
      if (!dataVerification.success) {
        throw new Error(`Data verification failed: ${dataVerification.error}`);
      }
      
      // 9. Test point-in-time recovery
      await this.testPointInTimeRecovery(testDbName, pointInTime);
      
      // 10. Cleanup
      await this.databasePool.dropDatabase(testDbName);
      
      result.status = 'PASSED';
      result.endTime = new Date();
      result.duration = result.endTime.getTime() - startTime.getTime();
      result.details = {
        backupCreated: backupResult.backupId,
        dataVerified: dataVerification.recordCount,
      };
      
    } catch (error) {
      result.status = 'FAILED';
      result.endTime = new Date();
      result.duration = result.endTime.getTime() - startTime.getTime();
      result.error = (error as Error).message;
    }
    
    // Store result
    await this.testResultsStore.save(result);
    
    // Notify
    await this.notificationService.sendTestResult(result);
    
    return result;
  }
  
  private async verifyTestData(database: string): Promise<{
    success: boolean;
    recordCount?: number;
    error?: string;
  }> {
    const count = await this.databasePool.query(
      database,
      'SELECT COUNT(*) FROM test_records'
    );
    
    const expectedCount = await this.getExpectedTestRecordCount();
    
    if (count < expectedCount) {
      return {
        success: false,
        error: `Expected at least ${expectedCount} records, found ${count}`,
      };
    }
    
    // Verify checksums
    const checksum = await this.databasePool.query(
      database,
      'SELECT md5(array_agg(data ORDER BY id)) FROM test_records'
    );
    
    return {
      success: true,
      recordCount: count,
    };
  }
}
```

## 6. Decision Matrices

### 6.1 DR Strategy Selection Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              DR Strategy Selection Matrix                               │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Budget        │ RTO      │ RPO      │ Recommended Strategy                             │
├───────────────┼──────────┼──────────┼──────────────────────────────────────────────────┤
│ Minimal       │ < 4 hrs  │ < 1 hour │ Backup & Restore + Pilot Light                  │
├───────────────┼──────────┼──────────┼──────────────────────────────────────────────────┤
│ Low           │ < 1 hour │ < 15 min │ Pilot Light + Automated failover               │
├───────────────┼──────────┼──────────┼──────────────────────────────────────────────────┤
│ Medium        │ < 30 min │ < 5 min  │ Warm Standby + Multi-AZ                         │
├───────────────┼──────────┼──────────┼──────────────────────────────────────────────────┤
│ High          │ < 15 min │ < 1 min  │ Hot Standby + Multi-Region Active-Active         │
├───────────────┼──────────┼──────────┼──────────────────────────────────────────────────┤
│ Enterprise    │ < 5 min  │ 0        │ Multi-Region Active-Active with sync replication│
└───────────────┴──────────┴──────────┴──────────────────────────────────────────────────┘
```

### 6.2 Backup Frequency Selection

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           Backup Frequency Selection Matrix                              │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ RPO         │ Recommended Backup Strategy                                               │
├─────────────┼──────────────────────────────────────────────────────────────────────────┤
│ 0 minutes   │ Synchronous replication (no backup needed, continuous copy)             │
├─────────────┼──────────────────────────────────────────────────────────────────────────┤
│ 15 minutes  │ Continuous WAL archiving + periodic base backups (every 15 min)         │
├─────────────┼──────────────────────────────────────────────────────────────────────────┤
│ 1 hour      │ Hourly backups + WAL archiving                                           │
├─────────────┼──────────────────────────────────────────────────────────────────────────┤
│ 4 hours     │ 4-hourly backups + nightly full backup                                   │
├─────────────┼──────────────────────────────────────────────────────────────────────────┤
│ 24 hours    │ Daily backup + weekly full backup                                        │
├─────────────┼──────────────────────────────────────────────────────────────────────────┤
│ > 24 hours  │ Weekly backup + monthly archive                                          │
└─────────────┴──────────────────────────────────────────────────────────────────────────┘
```

## 7. Anti-Patterns

### 7.1 DR Anti-Patterns

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                               DR Anti-Patterns to Avoid                                │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Anti-Pattern                    │ Problem                       │ Solution                │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No DR plan                      │ Chaos during disaster         │ Create and test DR plan │
│                                 │ Maximum downtime               │ regularly               │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Untested backups               │ Backup restoration fails       │ Regular DR testing     │
│                                 │ Data loss                      │ schedule               │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Single region deployment       │ Region outage = complete down  │ Multi-region setup     │
│                                 │                               │                         │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Backup stored with app         │ Backup also affected          │ Geo-separated backup   │
│                                 │                               │ storage                │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No RPO/RTO defined             │ No recovery goals             │ Define and document    │
│                                 │ Inappropriate strategy        │ business requirements  │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Manual failover                │ Long downtime                  │ Automate failover      │
│                                 │ Human error                    │ procedures             │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Ignoring network               │ Connectivity issues            │ Test network failover │
│                                 │ Block recovery                  │ separately             │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No monitoring of backups      │ Silent backup failures         │ Monitor backup jobs   │
│                                 │                               │ and verify success    │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Retention < regulatory max     │ Compliance violation           │ Align retention with   │
│                                 │                               │ regulations             │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Not testing restore on dev     │ Production restore fails       │ Regular end-to-end    │
│                                 │                               │ restore tests          │
└─────────────────────────────────┴───────────────────────────────┴────────────────────────┘
```

---

## Links

### AWS DR
- [AWS Disaster Recovery](https://aws.amazon.com/disaster-recovery/)
- [AWS Backup](https://aws.amazon.com/backup/)
- [AWS Route53 Failover](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/dns-failover.html)
- [RDS Multi-AZ](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/Concepts.MultiAZ.html)

### Azure DR
- [Azure Site Recovery](https://azure.microsoft.com/services/site-recovery/)
- [Azure Backup](https://azure.microsoft.com/services/backup/)
- [Azure SQL DR](https://docs.microsoft.com/en-us/azure/azure-sql/database/disaster-recovery-strategies)

### Google Cloud DR
- [Cloud SQL HA](https://cloud.google.com/sql/docs/mysql/high-availability)
- [GKE Disaster Recovery](https://cloud.google.com/kubernetes-engine/docs/concepts/disaster-recovery)
- [Cross-region replication](https://cloud.google.com/sql/docs/mysql/replication/cross-region-replica)

### General DR
- [DRII Best Practices](https://drii.org/best-practices)
- [ISO 22301 - Business Continuity](https://www.iso.org/iso-22301-business-continuity)
- [NIST SP 800-34](https://csrc.nist.gov/publications/detail/sp/800-34/final)

### Tools
- [Restic - Backup tool](https://restic.net/)
- [Velero - K8s backup](https://velero.io/)
- [Litestream - SQLite replication](https://litestream.io/)
- [pgBackRest - PostgreSQL backup](https://pgbackrest.org/)

### Testing
- [Chaos Monkey](https://github.com/Netflix/chaosmonkey)
- [LitmusChaos](https://litmuschaos.io/)
- [Gremlin](https://www.gremlin.com/)