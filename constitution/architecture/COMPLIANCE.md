# COMPLIANCE.md - Compliance Architecture Standards

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

---

## 1. SOC2 Compliance Patterns

### 1.1 SOC2 Common Criteria Implementation

```typescript
// compliance/soc2/audit-log.ts - Complete audit logging implementation

interface AuditEvent {
  id: string;
  timestamp: Date;
  eventType: AuditEventType;
  userId?: string;
  userEmail?: string;
  userRole?: string;
  ipAddress: string;
  userAgent: string;
  resource: ResourceInfo;
  action: ActionType;
  outcome: OutcomeType;
  details: Record<string, unknown>;
  metadata: EventMetadata;
}

enum AuditEventType {
  // CC1: COSO Principle 1 - Control Environment
  USER_LOGIN = 'USER_LOGIN',
  USER_LOGOUT = 'USER_LOGOUT',
  USER_LOGIN_FAILED = 'USER_LOGIN_FAILED',
  PASSWORD_CHANGED = 'PASSWORD_CHANGED',
  MFA_ENABLED = 'MFA_ENABLED',
  MFA_DISABLED = 'MFA_DISABLED',
  ROLE_ASSIGNED = 'ROLE_ASSIGNED',
  ROLE_REVOKED = 'ROLE_REVOKED',
  
  // CC2: COSO Principle 2 - Communication
  POLICY_VIEWED = 'POLICY_VIEWED',
  POLICY_ACCEPTED = 'POLICY_ACCEPTED',
  DOCUMENT_DOWNLOADED = 'DOCUMENT_DOWNLOADED',
  
  // CC3: COSO Principle 3 - Risk Assessment
  SENSITIVE_DATA_ACCESSED = 'SENSITIVE_DATA_ACCESSED',
  SENSITIVE_DATA_EXPORTED = 'SENSITIVE_DATA_EXPORTED',
  SENSITIVE_DATA_MODIFIED = 'SENSITIVE_DATA_MODIFIED',
  BULK_OPERATION = 'BULK_OPERATION',
  
  // CC4: COSO Principle 4 - Control Activities
  CONFIGURATION_CHANGED = 'CONFIGURATION_CHANGED',
  ACCESS_POLICY_CHANGED = 'ACCESS_POLICY_CHANGED',
  ENCRYPTION_KEY_ROTATED = 'ENCRYPTION_KEY_ROTATED',
  BACKUP_PERFORMED = 'BACKUP_PERFORMED',
  SECURITY_SCAN_TRIGGERED = 'SECURITY_SCAN_TRIGGERED',
  
  // CC5: COSO Principle 5 - Monitoring
  ANOMALOUS_ACTIVITY_DETECTED = 'ANOMALOUS_ACTIVITY_DETECTED',
  COMPLIANCE_CHECK_FAILED = 'COMPLIANCE_CHECK_FAILED',
  ALERT_TRIGGERED = 'ALERT_TRIGGERED',
}

interface ResourceInfo {
  type: string;
  id: string;
  name?: string;
  path?: string;
}

type ActionType = 'CREATE' | 'READ' | 'UPDATE' | 'DELETE' | 'EXECUTE' | 'LOGIN' | 'LOGOUT';
type OutcomeType = 'SUCCESS' | 'FAILURE' | 'DENIED' | 'ERROR';

interface EventMetadata {
  requestId: string;
  correlationId?: string;
  sessionId?: string;
  serviceName: string;
  serviceVersion?: string;
  environment: string;
  dataClassification?: string;
}

class AuditLogger {
  constructor(
    private storage: AuditStorage,
    private enricher: EventEnricher,
    private sanitizer: DataSanitizer
  ) {}
  
  async log(event: AuditEvent): Promise<void> {
    // Enrich event with additional context
    const enrichedEvent = await this.enricher.enrich(event);
    
    // Sanitize sensitive data
    const sanitizedEvent = this.sanitizer.sanitize(enrichedEvent);
    
    // Validate event
    this.validate(sanitizedEvent);
    
    // Store event
    await this.storage.write(sanitizedEvent);
    
    // Alert if needed
    if (this.shouldAlert(sanitizedEvent)) {
      await this.sendAlert(sanitizedEvent);
    }
  }
  
  private validate(event: AuditEvent): void {
    if (!event.id || !event.timestamp || !event.eventType) {
      throw new ValidationError('Invalid audit event: missing required fields');
    }
    
    // Validate event type is known
    if (!Object.values(AuditEventType).includes(event.eventType)) {
      throw new ValidationError(`Unknown audit event type: ${event.eventType}`);
    }
  }
  
  private shouldAlert(event: AuditEvent): boolean {
    const alertableTypes = [
      AuditEventType.USER_LOGIN_FAILED,
      AuditEventType.SENSITIVE_DATA_EXPORTED,
      AuditEventType.ANOMALOUS_ACTIVITY_DETECTED,
      AuditEventType.COMPLIANCE_CHECK_FAILED,
      AuditEventType.CONFIGURATION_CHANGED,
    ];
    
    return alertableTypes.includes(event.eventType);
  }
  
  private async sendAlert(event: AuditEvent): Promise<void> {
    // Send to security team
    console.log('SECURITY ALERT:', JSON.stringify(event));
  }
}

class EventEnricher {
  async enrich(event: AuditEvent): Promise<AuditEvent> {
    return {
      ...event,
      metadata: {
        ...event.metadata,
        enrichedAt: new Date(),
        serviceVersion: await this.getServiceVersion(),
        environment: await this.getEnvironment(),
      },
    };
  }
  
  private getServiceVersion(): string {
    return process.env.SERVICE_VERSION || 'unknown';
  }
  
  private getEnvironment(): string {
    return process.env.NODE_ENV || 'development';
  }
}

class DataSanitizer {
  // PII fields to mask
  private piiFields = [
    'password', 'ssn', 'social_security', 'credit_card',
    'secret', 'token', 'api_key', 'private_key',
  ];
  
  sanitize(event: AuditEvent): AuditEvent {
    return {
      ...event,
      details: this.sanitizeObject(event.details),
    };
  }
  
  private sanitizeObject(obj: unknown): unknown {
    if (typeof obj !== 'object' || obj === null) {
      return this.sanitizePrimitive(obj);
    }
    
    if (Array.isArray(obj)) {
      return obj.map(item => this.sanitizeObject(item));
    }
    
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      result[key] = this.sanitizeField(key, value);
    }
    
    return result;
  }
  
  private sanitizeField(key: string, value: unknown): unknown {
    const lowerKey = key.toLowerCase();
    
    for (const piiField of this.piiFields) {
      if (lowerKey.includes(piiField)) {
        return '[REDACTED]';
      }
    }
    
    return this.sanitizeObject(value);
  }
  
  private sanitizePrimitive(value: unknown): unknown {
    if (typeof value === 'string') {
      // Check for email addresses
      if (/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)) {
        return this.maskEmail(value);
      }
    }
    return value;
  }
  
  private maskEmail(email: string): string {
    const [local, domain] = email.split('@');
    const maskedLocal = local[0] + '***' + local[local.length - 1];
    return `${maskedLocal}@${domain}`;
  }
}

// Audit storage interface for multiple backends
interface AuditStorage {
  write(event: AuditEvent): Promise<void>;
  query(filter: AuditFilter): Promise<AuditEvent[]>;
  getById(id: string): Promise<AuditEvent | null>;
}

class PostgresAuditStorage implements AuditStorage {
  constructor(private pool: Pool) {}
  
  async write(event: AuditEvent): Promise<void> {
    await this.pool.query(
      `INSERT INTO audit_events (
        id, timestamp, event_type, user_id, user_email, user_role,
        ip_address, user_agent, resource_type, resource_id, resource_name,
        action, outcome, details, metadata, created_at
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NOW())`,
      [
        event.id,
        event.timestamp,
        event.eventType,
        event.userId,
        event.userEmail,
        event.userRole,
        event.ipAddress,
        event.userAgent,
        event.resource.type,
        event.resource.id,
        event.resource.name,
        event.action,
        event.outcome,
        JSON.stringify(event.details),
        JSON.stringify(event.metadata),
      ]
    );
  }
  
  async query(filter: AuditFilter): Promise<AuditEvent[]> {
    const conditions: string[] = [];
    const params: unknown[] = [];
    let paramIndex = 1;
    
    if (filter.eventTypes) {
      conditions.push(`event_type = ANY($${paramIndex})`);
      params.push(filter.eventTypes);
      paramIndex++;
    }
    
    if (filter.userId) {
      conditions.push(`user_id = $${paramIndex}`);
      params.push(filter.userId);
      paramIndex++;
    }
    
    if (filter.startDate) {
      conditions.push(`timestamp >= $${paramIndex}`);
      params.push(filter.startDate);
      paramIndex++;
    }
    
    if (filter.endDate) {
      conditions.push(`timestamp <= $${paramIndex}`);
      params.push(filter.endDate);
      paramIndex++;
    }
    
    const whereClause = conditions.length > 0 
      ? 'WHERE ' + conditions.join(' AND ')
      : '';
    
    const limit = filter.limit || 1000;
    const offset = filter.offset || 0;
    
    const result = await this.pool.query(
      `SELECT * FROM audit_events ${whereClause} ORDER BY timestamp DESC LIMIT ${limit} OFFSET ${offset}`,
      params
    );
    
    return result.rows.map(this.mapRowToEvent);
  }
  
  private mapRowToEvent(row: any): AuditEvent {
    return {
      id: row.id,
      timestamp: row.timestamp,
      eventType: row.event_type,
      userId: row.user_id,
      userEmail: row.user_email,
      userRole: row.user_role,
      ipAddress: row.ip_address,
      userAgent: row.user_agent,
      resource: {
        type: row.resource_type,
        id: row.resource_id,
        name: row.resource_name,
      },
      action: row.action,
      outcome: row.outcome,
      details: JSON.parse(row.details),
      metadata: JSON.parse(row.metadata),
    };
  }
}

interface AuditFilter {
  eventTypes?: AuditEventType[];
  userId?: string;
  startDate?: Date;
  endDate?: Date;
  resourceType?: string;
  resourceId?: string;
  limit?: number;
  offset?: number;
}
```

