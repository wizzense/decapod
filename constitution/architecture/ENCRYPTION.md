# ENCRYPTION.md - Encryption Architecture and Implementation

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

## Table of Contents

1. [TLS/SSL Configurations](#1-tlsssl-configurations)
2. [Key Management](#2-key-management)
3. [Encryption at Rest](#3-encryption-at-rest)
4. [Field-Level Encryption](#4-field-level-encryption)
5. [Complete Configuration Examples](#5-complete-configuration-examples)
6. [Decision Matrices](#6-decision-matrices)
7. [Anti-Patterns and Failure Modes](#7-anti-patterns-and-failure-modes)
8. [Production Checklist](#8-production-checklist)
9. [References](#9-references)

---

## 1. TLS/SSL Configurations

### 1.1 TLS Fundamentals

TLS (Transport Layer Security) provides encrypted communication between clients and servers. The current best practice is TLS 1.2 with strong cipher suites, or TLS 1.3 for maximum security.

**TLS 1.2 Handshake (Simplified)**
1. Client sends ClientHello with supported cipher suites
2. Server responds with ServerHello, certificate, and key exchange
3. Client verifies certificate against trusted CAs
4. Client generates session key using server's public key
5. Server decrypts using its private key
6. Both parties have shared session key for symmetric encryption

**TLS 1.3 Improvements**
- Reduced handshake from 2 RTT to 1 RTT (or 0-RTT with PSK)
- Removed weak cipher suites
- Removed RSA key exchange (forward secrecy always)
- Mandatory perfect forward secrecy

### 1.2 Certificate Authority Infrastructure

```yaml
# Certificate management infrastructure
certificate_authority:
  # Internal CA for development/testing
  internal_ca:
    name: Example Internal CA
    type: root
    key_size: 4096
    algorithm: RSA
    validity:
      start: "2024-01-01T00:00:00Z"
      end: "2034-01-01T00:00:00Z"
    paths:
      private_key: /etc/ca/private/root-ca.key
      certificate: /etc/ca/certs/root-ca.crt
      chain: /etc/ca/certs/root-ca-chain.crt
      
  # Intermediate CA for services
  intermediate_ca:
    name: Example Services Intermediate CA
    type: intermediate
    key_size: 4096
    algorithm: RSA
    validity:
      start: "2024-01-01T00:00:00Z"
      end: "2027-01-01T00:00:00Z"
    paths:
      private_key: /etc/ca/private/intermediate-ca.key
      certificate: /etc/ca/certs/intermediate-ca.crt
      signed_by: root_ca
      
  # Certificate profiles
  profiles:
    server_auth:
      key_usage:
        - digital_signature
        - key_encipherment
      extended_key_usage:
        - server_auth
      basic_constraints:
        is_ca: false
        path_length: null
        
    client_auth:
      key_usage:
        - digital_signature
      extended_key_usage:
        - client_auth
        
    code_signing:
      key_usage:
        - digital_signature
      extended_key_usage:
        - code_signing
```

### 1.3 TLS Server Configuration

```yaml
# TLS server configuration patterns
tls_configurations:
  # Modern TLS 1.3 only (recommended for internal services)
  modern:
    min_version: "TLSv1.3"
    max_version: "TLSv1.3"
    cipher_suites:
      - TLS_AES_256_GCM_SHA384
      - TLS_AES_128_GCM_SHA256
      - TLS_CHACHA20_POLY1305_SHA256
    curves:
      - X25519
      - secp384r1
      - secp256r1
    session_tickets: true
    ocsp_stapling: true
    prefer_server_cipher_order: true
    
  # Compatible TLS 1.2+ (recommended for external services)
  compatible:
    min_version: "TLSv1.2"
    max_version: "TLSv1.3"
    cipher_suites:
      - TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
      - TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
      - TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
      - TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
      - TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
      - TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
    curves:
      - X25519
      - secp384r1
      - secp256r1
    session_tickets: true
    ocsp_stapling: true
    prefer_server_cipher_order: true
    certificate_compression: true
    
  # Legacy TLS 1.2 with legacy cipher support (avoid if possible)
  legacy:
    min_version: "TLSv1.2"
    max_version: "TLSv1.2"
    cipher_suites:
      - TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
      - TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
      - TLS_RSA_WITH_AES_256_GCM_SHA384
      - TLS_RSA_WITH_AES_128_GCM_SHA256
    curves:
      - secp384r1
      - secp256r1
    session_tickets: true
    ocsp_stapling: true
```

### 1.4 Nginx TLS Configuration

```yaml
# Nginx TLS configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: nginx-tls-config
  namespace: platform
data:
  ssl.conf: |
    # SSL session settings
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;
    ssl_session_tickets on;
    ssl_session_ticket_key /etc/nginx/tls/ticket.key;
    
    # TLS configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers off;
    
    # ECDHE curves
    ssl_ecdh_curve X25519:secp384r1:secp256r1;
    
    # Modern cipher suite - TLS 1.3
    ssl_ciphers TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:TLS_AES_128_GCM_SHA256;
    
    # OCSP stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    resolver 10.96.0.10 8.8.8.8 valid=300s;
    resolver_timeout 5s;
    
    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
---
apiVersion: v1
kind: Secret
metadata:
  name: nginx-tls-ticket-key
  namespace: platform
type: Opaque
data:
  ticket.key: <base64-encoded-48-byte-random-key>
```

### 1.5 gRPC TLS Configuration

```yaml
# gRPC/TLS server configuration
grpc_tls:
  # Server options
  server:
    port: 50051
    tls:
      enabled: true
      certificate: /etc/grpc/tls/server.crt
      private_key: /etc/grpc/tls/server.key
      client_ca: /etc/grpc/tls/client-ca.crt  # For mTLS
      
      # TLS configuration
      config:
        min_version: TLSv1.2
        max_version: TLSv1.3
        cipher_suites:
          - TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
          - TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
          - TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
          - TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
          
    # Keepalive and timeouts
    keepalive:
      max_connection_idle: 5m
      max_connection_age: 30m
      max_connection_age_grace: 1m
      time: 1h
      timeout: 20s
      
  # Client options
  client:
    tls:
      enabled: true
      ca_certificate: /etc/grpc/tls/ca.crt
      server_name_override: grpc.example.com
      
      # Insecure fallback (development only!)
      insecure: false
      
    # Connection pool
    pool:
      max_connections: 100
      max_connections_per_host: 10
      max_idle_time: 5m
      max_idle_time_without_calls: 1m
```

---

## 2. Key Management

### 2.1 Key Hierarchy

```
Root CA (Offline, HSM)
  └── Intermediate CA (Online, HSM)
        └── Issuance CA (Service-specific)
              ├── TLS Server Certificates
              ├── TLS Client Certificates
              └── Code Signing Certificates
```

### 2.2 Key Management System Configuration

```yaml
# Key management system configuration
key_management:
  # HSM (Hardware Security Module) configuration
  hsm:
    type: cloudhsm  # Options: cloudhsm, pkcs11, aws_kms
    provider: aws
    region: us-east-1
    cluster_id: cluster-1234
    
    # Key generation and storage
    key_generation:
      algorithm: RSA
      key_size: 4096
      protection: HSM
      
    # Access control
    access:
      users:
        - name: ca-operator
          permissions: [sign, decrypt]
        - name: service-account
          permissions: [encrypt, verify]
          
  # Key rotation
  rotation:
    enabled: true
    schedules:
      root_ca: 87600h  # 10 years
      intermediate_ca: 8760h  # 1 year
      issuance_ca: 2160h  # 90 days
      tls_certificates: 720h  # 30 days
      session_keys: 24h  # 1 day
      
  # Key lifecycle
  lifecycle:
    key_states:
      pre_activation:  # Key generated but not used
        transition_to: active
        requires: manual_approval
      active:  # Key in use
        transition_to: deactivated, compromised
      deactivated:  # Key no longer used for signing
        transition_to: destroyed
        grace_period: 90d
      compromised:  # Key suspected to be leaked
        immediate_actions:
          - revoke_key
          - alert_security_team
          - initiate_incident_response
        transition_to: destroyed
      destroyed:  # Key permanently deleted
        audit_log: permanent
```

### 2.3 Certificate Lifecycle Management

```yaml
# Certificate lifecycle management
certificate_lifecycle:
  # Certificate issuance
  issuance:
    auto_enroll: true
    enrollment_method: ACME  # Options: ACME, SCEP, EST
    renewal_trigger: automatic
    renewal_window: 30d  # Renew 30 days before expiry
    
  # Certificate types and validity
  certificates:
    tls_server:
      validity: 90d
      renewal_window: 30d
      key_size: 2048  # RSA or 256-bit ECDSA
      algorithm: ECDSA
      curve: P-256
      subject_alternate_names:
        - DNS: service.example.com
        - DNS: "*.service.example.com"
        - IP: 10.0.0.1
        
    tls_client:
      validity: 365d
      renewal_window: 30d
      key_size: 2048
      include_email: true
      
    code_signing:
      validity: 730d  # 2 years
      key_size: 4096
      timestamp_required: true
      timestamp_server: http://timestamp.digicert.com
      
    smime:
      validity: 365d
      key_size: 2048
      include_email: true
      
  # Revocation
  revocation:
    methods:
      - CRL  # Certificate Revocation List
      - OCSP  # Online Certificate Status Protocol
    crl:
      url: http://crl.example.com/ca.crl
      update_interval: 24h
      overlap: 12h
    ocsp:
      url: http://ocsp.example.com
      nonce_enabled: true
      response_validity: 4d
      
  # Monitoring
  monitoring:
    expiration_check: daily
    warning_thresholds:
      critical: 7d
      warning: 30d
      info: 60d
    notifications:
      channels:
        - email: security@example.com
        - slack: "#cert-alerts"
        - pagerduty: true
```

### 2.4 Vault PKI Configuration

```hcl
# Vault PKI secrets engine configuration
# Configure via Vault CLI:

# Enable the PKI secrets engine
# vault secrets enable -path=pki pki

# Configure CA certificate and private key
# vault write pki/root/generate/internal \
#     common_name="Example Root CA" \
#     ttl=87600h

# Configure intermediate CA
# vault secrets enable -path=pki_int pki
# vault write pki_int/intermediate/generate/internal \
#     common_name="Example Services Intermediate CA" \
#     ttl=8760h

# Create role for service certificates
# vault write pki_int/roles/order-service \
#     allowed_domains="platform.svc.cluster.local" \
#     allow_subdomains=true \
#     allow_any_name=false \
#     allow_bare_domains=false \
#     ttl=720h \
#     max_ttl=2160h

# Configure CRL
# vault write pki_int/config/crl \
#     expiry="24h" \
#     ocsp_disable=false
```

---

## 3. Encryption at Rest

### 3.1 Database Encryption

```yaml
# PostgreSQL encryption configuration
database_encryption:
  postgresql:
    # Encryption at rest (handled by storage layer or PostgreSQL)
    encryption_at_rest:
      enabled: true
      provider: aws_kms  # Options: pg_encryption, aws_kms, azure_key_vault
      
    # Column-level encryption for sensitive fields
    column_encryption:
      enabled: true
      algorithm: AES-256-GCM
      key_management: vault_transit
      
      # Encrypted columns
      columns:
        - name: credit_card_number
          key_id: pii-encryption-key
          searchable: false
        - name: ssn
          key_id: pii-encryption-key
          searchable: false
        - name: password_hash
          key_id: password-encryption-key
          searchable: false
          
    # Transparent Data Encryption (TDE)
    transparent_encryption:
      enabled: true
      algorithm: AES-256
      key_rotation:
        enabled: true
        interval: 90d

# MySQL encryption configuration
mysql_encryption:
  # InnoDB tablespace encryption
  tablespace_encryption:
    enabled: true
    encryption_algorithm: AES-256
    keyring:
      type: vault
      vault_url: https://vault.platform.svc.cluster.local:8200
      kv_path: secret/data/mysql
      key_name: tablespace-master-key
      
  # Redo log encryption
  redo_log_encryption: true
  
  # Binlog encryption
  binlog_encryption: true
  
  # Doublewrite buffer encryption
  doublewrite_encryption: true
  
# MongoDB encryption
mongodb_encryption:
  # Encryption at rest (FLE - Field Level Encryption)
  fle:
    enabled: true
    encryption_key:
      provider: vault
      vault_url: https://vault.platform.svc.cluster.local:8200
      path: secret/data/mongodb
      key_name: master-key
      
    # Encrypted fields
    encrypted_fields:
      - path: customerData.creditCard
        algorithm: AEAD_AES_256_CBC_HMAC_SHA_512
      - path: customerData.ssn
        algorithm: AEAD_AES_256_CBC_HMAC_SHA_512
```

### 3.2 Storage Encryption

```yaml
# Kubernetes PersistentVolume encryption
storage_encryption:
  # AWS EBS encryption
  aws_ebs:
    enabled: true
    kms_key_id: arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012
    kms_key_arn: arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012
    volume_type: gp3
    encrypted: true
    
  # Azure Disk encryption
  azure_disk:
    enabled: true
    encryption_set_id: /subscriptions/.../diskEncryptionSets/my-des
    type: EncryptionAtRestWithPlatformKey
    
  # GCP Persistent Disk encryption
  gcp_pd:
    enabled: true
    kms_key_name: projects/my-project/locations/us-east1/keyRings/my-ring/cryptoKeys/my-key
    
  # S3 encryption
  s3:
    enabled: true
    encryption_type: SSE-KMS  # Options: SSE-S3, SSE-KMS, SSE-C
    kms_key_id: alias/s3-master-key
    bucket_key_enabled: true
    
  # NFS/CIFS encryption
  nfs:
    enabled: true
    protocol: nfsv4
    security:
      - mode: krb5i  # Options: none, sys, krb5, krb5i, krb5p
      - privacy: true
        
# Kubernetes StorageClass with encryption
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: encrypted-gp3
provisioner: ebs.csi.aws.com
parameters:
  type: gp3
  encrypted: "true"
  kmsKeyId: arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012
  csi.storage.k8s.io/fstype: ext4
volumeBindingMode: WaitForFirstConsumer
allowVolumeExpansion: true
reclaimPolicy: Retain
```

### 3.3 Application-Level Encryption

```python
# Application-level encryption using envelope encryption
import base64
import os
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.hkdf import HKDF

class EnvelopeEncryptor:
    """
    Implements envelope encryption pattern.
    Data Encryption Key (DEK) encrypts data.
    Key Encryption Key (KEK) encrypts DEK.
    """
    
    def __init__(self, kms_client, kek_arn):
        self.kms_client = kms_client
        self.kek_arn = kek_arn
        self.data_key_size = 32  # 256 bits
        
    def generate_data_key(self):
        """Generate a new data encryption key"""
        response = self.kms_client.generate_data_key(
            KeyId=self.kek_arn,
            KeySpec='AES_256',
            EncryptionContext={'application': 'order-service'}
        )
        return {
            'ciphertext': response['CiphertextBlob'],
            'plaintext': base64.b64encode(response['Plaintext']).decode(),
            'key_id': response['KeyId']
        }
        
    def encrypt(self, plaintext, data_key_plaintext):
        """Encrypt data using envelope encryption"""
        # Generate random IV
        iv = os.urandom(12)  # 96 bits for GCM
        
        # Derive key from data key
        derived_key = HKDF(
            algorithm=hashes.SHA256(),
            length=32,
            salt=iv,
            info=b'handshake data encryption',
        ).derive(data_key_plaintext.encode())
        
        # Encrypt with AES-GCM
        aesgcm = AESGCM(derived_key)
        ciphertext = aesgcm.encrypt(iv, plaintext.encode(), None)
        
        return {
            'iv': base64.b64encode(iv).decode(),
            'ciphertext': base64.b64encode(ciphertext).decode(),
            'version': 1
        }
        
    def decrypt(self, encrypted_data, data_key_plaintext, ciphertext_key):
        """Decrypt data using envelope encryption"""
        iv = base64.b64decode(encrypted_data['iv'])
        ciphertext = base64.b64decode(encrypted_data['ciphertext'])
        
        # Derive key from data key
        derived_key = HKDF(
            algorithm=hashes.SHA256(),
            length=32,
            salt=iv,
            info=b'handshake data encryption',
        ).derive(data_key_plaintext.encode())
        
        # Decrypt
        aesgcm = AESGCM(derived_key)
        plaintext = aesgcm.decrypt(iv, ciphertext, None)
        
        return plaintext.decode()
```

---

## 4. Field-Level Encryption

### 4.1 Field-Level Encryption Architecture

Field-level encryption protects sensitive data at the field level, ensuring that only authorized components can decrypt specific fields while the rest of the data remains accessible.

**Use Cases:**
- Credit card numbers
- Social Security Numbers (SSN)
- Personal Health Information (PHI)
- API keys and secrets
- Any PII (Personally Identifiable Information)

### 4.2 Implementation Patterns

```yaml
# Field-level encryption configuration
field_encryption:
  # Supported algorithms
  algorithms:
    - name: AES-256-GCM
      key_size: 256
      iv_size: 96
      tag_size: 128
      type: symmetric
      
    - name: AES-256-CBC
      key_size: 256
      iv_size: 128
      type: symmetric
      
  # Key management
  key_management:
    provider: vault  # Options: vault, aws_kms, azure_key_vault, gcp_kms
    transit_engine_path: transit
    encryption_key_name: field-encryption-key
    key_rotation:
      enabled: true
      period: 90d
      
  # Encrypted field definitions
  fields:
    credit_card:
      algorithm: AES-256-GCM
      searchable: false  # Cannot search encrypted CC numbers
      mask_in_logs: true
      mask_in_responses: true
      format: tokenized  # Token format for references
      
    ssn:
      algorithm: AES-256-GCM
      searchable: false
      mask_in_logs: true
      mask_in_responses: true
      format: last_four  # Only show last 4 digits
      
    email:
      algorithm: AES-256-GCM
      searchable: true  # Can use deterministic encryption for email lookup
      searchable_algorithm: AES-SIV
      mask_in_logs: true
      
    phone:
      algorithm: AES-256-GCM
      searchable: false
      mask_in_logs: true
      
    password_hash:
      algorithm: bcrypt  # Special handling for password hashes
      salt_size: 128
      rounds: 12
```

### 4.3 Code Implementation

```python
from cryptography.hazmat.primitives.ciphers.aead import AESGCM, AESCCM
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
from cryptography.hazmat.backends import default_backend
from dataclasses import dataclass
from typing import Optional
import base64
import json

@dataclass
class EncryptedField:
    """Represents an encrypted field"""
    ciphertext: str  # Base64-encoded ciphertext
    iv: str          # Base64-encoded initialization vector
    tag: Optional[str]  # Base64-encoded authentication tag (for GCM)
    version: int     # Encryption version for key rotation
    key_id: str      # Identifier of the key used

class FieldEncryptor:
    """Handles field-level encryption/decryption"""
    
    def __init__(self, key_provider):
        self.key_provider = key_provider
        
    def encrypt(
        self,
        plaintext: str,
        field_name: str,
        deterministic: bool = False
    ) -> EncryptedField:
        """
        Encrypt a field value.
        
        Args:
            plaintext: The value to encrypt
            field_name: Name of the field (used for context)
            deterministic: If True, use deterministic encryption (for searchable fields)
        
        Returns:
            EncryptedField containing all encryption metadata
        """
        # Get current encryption key
        key = self.key_provider.get_current_key(field_name)
        
        # Generate IV
        if deterministic:
            # Use field name as additional authenticated data for deterministic mode
            iv = self._derive_iv(key, field_name)
        else:
            iv = os.urandom(12)  # 96-bit IV for GCM
            
        # Encrypt
        aesgcm = AESGCM(key)
        ciphertext = aesgcm.encrypt(
            iv,
            plaintext.encode('utf-8'),
            field_name.encode('utf-8')  # AAD includes field name
        )
        
        return EncryptedField(
            ciphertext=base64.b64encode(ciphertext).decode(),
            iv=base64.b64encode(iv).decode(),
            tag=None,  # Tag is included in ciphertext for GCM
            version=key['version'],
            key_id=key['key_id']
        )
        
    def decrypt(self, encrypted_field: EncryptedField, field_name: str) -> str:
        """Decrypt an encrypted field"""
        # Get the key version used for encryption
        key = self.key_provider.get_key(encrypted_field.key_id)
        
        # Decode ciphertext and IV
        ciphertext = base64.b64decode(encrypted_field.ciphertext)
        iv = base64.b64decode(encrypted_field.iv)
        
        # Decrypt
        aesgcm = AESGCM(key)
        plaintext = aesgcm.decrypt(
            iv,
            ciphertext,
            field_name.encode('utf-8')  # Verify AAD
        )
        
        return plaintext.decode('utf-8')
        
    def encrypt_searchable(self, plaintext: str, field_name: str) -> EncryptedField:
        """
        Encrypt with deterministic output for searching.
        Uses AES-SIV for deterministic authenticated encryption.
        """
        key = self.key_provider.get_current_key(field_name, for_search=True)
        
        # Use field name as nonce derivation
        iv = self._derive_iv_for_search(key, field_name)
        
        aesgcm = AESGCM(key)
        ciphertext = aesgcm.encrypt(
            iv,
            plaintext.encode('utf-8'),
            field_name.encode('utf-8')
        )
        
        return EncryptedField(
            ciphertext=base64.b64encode(ciphertext).decode(),
            iv=base64.b64encode(iv).decode(),
            tag=None,
            version=key['version'],
            key_id=key['key_id']
        )
        
    def _derive_iv(self, key: bytes, context: str) -> bytes:
        """Derive deterministic IV from context"""
        hkdf = HKDF(
            algorithm=hashes.SHA256(),
            length=12,
            salt=context.encode(),
            info=b'deterministic-iv-derivation'
        )
        return hkdf.derive(key)
        
    def _derive_iv_for_search(self, key: bytes, context: str) -> bytes:
        """Derive IV for searchable encryption"""
        return self._derive_iv(key, context)
```

### 4.4 Database Field Encryption

```sql
-- PostgreSQL example with pgcrypto extension
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Create table with encrypted fields
CREATE TABLE customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL,
    
    -- Encrypted PII fields
    encrypted_ssn BYTEA NOT NULL,
    encrypted_credit_card BYTEA,
    encrypted_password_hash BYTEA,
    
    -- Searchable encrypted fields (deterministic)
    encrypted_email_search BYTEA,
    
    -- Key version tracking
    ssn_key_version INT DEFAULT 1,
    cc_key_version INT DEFAULT 1,
    
    -- Encrypted field metadata (IV, etc.) stored separately
    ssn_iv BYTEA NOT NULL,
    cc_iv BYTEA,
    email_search_iv BYTEA NOT NULL,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    CONSTRAINT email_unique UNIQUE (email)
);

-- Function to encrypt SSN on insert/update
CREATE OR REPLACE FUNCTION encrypt_ssn()
RETURNS TRIGGER AS $$
DECLARE
    key_bytes BYTEA;
    key_version INT;
BEGIN
    -- Get the current encryption key (from application key management)
    -- This would typically call an external key management system
    key_bytes := get_current_encryption_key('ssn');
    key_version := get_current_key_version('ssn');
    
    -- Encrypt SSN
    NEW.ssn_iv := gen_random_bytes(12);
    NEW.encrypted_ssn := pgp_sym_encrypt(
        NEW.encrypted_ssn::TEXT,  -- Would be passed as parameter
        encode(key_bytes, 'hex'),
        'aes-256-gcm, iv=' || encode(NEW.ssn_iv, 'hex')
    )::BYTEA;
    
    NEW.ssn_key_version := key_version;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Partial index for searching encrypted email
CREATE INDEX idx_customers_email_search 
ON customers (encrypted_email_search) 
WHERE encrypted_email_search IS NOT NULL;
```

---

## 5. Complete Configuration Examples

### 5.1 TLS Certificate Request Configuration

```yaml
# Certificate signing request configuration
certificate_request:
  # TLS server certificate CSR
  tls_server:
    subject:
      country: US
      state: California
      locality: San Francisco
      organization: Example Inc
      organizational_unit: Platform Engineering
      common_name: orders.example.com
      email_address: platform@example.com
      
    subject_alternate_names:
      dns:
        - orders.example.com
        - "*.orders.example.com"
        - orders-staging.example.com
      ip:
        - 10.0.0.1
        - 192.168.1.1
      email:
        - admin@orders.example.com
        
    key:
      algorithm: ECDSA
      curve: P-256
      reuse: false  # Generate new key per certificate
      
    extensions:
      key_usage:
        digital_signature: true
        key_encipherment: true
      extended_key_usage:
        server_auth: true
      basic_constraints:
        is_ca: false
        path_length: null
        
    signing:
      hash_algorithm: SHA256
      profile: server_auth
      
  # mTLS client certificate CSR
  tls_client:
    subject:
      country: US
      organization: Example Inc
      organizational_unit: Platform Engineering
      common_name: order-service
    subject_alternate_names:
      dns:
        - order-service.platform.svc.cluster.local
        - order-service
    key:
      algorithm: ECDSA
      curve: P-256
    extensions:
      key_usage:
        digital_signature: true
      extended_key_usage:
        client_auth: true
```

### 5.2 Kubernetes TLS Secret

```yaml
# Kubernetes TLS Secret (for Ingress, etc.)
apiVersion: v1
kind: Secret
metadata:
  name: orders-tls-secret
  namespace: platform
type: kubernetes.io/tls
data:
  # Base64-encoded PEM-encoded certificate
  tls.crt: |
    LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSURYVENDQWtXZ0F3SUJBZ0lVR....==
    
  # Base64-encoded PEM-encoded private key
  tls.key: |
    LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JSUV2UUlCQURBTkJna3Foa2lH....==
    
  # Optional: CA certificate chain
  ca.crt: |
    LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSURCVENDQWUyZ0F3SUJBZ0lV....==
    
# TLS Secret annotations for cert-manager
metadata:
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    cert-manager.io/issue-temporary-certificate: "false"
    cert-manager.io/private-key-algorithm: ECDSA
    cert-manager.io/private-key-size: "256"
```

### 5.3 Service Mesh mTLS Configuration

```yaml
# Istio PeerAuthentication for STRICT mTLS
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default-strict-mtls
  namespace: platform
spec:
  mtls:
    mode: STRICT
    
---
# Istio DestinationRule for TLS settings
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: order-service-tls
  namespace: platform
spec:
  host: order-service.platform.svc.cluster.local
  trafficPolicy:
    tls:
      mode: ISTIO_MUTUAL
      # Client certificate from SDS (Secret Discovery Service)
      clientCertificate: ""  # Uses SDS to fetch cert from Istiod
      privateKey: ""
      caCertificates: ""
      # Require a valid certificate
      verifySubjectAltName:
        - order-service.platform.svc.cluster.local
      # Subject names for SNI
      subjectAltNames:
        - order-service.platform.svc.cluster.local
---
# Istio AuthorizationPolicy
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: order-service-authz
  namespace: platform
spec:
  selector:
    matchLabels:
      app: order-service
  action: ALLOW
  rules:
    - from:
        - source:
            principals:
              - cluster.local/ns/platform/sa/order-service
              - cluster.local/ns/platform/sa/payment-service
      to:
        - operation:
            methods: ["GET", "POST"]
            paths: ["/api/v1/*"]
```

---

## 6. Decision Matrices

### 6.1 TLS Version Selection

| Requirement | TLS 1.3 | TLS 1.2 | TLS 1.1 | TLS 1.0 |
|-------------|----------|----------|---------|---------|
| Security | Excellent | Good | Weak | Insecure |
| Performance | Excellent | Good | Poor | Poor |
| Compatibility | Modern systems | Broad | Legacy | Legacy only |
| Forward secrecy | Mandatory | Recommended | Limited | None |
| 0-RTT support | Yes | No | No | No |
| Recommended | Yes | Fallback | No | Never |

### 6.2 Cipher Suite Selection

| Requirement | Recommended Ciphers | Avoid |
|-------------|-------------------|-------|
| TLS 1.3 only | AES-256-GCM, ChaCha20-Poly1305 | All others |
| TLS 1.2+ | ECDHE-RSA-AES-256-GCM-SHA384 | RC4, 3DES |
| Forward secrecy | ECDHE, DHE | RSA key exchange |
| Performance (mobile) | ChaCha20-Poly1305 | AES-256-GCM |
| Compliance | FIPS-compliant suites | Export ciphers |

### 6.3 Encryption at Rest Options

| Storage Type | Encryption Method | Key Management | Performance Impact |
|--------------|-------------------|----------------|-------------------|
| AWS EBS | KMS + XTS-AES-256 | AWS KMS | ~3-5% |
| Azure Disk | SSE with Azure Key Vault | Azure Key Vault | ~3-5% |
| GCP PD | Google-managed or CMEK | Cloud KMS | ~3-5% |
| Database (PostgreSQL) | pgcrypto or TDE | External KMS | Varies (5-30%) |
| S3 | SSE-S3, SSE-KMS, SSE-C | Various | Minimal |
| NFS | Kerberos + in-transit | Active Directory | ~10-15% |
| Memory | Application-level | Application | N/A |

---

## 7. Anti-Patterns and Failure Modes

### 7.1 Common Anti-Patterns

**Weak Cipher Suites**
```nginx
# BAD: Allowing weak ciphers
ssl_protocols TLSv1 TLSv1.1 TLSv1.2;
ssl_ciphers ALL:!aNULL:!MD5;
# This allows NULL ciphers, MD5, and weak RC4!
```

**Certificate Validation Disabled**
```python
# BAD: Disabling certificate verification
requests.get(url, verify=False)  # NEVER DO THIS
```

**Hardcoded Keys**
```python
# BAD: Hardcoded encryption key
ENCRYPTION_KEY = "super-secret-key-in-source-code"  # NEVER
```

**Insecure Randomness**
```python
# BAD: Using predictable randomness for keys
import random
key = bytes(random.getrandbits(8) for _ in range(32))  # NOT SECURE
```

### 7.2 Failure Modes

**Certificate Expiration**
```
Error: "SSL_ERROR_RX_RECORD_TOO_LONG"
Cause: Server certificate expired
Prevention:
- Monitor certificate expiration (30, 14, 7, 1 day warnings)
- Enable automatic renewal via cert-manager or similar
- Set calendar reminders for manual certificates
```

**Invalid Certificate Chain**
```
Error: "ERR_CERT_AUTHORITY_INVALID"
Cause: Intermediate CA not installed on client
Prevention:
- Always include full certificate chain in server cert
- Test certificate chain with SSL Labs
- Use certificate bundles properly
```

**Weak Key Generation**
```
Error: "Common Name length exceeds limit"
Cause: Key size too small (< 2048 for RSA)
Prevention:
- Use RSA 2048+ or ECDSA P-256 minimum
- Reject keys below 2048 bits
- Test with OpenSSL: openssl x509 -in cert.pem -text -noout
```

---

## 8. Production Checklist

### 8.1 TLS/SSL Checklist

- [ ] TLS 1.2 or 1.3 only enabled
- [ ] Weak cipher suites disabled
- [ ] Strong cipher suites configured
- [ ] Certificate chain properly configured
- [ ] OCSP stapling enabled
- [ ] HSTS header configured with preload
- [ ] Certificate expiration monitoring in place
- [ ] Automatic certificate renewal configured
- [ ] Certificate transparency logging enabled
- [ ] Regular SSL Labs testing performed

### 8.2 Key Management Checklist

- [ ] Keys stored securely (HSM or KMS)
- [ ] Key rotation schedule defined and automated
- [ ] Key access audited and monitored
- [ ] Key backup procedures documented
- [ ] Key recovery procedures tested
- [ ] Certificate revocation procedures in place
- [ ] CRL and OCSP endpoints configured
- [ ] Emergency key rotation capability exists

### 8.3 Encryption at Rest Checklist

- [ ] All persistent volumes encrypted
- [ ] Database encryption configured
- [ ] Field-level encryption for PII/PHI
- [ ] Encryption keys rotated regularly
- [ ] Key management integrated with Vault or cloud KMS
- [ ] Encryption status monitoring in place
- [ ] Data classification performed
- [ ] Decryption access controlled and audited

---

## 9. References

### TLS/SSL

- [Mozilla SSL Configuration Generator](https://ssl-config.mozilla.org/)
- [SSL Labs Best Practices](https://github.com/ssllabs/research/wiki/SSL-and-TLS-Deployment-Best-Practices)
- [RFC 7525 - TLS Recommendations](https://datatracker.ietf.org/doc/html/rfc7525)
- [TLS 1.3 RFC 8446](https://datatracker.ietf.org/doc/html/rfc8446)

### Key Management

- [NIST Key Management Guidelines](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)
- [AWS KMS Documentation](https://docs.aws.amazon.com/kms/)
- [HashiCorp Vault PKI](https://developer.hashicorp.com/vault/docs/secrets/pki)

### Encryption at Rest

- [PostgreSQL pgcrypto](https://www.postgresql.org/docs/current/pgcrypto.html)
- [MongoDB Field Level Encryption](https://www.mongodb.com/docs/manual/core/security-client-side-encryption/)
- [AWS EBS Encryption](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/EBSEncryption.html)

### Field-Level Encryption

- [Cloud KMS Field-Level Encryption](https://cloud.google.com/kms/docs/data-encryption)
- [AWS DynamoDB Encryption](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/EncryptionAtRest.html)
- [GCP Confidential Computing](https://cloud.google.com/confidential-computing)