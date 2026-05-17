# SECURITY.md - Security Architecture

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

### 1.2 Principle of Least Privilege
**Give minimum access necessary.**
- Users: Only permissions needed for role
- Services: Only API calls needed to function
- Applications: Only file/database access required
- Regular access reviews

### 1.3 Zero Trust
**Never trust, always verify.**
- No implicit trust based on network location
- Verify every request, every time
- Assume network is compromised
- Strong authentication everywhere

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

### 2.2 Attack Surface Analysis
**Identify entry points:**
- APIs and endpoints
- Authentication mechanisms
- File uploads/downloads
- Admin interfaces
- Third-party integrations
- Logging and monitoring

### 2.3 Threat Modeling Process
1. **Diagram:** Create data flow diagram
2. **Identify:** Entry points and trust boundaries
3. **STRIDE:** Apply threat categories
4. **Rate:** Risk severity (likelihood × impact)
5. **Mitigate:** Design countermeasures
6. **Validate:** Review and test

---

## 3. Authentication

### 3.1 Passwords
**Requirements:**
- Minimum length: 12+ characters
- Complexity: Mix of character types
- No common passwords (check against breach databases)
- Rate limiting on login attempts
- Account lockout after failures
- Secure storage (bcrypt, Argon2, scrypt)

**Patterns:**
- Password reset via email with token
- Multi-factor authentication (MFA)
- Password managers encouraged

### 3.2 Multi-Factor Authentication (MFA)
**Factors:**
- **Something you know:** Password, PIN
- **Something you have:** Phone, hardware key
- **Something you are:** Fingerprint, face

**Implementation:**
- TOTP (Time-based One-Time Password)
- Push notifications
- Hardware security keys (FIDO2/WebAuthn)
- SMS (least secure, but better than nothing)

### 3.3 Session Management
- **Token-based:** JWT, opaque tokens
- **Session IDs:** Server-side sessions
- **Secure flags:** HttpOnly, Secure, SameSite
- **Expiry:** Short-lived access tokens
- **Refresh tokens:** Long-lived, rotate on use
- **Logout:** Invalidate tokens server-side

### 3.4 OAuth 2.0 / OpenID Connect
**Use for:**
- Third-party authentication ("Login with Google")
- Delegated authorization
- API access on user's behalf

**Security considerations:**
- Use PKCE for mobile/SPA
- Validate state parameter
- Verify ID token signatures
- Use HTTPS redirect URIs only

---

## 4. Authorization

### 4.1 RBAC (Role-Based Access Control)
- **Roles:** Group permissions (admin, user, guest)
- **Users:** Assigned to roles
- **Permissions:** Actions on resources

**When to use:** Hierarchical organizations, clear roles

### 4.2 ABAC (Attribute-Based Access Control)
- **Attributes:** User, resource, environment properties
- **Policies:** Rules combining attributes
- **Dynamic:** Context-aware decisions

**When to use:** Complex authorization, fine-grained control

### 4.3 ACL (Access Control Lists)
- **Resources:** Have list of who can access
- **Permissions:** Read, write, execute
- **Direct:** User-resource mapping

**When to use:** File systems, simple resource ownership

### 4.4 Authorization Best Practices
- **Deny by default:** Whitelist, not blacklist
- **Fail closed:** Deny if authorization check fails
- **Validate server-side:** Don't trust client
- **Least privilege:** Grant minimum necessary
- **Regular reviews:** Audit permissions

---

## 5. Data Protection

### 5.1 Encryption at Rest
- **Database:** Transparent Data Encryption (TDE)
- **Files:** Encrypt before storage
- **Backups:** Encrypted backup storage
- **Keys:** Managed by KMS, not in code

### 5.2 Encryption in Transit
- **TLS 1.2+:** Minimum version
- **Certificate pinning:** Mobile apps
- **HSTS:** Enforce HTTPS
- **mTLS:** Service-to-service authentication