### 1.2 Access Control Implementation

```typescript
// compliance/soc2/access-control.ts - Complete RBAC implementation

interface Permission {
  resource: string;
  actions: Action[];
  conditions?: AccessCondition[];
}

interface AccessCondition {
  field: string;
  operator: 'equals' | 'contains' | 'in' | 'gt' | 'lt';
  value: unknown;
}

interface Role {
  id: string;
  name: string;
  permissions: Permission[];
  inheritsFrom?: string[];
  description: string;
  isSystemRole: boolean;
}

interface User {
  id: string;
  email: string;
  roles: string[];
  attributes: Record<string, unknown>;
  lastLoginAt: Date;
  mfaEnabled: boolean;
  status: 'ACTIVE' | 'SUSPENDED' | 'DELETED';
}

type Action = 'create' | 'read' | 'update' | 'delete' | 'execute' | 'admin';

class AccessControlService {
  private roles: Map<string, Role> = new Map();
  private userRoles: Map<string, string[]> = new Map();
  private userAttributes: Map<string, Record<string, unknown>> = new Map();
  
  constructor(
    private roleRepository: RoleRepository,
    private userRepository: UserRepository,
    private auditLogger: AuditLogger
  ) {
    this.loadRoles();
  }
  
  private async loadRoles(): Promise<void> {
    const roles = await this.roleRepository.findAll();
    for (const role of roles) {
      this.roles.set(role.id, role);
      this.userRoles.set(role.id, roles.filter(r => r.inheritsFrom?.includes(role.id)).map(r => r.id));
    }
  }
  
  async checkAccess(
    userId: string,
    resource: string,
    action: Action,
    context?: AccessContext
  ): Promise<AccessDecision> {
    const user = await this.userRepository.findById(userId);
    if (!user) {
      return { allowed: false, reason: 'User not found' };
    }
    
    if (user.status !== 'ACTIVE') {
      return { allowed: false, reason: 'User account is not active' };
    }
    
    if (!user.mfaEnabled) {
      return { allowed: false, reason: 'MFA is required' };
    }
    
    const permissions = this.getUserPermissions(user);
    
    for (const permission of permissions) {
      if (permission.resource === resource || this.resourceMatches(resource, permission.resource)) {
        if (permission.actions.includes(action) || permission.actions.includes('admin')) {
          // Check conditions
          if (permission.conditions && context) {
            if (!this.evaluateConditions(permission.conditions, context, user)) {
              return { allowed: false, reason: 'Access conditions not met' };
            }
          }
          
          return { allowed: true, reason: 'Access granted' };
        }
      }
    }
    
    return { allowed: false, reason: 'No matching permission found' };
  }
  
  private getUserPermissions(user: User): Permission[] {
    const permissions: Permission[] = [];
    const visited = new Set<string>();
    
    const addRolePermissions = (roleId: string) => {
      if (visited.has(roleId)) return;
      visited.add(roleId);
      
      const role = this.roles.get(roleId);
      if (!role) return;
      
      permissions.push(...role.permissions);
      
      if (role.inheritsFrom) {
        for (const parentId of role.inheritsFrom) {
          addRolePermissions(parentId);
        }
      }
    };
    
    for (const roleId of user.roles) {
      addRolePermissions(roleId);
    }
    
    return permissions;
  }
  
  private resourceMatches(requested: string, allowed: string): boolean {
    // Support wildcards: "orders:*" matches "orders:123"
    if (allowed.endsWith(':*')) {
      const base = allowed.slice(0, -1);
      return requested.startsWith(base);
    }
    return false;
  }
  
  private evaluateConditions(
    conditions: AccessCondition[],
    context: AccessContext,
    user: User
  ): boolean {
    for (const condition of conditions) {
      const value = this.getConditionValue(condition.field, context, user);
      if (!this.evaluateConditionValue(value, condition)) {
        return false;
      }
    }
    return true;
  }
  
  private getConditionValue(
    field: string,
    context: AccessContext,
    user: User
  ): unknown {
    switch (field) {
      case 'user.department':
        return user.attributes['department'];
      case 'user.role':
        return user.roles;
      case 'context.ipAddress':
        return context.ipAddress;
      case 'context.time':
        return context.timestamp;
      default:
        return undefined;
    }
  }
  
  private evaluateConditionValue(value: unknown, condition: AccessCondition): boolean {
    switch (condition.operator) {
      case 'equals':
        return value === condition.value;
      case 'contains':
        return typeof value === 'string' && value.includes(condition.value as string);
      case 'in':
        return Array.isArray(condition.value) && condition.value.includes(value);
      case 'gt':
        return typeof value === 'number' && value > (condition.value as number);
      case 'lt':
        return typeof value === 'number' && value < (condition.value as number);
      default:
        return false;
    }
  }
  
  async auditAccessCheck(
    userId: string,
    resource: string,
    action: Action,
    decision: AccessDecision,
    context?: AccessContext
  ): Promise<void> {
    await this.auditLogger.log({
      id: generateUUID(),
      timestamp: new Date(),
      eventType: AuditEventType.ACCESS_CHECK,
      userId,
      resource: { type: resource, id: '' },
      action: 'EXECUTE' as Action,
      outcome: decision.allowed ? 'SUCCESS' : 'DENIED',
      details: {
        resource,
        action,
        decision: decision.reason,
      },
      metadata: {
        requestId: context?.requestId,
        serviceName: 'access-control',
        environment: process.env.NODE_ENV || 'development',
      },
    });
  }
}

interface AccessDecision {
  allowed: boolean;
  reason: string;
}

interface AccessContext {
  requestId: string;
  ipAddress: string;
  timestamp: Date;
  attributes?: Record<string, unknown>;
}

// Predefined roles
const SYSTEM_ROLES: Role[] = [
  {
    id: 'admin',
    name: 'Administrator',
    description: 'Full system access',
    isSystemRole: true,
    permissions: [
      { resource: '*', actions: ['admin'] }
    ]
  },
  {
    id: 'user',
    name: 'Standard User',
    description: 'Basic user access',
    isSystemRole: true,
    permissions: [
      { resource: 'profile:*', actions: ['read', 'update'] },
      { resource: 'orders:*', actions: ['create', 'read'] },
    ]
  },
  {
    id: 'auditor',
    name: 'Auditor',
    description: 'Read-only access for compliance',
    isSystemRole: true,
    permissions: [
      { resource: '*', actions: ['read'] }
    ],
    conditions: [
      { field: 'context.time', operator: 'gt', value: 0 }
    ]
  },
];
```

