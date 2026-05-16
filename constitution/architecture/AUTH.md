# AUTH.md - Authentication Patterns

**Authority:** guidance (comprehensive authentication with exact token structures, flows, and security specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** OAuth 2.0, OIDC, JWT, mTLS, SAML, API keys, session management with exact specifications for pre-inference context

---

## 1. OAuth 2.0 & OpenID Connect

### 1.1 OAuth 2.0 Grant Types

#### Authorization Code Flow (Web Applications)
```
┌──────────                            ┌──────────────┐
│   Browser                           │   Auth Server │
│                                      │               │
│  1. GET /authorize?                  │               │
│     client_id=app                   │               │
│     redirect_uri=https://app/callback│               │
│     response_type=code               │               │
│     scope=openid profile email        │               │
│     state=random_state                │               │
│     code_challenge=S256(challenge)   │               │
│     code_challenge_method=S256       │               │
│─────────────────────────────────────►│               │
│                                      │               │
│  2. User authenticates               │               │
│     (forms, MFA if required)        │               │
│─────────────────────────────────────►│               │
│                                      │               │
│  3. POST /login (credentials)        │               │
│     username=user@example.com        │               │
│     password=SecurePass123!          │               │
│─────────────────────────────────────►│               │
│                                      │               │
│  4. 302 Redirect with code           │               │
│     Location: https://app/callback   │               │
│     ?code=auth_code_abc123           │               │
│     &state=random_state              │               │
│◄─────────────────────────────────────│               │
│                                      │               │
│  5. POST /token                      │               │
│     grant_type=authorization_code    │               │
│     code=auth_code_abc123            │               │
│     redirect_uri=https://app/callback│               │
│     client_id=app                    │               │
│     code_verifier=plain_text_challenge│             │
│─────────────────────────────────────►│               │
│                                      │               │
│  6. Response:                        │               │
│     access_token: eyJhbGciOi...      │               │
│     token_type: Bearer               │               │
│     expires_in: 3600                 │               │
│     refresh_token: dGhpcyBpcy...     │               │
│     id_token: eyJhbGciOi...          │               │
│◄─────────────────────────────────────│               │
└──────────                            └──────────────┘
```

#### PKCE Extension (Mobile Apps, SPAs)
```yaml
# PKCE (Proof Key for Code Exchange) is REQUIRED for:
# - Public clients (no client secret)
# - Mobile applications
# - Single Page Applications (SPAs)
# - Any scenario where authorization code could be intercepted

# Step 1: Generate code verifier and challenge
code_verifier: "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"  # 43-128 chars, high entropy
code_challenge: "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"  # BASE64URL(SHA256(code_verifier))
code_challenge_method: "S256"  # Always use S256, plain is deprecated

# The authorization request now includes:
# - code_challenge: Base64URL encoded SHA256 hash of code_verifier
# - code_challenge_method: "S256"

# Step 2: Token exchange requires code_verifier
POST /token
grant_type: authorization_code
code: auth_code_received
redirect_uri: https://app/callback
client_id: app_id
code_verifier: dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk  # Original plain text
```

#### Client Credentials Flow (Machine-to-Machine)
```yaml
# For service-to-service communication without user context
POST /token
grant_type: client_credentials
client_id: my-service
client_secret: very_secret_value
scope: api:read api:write

# Response:
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "api:read api:write"
}

# Usage:
GET /api/resource
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### Device Authorization Flow (CLI, Smart TV)
```yaml
# For devices with limited input capability
# Step 1: Device requests codes
POST /device/code
client_id: my-cli-app
scope: repo read:org

# Response:
{
  "device_code": "GmRhmhcxhwAzkoEqiMEg_DnyEysNkuNhszIySk9eS",
  "user_code": "WDJB-MJHT",
  "verification_uri": "https://example.com/device",
  "verification_uri_complete": "https://example.com/device?user_code=WDJB-MJHT",
  "expires_in": 1800,
  "interval": 5
}

# Step 2: User visits verification_uri and enters user_code
# Step 3: Device polls for token
POST /token
grant_type: urn:ietf:params:oauth:grant-type:device_code
device_code: GmRhmhcxhwAzkoEqiMEg_DnyEysNkuNhszIySk9eS
client_id: my-cli-app

# Keep polling until user completes auth:
# - error: authorization_pending (keep polling)
# - error: slow_down (increase interval)
# - success: receive tokens
```

### 1.2 Token Response Structure

```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwczovL2V4YW1wbGUuY29tIiwiYXVkIjoiYXBpLmV4YW1wbGUuY29tIiwic3ViIjoiMTIzNDU2Nzg5MCIsInJvbGUiOiJ1c2VyIiwiZW1haWwiOiJ1c2VyQGV4YW1wbGUuY29tIiwiaWF0IjoxNzA2NzAwMDAwLCJleHAiOjE3MDY3MDM2MDAsImp0aSI6IjEyMzQ3ODkwYWJjZGVmIn0.dGVzdF9zaWduYXR1cmU",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "tGz8sB7pCVk-guqB8E2m5aH5pQ3kL9xR6wM2vN8fQ0m",
  "id_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwczovL2V4YW1wbGUuY29tIiwiYXVkIjoiYXBpLmV4YW1wbGUuY29tIiwic3ViIjoiMTIzNDU2Nzg5MCIsIm5vbmNlIjoiM2RkMmFmMzMtMDQwZi00ZGFhLWE1M2MtYmY0MjFhZjVlNTNiIiwiaWF0IjoxNzA2NzAwMDAwLCJleHAiOjE3MDY3MDM2MDAsInN1YiI6IjEyMzQ1Njc4OTAiLCJub25jZSI6IjNkZDJhZjMzLTA0MGYtNGRhYS1hNTNjLWJmNDIxYWY1ZTUzYiIsImFkbWluIjp0cnVlLCJlbWFpbCI6InVzZXJAZXhhbXBsZS5jb20iLCJnaXZlbl9uYW1lIjoiVXNlciIsImZhbWlseV9uYW1lIjoiVGVzdCJ9.TEST_SIGNATURE",
  "scope": "openid profile email api:read api:write"
}
```

### 1.3 JWT Structure

```yaml
# JWT has three parts: header.payload.signature
# All are Base64URL encoded (not Base64)