### 5.3 Key Management
- **Never hardcode:** Use secret managers
- **Rotation:** Regular key rotation
- **Separation:** Different keys for different purposes
- **Access logging:** Audit key access
- **HSM:** Hardware Security Modules for high security

### 5.4 Data Classification
- **Public:** No restrictions
- **Internal:** Company use only
- **Confidential:** Restricted access
- **Restricted:** Compliance requirements (PII, PHI)

**Protection by classification:**
- Encryption requirements
- Access controls
- Logging and monitoring
- Retention policies

---

## 6. Input Validation

### 6.1 Validation Principles
- **Whitelist:** Allow known good, reject everything else
- **Sanitize:** Remove or escape dangerous content
- **Validate early:** At application boundary
- **Fail securely:** Reject invalid input

### 6.2 SQL Injection Prevention
- **Parameterized queries:** Never concatenate SQL
- **ORMs:** Use built-in query builders
- **Stored procedures:** Limit direct table access
- **Least privilege:** Database user permissions

### 6.3 XSS (Cross-Site Scripting) Prevention
- **Output encoding:** Escape based on context (HTML, JS, CSS, URL)
- **Content Security Policy (CSP):** Restrict script sources
- **HttpOnly cookies:** Prevent JavaScript access
- **Validate input:** Reject suspicious patterns

### 6.4 CSRF (Cross-Site Request Forgery) Prevention
- **CSRF tokens:** Unique per session
- **SameSite cookies:** Lax or Strict
- **Referrer checking:** Validate request source
- **Double-submit cookie:** Token in cookie and header

### 6.5 Command Injection Prevention
- **Avoid shell execution:** Use library functions
- **Input validation:** Strict whitelist
- **Escape arguments:** If shell execution required
- **Least privilege:** Limited execution permissions

---

## 7. Secure Development

### 7.1 Secure Coding Practices
- **Input validation:** All untrusted input
- **Output encoding:** Context-appropriate encoding
- **Authentication:** Verify identity
- **Authorization:** Check permissions
- **Error handling:** Don't leak sensitive info
- **Logging:** Security events, no sensitive data
- **Dependencies:** Regular updates, vulnerability scanning

### 7.2 Secrets Management
**Never commit secrets to code:**
- API keys
- Database passwords
- Private keys
- Encryption keys

**Use:**
- Environment variables
- Secret managers (Vault, AWS Secrets Manager)
- Encrypted configuration
- Runtime injection

### 7.3 Dependency Security
- **Inventory:** Know what you're using
- **Scanning:** Automated vulnerability detection
- **Updates:** Regular dependency updates
- **Pinning:** Lock versions for reproducibility
- **Minimal:** Only necessary dependencies

### 7.4 Security Testing
- **SAST:** Static Application Security Testing
- **DAST:** Dynamic Application Security Testing
- **Dependency scanning:** Known vulnerabilities
- **Penetration testing:** External security assessment
- **Fuzzing:** Automated input testing

---

## 8. Infrastructure Security

### 8.1 Network Security
- **VPCs:** Isolate resources
- **Subnets:** Public/private separation
- **Security groups:** Instance-level firewalls
- **NACLs:** Subnet-level rules
- **WAF:** Web Application Firewall
- **DDoS protection:** AWS Shield, Cloudflare

### 8.2 Container Security
- **Minimal images:** Reduce attack surface
- **No root:** Run as non-root user
- **Read-only filesystem:** Prevent modifications
- **Secrets:** Don't bake into images
- **Scanning:** Image vulnerability scanning
- **Runtime protection:** Detect anomalous behavior

### 8.3 Cloud Security
- **IAM:** Least privilege access
- **Encryption:** At rest and in transit
- **Logging:** CloudTrail, audit logs
- **Monitoring:** Security dashboards
- **Compliance:** Automated compliance checks

---

## 9. Incident Response

### 9.1 Preparation
- **Playbooks:** Documented response procedures
- **Tools:** Forensics, log analysis
- **Contacts:** Security team, legal, PR
- **Training:** Regular drills