## 2. GDPR Compliance Patterns

### 2.1 Data Subject Rights Implementation

```typescript
// compliance/gdpr/data-subject-rights.ts

interface DataSubjectRequest {
  id: string;
  type: RequestType;
  requesterEmail: string;
  requesterId?: string;
  status: RequestStatus;
  requestedAt: Date;
  completedAt?: Date;
  verificationMethod?: string;
  verifiedAt?: Date;
  rejectionReason?: string;
  dataProvided?: DataProvisionMethod;
}

enum RequestType {
  ACCESS = 'ACCESS',           // Right to access - Art. 15
  RECTIFICATION = 'RECTIFICATION', // Right to rectification - Art. 16
  ERASURE = 'ERASURE',         // Right to erasure - Art. 17
  RESTRICTION = 'RESTRICTION', // Right to restriction - Art. 18
  PORTABILITY = 'PORTABILITY', // Right to data portability - Art. 20
  OBJECTION = 'OBJECTION',     // Right to object - Art. 21
}

enum RequestStatus {
  PENDING = 'PENDING',
  VERIFYING_IDENTITY = 'VERIFYING_IDENTITY',
  VERIFIED = 'VERIFIED',
  PROCESSING = 'PROCESSING',
  COMPLETED = 'COMPLETED',
  REJECTED = 'REJECTED',
  FAILED = 'FAILED',
}

type DataProvisionMethod = 'EMAIL' | 'PORTAL' | 'API';

class DataSubjectRightsService {
  constructor(
    private requestRepository: DataSubjectRequestRepository,
    private userRepository: UserRepository,
    private dataInventory: DataInventory,
    private identityVerification: IdentityVerificationService,
    private notificationService: NotificationService,
    private auditLogger: AuditLogger
  ) {}
  
  async submitRequest(
    email: string,
    type: RequestType,
    verificationData: VerificationData
  ): Promise<string> {
    // Verify identity
    const verified = await this.identityVerification.verify(
      email,
      verificationData
    );
    
    if (!verified) {
      throw new VerificationFailedError('Identity verification failed');
    }
    
    // Create request
    const request: DataSubjectRequest = {
      id: generateUUID(),
      type,
      requesterEmail: email,
      status: RequestStatus.VERIFIED,
      requestedAt: new Date(),
      verifiedAt: new Date(),
    };
    
    await this.requestRepository.save(request);
    
    // Queue for processing
    await this.queueProcessing(request);
    
    // Send acknowledgment
    await this.notificationService.sendEmail(email, 'request_acknowledged', {
      requestId: request.id,
      requestType: type,
    });
    
    return request.id;
  }
  
  async processAccessRequest(requestId: string): Promise<void> {
    const request = await this.requestRepository.findById(requestId);
    if (!request) {
      throw new NotFoundError('Request not found');
    }
    
    await this.requestRepository.updateStatus(requestId, RequestStatus.PROCESSING);
    
    try {
      // Find all data for this user
      const userData = await this.collectUserData(request.requesterEmail);
      
      // Compile data package
      const dataPackage = this.compileDataPackage(userData);
      
      // Provide data to subject
      await this.provideData(request, dataPackage);
      
      await this.requestRepository.updateStatus(requestId, RequestStatus.COMPLETED, {
        completedAt: new Date(),
      });
      
      // Audit
      await this.auditDataAccess(request, 'FULFILLED');
      
    } catch (error) {
      await this.requestRepository.updateStatus(requestId, RequestStatus.FAILED);
      throw error;
    }
  }
  
  async processErasureRequest(requestId: string): Promise<void> {
    const request = await this.requestRepository.findById(requestId);
    if (!request) {
      throw new NotFoundError('Request not found');
    }
    
    await this.requestRepository.updateStatus(requestId, RequestStatus.PROCESSING);
    
    try {
      // Find all data locations
      const dataLocations = await this.dataInventory.findUserDataLocations(
        request.requesterEmail
      );
      
      // Erase from each location
      for (const location of dataLocations) {
        await this.eraseFromLocation(location, request);
      }
      
      await this.requestRepository.updateStatus(requestId, RequestStatus.COMPLETED, {
        completedAt: new Date(),
      });
      
      // Audit
      await this.auditDataErasure(request, dataLocations);
      
    } catch (error) {
      await this.requestRepository.updateStatus(requestId, RequestStatus.FAILED);
      throw error;
    }
  }
  
  private async collectUserData(email: string): Promise<UserDataCollection> {
    const user = await this.userRepository.findByEmail(email);
    
    return {
      profile: {
        id: user.id,
        email: user.email,
        name: user.name,
        createdAt: user.createdAt,
        // Include all profile fields
      },
      orders: await this.getUserOrders(user.id),
      activities: await this.getUserActivities(user.id),
      preferences: await this.getUserPreferences(user.id),
      // Include all data categories
    };
  }
  
  private compileDataPackage(data: UserDataCollection): DataPackage {
    // Format according to GDPR requirements
    return {
      format: 'json',
      schemaVersion: '1.0',
      generatedAt: new Date().toISOString(),
      data,
    };
  }
  
  private async eraseFromLocation(
    location: DataLocation,
    request: DataSubjectRequest
  ): Promise<void> {
    // Check if retention period allows erasure
    if (location.retentionPolicy && location.retentionPolicy.legalBasis) {
      if (this.isRetentionRequired(location.retentionPolicy)) {
        // Cannot erase, note this
        return;
      }
    }
    
    await location.storage.erase(location.dataIds);
  }
  
  private isRetentionRequired(policy: RetentionPolicy): boolean {
    // Check if any legal basis requires retention
    const retentionBases = [
      'LEGAL_OBligation',
      'TAX_ACCOUNTING',
      'LITIGATION',
      'CONTRACT_PERFORMANCE',
    ];
    
    return retentionBases.includes(policy.legalBasis);
  }
}

interface DataLocation {
  system: string;
  storage: StorageAdapter;
  dataIds: string[];
  retentionPolicy?: RetentionPolicy;
}

interface RetentionPolicy {
  legalBasis?: string;
  retentionPeriod?: number;
  expiresAt?: Date;
}
```