# Part 1: Header
{
  "alg": "RS256",           # RS256 | RS384 | RS512 | ES256 | ES384 | ES512 | HS256
  "typ": "JWT",             # Always "JWT"
  "kid": "key-id-123",      # Key ID for key rotation
  "jku": "https://auth.example.com/.well-known/jwks.json"  # Key set URL (optional)
}

# Part 2: Payload (Claims)
{
  # Registered claims (standard):
  "iss": "https://auth.example.com",           # Issuer
  "sub": "1234567890",                          # Subject (user ID)
  "aud": ["api.example.com", "app.example.com"], # Audience (array or string)
  "exp": 1706703600,                           # Expiration time (Unix timestamp)
  "nbf": 1706700000,                           # Not before (optional)
  "iat": 1706700000,                           # Issued at
  "jti": "unique-token-id-123",                 # JWT ID (for revocation)
  
  # Public claims (custom):
  "email": "user@example.com",
  "email_verified": true,
  "name": "User Test",
  "given_name": "User",
  "family_name": "Test",
  "picture": "https://example.com/avatar.jpg",
  "locale": "en-US",
  "zoneinfo": "America/New_York",
  
  # Authorization claims:
  "roles": ["user", "admin"],
  "permissions": ["read", "write", "delete"],
  "scope": "openid profile email api:read",
  "org_id": "org_abc123",
  "tenant_id": "tenant_xyz789",
  
  # Additional context:
  "amr": ["pwd", "mfa"],          # Authentication methods reference
  "auth_time": 1706700000,        # When authentication occurred
  "nonce": "random-nonce-value",  # For replay attack prevention
  "at_hash": "abc123",            # Access token hash (in ID token)
  "c_hash": "def456",             # Code hash (in ID token)
  
  # Custom private claims:
  "custom_claim": "any-value"
}