### 9.2 Detection
- **Monitoring:** SIEM, anomaly detection
- **Alerting:** Paging for security events
- **Logging:** Centralized, tamper-proof
- **Honeypots:** Detect attackers early

### 9.3 Response
1. **Contain:** Stop the attack
2. **Eradicate:** Remove threat
3. **Recover:** Restore services
4. **Learn:** Post-incident review

### 9.4 Post-Incident
- **Root cause analysis:** What happened, why
- **Timeline:** When did it start, how discovered
- **Impact assessment:** What was affected
- **Remediation:** Prevent recurrence
- **Communication:** Notify affected parties

---

## 10. Anti-Patterns

- **Security through obscurity:** Assuming secrecy = security
- **Hardcoded credentials:** In code, configs, logs
- **No input validation:** Trusting all input
- **Verbose error messages:** Leaking implementation details
- **No rate limiting:** Brute force vulnerability
- **Weak cryptography:** MD5, SHA1, DES
- **No logging:** Can't detect or investigate breaches
- **Overly permissive CORS:** Allowing any origin
- **No HTTPS:** Transmitting secrets in plaintext
- **Ignoring security updates:** Running vulnerable dependencies

---

## 11. Agent System Defense Layers

When building systems where agents process external data (user input, API responses, file contents, tool output), all data must pass through ordered defense layers. No single layer is sufficient.

### The Five-Layer Model

1. **Validation** — Length limits, encoding checks, structural validation. Reject malformed input before any processing occurs.
2. **Sanitization** — Escape dangerous content, neutralize injection patterns. Remove or defang anything that could alter control flow.
3. **Policy Enforcement** — Apply rules with severity levels and enforcement actions. Policies are configurable but defaults are deny.
4. **Output Wrapping** — Structural boundaries between trusted and untrusted content. Untrusted data is always wrapped in markers that prevent it from being interpreted as instructions.
5. **Leak Detection** — Scan outbound data for secrets before transmission. Use fast literal prefix scans (e.g., `sk-`, `AKIA`, `ghp_`) followed by expensive regex only on candidates.

### Registry Protection

Registries (plugin names, constitution paths, tool names) must protect against shadowing:

- **Protected names**: Core/builtin names cannot be overridden by dynamic registration.
- **Shadow rejection**: Attempts to register a name that shadows a builtin must be rejected with a warning, not silently ignored.
- **Emit, don't swallow**: Every rejected registration attempt must produce a visible warning. Silent failure is a security anti-pattern.

---

## 12. Supply Chain Security (BINDING for production systems)

*Supply chain attacks are among the most dangerous threats - they compromise trust at the source.*

### 12.1 Software Bill of Materials (SBOM)

**Generation (BINDING for all deployed artifacts):**
- Generate SBOM for every release using SPDX or CycloneDX format
- Include all transitive dependencies, not just direct imports
- Sign SBOMs and distribute alongside artifacts
- Maintain SBOM versions tied to version control commits

**Consumption:**
- Verify SBOM before installing dependencies
- Alert on new vulnerabilities affecting components in SBOM
- Track SBOM drift between build and deploy

### 12.2 SLSA Supply Chain Levels (BINDING for critical systems)

| Level | Requirement | Threat Mitigated |
|-------|-------------|------------------|
| L0 | No guarantees | None |
| L1 | Provenance document | Tampering after build |
| L2 | Signed provenance, hermetic build | Tampering during build |
| L3 | Hardened build service | Tampering by privileged user |
| L4 | Two-party review + hermetic | All of above + insider threat |

**Implementation:**
- Use build systems that produce verifiable provenance (GitHub Actions with SLSA, Bazel)
- Require provenance verification in CI before deployment
- Maintain build integrity through hermetic, isolated builds

### 12.3 Dependency Security (BINDING)

**Allowlist over blocklist:**
- Use lockfiles that hash every dependency
- Pin to specific versions, not ranges
- Audit new dependencies before addition (not just vulnerability scans)
- Prefer well-maintained packages with multiple maintainers