### 2.2 Data Inventory Implementation

```typescript
// compliance/gdpr/data-inventory.ts

interface DataInventoryEntry {
  id: string;
  name: string;
  description: string;
  dataCategory: DataCategory;
  dataClassification: DataClassification;
  storageLocations: StorageLocation[];
  retentionPeriod: RetentionPeriod;
  legalBasis: LegalBasis;
  subjectTypes: SubjectType[];
  purposes: Purpose[];
  thirdPartySharing: ThirdPartySharing[];
  securityMeasures: SecurityMeasure[];
  lastReviewed: Date;
  nextReview: Date;
}

interface StorageLocation {
  id: string;
  type: 'DATABASE' | 'FILE_STORAGE' | 'CACHE' | 'BACKUP' | 'ANALYTICS';
  system: string;
  location: string;
  encryption: EncryptionInfo;
  accessControls: AccessControlInfo;
}

interface RetentionPeriod {
  duration: number;
  unit: 'DAYS' | 'MONTHS' | 'YEARS';
  startsFrom: 'CREATION' | 'LAST_INTERACTION' | 'ACCOUNT_DELETION';
  legalRetention?: string;
}

interface LegalBasis {
  gdprArticle: string;
  description: string;
  isLegitimateInterest?: {
    interest: string;
    necessity: string;
    balancingTest: string;
  };
}

type DataClassification = 'PUBLIC' | 'INTERNAL' | 'CONFIDENTIAL' | 'RESTRICTED';
type DataCategory = 'PERSONAL' | 'SENSITIVE' | 'SPECIAL_CATEGORY' | 'NON_PERSONAL';
type SubjectType = 'CUSTOMER' | 'EMPLOYEE' | 'VENDOR' | 'OTHER';

interface Purpose {
  name: string;
  description: string;
  legalBasis: string;
}

interface ThirdPartySharing {
  recipient: string;
  purpose: string;
  legalBasis: string;
  dataShared: string[];
  hasContract: boolean;
  safeguards: string[];
}

interface SecurityMeasure {
  name: string;
  type: 'TECHNICAL' | 'ORGANIZATIONAL';
  implementation: string;
}

class DataInventoryService {
  private inventory: Map<string, DataInventoryEntry> = new Map();
  
  constructor(
    private storageRepository: DataInventoryRepository,
    private discoveryService: DataDiscoveryService
  ) {
    this.loadInventory();
  }
  
  async registerDataProcessing(data: RegisterDataInput): Promise<string> {
    const entry: DataInventoryEntry = {
      id: generateUUID(),
      name: data.name,
      description: data.description,
      dataCategory: data.category,
      dataClassification: data.classification,
      storageLocations: data.locations,
      retentionPeriod: data.retention,
      legalBasis: data.legalBasis,
      subjectTypes: data.subjects,
      purposes: data.purposes,
      thirdPartySharing: data.thirdPartySharing || [],
      securityMeasures: data.securityMeasures,
      lastReviewed: new Date(),
      nextReview: this.calculateNextReview(data),
    };
    
    await this.storageRepository.save(entry);
    this.inventory.set(entry.id, entry);
    
    return entry.id;
  }
  
  async findUserDataLocations(email: string): Promise<DataLocation[]> {
    const locations: DataLocation[] = [];
    
    for (const entry of this.inventory.values()) {
      for (const location of entry.storageLocations) {
        const hasData = await this.discoveryService.checkForUserData(
          location,
          email
        );
        
        if (hasData) {
          locations.push({
            ...location,
            dataIds: await this.discoveryService.getDataIds(location, email),
          });
        }
      }
    }
    
    return locations;
  }
  
  async performDPIA(dataProtectionImpactAssessment: DPIAInput): Promise<DPIAResult> {
    const risks: Risk[] = [];
    
    // Check data volume
    if (dataProtectionImpactAssessment.dataVolume > 10000) {
      risks.push({
        id: 'HIGH_VOLUME',
        description: 'Large scale processing',
        severity: 'HIGH',
        likelihood: 'HIGH',
        impact: 'HIGH',
      });
    }
    
    // Check for special categories
    if (dataProtectionImpactAssessment.includesSpecialCategory) {
      risks.push({
        id: 'SPECIAL_CATEGORY',
        description: 'Processing of special category data',
        severity: 'CRITICAL',
        likelihood: 'HIGH',
        impact: 'HIGH',
      });
    }
    
    // Check profiling/automated decision making
    if (dataProtectionImpactAssessment.includesProfiling) {
      risks.push({
        id: 'PROFILING',
        description: 'Automated decision-making or profiling',
        severity: 'HIGH',
        likelihood: 'MEDIUM',
        impact: 'HIGH',
      });
    }
    
    // Check cross-border transfers
    if (dataProtectionImpactAssessment.includesTransfer) {
      risks.push({
        id: 'TRANSFER',
        description: 'International data transfer',
        severity: 'MEDIUM',
        likelihood: 'HIGH',
        impact: 'MEDIUM',
      });
    }
    
    const mitigationMeasures = await this.suggestMitigations(risks);
    
    return {
      id: generateUUID(),
      assessmentDate: new Date(),
      risks,
      mitigationMeasures,
      overallRiskLevel: this.calculateOverallRisk(risks),
      recommendation: risks.some(r => r.severity === 'CRITICAL') 
        ? 'HIGH_RISK_PROCESSING_REQUIRES_DPO_CONSULTATION'
        : 'PROCEED_WITH_MITIGATIONS',
    };
  }
}
```