# Part 3: Signature
# RS256: RSASSA-PKCS1-v1_5 with SHA-256
# The signature is computed over: BASE64URL(header)."."BASE64URL(payload)
# Then encrypted with the private key
```

### 1.4 ID Token Validation (OIDC)

```yaml
# MUST validate ALL of the following:
# 1. Signature verification
#    - Fetch JWKS from issuer's well-known endpoint
#    - Find key by "kid" in token header
#    - Verify signature using appropriate algorithm
openssl dgst -sha256 -verify public.pem -signature token.sig token.txt

# 2. Issuer validation
if token.iss != "https://auth.example.com":
    raise InvalidIssuerError()

# 3. Audience validation
if expected_audience not in token.aud:
    raise InvalidAudienceError()

# 4. Expiration check
if current_time > token.exp:
    raise TokenExpiredError()

# 5. Not-before check (if present)
if current_time < token.nbf:
    raise TokenNotYetValidError()

# 6. Issued-at sanity check (within acceptable skew)
if abs(current_time - token.iat) > 5 * 60:  # 5 minutes
    raise SuspiciousTimeError()

# 7. Nonce validation (if present in original auth request)
if nonce != token.nonce:
    raise InvalidNonceError()
```

---

## 2. Token Storage & Security

### 2.1 Secure Token Storage

```yaml
# BROWSER (SPAs):
# ✅ Use HttpOnly, Secure cookies (for access tokens)
# ✅ Memory storage for short-lived tokens
# ❌ localStorage is vulnerable to XSS
# ❌ sessionStorage is vulnerable to XSS

# Recommended: Cookies with appropriate settings
Set-Cookie: access_token=xxx; 
  HttpOnly;     # Prevent JavaScript access
  Secure;       # HTTPS only
  SameSite=Strict;  # CSRF protection (or Lax for GET requests)
  Path=/;
  Max-Age=3600;
  Domain=api.example.com;

# MOBILE (iOS/Android):
# ✅ iOS: Keychain (kSecAttrAccessibleWhenUnlockedThisDeviceOnly)
# ✅ Android: EncryptedSharedPreferences (Jetpack Security)
# ❌ SharedPreferences (unencrypted)
# ❌ UserDefaults (unencrypted)

# ANDROID example (Jetpack Security):
val masterKey = MasterKey.Builder(context)
    .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
    .build()

val sharedPreferences = EncryptedSharedPreferences.create(
    context,
    "secure_prefs",
    masterKey,
    EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
    EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
)
sharedPreferences.edit().putString("access_token", token).apply()

# DESKTOP:
# ✅ System credential manager (Keychain, libsecret on Linux, DPAPI on Windows)
# ✅ Platform-specific encryption (macOS Keychain, Windows DPAPI)
# ❌ Plain text files
# ❌ Config files in home directory
```

### 2.2 Token Lifecycle

```yaml
# Access Token: Short-lived (15 minutes - 1 hour)
# - Included in API requests
# - Cannot be revoked (stateless)
# - Must be secured (not logged, not stored in URL)

# Refresh Token: Long-lived (1 day - 30 days)
# - Used to obtain new access tokens
# - Stored securely server-side (or as opaque token)
# - Can be revoked (stateful)
# - Rotation on use (issue new refresh, invalidate old)

# ID Token: Short-lived (15 minutes - 1 hour)
# - Contains user claims
# - Verified by client, not sent to APIs
# - Not for API authentication

# Token Refresh Flow:
POST /token
grant_type: refresh_token
refresh_token: dGhpcyBpcyB0aGUgcmVmcmVzaCB0b2tlbg...
client_id: app_id
client_secret: secret  # For confidential clients

# Response:
{
  "access_token": "new_access_token...",
  "refresh_token": "new_refresh_token...",  # Token rotation
  "token_type": "Bearer",
  "expires_in": 3600,
  "id_token": "new_id_token..."  # If openid scope was requested
}

# The old refresh token is immediately invalidated
# This provides security: stolen refresh token only usable once
```

### 2.3 Token Revocation

```yaml
# RFC 7009 - Token Revocation
POST /revoke
Content-Type: application/x-www-form-urlencoded
Authorization: Basic base64(client_id:client_secret)

