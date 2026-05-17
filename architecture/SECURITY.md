# SECURITY.md - Security Architecture (DENSE)

**Authority:** guidance (security patterns, threat modeling, and defense in depth)
**Layer:** Guides
**Binding:** No
**Scope:** security principles, threat modeling, and defensive patterns
**Non-goals:** specific security tools, compliance checklists

---

## 1. Security Principles

### 1.1 Defense in Depth
**No single point of failure.**
- Multiple layers of security
- If one layer fails, others protect
- No "silver bullet" security measure
- Assume breach will happen

**Layers:**
1. **Perimeter:** Firewalls, WAF, DDoS protection
2. **Network:** Segmentation, VPCs, encryption
3. **Application:** Input validation, auth, authorization
4. **Data:** Encryption, access controls, masking
5. **Physical:** Data center security (cloud handles this)

**Defense-in-Depth Implementation Matrix:**

| Layer | Control | Implementation | Verification |
|-------|---------|----------------|--------------|
| Perimeter | WAF | AWS WAF, Cloudflare, Imposter | OWASP Top 10 coverage |
| Perimeter | DDoS | AWS Shield, Cloudflare | Attack simulation |
| Network | Segmentation | VPC, Security Groups, NACLs | Network topology audit |
| Network | Encryption | mTLS, WireGuard | Certificate rotation |
| Application | AuthN | OAuth 2.0, OIDC, MFA | AuthN testing |
| Application | AuthZ | RBAC, ABAC, Zanzibar | Privilege escalation testing |
| Data | Encryption | AES-256, KMS | Key rotation test |
| Data | Masking | Column-level encryption | Data access audit |

### 1.2 Principle of Least Privilege
**Give minimum access necessary.**
- Users: Only permissions needed for role
- Services: Only API calls needed to function
- Applications: Only file/database access required
- Regular access reviews

**IAM Policy Schema:**

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "IAMPolicy",
  "type": "object",
  "required": ["Version", "Statement"],
  "properties": {
    "Version": {
      "type": "string",
      "const": "2012-10-17"
    },
    "Statement": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["Effect", "Action", "Resource"],
        "properties": {
          "Sid": {
            "type": "string",
            "maxLength": 64
          },
          "Effect": {
            "type": "string",
            "enum": ["Allow", "Deny"]
          },
          "Principal": {
            "oneOf": [
              {"type": "string"},
              {"type": "object"},
              {"type": "array"}
            ]
          },
          "Action": {
            "oneOf": [
              {"type": "string"},
              {"type": "array"}
            ],
            "description": "AWS API actions. Use specific actions, not *"
          },
          "NotAction": {
            "oneOf": [
              {"type": "string"},
              {"type": "array"}
            ],
            "description": "Actions to exclude from this policy"
          },
          "Resource": {
            "oneOf": [
              {"type": "string"},
              {"type": "array"}
            ],
            "description": "Specific resource ARNs, not wildcards"
          },
          "NotResource": {
            "oneOf": [
              {"type": "string"},
              {"type": "array"}
            ],
            "description": "Resources to exclude"
          },
          "Condition": {
            "type": "object",
            "description": "Conditions that must be true for this policy to apply"
          }
        }
      }
    }
  }
}
```

**Example Service Policy (Least Privilege):**

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AllowDynamoDBRead",
      "Effect": "Allow",
      "Action": [
        "dynamodb:GetItem",
        "dynamodb:Query",
        "dynamodb:Scan"
      ],
      "Resource": "arn:aws:dynamodb:us-east-1:123456789:table/production-users"
    },
    {
      "Sid": "AllowDynamoDBReadOnlyAttributes",
      "Effect": "Allow",
      "Action": [
        "dynamodb:DescribeTable",
        "dynamodb:GetItem",
        "dynamodb:Query"
      ],
      "Resource": "arn:aws:dynamodb:us-east-1:123456789:table/production-users/index/*"
    },
    {
      "Sid": "DenyDynamoDBWrite",
      "Effect": "Deny",
      "Action": [
        "dynamodb:PutItem",
        "dynamodb:UpdateItem",
        "dynamodb:DeleteItem",
        "dynamodb:BatchWriteItem"
      ],
      "Resource": "arn:aws:dynamodb:us-east-1:123456789:table/production-users",
      "Condition": {
        "ArnNotEquals": {
          "aws:PrincipalARN": "arn:aws:iam::123456789:role/admin-write-role"
        }
      }
    },
    {
      "Sid": "AllowS3ReadOnly",
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:GetObjectVersion",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::production-assets",
        "arn:aws:s3:::production-assets/*"
      ],
      "Condition": {
        "IpAddress": {
          "aws:SourceIp": ["10.0.0.0/8", "172.16.0.0/12"]
        }
      }
    },
    {
      "Sid": "DenyDeleteWithout MFA",
      "Effect": "Deny",
      "Action": [
        "dynamodb:DeleteItem",
        "s3:DeleteObject",
        "s3:DeleteObjectVersion"
      ],
      "Resource": "*",
      "Condition": {
        "Bool": {
          "aws:MultiFactorAuthPresent": "false"
        }
      }
    }
  ]
}
```

### 1.3 Zero Trust
**Never trust, always verify.**
- No implicit trust based on network location
- Verify every request, every time
- Assume network is compromised
- Strong authentication everywhere

**Zero Trust Architecture Schema:**

```yaml
# Zero Trust Network Architecture
ZeroTrustArchitecture:
  identity:
    provider: oidc
    issuer: https://auth.example.com
    mfa:
      required: true
      methods:
        - hardware_key  # FIDO2/WebAuthn primary
        - totp          # TOTP fallback
        - push          # Mobile push notification
    
    device_posture:
      enrolled_devices_required: true
      minimum_os_version:
        macos: "13.0"
        ios: "16.0"
        android: "13"
      encryption_required: true
      screen_lock_required: true
      firewall_required: true
    
    continuous_authentication:
      enabled: true
      reauth_on_risk: true
      risk_signals:
        - new_device
        - new_location
        - unusual_time
        - impossible_travel
        - credential stuffing_detected
  
  network:
    microsegmentation:
      enabled: true
      granularity: workload
      
    east_west_traffic:
      require_authentication: true
      mtls_required: true
      authorization_policy: every_request
      
    north_south_traffic:
      waf_required: true
      rate_limiting: true
      bot_protection: true
      
    identity_aware_proxy:
      enabled: true
      single_sign_on: true
      context_aware_access: true
  
  data:
    classification_levels:
      - public
      - internal
      - confidential
      - restricted  # PII, PHI, financial
      
    encryption:
      at_rest: aes-256-gcm
      in_transit: tls-1.3
      key_rotation_days: 90
      
    access_control:
      need_to_know: true
      break_glass_procedure: true
      
  monitoring:
    audit_all_access: true
    anomaly_detection: true
    user_behavior_analytics: true
    threat_intelligence: integrated
```