## 3. HIPAA Compliance Patterns

### 3.1 PHI Access Control Implementation

```typescript
// compliance/hipaa/phi-access.ts

interface PHIRecord {
  id: string;
  patientId: string;
  recordType: PHIRecordType;
  data: ProtectedHealthInformation;
  createdAt: Date;
  createdBy: string;
  lastAccessedAt: Date;
  lastAccessedBy?: string;
  auditTrail: PHIAccessEvent[];
}

enum PHIRecordType {
  MEDICAL_RECORD = 'MEDICAL_RECORD',
  BILLING = 'BILLING',
  INSURANCE = 'INSURANCE',
  LAB_RESULT = 'LAB_RESULT',
  PRESCRIPTION = 'PRESCRIPTION',
  IMAGING = 'IMAGING',
  NOTES = 'NOTES',
}

interface ProtectedHealthInformation {
  // PHI fields
  patientName?: string;
  dateOfBirth?: Date;
  socialSecurityNumber?: string;
  address?: string;
  phoneNumber?: string;
  email?: string;
  medicalRecordNumber?: string;
  healthPlanNumber?: string;
  accountNumber?: string;
  certificateLicense?: string;
  vehicleId?: string;
  deviceId?: string;
  webUrl?: string;
  IPAddress?: string;
  biometricId?: string;
  photo?: string;
  anyUniqueIdentifier?: string;
  
  // Clinical data
  diagnosis?: string;
  treatment?: string;
  medications?: string[];
  allergies?: string[];
  labResults?: LabResult[];
  vitalSigns?: VitalSigns;
}

interface PHIAccessEvent {
  timestamp: Date;
  userId: string;
  userRole: string;
  action: PHIAccessAction;
  purpose: AccessPurpose;
  outcome: 'SUCCESS' | 'FAILURE';
  ipAddress: string;
  userAgent: string;
}

type PHIAccessAction = 'CREATE' | 'READ' | 'UPDATE' | 'DELETE' | 'PRINT' | 'EXPORT';
type AccessPurpose = 'TREATMENT' | 'PAYMENT' | 'OPERATIONS' | 'RESEARCH' | 'MARKETING' | 'SELF_PAY';

class PHIAccessControl implements PHIAccessControlInterface {
  constructor(
    private recordRepository: PHIRecordRepository,
    private userRepository: UserRepository,
    private auditLogger: PHIAuditLogger,
    private encryptionService: EncryptionService
  ) {}
  
  async accessRecord(
    userId: string,
    recordId: string,
    purpose: AccessPurpose,
    reason?: string
  ): Promise<PHIRecord> {
    // Check user authorization
    const user = await this.userRepository.findById(userId);
    if (!user) {
      throw new UnauthorizedError('User not found');
    }
    
    // Verify user is covered entity
    if (!user.isCoveredEntity) {
      throw new UnauthorizedError('User not authorized for PHI access');
    }
    
    // Check purpose is allowed
    if (!this.isValidPurpose(purpose)) {
      throw new InvalidPurposeError('Invalid access purpose');
    }
    
    // Log access purpose
    if (purpose === 'OPERATIONS' && reason) {
      await this.logOperationPurpose(userId, reason);
    }
    
    // Retrieve record
    const record = await this.recordRepository.findById(recordId);
    if (!record) {
      throw new NotFoundError('PHI record not found');
    }
    
    // Verify patient match (if required)
    if (user.restrictedToPatients) {
      if (!this.isUserAuthorizedForPatient(userId, record.patientId)) {
        throw new UnauthorizedError('User not authorized for this patient');
      }
    }
    
    // Record access
    await this.recordRepository.recordAccess(recordId, userId, purpose);
    
    // Audit access
    await this.auditLogger.logAccess({
      userId,
      recordId,
      patientId: record.patientId,
      purpose,
      outcome: 'SUCCESS',
      timestamp: new Date(),
    });
    
    // Return record (potentially with decryption)
    return record;
  }
  
  private isValidPurpose(purpose: AccessPurpose): boolean {
    const allowedPurposes: AccessPurpose[] = [
      'TREATMENT',
      'PAYMENT', 
      'OPERATIONS',
      'RESEARCH',
      'SELF_PAY',
    ];
    
    // Marketing requires explicit patient authorization
    return allowedPurposes.includes(purpose);
  }
  
  async createPHIBreakWall(
    userId: string,
    recordId: string,
    justification: string
  ): Promise<void> {
    // Log breaking the wall
    await this.auditLogger.logBreakWall({
      userId,
      recordId,
      justification,
      timestamp: new Date(),
    });
    
    // Update record
    await this.recordRepository.setBreakWall(recordId, {
      brokenBy: userId,
      brokenAt: new Date(),
      justification,
    });
  }
}

interface MinimumNecessaryContext {
  userRole: string;
  purpose: AccessPurpose;
  patientId?: string;
  requestedFields?: string[];
}
```

### 3.2 HIPAA Audit Logging

```typescript
// compliance/hipaa/audit-log.ts

class HIPAABeautyAuditLogger {
  async logPHIAccess(event: PHIAccessLogEvent): Promise<void> {
    const entry: HIPAABeatLogEntry = {
      // Required fields per HIPAA §164.312(b)
      id: generateUUID(),
      date: event.timestamp,
      time: event.timestamp.toISOString(),
      
      // Who accessed
      userId: event.userId,
      userName: event.userName,
      userRole: event.userRole,
      
      // What was accessed
      patientId: event.patientId,
      recordType: event.recordType,
      recordId: event.recordId,
      
      // Action taken
      action: event.action,
      description: event.description,
      
      // Purpose
      accessPurpose: event.purpose,
      justification: event.justification,
      
      // Outcome
      outcome: event.outcome,
      errorDescription: event.errorDescription,
      
      // Security
      ipAddress: event.ipAddress,
      userAgent: event.userAgent,
      workstationId: event.workstationId,
      
      // Metadata
      correlationId: event.correlationId,
      requestId: event.requestId,
    };
    
    await this.saveAuditEntry(entry);
    
    // Check for suspicious activity
    if (this.isSuspiciousActivity(event)) {
      await this.alertSecurityTeam(event);
    }
  }
  
  private isSuspiciousActivity(event: PHIAccessLogEvent): boolean {
    // Check for bulk access
    const recentAccessCount = await this.getRecentAccessCount(
      event.userId,
      event.patientId
    );
    
    if (recentAccessCount > 100) {
      return true;
    }
    
    // Check for access outside normal hours
    const hour = new Date().getHours();
    if (hour < 6 || hour > 22) {
      return true;
    }
    
    // Check for bulk export
    if (event.action === 'EXPORT' && event.recordType === 'BILLING') {
      return true;
    }
    
    return false;
  }
}
```