token: the_token_to_revoke
token_type_hint: access_token  # Optional: access_token | refresh_token

# Response: 200 OK (always, even if token was invalid)
# For refresh tokens, server should also revoke related tokens

# Implementation considerations:
# - Store revoked tokens in Redis with TTL = token remaining lifetime
# - Check token blacklist on every API request
# - Alternatively, use shorter-lived tokens to reduce revocation need
```

---

## 3. API Key Authentication

### 3.1 API Key Types & Usage

```yaml
# Type 1: User-bound API Keys (tied to user identity)
# Pros: Auditable per-user, can revoke per-user
# Cons: User may share, harder to rotate

# Header format:
X-API-Key: sk_live_abc123def456ghi789

# Or in Authorization header:
Authorization: ApiKey sk_live_abc123def456ghi789

# Type 2: Service-bound API Keys (tied to service/application)
# Pros: Easier rotation, no user sharing
# Cons: Cannot audit per-user actions

# Type 3: Hierarchical Keys (multiple environments)
# sk_live_xxx (production)
# sk_test_xxx (test/sandbox)
# sk_dev_xxx (development only)

# Key format conventions:
# API Key: sk_live_4eC59HqMpZf7nQ6t
# Secret Key: sk_prod_Zxf8gT3vL9mR2wK5pB7cD4sA1qE6jH0
# Public Key: pk_live_7rT4pW9xF1mK3jL6nB8vC2zQ5yE0uO
```

### 3.2 API Key Security

```yaml
# Storage (Server-side):
# ✅ Hash before storage (like passwords)
#    - SHA-256 of the key
#    - Store: hash(api_key) in database
#    - Compare: hash(submitted_key) == stored_hash
# ✅ Never log API keys
# ✅ Never return API keys in API responses (only show on creation)

# Transmission:
# ✅ Always use HTTPS
# ✅ Send in headers, never in URL (gets logged)
# ❌ Never in query parameters (bookmarks, logs, referrer)
# ❌ Never in body (might get logged)

# Rate Limiting:
# - Per API key rate limits
# - Implement circuit breaker on auth service
# - Log and alert on unusual patterns

# Rotation:
# - Support multiple active keys per user (for rotation)
# - Grace period before invalidating old key
# - Notification before rotation
```

---

## 4. Session Management

### 4.1 Server-Side Sessions

```yaml
# Session Store (Redis example):
# Key: session:{session_id}
# TTL: 24 hours

HSET session:abc123 \
  user_id "1234567890" \
  email "user@example.com" \
  roles "admin,user" \
  created_at "1706700000" \
  last_active "1706703600" \
  ip_address "192.168.1.1" \
  user_agent "Mozilla/5.0..."

# Session cookie:
Set-Cookie: session_id=abc123; 
  HttpOnly;      # Prevent XSS
  Secure;        # HTTPS only
  SameSite=Strict;
  Path=/;
  Max-Age=86400;  # 24 hours
  Domain=example.com;

# Session validation:
1. Extract session_id from cookie
2. Check in Redis: GET session:abc123
3. If not found → Invalid session (logout)
4. If found → Load session data, attach to request context
5. Update last_active timestamp
```

### 4.2 Session Security

```yaml
# Session Hijacking Prevention:
# 1. Bind session to IP address (with caution for mobile)
if session.ip_address != request.ip:
    # Consider device fingerprinting for mobile
    # Allow some IP subnets but alert on changes
    log_security_event("IP changed for session", session_id)

# 2. Bind session to User-Agent
if session.user_agent != request.user_agent:
    invalidate_session(session_id)

# 3. Regenerate session ID after authentication
#    (prevents session fixation attacks)
session_id = generate_secure_random_id()
DELETE session:old_session_id
CREATE session:new_session_id with same data

# 4. Concurrent session limits
session_count = INCR user_sessions:{user_id}
if session_count > max_concurrent_sessions:
    # Force logout oldest session
    oldest_session = LRANGE user_session_list:{user_id} 0 0
    DELETE session:{oldest_session}