**Provenance verification:**
- Verify source repository, maintainer identity, and release integrity
- Reject dependencies from forks without explicit review
- Monitor for typosquatting and dependency confusion attacks

### 12.4 Secret Scanning (BINDING in CI)

**Prevent commits:**
- Pre-commit hooks that scan for secrets before allowing commit
- CI checks that fail on any detected secret (true positives)
- No exceptions for test/fake secrets - train against real patterns

**Detect exposure:**
- Scan entire git history for secrets (git-secrets, TruffleHog)
- Alert on secret found, don't just fail
- Rotate immediately - assume compromise on detection

---

## 13. Cryptographic Standards (BINDING for any cryptographic implementation)

### 13.1 Symmetric Encryption

**Algorithms:**
| Algorithm | Key Length | Status | Use Case |
|-----------|------------|--------|----------|
| AES-256-GCM | 256-bit | RECOMMENDED | General encryption at rest |
| AES-256-GCM-SIV | 256-bit | ACCEPTABLE | Nonce-misuse resistance |
| ChaCha20-Poly1305 | 256-bit | RECOMMENDED | High performance, mobile |
| AES-128 | 128-bit | MINIMUM | Legacy compatibility only |

**Prohibited:** DES, 3DES, AES-ECB, RC4, Blowfish

**Implementation:**
- Always use authenticated encryption (GCM, Poly1305)
- Generate IVs using crypto RNG, never reuse
- Store keys in KMS, never in code or config files

### 13.2 Asymmetric Encryption

**Key Exchange:**
| Algorithm | Key Size | Status | Notes |
|-----------|----------|--------|-------|
| X25519 | 256-bit | RECOMMENDED | ECDH, fast, secure |
| ECDH P-384 | 384-bit | ACCEPTABLE | Legacy compatibility |
| FFDH-4096 | 4096-bit | ACCEPTABLE | When ECC unavailable |

**Digital Signatures:**
| Algorithm | Key Size | Status | Use Case |
|-----------|----------|--------|----------|
| Ed25519 | 256-bit | RECOMMENDED | Signatures, identity |
| ECDSA P-384 | 384-bit | ACCEPTABLE | Legacy systems |
| RSA-4096 | 4096-bit | MINIMUM | When ECC unavailable |

**Prohibited:** RSA-2048 and below, RSA with PKCSv1.5 padding

### 13.3 Hashing

| Algorithm | Status | Use Case |
|-----------|--------|----------|
| SHA-256 | MINIMUM | General hashing |
| SHA-384 | RECOMMENDED | When 256-bit insufficient |
| BLAKE3 | RECOMMENDED | Fast hashing, large data |
| Argon2id | RECOMMENDED | Password hashing |
| scrypt | ACCEPTABLE | Password hashing |

**Prohibited:** MD5, SHA-1 (except in HMAC-SHA1), Tiger

### 13.4 Password Storage (BINDING)

**Algorithm choice:**
- Argon2id (primary) - memory-hard, side-channel resistant
- scrypt (acceptable) - when Argon2 unavailable
- bcrypt (minimum) - legacy compatibility only
- NEVER use PBKDF2 with iterations < 600,000

**Implementation:**
- Generate unique salt per password (minimum 16 bytes)
- Cost parameters tuned to take >250ms on deployment hardware
- Verify against breach databases (HaveIBeenPwned API)

### 13.5 TLS/SSL

**Versions:** TLS 1.3 only for new deployments; TLS 1.2 minimum for compatibility

**Cipher Suites (TLS 1.3):**
- TLS_AES_256_GCM_SHA384
- TLS_AES_128_GCM_SHA256
- TLS_CHACHA20_POLY1305_SHA256

**For TLS 1.2:**
- Require forward secrecy (ECDHE or DHE)
- Reject connections without SNI
- Certificate verification mandatory
- HSTS required (max-age >= 1 year)

### 13.6 Key Management (BINDING)