## 4. Audit Logging Implementation

### 4.1 Comprehensive Audit System

```typescript
// compliance/audit/audit-system.ts

interface AuditLogEntry {
  id: string;
  timestamp: Date;
  version: string;
  
  // Actor
  actor: ActorInfo;
  
  // Action
  action: AuditAction;
  resource: ResourceInfo;
  
  // Context
  context: ActionContext;
  
  // Result
  outcome: OutcomeInfo;
  
  // Data
  previousState?: unknown;
  newState?: unknown;
  changedFields?: string[];
  
  // Compliance
  compliance: ComplianceInfo;
  
  // Metadata
  metadata: Record<string, unknown>;
}

interface ActorInfo {
  id: string;
  type: 'USER' | 'SYSTEM' | 'SERVICE_ACCOUNT';
  email?: string;
  name?: string;
  role?: string;
  ipAddress: string;
  userAgent?: string;
  sessionId?: string;
}

interface AuditAction {
  type: 'CREATE' | 'READ' | 'UPDATE' | 'DELETE' | 'EXECUTE' | 'LOGIN' | 'LOGOUT' | 'EXPORT';
  name: string;
  description?: string;
}

interface ResourceInfo {
  type: string;
  id: string;
  name?: string;
  path?: string;
  parentType?: string;
  parentId?: string;
}

interface ActionContext {
  requestId: string;
  correlationId?: string;
  service: string;
  serviceVersion?: string;
  endpoint?: string;
  httpMethod?: string;
  userAgent?: string;
  timestamp: Date;
}

interface OutcomeInfo {
  status: 'SUCCESS' | 'FAILURE' | 'DENIED' | 'ERROR';
  errorCode?: string;
  errorMessage?: string;
  durationMs?: number;
}

interface ComplianceInfo {
  regulations: string[];
  dataClassification: 'PUBLIC' | 'INTERNAL' | 'CONFIDENTIAL' | 'RESTRICTED' | 'PHI' | 'PII';
  retentionDays?: number;
  legalHold?: boolean;
}

class ComprehensiveAuditLogger {
  private queue: AuditLogEntry[] = [];
  private flushInterval: number = 5000;
  private batchSize: number = 100;
  
  constructor(
    private primaryStorage: AuditStorage,
    private backupStorage: AuditStorage,
    private alertService: AlertService
  ) {
    this.startFlushWorker();
  }
  
  async log(entry: AuditLogEntry): Promise<void> {
    // Validate entry
    this.validate(entry);
    
    // Enrich entry
    const enrichedEntry = this.enrich(entry);
    
    // Add to queue
    this.queue.push(enrichedEntry);
    
    // Flush if batch size reached
    if (this.queue.length >= this.batchSize) {
      await this.flush();
    }
    
    // Alert if critical event
    if (this.isCriticalEvent(enrichedEntry)) {
      await this.alertService.send({
        type: 'CRITICAL_AUDIT_EVENT',
        entry: enrichedEntry,
      });
    }
  }
  
  private async flush(): Promise<void> {
    if (this.queue.length === 0) return;
    
    const entries = this.queue.splice(0, this.batchSize);
    
    try {
      // Write to primary storage
      await this.primaryStorage.writeBatch(entries);
      
      // Write to backup storage for redundancy
      await this.backupStorage.writeBatch(entries);
      
    } catch (error) {
      // Put back in queue for retry
      this.queue.unshift(...entries);
      throw error;
    }
  }
  
  private startFlushWorker(): void {
    setInterval(() => {
      this.flush().catch(console.error);
    }, this.flushInterval);
  }
  
  private validate(entry: AuditLogEntry): void {
    if (!entry.id || !entry.timestamp || !entry.actor || !entry.action) {
      throw new ValidationError('Invalid audit entry: missing required fields');
    }
  }
  
  private enrich(entry: AuditLogEntry): AuditLogEntry {
    return {
      ...entry,
      version: '1.0',
      context: {
        ...entry.context,
        serviceVersion: process.env.SERVICE_VERSION || 'unknown',
      },
    };
  }
  
  private isCriticalEvent(entry: AuditLogEntry): boolean {
    const criticalActions = [
      'USER_LOGIN_FAILED',
      'PASSWORD_CHANGED',
      'ROLE_CHANGED',
      'SENSITIVE_DATA_ACCESSED',
      'DATA_EXPORTED',
      'CONFIGURATION_CHANGED',
      'ADMIN_ACCESS',
    ];
    
    return criticalActions.includes(entry.action.name);
  }
  
  async query(filter: AuditQuery): Promise<AuditQueryResult> {
    return this.primaryStorage.query(filter);
  }
}
```

## 5. Data Retention Implementation

### 5.1 Retention Policy Engine