# Session Timeout:
# - Idle timeout: 30 minutes (or 15 for admin)
# - Absolute timeout: 24 hours
# - Force re-authentication for sensitive operations
```

---

## 5. Multi-Factor Authentication (MFA)

### 5.1 TOTP Implementation

```yaml
# TOTP: Time-based One-Time Password (RFC 6238)

# Shared Secret (Base32 encoded):
# Stored in password database, encrypted
shared_secret: "JBSWY3DPEHPK3PXP"  # Base32("Hello!") example

# TOTP Generation (server-side):
import pyotp

totp = pyotp.TOTP(shared_secret)
current_otp = totp.at(time.time())  # 6-digit code
# Or verify:
is_valid = totp.verify(user_provided_otp)  # Handles +/- 1 interval

# TOTP URI (for QR code generation):
otpauth://totp/Example:user@example.com?\
  secret=JBSWY3DPEHPK3PXP\
  &issuer=Example\
  &algorithm=SHA1\
  &digits=6\
  &period=30

# QR Code payload:
{
  "otpauth": "totp",
  "secret": "JBSWY3DPEHPK3PXP",
  "issuer": "Example",
  "accountname": "user@example.com"
}

# TOTP Validation Window:
# Default: TOTP window = 1 (current + 1 before, 1 after)
# For clock drift, increase window to 3 or 5
is_valid = totp.verify(user_otp, valid_window=2)
# This allows 4.5 minutes (30s * 5 interval) of clock drift
```

### 5.2 WebAuthn/FIDO2 (Passwordless)

```yaml
# Registration:
# 1. Server generates challenge and options
POST /webauthn/register/options
{
  "user": {
    "id": "user_123",
    "name": "user@example.com",
    "displayName": "User Test"
  },
  "rp": {
    "name": "Example App",
    "id": "example.com",
    "icon": "https://example.com/icon.png"
  },
  "pubKeyCredParams": [
    {"alg": -7, "type": "public-key"},  # ES256
    {"alg": -257, "type": "public-key"}  # RS256
  ],
  "timeout": 60000,
  "attestation": "none",  # none | indirect | direct | enterprise
  "authenticatorSelection": {
    "authenticatorAttachment": "platform",  # platform | cross-platform
    "requireResidentKey": true,
    "residentKey": "required",
    "userVerification": "preferred"  # required | preferred | discouraged
  },
  "excludeCredentials": [],  # Prevent duplicate registrations
  "challenge": "random_challenge_from_server"
}

# 2. Client creates credential
const credential = await navigator.credentials.create({
  publicKey: {
    rp: { id: "example.com", name: "Example App" },
    user: { id: Uint8Array.from("user_123", c => c.charCodeAt(0)), name: "user@example.com" },
    challenge: Uint8Array.from(base64url_decode(challenge)),
    pubKeyCredParams: [{ alg: -7, type: "public-key" }],
    authenticatorSelection: {
      authenticatorAttachment: "platform",
      requireResidentKey: true,
      userVerification: "preferred"
    }
  }
});

# 3. Server stores credential
POST /webauthn/register/result
{
  "id": "credential_id",
  "rawId": "base64url_encoded_id",
  "type": "public-key",
  "response": {
    "attestationObject": "base64url_cbor_attestation",
    "clientDataJSON": "base64url_json"
  }
}

# Server validates:
# 1. Verify attestation signature
# 2. Verify challenge matches
# 3. Verify rpId matches expected
# 4. Verify counter incremented (anti-replay)
# 5. Store credential public key

# Authentication:
POST /webauthn/auth/options
{
  "challenge": "server_challenge",
  "rpId": "example.com",
  "timeout": 60000,
  "userVerification": "preferred",
  "allowCredentials": [
    { "id": "credential_id", "type": "public-key" }
  ]
}

# Client:
const assertion = await navigator.credentials.get({
  publicKey: {
    challenge: Uint8Array.from(base64url_decode(challenge)),
    rpId: "example.com",
    allowCredentials: [{ id: credential_id, type: "public-key" }],
    userVerification: "preferred"
  }
});