**Key Lifecycle:**
1. **Generation:** Hardware RNG or HSM, never software RNG for production keys
2. **Storage:** HSM for master keys; KMS for service keys
3. **Distribution:** Use envelope encryption, never export raw keys
4. **Rotation:** Automatic rotation for symmetric keys; planned rotation for asymmetric
5. **Revocation:** Immediate revocation and re-encryption on suspected compromise
6. **Destruction:** Secure wipe with verification

**Key Hierarchy:**
- Master Key (HSM) → Key Encrypting Key → Data Encryption Key
- Never use master key directly for data encryption

---

## 14. SECCOMP and System Call Filtering (BINDING for containerized workloads)

### 14.1 Principle of Least Privilege for System Calls

**Default deny:**
- Block all system calls not explicitly required
- Use seccomp profile that whitelists only needed calls
- Audit unexpected syscalls as potential indicators of compromise

### 14.2 Minimal System Call Sets

**For untrusted workloads:**
```
# Allowed base syscalls
read, write, close, sigaltstack, mmap, mprotect, brk, access
exit, arch_prctl, set_tid_address, set_robust_list, prlimit64
rt_sigprocmask, rt_sigreturn, clock_gettime, restart_syscall
exit_group, epoll_wait, ppoll, clock_nanosleep
```

**Additional when needed:**
```
# Network access
socket, connect, bind, listen, accept, send, recv, shutdown
# File system (read-only)
openat, fstat, readlink, lseek
# File system (write - minimal)
openat (O_WRONLY only), unlink (rare)
# Memory management
munmap, mremap, statfs
```

### 14.3 Container Runtime Security

**Docker/OCI runtime:**
- Run containers with `--security-opt seccomp=unconfined` only when necessary
- Default seccomp profile blocks ~44 syscalls
- Apply AppArmor or SELinux profiles for additional隔离

**Kubernetes:**
- Use PodSecurityPolicies or Pod Security Standards
- Disable privileged containers
- Enforce seccomp profiles at cluster level

### 14.4 Capability Dropping (BINDING)

**Required capability drops:**
- NET_RAW (prevent spoofing)
- SYS_ADMIN (mount operations)
- SYS_MODULE (load kernel modules)
- DAC_READ_SEARCH (bypass file permissions)
- NET_ADMIN (network configuration)

**Audit capabilities:**
- Regularly audit granted capabilities vs. actual requirements
- Remove unused capabilities from running containers
- Alert on capability escalation

---

## 15. Security Monitoring and SIEM (BINDING for production)

### 15.1 Security Event Logging (BINDING)

**Log everything:**
- Authentication attempts (success and failure)
- Authorization decisions (especially denials)
- Configuration changes
- Privilege escalations
- Network connections (source, destination, port, protocol)
- Data access (especially sensitive data)
- Admin operations
- Security tool alerts

**Log format:**
- Structured JSON with timestamps (ISO 8601)
- Include: who, what, when, where, source IP, user agent
- Never log: passwords, tokens, PII (unless required for compliance)
- Tamper-proof logging with write-once storage

### 15.2 Detection Rules

**Critical alerts (immediate response):**
- Multiple failed logins from same IP
- Authentication from unusual location
- Privilege escalation detected
- Data exfiltration indicators
- Malware/trojan detection
- Lateral movement detection

**Monitoring:**
- Brute force attacks
- Port scanning
- Unusual process execution
- File integrity violations
- Network anomaly detection
- User behavior analytics (UEBA)

### 15.3 SIEM Requirements (BINDING for enterprise)

**Collection:**
- Agent-based and agentless collection
- Real-time event streaming
- Log aggregation from all sources (minimum 1 year retention)
- Cloud and on-premises coverage

**Analytics:**
- Correlation rules across data sources
- Machine learning for anomaly detection
- Threat intelligence integration
- Automated alerting with severity classification

**Response integration:**
- SOAR playbook integration for automated response
- Ticketing system integration
- Executive dashboard for security posture

---

## 16. Threat Intelligence (ADVISORY for standard operations, BINDING for SOC)

### 16.1 Intelligence Sources

**Internal:**
- Security event logs
- Incident postmortems
- Vulnerability assessments
- Red team findings