```typescript
// compliance/retention/policy-engine.ts

interface RetentionPolicy {
  id: string;
  name: string;
  description: string;
  appliesTo: ResourceSelector;
  rules: RetentionRule[];
  status: 'ACTIVE' | 'SUSPENDED' | 'DELETED';
  createdAt: Date;
  lastReviewed: Date;
}

interface RetentionRule {
  id: string;
  condition: RetentionCondition;
  action: RetentionAction;
  priority: number;
  reason: string;
}

interface RetentionCondition {
  type: 'AGE' | 'SIZE' | 'COUNT' | 'CUSTOM';
  field?: string;
  operator: 'GREATER_THAN' | 'LESS_THAN' | 'EQUALS' | 'CONTAINS';
  value: string | number;
  duration?: {
    amount: number;
    unit: 'DAYS' | 'MONTHS' | 'YEARS';
  };
}

interface RetentionAction {
  type: 'DELETE' | 'ARCHIVE' | 'ANONYMIZE' | 'RESTRICT_ACCESS';
  target?: string;
  archiveDestination?: string;
  anonymizationConfig?: AnonymizationConfig;
}

interface ResourceSelector {
  resourceTypes: string[];
  tags?: Record<string, string>;
  createdBefore?: Date;
  createdAfter?: Date;
}

class RetentionPolicyEngine {
  constructor(
    private policyRepository: RetentionPolicyRepository,
    private dataScanner: DataScanner,
    private deletionService: DeletionService,
    private archiveService: ArchiveService,
    private auditLogger: AuditLogger,
    private notificationService: NotificationService
  ) {}
  
  async evaluatePolicies(): Promise<RetentionAction[]> {
    const actions: RetentionAction[] = [];
    
    // Get active policies
    const policies = await this.policyRepository.findActive();
    
    for (const policy of policies) {
      // Find matching resources
      const resources = await this.dataScanner.findMatchingResources(policy.appliesTo);
      
      // Evaluate each resource against rules
      for (const resource of resources) {
        for (const rule of policy.rules.sort((a, b) => a.priority - b.priority)) {
          if (this.evaluateCondition(rule.condition, resource)) {
            actions.push(rule.action);
            
            // Execute action (async)
            this.executeAction(rule.action, resource);
            
            // Only apply first matching rule
            break;
          }
        }
      }
    }
    
    return actions;
  }
  
  private evaluateCondition(condition: RetentionCondition, resource: DataResource): boolean {
    if (condition.type === 'AGE') {
      const age = this.calculateAge(resource, condition.duration.unit);
      const threshold = condition.duration.amount;
      
      switch (condition.operator) {
        case 'GREATER_THAN':
          return age > threshold;
        case 'LESS_THAN':
          return age < threshold;
        case 'EQUALS':
          return age === threshold;
      }
    }
    
    return false;
  }
  
  private async executeAction(action: RetentionAction, resource: DataResource): Promise<void> {
    const executionId = generateUUID();
    
    try {
      switch (action.type) {
        case 'DELETE':
          await this.deletionService.delete(resource, {
            executionId,
            reason: 'Retention policy',
          });
          break;
          
        case 'ARCHIVE':
          await this.archiveService.archive(resource, action.archiveDestination);
          break;
          
        case 'ANONYMIZE':
          await this.anonymizeResource(resource, action.anonymizationConfig);
          break;
          
        case 'RESTRICT_ACCESS':
          await this.restrictAccess(resource);
          break;
      }
      
      await this.auditLogger.logRetentionAction({
        executionId,
        resourceId: resource.id,
        actionType: action.type,
        outcome: 'SUCCESS',
      });
      
    } catch (error) {
      await this.auditLogger.logRetentionAction({
        executionId,
        resourceId: resource.id,
        actionType: action.type,
        outcome: 'FAILURE',
        error: (error as Error).message,
      });
      
      await this.notificationService.notifyRetentionFailure(resource, action, error);
    }
  }
  
  private async anonymizeResource(
    resource: DataResource,
    config: AnonymizationConfig
  ): Promise<void> {
    const rules: AnonymizationRule[] = config.rules;
    
    for (const rule of rules) {
      await this.applyAnonymizationRule(resource, rule);
    }
  }
}
```

## 6. Complete Compliance Checklist Implementation

### 6.1 Compliance Verification System

```typescript
// compliance/verification/checklist-system.ts

interface ComplianceCheck {
  id: string;
  framework: ComplianceFramework;
  category: string;
  requirement: string;
  description: string;
  severity: 'CRITICAL' | 'HIGH' | 'MEDIUM' | 'LOW';
  checks: CheckDefinition[];
  lastChecked?: Date;
  status: CheckStatus;
  findings: Finding[];
  remediation: RemediationStep[];
}

interface CheckDefinition {
  id: string;
  name: string;
  type: 'AUTOMATED' | 'MANUAL' | 'HYBRID';
  implementation: string;
  schedule?: string;
  sampleSize?: number;
}

interface Finding {
  id: string;
  severity: 'CRITICAL' | 'HIGH' | 'MEDIUM' | 'LOW' | 'INFO';
  title: string;
  description: string;
  resource?: string;
  evidence: Evidence[];
  detectedAt: Date;
  resolvedAt?: Date;
}

interface RemediationStep {
  id: string;
  description: string;
  status: 'PENDING' | 'IN_PROGRESS' | 'COMPLETED';
  assignee?: string;
  dueDate?: Date;
  completedAt?: Date;
}

type ComplianceFramework = 'SOC2' | 'GDPR' | 'HIPAA' | 'PCI_DSS' | 'ISO27001' | 'CUSTOM';
type CheckStatus = 'PASS' | 'FAIL' | 'WARNING' | 'NOT_APPLICABLE' | 'IN_PROGRESS';

class ComplianceVerificationSystem {
  constructor(
    private checkRepository: ComplianceCheckRepository,
    private scanner: SecurityScanner,
    private evidenceCollector: EvidenceCollector,
    private ticketingSystem: TicketingSystem
  ) {}
  
  async runCheck(checkId: string): Promise<void> {
    const check = await this.checkRepository.findById(checkId);
    if (!check) {
      throw new NotFoundError('Check not found');
    }
    
    // Update status
    await this.checkRepository.updateStatus(checkId, 'IN_PROGRESS');
    
    const findings: Finding[] = [];
    
    for (const definition of check.checks) {
      try {
        const result = await this.executeCheck(definition);
        
        if (result.failed) {
          findings.push({
            id: generateUUID(),
            severity: result.severity,
            title: result.title,
            description: result.description,
            resource: result.resource,
            evidence: result.evidence,
            detectedAt: new Date(),
          });
        }
        
      } catch (error) {
        findings.push({
          id: generateUUID(),
          severity: 'HIGH',
          title: 'Check execution failed',
          description: (error as Error).message,
          evidence: [],
          detectedAt: new Date(),
        });
      }
    }
    
    // Update check with findings
    const status = this.determineStatus(findings);
    await this.checkRepository.updateResults(checkId, findings, status);
    
    // Create tickets for failed checks
    for (const finding of findings.filter(f => f.severity === 'CRITICAL' || f.severity === 'HIGH')) {
      await this.ticketingSystem.createTicket({
        title: `[${check.requirement}] ${finding.title}`,
        description: finding.description,
        priority: finding.severity === 'CRITICAL' ? 'URGENT' : 'HIGH',
        labels: [check.framework, check.category],
      });
    }
  }
  
  private async executeCheck(definition: CheckDefinition): Promise<CheckResult> {
    switch (definition.type) {
      case 'AUTOMATED':
        return this.scanner.run(definition.implementation);
        
      case 'MANUAL':
        return { failed: false, findings: [] }; // Manual checks need human review
        
      case 'HYBRID':
        const automatedResult = await this.scanner.run(definition.implementation);
        const evidence = await this.evidenceCollector.collect(definition.id);
        return { ...automatedResult, evidence };
    }
  }
  
  private determineStatus(findings: Finding[]): CheckStatus {
    if (findings.some(f => f.severity === 'CRITICAL')) {
      return 'FAIL';
    }
    if (findings.some(f => f.severity === 'HIGH')) {
      return 'WARNING';
    }
    return 'PASS';
  }
  
  async generateReport(framework: ComplianceFramework): Promise<ComplianceReport> {
    const checks = await this.checkRepository.findByFramework(framework);
    
    return {
      framework,
      generatedAt: new Date(),
      summary: {
        total: checks.length,
        passed: checks.filter(c => c.status === 'PASS').length,
        failed: checks.filter(c => c.status === 'FAIL').length,
        warnings: checks.filter(c => c.status === 'WARNING').length,
      },
      checks: checks.map(c => ({
        requirement: c.requirement,
        status: c.status,
        findings: c.findings,
        lastChecked: c.lastChecked,
      })),
      evidence: await this.evidenceCollector.getEvidenceForFramework(framework),
    };
  }
}

interface CheckResult {
  failed: boolean;
  severity?: 'CRITICAL' | 'HIGH' | 'MEDIUM' | 'LOW';
  title?: string;
  description?: string;
  resource?: string;
  evidence: Evidence[];
}

interface Evidence {
  type: 'SCREENSHOT' | 'LOG' | 'CONFIG' | 'QUERY_RESULT';
  data: unknown;
  collectedAt: Date;
}
```