# Server validates:
# 1. Verify signature using stored public key
# 2. Verify challenge matches
# 3. Verify rpId matches
# 4. Verify counter > stored counter
# 5. Extract user ID from credential
```

---

## 6. mTLS (Mutual TLS)

### 6.1 mTLS Certificate Structure

```yaml
# Server Certificate (typical):
Subject: CN=api.example.com
Subject Alternative Names: DNS:api.example.com, DNS:*.example.com
Issuer: CN=Let's Encrypt Authority X3, O=Let's Encrypt, C=US
Validity: 2024-01-01 to 2024-04-01
Public Key: RSA 2048-bit
Signature Algorithm: SHA256withRSA

# Client Certificate:
Subject: CN=client@example.com, O=My Organization, OU=Clients
Subject Alternative Names: email:client@example.com
Issuer: CN=My Organization CA, O=My Organization, C=US
Validity: 2024-01-01 to 2025-01-01
Public Key: ECDSA P-256
Signature Algorithm: SHA256withECDSA
Extended Key Usage: TLS Web Client Authentication (1.3.6.1.5.5.7.3.2)
```

### 6.2 mTLS Configuration

```yaml
# Go gRPC mTLS server configuration:
creds, err := credentials.newTLS(&tls.Config{
    // Require client certificate
    ClientAuth: tls.RequireAndVerifyClientCert,
    
    // Certificates to present to clients
    Certificates: []tls.Certificate{serverCert},
    
    // CA to verify client certificates
    ClientCAs: caCertPool,
    
    // Minimum TLS version
    MinVersion: tls.VersionTLS12,
    
    // Cipher suites (specific list for compliance)
    CipherSuites: []uint16{
        tls.TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
        tls.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
        tls.TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
        tls.TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
    },
    
    // Curve preferences (specific curves only)
    CurvePreferences: []tls.CurveID{
        tls.CurveP521,
        tls.CurveP384,
        tls.CurveP256,
    },
    
    // Session tickets for resumption
    SessionTicketsDisabled: false,
    TicketKeyName: []byte("session-ticket-key"),
})

# NGINX mTLS configuration:
server {
    listen 443 ssl;
    server_name api.example.com;
    
    ssl_certificate /etc/ssl/certs/server.crt;
    ssl_certificate_key /etc/ssl/private/server.key;
    ssl_client_certificate /etc/ssl/certs/ca.crt;  # CA for client verification
    ssl_verify_client on;  # Require client cert
    ssl_verify_depth 2;  # CA chain depth
    
    # Verify client certificate
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;
    
    # OCSP stapling
    ssl_stapling on;
    ssl_stapling_verify on;
}
```

### 6.3 SPIFFE/SPIRE for Service Mesh

```yaml
# Workload registration (SPIRE server config):
apiVersion: spire.spiffe.io/v1alpha1
kind: ClusterSPIFFEID
metadata:
  name: web-server-identity
spec:
  spiffeIDTemplate: "spiffe://example.com/ns/{{.PodMeta.Namespace}}/sa/{{.PodSpec.ServiceAccountName}}"
  podSelector:
    matchLabels:
      app: web-server
  namespaceSelector:
    matchLabels:
      kubernetes.io/metadata.name: production

# This creates SVIDs like:
# spiffe://example.com/ns/production/sa/web-server

# Service mesh mTLS (Istio + SPIRE):
# 1. SPIRE agent attests pod and provides SVID
# 2. Istio Citadel (or Vault) uses SVID for mTLS
# 3. All service-to-service communication uses mTLS