**External:**
- Commercial threat feeds (Mandiant, Recorded Future)
- Government feeds (CISA, FBI)
- ISACs (Information Sharing and Analysis Centers)
- Open source feeds (AlienVault OTX, MISP)
- Industry-specific ISACs

### 16.2 Sharing (ADVISORY)

**Share responsibly:**
- Participate in industry ISACs
- Share indicators with trusted partners
- Report to government authorities (FBI, CISA)
- Contribute to open source security lists

**Protect sensitive data:**
- Sanitize shared indicators (remove PII)
- Use TAXII/STIX for standardized sharing
- Apply traffic light protocol (TLP) markings

---

## 17. Security Research and Seminal Papers (REFERENCE)

### Foundational Texts

1. **"The Protection of Information in Computer Systems"** - Saltzer & Schroeder, 1975
   - First formal treatment of protection principles
   - Least privilege, open design, separation of privilege

2. **"Security Engineering"** - Ross Anderson, 2020
   - Comprehensive security engineering textbook
   - Threat modeling, cryptography, protocols, economics

3. **"The Tangled Web"** - Michal Zalewski, 2011
   - Browser security fundamentals
   - Origin policy,Same-origin, CSP

4. **"The Art of Software Security Assessment"** - Dowd & McDonald, 2006
   - Code review methodology
   - Vulnerability classes and detection

### Cryptography

5. **"Handbook of Applied Cryptography"** - Menezes et al., 1996
   - Comprehensive crypto reference
   - Algorithm specifications and security proofs

6. **"Cryptographic Hash Functions"** - Bart Preneel, 1999
   - Hash function design principles
   - Collision resistance foundations

### Network Security

7. **"Transport Layer Security (TLS) Protocol"** - Dierks & Rescorla, 2008
   - TLS 1.2 specification
   - Cipher suite negotiation, handshake protocol

8. **"The NSA's SKI"** - Multiple authors
   - Key exchange vulnerabilities
   - Forward secrecy importance

### Browser and Web

9. **"CSP 1.0 Specification"** - World Wide Web Consortium
   - Content Security Policy
   - Mitigation against XSS

10. **"Same-Origin Policy"** - Mozilla Developer Network
    - Browser security model
    - Cross-origin restrictions

### Threat Modeling

11. **"Threat Modeling: Designing for Security"** - Adam Shostack, 2014
    - STRIDE methodology
    - DFD-based threat identification

12. **"Patas"** - MEHTA, 2015
    - Process for attack simulation and threat analysis
    - Risk-based threat modeling

### Supply Chain

13. **"The Notorious Nine: Cloud Computing Threats"** - CSA, 2016
    - Cloud-specific threats
    - Shared responsibility model

14. **"SLSA Framework"** - Google, 2021
    - Supply chain integrity framework
    - Provenance generation and verification

---

## 18. Compliance Frameworks (ADVISORY, BINDING when scope requires)

### 18.1 SOC 2 (Service Organization Control 2)

**Trust Service Criteria:**
- Security (common criteria)
- Availability
- Processing Integrity
- Confidentiality
- Privacy

**Requirements:**
- Annual audit by certified third party
- Continuous monitoring
- Incident response procedures
- Access management
- Change management

### 18.2 HIPAA (Health Insurance Portability and Accountability Act)

**Requirements:**
- Technical safeguards (encryption, access controls, audit trails)
- Administrative safeguards (policies, training, risk assessment)
- Physical safeguards (facility access, workstation security)
- Breach notification within 60 days

**Protected Data:**
- PHI (Protected Health Information)
- EPHI (Electronic PHI)

### 18.3 GDPR (General Data Protection Regulation)

**Requirements:**
- Lawful basis for processing
- Data subject rights
- Privacy by design
- Data protection impact assessments
- Breach notification within 72 hours
- Cross-border transfer restrictions

**Key Concepts:**
- Data minimization
- Purpose limitation
- Storage limitation
- Accuracy

### 18.4 PCI-DSS (Payment Card Industry Data Security Standard)