## 7. Decision Matrices

### 7.1 Data Classification Decision Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           Data Classification Decision Matrix                           │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Data Type                     │ Classification    │ Handling Requirements             │
├───────────────────────────────┼───────────────────┼────────────────────────────────────┤
│ Public content                │ PUBLIC            │ No restrictions                    │
├───────────────────────────────┼───────────────────┼────────────────────────────────────┤
│ Internal docs                 │ INTERNAL          │ Auth required                     │
├───────────────────────────────┼───────────────────┼────────────────────────────────────┤
│ Customer PII                  │ CONFIDENTIAL      │ Encryption, access control, audit │
├───────────────────────────────┼───────────────────┼────────────────────────────────────┤
│ Financial data                │ RESTRICTED        │ Encryption, MFA, audit, retention │
├───────────────────────────────┼───────────────────┼────────────────────────────────────┤
│ Health records (HIPAA)        │ PHI               │ Full HIPAA compliance             │
├───────────────────────────────┼───────────────────┼────────────────────────────────────┤
│ EU citizen data (GDPR)        │ RESTRICTED        │ GDPR controls, data residency    │
├───────────────────────────────┼───────────────────┼────────────────────────────────────┤
│ Payment card data (PCI)       │ CARDHOLDER_DATA   │ PCI DSS compliance                │
├───────────────────────────────┼───────────────────┼────────────────────────────────────┤
│ Authentication credentials    │ RESTRICTED        │ Hashing, encryption, no logging   │
├───────────────────────────────┼───────────────────┼────────────────────────────────────┤
│ Trade secrets                 │ RESTRICTED        │ Encryption, access logging        │
└───────────────────────────────┴───────────────────┴────────────────────────────────────┘
```

### 7.2 Compliance Framework Selection

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                         Compliance Framework Selection Matrix                            │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Business Type              │ Required Frameworks               │ Recommended Add-ons     │
├────────────────────────────┼────────────────────────────────────┼────────────────────────┤
│ SaaS (US customers)        │ SOC2 Type II                      │ GDPR if EU customers   │
├────────────────────────────┼────────────────────────────────────┼────────────────────────┤
│ EU-based business          │ GDPR                              │ SOC2 for US expansion  │
├────────────────────────────┼────────────────────────────────────┼────────────────────────┤
│ Healthcare (US)            │ HIPAA                             │ SOC2, HITRUST          │
├────────────────────────────┼────────────────────────────────────┼────────────────────────┤
│ E-commerce                 │ PCI DSS                           │ SOC2                   │
├────────────────────────────┼────────────────────────────────────┼────────────────────────┤
│ Financial services         │ SOC2, PCI DSS                    │ ISO 27001              │
├────────────────────────────┼────────────────────────────────────┼────────────────────────┤
│ Government contractor      │ FedRAMP, NIST                    │ SOC2, ISO 27001        │
└────────────────────────────┴────────────────────────────────────┴────────────────────────┘
```

## 8. Anti-Patterns

### 8.1 Compliance Anti-Patterns

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                            Compliance Anti-Patterns to Avoid                            │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Anti-Pattern                    │ Problem                       │ Solution                │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No audit logging               │ Compliance violation           │ Implement comprehensive│
│                                 │ No evidence for audit          │ audit logging          │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Weak access controls           │ Unauthorized access            │ RBAC, MFA, least priv  │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No data classification         │ Improper handling              │ Classify all data      │
│                                 │ Missing controls               │ first                  │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Missing retention policies     │ Data accumulation              │ Define retention for   │
│                                 │ Compliance risk                │ each data type        │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No encryption at rest          │ Data exposure                  │ Encrypt sensitive data │
│                                 │ Regulatory violation           │ at rest and in transit │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Ignoring data subject rights   │ GDPR violations                │ Implement rights mgmt  │
│                                 │ Heavy fines                    │ workflows              │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Manual compliance checks       │ Human error                    │ Automate where possible│
│                                 │ Inconsistency                 │ Use continuous monitor│
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No third-party oversight       │ Vendor risk                   │ Vendor assessments     │
│                                 │ Supply chain issues            │ and monitoring         │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Incomplete DPIA               │ GDPR violation                 │ Conduct thorough DPIAs │
│                                 │ Missing risk mitigation        │ for all high-risk      │
│                                 │                               │ processing             │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No incident response plan      │ Breach chaos                   │ Create and test IRP    │
│                                 │ Regulatory delays              │ regularly              │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Storing what you don't need    │ Increased risk                 │ Data minimization      │
│                                 │ Higher retention costs         │ principle              │
└─────────────────────────────────┴───────────────────────────────┴────────────────────────┘
```

---

## Links

### SOC2
- [SOC2 Trust Services Criteria](https://www.aicpa.org/soc2)
- [SOC2 Audit Guide](https://www.socreports.com/)
- [SSAE 18 Standards](https://www.aicpa.org/soc18)

### GDPR
- [GDPR Official Text](https://gdpr-info.eu/)
- [ICO GDPR Guidance](https://ico.org.uk/for-organisations/guide-to-data-protection/)
- [GDPR Requirements Checklist](https://www.enisa.europa.eu/publications/gdpr-compliance)

### HIPAA
- [HHS HIPAA Guidance](https://www.hhs.gov/hipaa/index.html)
- [HIPAA Security Rule](https://www.hhs.gov/hipaa/for-professionals/security/index.html)
- [HIPAA Audit Protocol](https://www.hhs.gov/hipaa/for-professionals/guidance/audit-evaluation/index.html)

### PCI DSS
- [PCI DSS Standards](https://www.pcisecuritystandards.org/)
- [PCI DSS Documentation](https://docs.prism习)

### ISO 27001
- [ISO 27001 Standard](https://www.iso.org/isoiec-27001-information-security.html)
- [ISO 27001 Documentation](https://www.iso27001standard.com/)

### Compliance Tools
- [Vanta - Compliance automation](https://www.vanta.com/)
- [Drata - Compliance automation](https://www.drata.com/)
- [Secureframe - Compliance](https://www.secureframe.com/)
- [OneTrust - Privacy compliance](https://www.onetrust.com/)

### Audit Logging
- [Elasticsearch for Audit](https://www.elastic.co/auditbeat)
- [Splunk Audit Logging](https://www.splunk.com/)
- [AWS CloudTrail](https://aws.amazon.com/cloudtrail/)