### 1.4 Security by Design
**Security is not a feature; it's a property.**
- Consider security from design phase
- Threat model before implementation
- Security requirements are functional requirements
- Security reviews for architectural changes

### 1.5 Production Mindset
Security is a property of the system, not a feature layer. Systems that require security to be "added" before release have already failed at architecture:

- **Assume the perimeter is already breached:** Design every component assuming a network-adjacent attacker exists. Lateral movement must be architecturally impossible, not just blocked by policy. Microsegmentation, mTLS, and zero-trust identity make this enforceable.
- **Trust is technical debt:** Every trusted component or interface is a potential pivot point. Minimize trust boundaries explicitly. Document what is trusted, why, and what the consequences of that trust being violated are.
- **Compliance is the floor, not the ceiling:** Meeting SOC2 or HIPAA means you satisfy a minimum legal standard. Real security requires adversarial thinking. Red-team your own architecture before an attacker does.
- **Security must be automated to scale:** Manual security reviews on every PR are a bottleneck that developers will eventually route around. SAST, DAST, dependency scanning, and secret detection must run in CI on every change, without exceptions.
- **Policy exceptions are vulnerabilities:** An exception to a security policy is a vulnerability with documentation. If a policy is consistently too strict to follow, fix the policy through a formal process — do not grant individual exceptions.
- **Identity is the perimeter in cloud-native systems:** IP-based trust is meaningless in elastic, multi-tenant infrastructure. Use strong cryptographic identity (mTLS, SPIFFE/SPIRE) for every service-to-service interaction.
- **Immutable infrastructure limits blast radius:** A compromised instance must not be patched in place. Kill it and redeploy from a known-good image. This is only possible if compute is stateless and infrastructure is defined in code.
- **Secure defaults are the only reliable defaults:** Any configuration, API, or library that requires explicit action to enable security will eventually ship insecure. Defaults must be secure. Opt-in for relaxed behavior, never opt-in for security.
- **Agents must operate with minimum necessary context:** When agents process external data or operate on the codebase, they must have access only to the files, tools, and credentials their specific task requires. Over-privileged agents are a significant attack surface. Scope everything.
- **Validation is the final gate:** In Decapod, `decapod validate` is the last line of automated defense. A change that violates a security specification cannot be promoted. This gate is non-negotiable.

---

## 2. Threat Modeling

### 2.1 STRIDE Methodology
**Threat categories:**
- **S**poofing: Pretending to be someone else
- **T**ampering: Modifying data/code
- **R**epudiation: Denying actions
- **I**nformation Disclosure: Leaking data
- **D**enial of Service: Making system unavailable
- **E**levation of Privilege: Gaining unauthorized access

**STRIDE Mitigation Matrix:**

| Threat | Mitigation | Technology |
|--------|------------|------------|
| Spoofing | Authentication | OAuth 2.0, OIDC, mTLS, certificates |
| Tampering | Integrity verification | Digital signatures, HMAC, MAC |
| Repudiation | Audit logging, digital signatures | Cryptographic audit logs |
| Information Disclosure | Encryption, access controls | TLS, ACLs, KMS |
| Denial of Service | Redundancy, rate limiting | Multi-AZ, WAF, CDN |
| Elevation of Privilege | Authorization, least privilege | RBAC, ABAC, Zanzibar |

### 2.2 Attack Surface Analysis
**Identify entry points:**
- APIs and endpoints
- Authentication mechanisms
- File uploads/downloads
- Admin interfaces
- Third-party integrations
- Logging and monitoring