# Certificate structure:
{
  "spiffe_id": "spiffe://example.com/ns/production/sa/web-server",
  "subject": {
    "common_name": "spiffe://example.com/ns/production/sa/web-server",
    "organization": "example"
  },
  "sans": [
    "spiffe://example.com/ns/production/sa/web-server",
    "pod-12345.production.pod.svc.cluster.local"
  ],
  "ttl": "1h",
  "signing_cert_issuer": "spiffe://example.com"
}
```

---

## 7. SAML 2.0

### 7.1 SAML Assertion Structure

```xml
<samlp:Response xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" ID="_abc123" Version="2.0">
  <saml:Issuer xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">https://idp.example.com</saml:Issuer>
  <samlp:Status>
    <samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/>
  </samlp:Status>
  <saml:Assertion xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_def456" Version="2.0">
    <saml:Issuer>https://idp.example.com</saml:Issuer>
    <saml:Subject>
      <saml:NameID Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress">
        user@example.com
      </saml:NameID>
      <saml:SubjectConfirmation Method="urn:oasis:names:tc:SAML:2.0:cm:bearer">
        <saml:SubjectConfirmationData 
          NotOnOrAfter="2024-01-01T12:00:00Z"
          Recipient="https://app.example.com/saml/callback"/>
      </saml:SubjectConfirmation>
    </saml:Subject>
    <saml:Conditions NotBefore="2024-01-01T11:55:00Z" NotOnOrAfter="2024-01-01T12:05:00Z">
      <saml:AudienceRestriction>
        <saml:Audience>https://app.example.com</saml:Audience>
      </saml:AudienceRestriction>
    </saml:Conditions>
    <saml:AuthnStatement AuthnInstant="2024-01-01T11:58:00Z">
      <saml:AuthnContext>
        <saml:AuthnContextClassRef>urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport</saml:AuthnContextClassRef>
      </saml:AuthnContext>
    </saml:AuthnStatement>
    <saml:AttributeStatement>
      <saml:Attribute Name="email">
        <saml:AttributeValue>user@example.com</saml:AttributeValue>
      </saml:Attribute>
      <saml:Attribute Name="firstName">
        <saml:AttributeValue>User</saml:AttributeValue>
      </saml:Attribute>
      <saml:Attribute Name="roles">
        <saml:AttributeValue>user</saml:AttributeValue>
        <saml:AttributeValue>admin</saml:AttributeValue>
      </saml:Attribute>
    </saml:AttributeStatement>
  </saml:Assertion>
</samlp:Response>
```

### 7.2 SAML SSO Flow

```yaml
# 1. SP Initiated SSO:
#    User accesses SP → SP redirects to IdP with AuthnRequest
#    User authenticates at IdP → IdP posts SAML Response to SP

# AuthnRequest (Redirect binding):
GET /sso/saml2?SAMLRequest=base64_deflate(xml)&&RelayState=return_url

# SAMLRequest content:
<samlp:AuthnRequest 
  xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
  ID="_auth123"
  Version="2.0"
  IssueInstant="2024-01-01T11:50:00Z"
  AssertionConsumerServiceURL="https://app.example.com/saml/callback"
  ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST">
  <saml:Issuer xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion">https://app.example.com</saml:Issuer>
  <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress" AllowCreate="true"/>
</samlp:AuthnRequest>

# 2. IdP processes and returns SAML Response (POST binding):
POST /sso/saml2
SAMLResponse: base64(signed_xml_assertion)
RelayState: return_url

# 3. SP validates and creates session:
#    - Verify signature using IdP's public key
#    - Verify issuer matches expected IdP
#    - Verify destination matches ACS URL
#    - Verify NotOnOrAfter and NotBefore conditions
#    - Verify AudienceRestriction matches SP entity ID
#    - Extract NameID and attributes
#    - Create local session
```

---

## 8. Security Anti-Patterns

### 8.1 Critical Mistakes

```yaml
# ❌ NEVER store passwords in plain text
# ✅ MUST use bcrypt (cost factor 10-12), Argon2id, or scrypt

# Bad:
password == "plaintext"  # NEVER DO THIS
password == hash  # Still vulnerable if hash is known

# Good:
bcrypt.checkpw(submitted_password, stored_hash)  # Constant-time comparison

# ❌ NEVER use MD5, SHA1, or SHA256 for password hashing
# These are fast hashes, susceptible to GPU cracking
# Use slow KDFs designed for passwords

# ❌ NEVER implement your own crypto
# Use established libraries: libsodium, OpenSSL, cryptography.io
# Custom implementations almost always have vulnerabilities

# ❌ NEVER log sensitive data
# - Passwords, tokens, API keys, PII
# - Use structured logging with sanitization
logger.info("Login attempt", extra={"user": user_email, "ip": ip})
# Log token type, not the token value
logger.debug("Token issued", extra={"type": "access", "user": user_id})