**Requirements:**
- Secure network (firewalls, encryption)
- Cardholder data protection
- Vulnerability management
- Access control
- Network monitoring
- Information security policy

---

## 19. Memory Safety (BINDING for new systems in memory-unsafe languages)

### 19.1 Unsafe Languages (C/C++) Require Explicit Mitigation

**When using C/C++:**
- Use AddressSanitizer (ASan) in development and testing
- Use MemorySanitizer (MSan) for undefined behavior detection
- Use Control Flow Integrity (CFI) to prevent jump hijacking
- Enable stack canaries for buffer overflow detection
- Use `-fPIE -pie` for position-independent executables

### 19.2 Safe Alternatives

**Prefer safe languages:**
- Rust (memory safety without GC)
- Go (memory safety, GC)
- Java (bytecode verification, sandbox)
- C# (managed code, memory safety)

**When unsafe is required:**
- Isolate in separate process with minimal privileges
- Use hardware memory protection (MMU)
- Apply seccomp to limit syscalls

### 19.3 Common Vulnerability Classes

| Vulnerability | Root Cause | Mitigation |
|--------------|------------|------------|
| Buffer overflow | Missing bounds check | Safe languages, ASan, bounds check |
| Use after free | Dangling pointer | Safe languages, MSan, memory pools |
| Double free | Double deallocation | Safe languages, MSan, allocator metadata |
| Format string | User input in format | Safe languages, bounds-checked I/O |
| Integer overflow | Bounds check bypass | Safe languages, runtime checks |

---

## 20. Security Architecture Patterns (ADVISORY)

### 20.1 Gateway Pattern

**Benefits:**
- Centralized security policy enforcement
- Single point of authentication/authorization
- Unified logging and monitoring
- Reduced attack surface on services

**Implementation:**
- API Gateway with built-in security (Kong, Apigee)
- Service mesh with mTLS (Istio, Linkerd)
- WAF as first line of defense

### 20.2 Sidecar Pattern

**Benefits:**
- Language-agnostic security
- Decoupled from application logic
- Independent scaling and updates

**Implementation:**
- Service mesh proxies (Envoy)
- Secret injection sidecars
- Certificate management agents

### 20.3 Zero Trust Network Architecture (BINDING for production)

**Core principles:**
- Never trust, always verify
- Assume breach mentality
- Least privilege access
- Microsegmentation

**Implementation:**
- Service identity (SPIFFE/SPIRE)
- mTLS for all service-to-service communication
- Continuous authentication
- Policy engine (Open Policy Agent)
- Identity-aware proxy for user access

---

## Links

- [SECURITY](../specs/SECURITY.md) - Security doctrine (binding)
- [ARCHITECTURE](../methodology/ARCHITECTURE.md) - binding architecture
- [WEB](WEB.md) - Web security
- [CLOUD](CLOUD.md) - Cloud security
- [CODING_STANDARDS](CODING_STANDARDS.md) - Coding standards with security implications
- [COMPLIANCE](COMPLIANCE.md) - Compliance frameworks
- [DR](DR.md) - Disaster recovery patterns

### Parent Docs
- [DECAPOD](../core/DECAPOD.md) - Router and navigation charter
- [INTERFACES](../core/INTERFACES.md) - Interface contracts
- [INTENT](../specs/INTENT.md) - Intent specification

---

## Project Override Context

Project security architecture emphasis:
- Minimize trust by default: least-privilege capabilities and explicit allowlists.
- Keep secrets out of model-visible context; inject only where execution requires them.
- Distinguish sandboxed tool execution from externally hosted connectors, and apply stricter controls to the latter.
- Require auditable approval flows for high-risk actions and irreversible operations.
- Supply chain integrity: verify all dependencies, generate SBOMs for all artifacts.
- Cryptographic standards: AES-256-GCM or ChaCha20-Poly1305 for encryption, Ed25519 for signatures.
- Memory safety: prefer safe languages; when unsafe is required, use ASan/MSan in testing.