**Attack Surface Schema:**

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttackSurface",
  "type": "object",
  "required": ["application", "entryPoints"],
  "properties": {
    "application": {
      "type": "object",
      "properties": {
        "name": {"type": "string"},
        "version": {"type": "string"},
        "description": {"type": "string"}
      }
    },
    "entryPoints": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["type", "name", "transport", "authentication"],
        "properties": {
          "type": {
            "enum": [
              "api_rest", "api_graphql", "api_grpc", 
              "web_ui", "admin_ui", "websocket",
              "file_upload", "webhook", "internal_service"
            ]
          },
          "name": {"type": "string"},
          "description": {"type": "string"},
          "transport": {
            "type": "object",
            "properties": {
              "protocol": {"enum": ["https", "wss", "http", "tcp", "uds"]},
              "port": {"type": "integer"},
              "tls_version": {"type": "string"},
              "mtls_required": {"type": "boolean"}
            }
          },
          "authentication": {
            "type": "object",
            "properties": {
              "type": {
                "enum": ["none", "api_key", "basic", "bearer", "oauth2", "saml", "mtls"]
              },
              "session_management": {
                "type": "string",
                "enum": ["none", "stateful", "stateless_jwt", "refresh_token"]
              },
              "mfa_required": {"type": "boolean"},
              "password_policy": {"type": "string"}
            }
          },
          "authorization": {
            "type": "string",
            "enum": ["open", "authenticated", "role_based", "attribute_based"]
          },
          "rate_limiting": {
            "type": "object",
            "properties": {
              "enabled": {"type": "boolean"},
              "requests_per_minute": {"type": "integer"},
              "burst": {"type": "integer"}
            }
          },
          "input_validation": {
            "type": "object",
            "properties": {
              "schema_validation": {"type": "boolean"},
              "sanitization": {"type": "boolean"},
              "max_request_size_bytes": {"type": "integer"}
            }
          },
          "trust_level": {
            "type": "string",
            "enum": ["internal", "partner", "public"],
            "description": "Assumed trust level of this entry point"
          }
        }
      }
    },
    "data_assets": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "classification": {
            "enum": ["public", "internal", "confidential", "restricted"]
          },
          "storage_location": {"type": "string"},
          "encryption": {"type": "string"},
          "retention_days": {"type": "integer"}
        }
      }
    },
    "trust_boundaries": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "from": {"type": "string"},
          "to": {"type": "string"},
          "data_flow": {"type": "string"}
        }
      }
    }
  }
}
```

### 2.3 Threat Modeling Process
1. **Diagram:** Create data flow diagram
2. **Identify:** Entry points and trust boundaries
3. **STRIDE:** Apply threat categories
4. **Rate:** Risk severity (likelihood × impact)
5. **Mitigate:** Design countermeasures
6. **Validate:** Review and test

**Threat Model Example:**

```json
{
  "threat_model": {
    "application": "User Management Service",
    "date": "2026-05-16",
    "reviewers": ["security-team@example.com"],
    
    "data_flow_diagram": {
      "external_entities": [
        {"id": "user", "name": "End User", "trust_level": "untrusted"},
        {"id": "admin", "name": "Administrator", "trust_level": "authenticated"}
      ],
      "processes": [
        {"id": "api_gateway", "name": "API Gateway", "trust_level": "trusted"},
        {"id": "user_service", "name": "User Service", "trust_level": "trusted"},
        {"id": "auth_service", "name": "Auth Service", "trust_level": "trusted"},
        {"id": "database", "name": "User Database", "trust_level": "trusted"}
      ],
      "data_stores": [
        {"id": "users_db", "name": "Users Table", "classification": "restricted"},
        {"id": "sessions_redis", "name": "Session Store", "classification": "internal"},
        {"id": "audit_log", "name": "Audit Log", "classification": "confidential"}
      ],
      "flows": [
        {"from": "user", "to": "api_gateway", "label": "HTTPS"},
        {"from": "api_gateway", "to": "auth_service", "label": "mTLS"},
        {"from": "auth_service", "to": "sessions_redis", "label": "Redis"},
        {"from": "api_gateway", "to": "user_service", "label": "mTLS"},
        {"from": "user_service", "to": "users_db", "label": "PostgreSQL"}
      ]
    },
    
    "threats": [
      {
        "id": "T1",
        "category": "STRIDE",
        "type": "Spoofing",
        "title": "User credential theft via phishing",
        "description": "Attacker impersonates legitimate user by stealing credentials",
        "likelihood": "high",
        "impact": "critical",
        "risk_score": "critical",
        "affected_components": ["api_gateway"],
        "mitigation": {
          "control": "MFA required for all users",
          "implementation": "TOTP or hardware key (FIDO2)",
          "validation": "MFA enrollment rate > 95%"
        }
      },
      {
        "id": "T2",
        "category": "STRIDE",
        "type": "Tampering",
        "title": "Request parameter manipulation",
        "description": "Attacker modifies request parameters to access unauthorized data",
        "likelihood": "medium",
        "impact": "high",
        "risk_score": "high",
        "affected_components": ["user_service"],
        "mitigation": {
          "control": "Input validation + schema enforcement",
          "implementation": "JSON Schema validation on all endpoints",
          "validation": "Fuzzing tests pass"
        }
      },
      {
        "id": "T3",
        "category": "STRIDE",
        "type": "Information Disclosure",
        "title": "Database injection via ORM exploit",
        "description": "Attacker exploits ORM vulnerability to extract data",
        "likelihood": "low",
        "impact": "critical",
        "risk_score": "high",
        "affected_components": ["user_service", "users_db"],
        "mitigation": {
          "control": "ORM query validation + parameterized queries",
          "implementation": "Only stored procedure access, no raw SQL",
          "validation": "DAST with SQL injection probes"
        }
      },
      {
        "id": "T4",
        "category": "STRIDE",
        "type": "Denial of Service",
        "title": "Login endpoint brute force",
        "description": "Attacker attempts brute force on login endpoint",
        "likelihood": "high",
        "impact": "medium",
        "risk_score": "high",
        "affected_components": ["api_gateway", "auth_service"],
        "mitigation": {
          "control": "Rate limiting + account lockout",
          "implementation": "5 failed attempts = 15 min lockout, progressive backoff",
          "validation": "Load test at 1000 req/s shows graceful degradation"
        }
      },
      {
        "id": "T5",
        "category": "STRIDE",
        "type": "Elevation of Privilege",
        "title": "IDOR on user profile access",
        "description": "User accesses another user's profile via direct object reference",
        "likelihood": "medium",
        "impact": "high",
        "risk_score": "high",
        "affected_components": ["user_service"],
        "mitigation": {
          "control": "Authorization check on every resource access",
          "implementation": "User can only access own resources (checked in service layer)",
          "validation": "Integration tests verify isolation"
        }
      }
    ],
    
    "security_controls": [
      {
        "control_id": "SC1",
        "name": "Multi-Factor Authentication",
        "category": "Authentication",
        "status": "required",
        "implementation": "All user accounts must have at least one MFA method enrolled"
      },
      {
        "control_id": "SC2",
        "name": "TLS 1.3 for all traffic",
        "category": "Transport Security",
        "status": "required",
        "implementation": "TLS 1.2 minimum, 1.3 preferred. Weak ciphers disabled"
      },
      {
        "control_id": "SC3",
        "name": "JWT with short expiry",
        "category": "Session Management",
        "status": "required",
        "implementation": "15 minute access tokens, 7 day refresh tokens with rotation"
      },
      {
        "control_id": "SC4",
        "name": "Audit logging for all mutations",
        "category": "Audit",
        "status": "required",
        "implementation": "Every state change logged with actor, action, timestamp"
      }
    ]
  }
}
```

---

## 3. Authentication

### 3.1 Password Requirements Schema

```json
{
  "PasswordPolicySchema": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "PasswordPolicy",
    "type": "object",
    "required": ["min_length", "complexity", "breach_detection"],
    "properties": {
      "min_length": {
        "type": "integer",
        "minimum": 12,
        "maximum": 128,
        "description": "Minimum password length"
      },
      "max_length": {
        "type": "integer",
        "minimum": 64,
        "maximum": 1024,
        "default": 128
      },
      "complexity": {
        "type": "object",
        "properties": {
          "require_uppercase": {
            "type": "boolean",
            "default": true
          },
          "require_lowercase": {
            "type": "boolean",
            "default": true
          },
          "require_digit": {
            "type": "boolean",
            "default": true
          },
          "require_special": {
            "type": "boolean",
            "default": true
          },
          "disallowed_patterns": {
            "type": "array",
            "items": {
              "type": "string"
            },
            "description": "Common patterns to disallow (e.g., 'password', '123456')"
          }
        }
      },
      "breach_detection": {
        "type": "object",
        "properties": {
          "enabled": {
            "type": "boolean",
            "default": true
          },
          "api": {
            "type": "string",
            "description": "URL to breach detection API (e.g., Have I Been Pwned)"
          },
          "check_on_login": {
            "type": "boolean",
            "default": true
          },
          "check_on_password_set": {
            "type": "boolean",
            "default": true
          }
        }
      },
      "expiration": {
        "type": "object",
        "properties": {
          "enabled": {
            "type": "boolean",
            "default": false
          },
          "max_age_days": {
            "type": "integer",
            "default": 90
          },
          "warn_days_before_expiry": {
            "type": "integer",
            "default": 14
          }
        }
      },
      "history": {
        "type": "object",
        "properties": {
          "enabled": {
            "type": "boolean",
            "default": true
          },
          "prevent_reuse_count": {
            "type": "integer",
            "minimum": 1,
            "maximum": 24,
            "default": 12
          }
        }
      },
      "rate_limiting": {
        "type": "object",
        "properties": {
          "max_attempts": {
            "type": "integer",
            "default": 5
          },
          "lockout_duration_minutes": {
            "type": "integer",
            "default": 15
          },
          "progressive_lockout": {
            "type": "boolean",
            "default": true
          }
        }
      },
      "storage": {
        "type": "object",
        "properties": {
          "algorithm": {
            "type": "string",
            "enum": ["argon2id", "bcrypt", "scrypt", "pbkdf2"],
            "default": "argon2id"
          },
          "parameters": {
            "type": "object",
            "properties": {
              "argon2id": {
                "type": "object",
                "properties": {
                  "memory_kib": {"type": "integer", "default": 65536},
                  "iterations": {"type": "integer", "default": 3},
                  "parallelism": {"type": "integer", "default": 4}
                }
              },
              "bcrypt": {
                "type": "object",
                "properties": {
                  "cost": {"type": "integer", "default": 12}
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

### 3.2 Multi-Factor Authentication (MFA)

**MFA Configuration Schema:**

```json
{
  "MFAConfigurationSchema": {
    "type": "object",
    "required": ["methods", "policy"],
    "properties": {
      "methods": {
        "type": "array",
        "items": {
          "type": "object",
          "required": ["type", "enabled"],
          "properties": {
            "type": {
              "enum": [
                "totp",        # Time-based One-Time Password
                "hotp",        # HMAC-based One-Time Password
                "push",        # Push notification
                "sms",         # SMS (least secure)
                "email",       # Email code
                "fido2",       # Hardware security key
                "passkey"      # WebAuthn/Passkey
              ]
            },
            "enabled": {"type": "boolean"},
            "rank": {
              "type": "integer",
              "description": "Priority (lower = higher priority/stronger)"
            },
            "config": {
              "type": "object",
              "properties": {
                "issuer_name": {
                  "type": "string",
                  "description": "Issuer in TOTP QR code"
                },
                "algorithm": {
                  "type": "string",
                  "enum": ["SHA1", "SHA256", "SHA512"],
                  "default": "SHA1"
                },
                "digits": {
                  "type": "integer",
                  "enum": [6, 8],
                  "default": 6
                },
                "period_seconds": {
                  "type": "integer",
                  "default": 30
                }
              }
            }
          }
        }
      },
      "policy": {
        "type": "object",
        "properties": {
          "required_for_roles": {
            "type": "array",
            "items": {
              "type": "string"
            },
            "description": "Roles that MUST have MFA enrolled"
          },
          "required_for_api": {
            "type": "boolean",
            "default": false
          },
          "allow_recovery_codes": {
            "type": "boolean",
            "default": true
          },
          "recovery_code_count": {
            "type": "integer",
            "default": 10
          },
          "grace_period_hours": {
            "type": "integer",
            "default": 0,
            "description": "Hours before MFA is enforced for existing users"
          }
        }
      }
    }
  }
}
```

### 3.3 Session Management

**JWT Configuration Schema:**

```json
{
  "JWTSchema": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "JWTConfiguration",
    "type": "object",
    "required": ["access_token", "refresh_token"],
    "properties": {
      "access_token": {
        "type": "object",
        "properties": {
          "algorithm": {
            "type": "string",
            "enum": ["RS256", "ES256"],
            "description": "Asymmetric algorithms only (HS256 is forbidden)"
          },
          "expiry_seconds": {
            "type": "integer",
            "minimum": 60,
            "maximum": 3600,
            "default": 900,
            "description": "15 minutes default"
          },
          "claims": {
            "type": "object",
            "required": ["sub", "iat", "exp", "jti"],
            "properties": {
              "sub": {
                "type": "string",
                "description": "Subject (user ID)"
              },
              "iat": {
                "type": "integer",
                "description": "Issued at timestamp"
              },
              "exp": {
                "type": "integer",
                "description": "Expiration timestamp"
              },
              "jti": {
                "type": "string",
                "description": "JWT ID (for revocation)"
              },
              "iss": {
                "type": "string",
                "description": "Issuer URL"
              },
              "aud": {
                "type": "string",
                "description": "Audience (client ID)"
              },
              "scope": {
                "type": "array",
                "items": {"type": "string"},
                "description": "OAuth scopes"
              },
              "roles": {
                "type": "array",
                "items": {"type": "string"}
              },
              "sid": {
                "type": "string",
                "description": "Session ID"
              }
            }
          }
        }
      },
      "refresh_token": {
        "type": "object",
        "properties": {
          "type": {
            "type": "string",
            "enum": ["opaque", "reference", "jwt"],
            "description": "Opaque = stored in DB, JWT = self-contained"
          },
          "expiry_seconds": {
            "type": "integer",
            "minimum": 3600,
            "maximum": 604800,
            "default": 604800,
            "description": "7 days default"
          },
          "single_use": {
            "type": "boolean",
            "default": true,
            "description": "Rotate refresh token on use"
          },
          "family": {
            "type": "string",
            "description": "Token family for rotation tracking"
          }
        }
      }
    }
  }
}
```

**Session Security Headers:**

```
# Required Security Headers
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'none'; frame-ancestors 'none'
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=(), camera=()

# Session Cookies
Set-Cookie: session_id=abc123; 
  Path=/; 
  HttpOnly; 
  Secure; 
  SameSite=Strict; 
  Max-Age=86400
```

---

## 4. Authorization

### 4.1 RBAC Configuration Schema

```json
{
  "RBACConfigurationSchema": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "RBACConfiguration",
    "type": "object",
    "required": ["roles", "permissions"],
    "properties": {
      "roles": {
        "type": "array",
        "items": {
          "type": "object",
          "required": ["name", "permissions"],
          "properties": {
            "name": {
              "type": "string",
              "pattern": "^[a-z][a-z0-9_]{1,62}[a-z0-9]$"
            },
            "description": {"type": "string"},
            "permissions": {
              "type": "array",
              "items": {
                "type": "string"
              }
            },
            "parent_role": {
              "type": "string",
              "description": "Role to inherit from"
            },
            "attributes": {
              "type": "object",
              "description": "ABAC-style attributes"
            }
          }
        }
      },
      "permissions": {
        "type": "array",
        "items": {
          "type": "object",
          "required": ["id", "action", "resource"],
          "properties": {
            "id": {
              "type": "string"
            },
            "action": {
              "oneOf": [
                {"type": "string"},
                {"type": "array", "items": {"type": "string"}}
              ],
              "enum": [
                "create", "read", "update", "delete",
                "list", "search", "export", "admin"
              ]
            },
            "resource": {
              "type": "string",
              "pattern": "^service:resource:qualifier$"
            },
            "conditions": {
              "type": "array",
              "items": {
                "type": "object"
              }
            }
          }
        }
      }
    }
  }
}
```

**RBAC Permission Matrix:**

| Role | users:read | users:create | users:update | users:delete | admin:system |
|------|------------|--------------|--------------|--------------|--------------|
| viewer | ✓ | ✗ | ✗ | ✗ | ✗ |
| editor | ✓ | ✓ | own | ✗ | ✗ |
| manager | ✓ | ✓ | team | ✗ | ✗ |
| admin | ✓ | ✓ | ✓ | ✓ | ✗ |
| superadmin | ✓ | ✓ | ✓ | ✓ | ✓ |

### 4.2 ABAC Policy Schema

```json
{
  "ABACPolicySchema": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "ABACPolicy",
    "type": "object",
    "properties": {
      "policies": {
        "type": "array",
        "items": {
          "type": "object",
          "required": ["effect", "principal", "resource", "actions", "conditions"],
          "properties": {
            "policy_id": {"type": "string"},
            "description": {"type": "string"},
            "effect": {
              "enum": ["permit", "deny"]
            },
            "principal": {
              "type": "object",
              "properties": {
                "match": {
                  "type": "array",
                  "items": {
                    "type": "object",
                    "properties": {
                      "attr": {"type": "string"},
                      "op": {"type": "string"},
                      "value": {}
                    }
                  }
                }
              }
            },
            "resource": {
              "type": "object",
              "properties": {
                "match": {
                  "type": "array",
                  "items": {
                    "type": "object",
                    "properties": {
                      "attr": {"type": "string"},
                      "op": {"type": "string"},
                      "value": {}
                    }
                  }
                }
              }
            },
            "actions": {
              "type": "array",
              "items": {"type": "string"}
            },
            "conditions": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "kind": {
                    "enum": ["time", "ip", "context", "expression"]
                  },
                  "attr": {"type": "string"},
                  "op": {"type": "string"},
                  "value": {}
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

**ABAC Example Policies:**

```yaml
policies:
  - policy_id: "manager_team_access"
    description: "Managers can only access resources in their team"
    effect: permit
    principal:
      match:
        - attr: "role"
          op: eq
          value: "manager"
    resource:
      match:
        - attr: "team"
          op: eq
          value: "${principal.team}"
    actions: ["read", "update", "list"]
    conditions:
      - kind: "time"
        attr: "request_time"
        op: between
        value: ["09:00", "18:00"]
  
  - policy_id: "owner_full_access"
    description: "Resource owners have full access to their resources"
    effect: permit
    principal:
      match: []
    resource:
      match:
        - attr: "owner_id"
          op: eq
          value: "${principal.user_id}"
    actions: ["*"]
  
  - policy_id: "pii_access_requires_mfa"
    description: "Access to PII data requires MFA"
    effect: permit
    principal:
      match:
        - attr: "mfa_enrolled"
          op: eq
          value: true
    resource:
      match:
        - attr: "data_classification"
          op: eq
          value: "restricted"
    actions: ["read"]
    conditions:
      - kind: "context"
        attr: "request_mfa_verified"
        op: eq
        value: true
```

---

## 5. Data Protection

### 5.1 Encryption at Rest Configuration

```yaml
# Encryption at Rest Configuration Schema
EncryptionAtRest:
  default_algorithm: AES-256-GCM
  
  kms_configuration:
    key_type: customer_managed
    rotation_days: 90
    key_admins:
      - role: security-admin
      - role: kms-admin
    key_usage:
      - encrypt
      - decrypt
      - re-encrypt
    
  database_encryption:
    postgresql:
      method: pgcrypto
      algorithm: aes-256
      key_rotation: automatic
    mysql:
      method: innodb_encryption
      algorithm: aes-256
    dynamodb:
      method: kms
      encryption_at_rest: true
    
  storage_encryption:
    ebs:
      enabled: true
      encryption_type: gp3
    s3:
      enabled: true
      sse_algorithm: AES256
      bucket_key_enabled: true
    rds:
      enabled: true
      storage_encrypted: true
      
  application_level:
    sensitive_fields:
      - name: "*.password"
        algorithm: AES-256-GCM
      - name: "*.ssn"
        algorithm: AES-256-GCM
      - name: "*.credit_card"
        algorithm: AES-256-GCM
      - name: "*.api_key"
        algorithm: AES-256-GCM
```

### 5.2 TLS Configuration

```json
{
  "TLSConfiguration": {
    "min_version": "TLS1.2",
    "recommended_version": "TLS1.3",
    "cipher_suites": {
      "TLS1.3": [
        "TLS_AES_256_GCM_SHA384",
        "TLS_AES_128_GCM_SHA256",
        "TLS_CHACHA20_POLY1305_SHA256"
      ],
      "TLS1.2": [
        "ECDHE-ECDSA-AES256-GCM-SHA384",
        "ECDHE-RSA-AES256-GCM-SHA384",
        "ECDHE-ECDSA-CHACHA20-POLY1305",
        "ECDHE-RSA-CHACHA20-POLY1305",
        "ECDHE-ECDSA-AES128-GCM-SHA256",
        "ECDHE-RSA-AES128-GCM-SHA256"
      ],
      "disallowed": [
        "TLS_RSA_WITH_3DES_EDE_CBC_SHA",
        "TLS_RSA_WITH_AES_128_CBC_SHA",
        "TLS_RSA_WITH_AES_256_CBC_SHA",
        "TLS_ECDHE_RSA_WITH_3DES_EDE_CBC_SHA"
      ]
    },
    "certificate_requirements": {
      "min_key_size": 2048,
      "max_key_size": 4096,
      "allowed_key_types": ["RSA", "EC"],
      "ec_curve": "prime256v1",
      "subject_alternative_names": {
        "required": true,
        "include": [
          "DNS name",
          "IP address (if applicable)"
        ]
      }
    },
    "hsts": {
      "enabled": true,
      "max_age_seconds": 31536000,
      "include_subdomains": true,
      "preload": true
    }
  }
}
```

---

## 6. Input Validation

### 6.1 Input Validation Schema

```json
{
  "InputValidationSchema": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "InputValidationRule",
    "type": "object",
    "required": ["field", "type", "rules"],
    "properties": {
      "field": {
        "type": "string",
        "description": "Field path (e.g., 'user.email', 'body.name')"
      },
      "type": {
        "type": "string",
        "enum": [
          "string", "number", "integer", "boolean",
          "array", "object", "email", "uri", "uuid", "date", "datetime"
        ]
      },
      "rules": {
        "type": "object",
        "properties": {
          "required": {"type": "boolean"},
          "min_length": {"type": "integer"},
          "max_length": {"type": "integer"},
          "pattern": {
            "type": "string",
            "description": "Regex pattern (ECMAScript syntax)"
          },
          "minimum": {"type": "number"},
          "maximum": {"type": "number"},
          "enum": {
            "type": "array"
          },
          "min_items": {"type": "integer"},
          "max_items": {"type": "integer"},
          "unique_items": {"type": "boolean"},
          "items": {
            "$ref": "#/definitions/InputValidationRule"
          },
          "properties": {
            "type": "object",
            "additionalProperties": {
              "$ref": "#/definitions/InputValidationRule"
            }
          },
          "custom_validators": {
            "type": "array",
            "items": {
              "type": "string",
              "description": "Function name of custom validator"
            }
          }
        }
      },
      "sanitization": {
        "type": "object",
        "properties": {
          "trim": {"type": "boolean"},
          "lowercase": {"type": "boolean"},
          "uppercase": {"type": "boolean"},
          "remove_html": {"type": "boolean"},
          "remove_scripts": {"type": "boolean"},
          "escape_sql": {"type": "boolean"}
        }
      }
    },
    "definitions": {
      "InputValidationRule": {
        "type": "object",
        "required": ["type"],
        "properties": {
          "type": {"type": "string"},
          "rules": {
            "type": "object"
          },
          "sanitization": {
            "type": "object"
          }
        }
      }
    }
  }
}
```

**Example Input Validation Configuration:**

```yaml
# Input Validation for User Registration Endpoint
endpoint: POST /api/v1/users/register
validations:
  - field: body.email
    type: string
    rules:
      required: true
      max_length: 254
      pattern: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
    sanitization:
      trim: true
      lowercase: true
  
  - field: body.password
    type: string
    rules:
      required: true
      min_length: 12
      max_length: 128
      custom_validators:
        - validate_password_strength
        - validate_not_in_breach_db
  
  - field: body.first_name
    type: string
    rules:
      required: true
      min_length: 1
      max_length: 100
      pattern: "^[a-zA-Z0-9\\s\\-']+$"
    sanitization:
      trim: true
  
  - field: body.last_name
    type: string
    rules:
      required: true
      min_length: 1
      max_length: 100
      pattern: "^[a-zA-Z0-9\\s\\-']+$"
    sanitization:
      trim: true
  
  - field: body.date_of_birth
    type: date
    rules:
      required: true
      min: "1900-01-01"
      max: "2026-01-01"
  
  - field: body.interests
    type: array
    rules:
      required: false
      min_items: 0
      max_items: 10
      unique_items: true
      items:
        type: string
        rules:
          enum: ["technology", "sports", "music", "art", "travel", "food", "reading"]
  
  - field: body.metadata
    type: object
    rules:
      required: false
      max_properties: 10
      properties:
        source:
          type: string
          rules:
            max_length: 50
        referrer:
          type: string
          rules:
            max_length: 500
```

### 6.2 SQL Injection Prevention

```json
{
  "SQLInjectionPrevention": {
    "rules": [
      {
        "rule": "NEVER_concatenate_user_input_into_SQL",
        "severity": "critical",
        "alternatives": [
          "Use parameterized queries (prepared statements)",
          "Use ORM query builders",
          "Use stored procedures with parameters"
        ]
      },
      {
        "rule": "ALWAYS_validate_parameter_types",
        "severity": "high",
        "implementation": "Validate that numeric IDs are integers, strings match patterns"
      },
      {
        "rule": "USE_least_privilege_database_user",
        "severity": "high",
        "implementation": "Application DB user has INSERT/SELECT/UPDATE on specific tables only"
      },
      {
        "rule": "ESCAPE_SQL_characters_if_dynamically_building_SQL",
        "severity": "critical",
        "allowed_methods": ["pqxx::connection::quote", "psycopg2.extensions.AsIs"]
      }
    ],
    "allowed_patterns": {
      "good": [
        "SELECT * FROM users WHERE id = $1",
        "SELECT * FROM users WHERE email = ?",
        "db.table.where(email: user_email).first",
        "INSERT INTO users (name, email) VALUES ($1, $2)"
      ],
      "bad": [
        "SELECT * FROM users WHERE id = #{user_input}",
        "SELECT * FROM users WHERE name = '" + name + "'",
        "SELECT * FROM #{table_name} WHERE id = #{id}"
      ]
    }
  }
}
```

---

## 7. Agent System Defense Layers

When building systems where agents process external data (user input, API responses, file contents, tool output), all data must pass through ordered defense layers. No single layer is sufficient.

### 7.1 The Five-Layer Model

| Layer | Purpose | Failure If Missing | Implementation |
|-------|---------|-------------------|----------------|
| **Validation** | Reject malformed input | Malformed data corrupts processing | Schema validation, type checking, range checks |
| **Sanitization** | Neutralize dangerous content | Injection attacks succeed | Escape sequences, remove dangerous patterns |
| **Policy Enforcement** | Apply configurable rules | Policy violations go undetected | Severity levels, enforcement actions |
| **Output Wrapping** | Prevent content interpretation as code | Untrusted data executes as commands | Boundary markers, encoding |
| **Leak Detection** | Prevent secret exfiltration | Secrets escape to logs/external systems | Prefix scan, regex on candidates |

### 7.2 Defense Layer Specification

```json
{
  "AgentDefenseLayers": {
    "layer_1_validation": {
      "name": "Validation",
      "order": 1,
      "checks": [
        {
          "name": "length_check",
          "implementation": "Reject if size > max_size_bytes",
          "max_size_bytes": 1048576
        },
        {
          "name": "encoding_check",
          "implementation": "Validate UTF-8, reject invalid byte sequences",
          "reject_invalid": true
        },
        {
          "name": "schema_validation",
          "implementation": "JSON Schema or Protobuf validation",
          "strict_mode": true
        },
        {
          "name": "type_check",
          "implementation": "Verify field types match expected schema"
        }
      ],
      "on_failure": "reject"
    },
    "layer_2_sanitization": {
      "name": "Sanitization",
      "order": 2,
      "checks": [
        {
          "name": "html_escape",
          "implementation": "Escape < > & \" ' for HTML context"
        },
        {
          "name": "url_escape",
          "implementation": "Percent-encode special URL characters"
        },
        {
          "name": "sql_escape",
          "implementation": "Escape SQL special characters (defense-in-depth, not primary)"
        },
        {
          "name": "command_escape",
          "implementation": "Escape shell special characters"
        },
        {
          "name": "remove_scripts",
          "implementation": "Strip <script> tags and event handlers"
        }
      ],
      "on_failure": "escape"
    },
    "layer_3_policy_enforcement": {
      "name": "Policy Enforcement",
      "order": 3,
      "severity_levels": [
        "allow",
        "warn",
        "block",
        "quarantine"
      ],
      "policies": [
        {
          "name": "max_external_calls_per_task",
          "limit": 50,
          "severity": "warn"
        },
        {
          "name": "max_file_writes_per_task",
          "limit": 20,
          "severity": "warn"
        },
        {
          "name": "prohibited_file_patterns",
          "patterns": [
            "^/etc/passwd$",
            "^/etc/shadow$",
            "^\\.ssh/",
            "^\\.aws/"
          ],
          "severity": "block"
        }
      ],
      "on_failure": "severity_dependent"
    },
    "layer_4_output_wrapping": {
      "name": "Output Wrapping",
      "order": 4,
      "implementation": "Wrap untrusted content in boundary markers",
      "markers": {
        "start": "<!-- UNTRUSTED_CONTENT_START -->",
        "end": "<!-- UNTRUSTED_CONTENT_END -->"
      },
      "on_failure": "strip"
    },
    "layer_5_leak_detection": {
      "name": "Leak Detection",
      "order": 5,
      "detection_method": "prefix_scan_then_regex",
      "prefixes": [
        {"pattern": "sk-", "description": "OpenAI API key"},
        {"pattern": "AKIA", "description": "AWS Access Key ID"},
        {"pattern": "ghp_", "description": "GitHub Personal Access Token"},
        {"pattern": "xox[baprs]-", "description": "Slack Token"},
        {"pattern": "AIza", "description": "Google API Key"},
        {"pattern": "-----BEGIN", "description": "Private Key"},
        {"pattern": "password=", "description": "Password in URL"}
      ],
      "on_failure": "redact",
      "alert_channel": "security-alerts"
    }
  }
}
```

### 7.3 Registry Protection

**Protected Names Configuration:**

```json
{
  "RegistryProtection": {
    "protected_namespaces": {
      "core": {
        "description": "Core/builtin names that cannot be overridden",
        "names": [
          "validate",
          "execute",
          "query",
          "subscribe",
          "init",
          "shutdown",
          "health",
          "status"
        ],
        "shadow_action": "reject"
      },
      "system": {
        "description": "System-level reserved names",
        "names": [
          "system",
          "config",
          "store",
          "broker"
        ],
        "shadow_action": "reject"
      }
    },
    "shadow_rejection_response": {
      "status_code": 409,
      "error": "NAME_SHADOW_ATTEMPT",
      "message": "Attempted to register name '{name}' which shadows protected builtin",
      "log_level": "warn"
    }
  }
}
```

---

## 8. Container Security

### 8.1 Container Security Configuration

```yaml
# Container Security Policy
ContainerSecurity:
  image_requirements:
    minimal_base:
      enabled: true
      allowed_base_images:
        - "alpine:3.19"
        - "ubuntu:24.04"
        - "distroless/nodejs"
        - "amazonlinux:2023"
    
    vulnerability_scanning:
      enabled: true
      max_severity_allowed: "HIGH"
      block_on_critical: true
      scan_on_push: true
    
    signed_images:
      enabled: true
      require_cosign_signature: true
      key_path: "cosign.pub"
    
    no_root:
      enabled: true
      run_as_non_root: true
      run_as_user: 10000
  
  runtime_security:
    seccomp:
      profile: "RuntimeDefault"
    
    apparmor:
      enabled: true
      profile: "runtime/default"
    
    capabilities:
      drop_all: true
      allow:
        - NET_BIND_SERVICE
        - SYS_CHROOT
    
    readonly_rootfs: true
    
    privileged: false
  
  resource_security:
    read_only_root_filesystem: true
    tmpfs_mounts:
      - /tmp
      - /var/run
    volumes:
      - name: scratch
        empty_dir: {}
  
  network_security:
    network_policy_required: true
    default_deny: true
    allowed_egress:
      - to:
          - namespaceSelector:
              matchLabels:
                name: production
        ports:
          - protocol: TCP
            port: 5432
          - protocol: TCP
            port: 6379
      - to:
          - namespaceSelector: {}
        ports:
          - protocol: TCP
            port: 443
          - protocol: TCP
            port: 80
```

---

## 9. Incident Response

### 9.1 Incident Response Playbook Schema

```json
{
  "IncidentResponseSchema": {
    "type": "object",
    "properties": {
      "incident_types": {
        "type": "object",
        "properties": {
          "data_breach": {
            "severity": "critical",
            "response_team": ["security", "legal", "pr", "executive"],
            "escalation_threshold_minutes": 15,
            "communications": {
              "internal": "immediate",
              "affected_users": "within_72_hours",
              "regulators": "within_72_hours"
            }
          },
          "service_outage": {
            "severity": "high",
            "response_team": ["engineering", "operations"],
            "escalation_threshold_minutes": 30,
            "communications": {
              "internal": "immediate",
              "status_page": "immediate"
            }
          },
          "security_vulnerability": {
            "severity": "high",
            "response_team": ["security", "engineering"],
            "escalation_threshold_minutes": 60
          },
          "malware_detection": {
            "severity": "critical",
            "response_team": ["security", "operations", "legal"],
            "escalation_threshold_minutes": 5
          }
        }
      },
      "response_phases": {
        "containment": {
          "duration_target_minutes": 30,
          "actions": [
            "Isolate affected systems",
            "Preserve evidence",
            "Block attacker access",
            "Activate backup systems"
          ]
        },
        "eradication": {
          "duration_target_minutes": 120,
          "actions": [
            "Remove malware/backdoors",
            "Patch vulnerabilities",
            "Reset compromised credentials",
            "Harden affected systems"
          ]
        },
        "recovery": {
          "duration_target_minutes": 480,
          "actions": [
            "Restore from clean backups",
            "Verify system integrity",
            "Gradual traffic restoration",
            "Monitor for re-infection"
          ]
        },
        "lessons_learned": {
          "duration_target_days": 7,
          "actions": [
            "Complete timeline analysis",
            "Identify root cause",
            "Document improvements",
            "Update playbooks"
          ]
        }
      },
      "forensics": {
        "evidence_collection": {
          "memory_dump": true,
          "disk_images": true,
          "network_logs": true,
          "cloud_logs": true
        },
        "chain_of_custody": {
          "documentation_required": true,
          "hash_verification": true,
          "secure_storage": "s3://forensics-bucket"
        }
      }
    }
  }
}
```

### 9.2 Security Event Detection Matrix

| Event | Severity | Detection | Response Time | Automated Response |
|-------|----------|-----------|---------------|-------------------|
| Failed login > 5 in 1 min | Medium | SIEM | 5 min | Temporarily block IP |
| Failed login > 20 in 5 min | High | SIEM | 1 min | Lock account |
| SQL injection attempt | High | WAF | 0 min | Block request |
| XSS attempt | Medium | WAF | 0 min | Sanitize/block |
| Privilege escalation | Critical | EDR | 1 min | Alert + isolate |
| Data exfiltration | Critical | DLP | 1 min | Block + alert |
| New admin account | High | IAM | 5 min | Investigate |
| Secret in code commit | Critical | Pre-commit hook | 0 min | Block commit |
| Root access attempt | Critical | Audit log | 1 min | Alert + investigate |

---

## 10. Security Testing

### 10.1 Security Testing Schema

```json
{
  "SecurityTestSchema": {
    "sast": {
      "enabled": true,
      "tools": [
        {"name": "semgrep", "rules": "security"},
        {"name": "codeql", "languages": ["python", "typescript", "go", "rust"]}
      ],
      "block_on": ["critical", "high"],
      "exclude_patterns": [
        "**/test/**",
        "**/*_test.go",
        "**/migrations/**"
      ]
    },
    "dast": {
      "enabled": true,
      "tools": [
        {"name": "zap", "authenticated": true},
        {"name": "nuclei", "templates": "security"}
      ],
      "frequency": "daily",
      "target": "https://staging.example.com"
    },
    "dependency_scanning": {
      "enabled": true,
      "tools": [
        {"name": "snyk"},
        {"name": "trivy"},
        {"name": "osv"}
      ],
      "block_on_severity": "high",
      "check_license": true,
      "prohibited_licenses": [
        "GPL-3.0",
        "AGPL-3.0",
        "SSPL-1.0"
      ]
    },
    "secret_scanning": {
      "enabled": true,
      "tools": [
        {"name": "gitguardian"},
        {"name": "trufflehog"}
      ],
      "block_on": ["high", "critical"],
      "include_patterns": ["*.tf", "*.yaml", "*.json", "*.go", "*.py"]
    },
    "infrastructure_scanning": {
      "enabled": true,
      "tools": [
        {"name": "checkov"},
        {"name": "terrascan"}
      ],
      "block_on": ["high", "critical"]
    }
  }
}
```

---

## 11. Anti-Patterns

| Anti-Pattern | Specific Failure Mode | Prevention |
|--------------|----------------------|------------|
| **Security through obscurity** | Kerckhoffs's principle: security relies on attacker not knowing algorithm | Assume attacker knows everything except key |
| **Hardcoded credentials** | Code in git history, rotated keys still compromised | Secrets manager, environment variables, rotation |
| **No input validation** | SQL injection, XSS, command injection | Whitelist validation, parameterized queries |
| **Verbose error messages** | Stack traces reveal internal structure | Generic error messages in production |
| **No rate limiting** | Brute force, DoS, resource exhaustion | Per-IP, per-user, per-endpoint limits |
| **Weak cryptography** | MD5 collision, SHA1 chosen prefix | Use NIST-approved algorithms only |
| **No logging** | Breach goes undetected for months | Centralized, tamper-proof audit logs |
| **Overly permissive CORS** | Any origin can access API | Whitelist specific origins |
| **No HTTPS** | Man-in-the-middle, credential theft | TLS everywhere, HSTS |
| **Ignoring security updates** | Known CVEs exploited | Automated patch management |
| **Rolling your own crypto** | Side-channel attacks, implementation bugs | Use battle-tested libraries |
| **Trusting user input** | Every input is potentially malicious | Never trust, always validate/sanitize |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/WEB.md` - Web security patterns
- `architecture/CLOUD.md` - Cloud security
- `architecture/OBSERVABILITY.md` - Security monitoring
- `architecture/CONCURRENCY.md` - Distributed security patterns

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition
- `specs/SECURITY.md` - Security doctrine (binding)

### Interface Contracts
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/CONTROL_PLANE.md` - Agent sequencing
- `interfaces/STORE_MODEL.md` - Store semantics

### Methodology
- `methodology/ARCHITECTURE.md` - Architecture methodology
- `methodology/THREAT_MODELING.md` - Threat modeling process