# ❌ NEVER accept tokens in URLs
# URLs get logged in server logs, proxies, browser history
# ❌ Use POST body for token transmission (except form-encoded)
# ✅ Use Authorization header

# ❌ NEVER use predictable session IDs
# ❌ Don't use: user_id, timestamp, random() with small range
# ✅ Use: cryptographically secure random (32+ bytes)
session_id = os.urandom(32).hex()  # 64 character hex string

# ❌ NEVER skip SSL certificate validation (in production)
# ❌ Don't use AllowInsecure=True, verify=False
# This enables MITM attacks
```

### 8.2 Timing Attack Prevention

```yaml
# Constant-time comparison for tokens and passwords:
import hmac

def secure_compare(a: bytes, b: bytes) -> bool:
    """Compare two values in constant time to prevent timing attacks."""
    if len(a) != len(b):
        # Return early but with same-time comparison
        return hmac.compare_digest(a, a)  # Always same time given same length
    return hmac.compare_digest(a, b)

# Use for:
# - Token validation
# - HMAC verification
# - API key comparison
# - Session ID comparison

# Bad (timing leak):
if stored_token == submitted_token:  # String comparison, early exit
    return True
return False

# Good:
return hmac.compare_digest(stored_token, submitted_token)

# JWT signature verification:
# Use library that handles constant-time comparison
# e.g., PyJWT, jose-python, node-jsonwebtoken
```

---

## 9. Decision Frameworks

### 9.1 Auth Method Selection Matrix

| Use Case | Recommended Method | Alternative |
|----------|-------------------|-------------|
| Web app with server backend | OAuth 2.0 + OIDC (Authorization Code) | Session-based auth |
| SPA (browser) | OAuth 2.0 + PKCE (Authorization Code) | Same-site cookies |
| Mobile app | OAuth 2.0 + PKCE | Biometric + encrypted storage |
| CLI tool | OAuth 2.0 Device Authorization Flow | Personal access tokens |
| Service-to-service (backend) | OAuth 2.0 Client Credentials + mTLS | API keys (hashed) |
| IoT/embedded | mTLS with hardware security | Pre-shared keys |
| Enterprise SSO | SAML 2.0 or OIDC | OIDC preferred for new |
| Passwordless | WebAuthn/FIDO2 | Magic links |

### 9.2 Token Lifetime Selection

| Token Type | Lifetime | Rationale |
|-----------|----------|----------|
| Access token (high security) | 5-15 min | Short window for compromise |
| Access token (standard) | 15-60 min | Balance security/usability |
| Refresh token (web) | 1-24 hours | Match session length |
| Refresh token (mobile) | 30-90 days | Long-lived convenience |
| API key (user-bound) | Until revoked | Manual rotation |
| API key (service) | 90-365 days | Rotation schedule |
| Session ID | 8-24 hours | Standard session length |
| CSRF token | Same as session | Session-scoped |

### 9.3 Password Policy Framework

```yaml
# Modern password policy (NIST SP 800-63B):
# - Minimum 8 characters (no maximum)
# - Check against known breached passwords
# - No composition rules (no "must have upper, lower, digit")
#   - Users use predictable patterns like "Password123!"
# - No password hints
# - Allow paste in password fields (encourages managers)
# - Allow spell-check in password fields
# - MFA required for sensitive accounts

# Password strength estimation:
# - Use zxcvbn-like scoring
# - Reject passwords with score < 3
# - Consider contextual penalties (username in password)

# Breached password check:
# - HaveIBeenPwned API (k-anonymity)
# - Internal breached password database
# - Check during registration AND login (if large breach detected)
```

---

## Links

- `architecture/AUTHZ.md` - Authorization patterns (RBAC, ABAC)
- `architecture/SECRETS.md` - Secrets management
- `architecture/NETWORK_SECURITY.md` - mTLS, network security
- `architecture/ENCRYPTION.md` - Encryption standards
- `specs/SECURITY.md` - Security doctrine (binding)

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-15 | Initial comprehensive authentication reference |