# DECAPOD Knowledge Base - Engineering Paved Roads

**Authority:** guidance (dense engineering knowledge base with pre-inference depth)
**Layer:** Core Router
**Binding:** No
**Scope:** Comprehensive engineering knowledge organized as navigable paved roads for agent pre-inference context
**Non-goals:** Tutorial-level introductions; assumes engineering foundation knowledge

---

## Purpose

This knowledge base provides Decapod agents with dense, specific engineering context for pre-inference payloads. Unlike high-level overview documents, leaf articles here contain:

- **Exact specifications** (API shapes, schema definitions, configuration formats)
- **Concrete patterns** (production-proven implementation templates)
- **Decision matrices** (when to use X vs Y with specific tradeoffs)
- **Anti-patterns with remedies** (what breaks and how to fix)
- **Code-level references** (exact constructs, not conceptual descriptions)

The goal is for Decapod to carve out and present specific contextual slices to agents, enabling precise architectural and implementation decisions without ambiguity.

---

## Knowledge Base Index

### Infrastructure & Platform

| Topic | Leaf Document | Density Level |
|-------|--------------|---------------|
| Kubernetes Orchestration | `architecture/KUBERNETES.md` | ✅ Comprehensive - manifests, operators, networking (1200+ lines) |
| Authentication Patterns | `architecture/AUTH.md` | ✅ Comprehensive - OAuth, JWT, SAML, mTLS (900+ lines) |
| API Design | `architecture/API_DESIGN.md` | ✅ Comprehensive - REST, GraphQL, gRPC patterns (1000+ lines) |
| Cloud Architecture | `architecture/CLOUD.md` | Updated - multi-cloud patterns |
| Database & Storage | `architecture/DATA.md` | Substantial - data modeling patterns |

### API & Integration

| Topic | Leaf Document | Density Level |
|-------|--------------|---------------|
| REST API Design | `architecture/API_DESIGN.md` | Comprehensive - versioning, pagination, error handling |
| GraphQL | `architecture/GRAPHQL.md` | Pattern-heavy - schema design, federation |
| gRPC & Protocol Buffers | `architecture/GRPC.md` | Deep - proto patterns, streaming |
| Webhooks & Events | `architecture/WEBHOOKS.md` | Specific - delivery, retries, signatures |
| Message Queues | `architecture/MESSAGING.md` | Comprehensive - Kafka, RabbitMQ, SQS patterns |

### Data Architecture

| Topic | Leaf Document | Density Level |
|-------|--------------|---------------|
| Data Modeling | `architecture/DATA_MODELING.md` | Deep - normalization, schema design |
| Data Pipelines | `architecture/DATA_PIPELINES.md` | Comprehensive - ETL, streaming, governance |
| Cache Strategies | `architecture/CACHING.md` | Specific - patterns, invalidation, Redis/Memcached |
| Search Architecture | `architecture/SEARCH.md` | Deep - Elasticsearch, full-text patterns |

### Security & Compliance

| Topic | Leaf Document | Density Level |
|-------|--------------|---------------|
| Authentication Patterns | `architecture/AUTH.md` | Comprehensive - OAuth, JWT, SAML, mTLS |
| Authorization Models | `architecture/AUTHZ.md` | Deep - RBAC, ABAC, policy engines |
| Secrets Management | `architecture/SECRETS.md` | Specific - Vault, AWS Secrets Manager, rotation |
| Network Security | `architecture/NETWORK_SECURITY.md` | Comprehensive - mTLS, SPIFFE, zero-trust |
| Encryption Standards | `architecture/ENCRYPTION.md` | Deep - at-rest, in-transit, key management |

### Observability

| Topic | Leaf Document | Density Level |
|-------|--------------|---------------|
| Metrics & Monitoring | `architecture/METRICS.md` | Comprehensive - Prometheus, statsD, alerting |
| Distributed Tracing | `architecture/TRACING.md` | Deep - OpenTelemetry, sampling strategies |
| Logging Patterns | `architecture/LOGGING.md` | Specific - structured logging, log levels, aggregation |
| Alerting & On-Call | `architecture/ALERTING.md` | Comprehensive - SLOs, error budgets, runbooks |

### Reliability & Operations

| Topic | Leaf Document | Density Level |
|-------|--------------|---------------|
| Chaos Engineering | `architecture/CHAOS.md` | Specific - failure injection, game days |
| Disaster Recovery | `architecture/DR.md` | Comprehensive - RPO/RTO, backup strategies |
| Load Balancing | `architecture/LOAD_BALANCING.md` | Deep - algorithms, health checks, failover |
| Rate Limiting | `architecture/RATE_LIMITING.md` | Specific - algorithms, distributed patterns |
| Circuit Breakers | `architecture/CIRCUIT_BREAKERS.md` | Deep - state machines, half-open, bulkheads |

### Deployment & Delivery

| Topic | Leaf Document | Density Level |
|-------|--------------|---------------|
| CI/CD Pipeline Design | `architecture/CI_CD_PIPELINES.md` | Comprehensive - stages, artifacts, gates |
| Deployment Strategies | `architecture/DEPLOYMENTS.md` | Specific - blue-green, canary, rolling |
| GitOps Patterns | `architecture/GITOPS.md` | Deep - ArgoCD, Flux, reconciliation |
| Container Orchestration | `architecture/KUBERNETES.md` | Comprehensive - see above |

### Testing & Quality

| Topic | Leaf Document | Density Level |
|-------|--------------|---------------|
| Testing Strategy | `architecture/TESTING_STRATEGY.md` | Comprehensive - pyramid, types, frameworks |
| Contract Testing | `architecture/CONTRACT_TESTING.md` | Deep - Pact, schema validation |
| Performance Testing | `architecture/PERFORMANCE_TESTING.md` | Specific - load profiles, benchmarks |
| Chaos & Resilience Testing | `architecture/CHAOS_TESTING.md` | Deep - fault injection, game days |

### Frontend & User Experience

| Topic | Leaf Document | Density Level |
|-------|--------------|---------------|
| Frontend Architecture | `architecture/FRONTEND.md` | Comprehensive - React, Vue, state management |
| UI Component Design | `architecture/UI_COMPONENTS.md` | Specific - design systems, accessibility |
| Performance Optimization | `architecture/FE_PERFORMANCE.md` | Deep - Core Web Vitals, lazy loading |

### Architecture & Design

| Topic | Leaf Document | Density Level |
|-------|--------------|---------------|
| Microservices Patterns | `architecture/MICROSERVICES.md` | Comprehensive - decomposition, boundaries |
| Domain-Driven Design | `architecture/DDD.md` | Deep - bounded contexts, aggregates, events |
| Event-Driven Architecture | `architecture/EVENT_DRIVEN.md` | Specific - CQRS, event sourcing, choreography |
| API Gateway Patterns | `architecture/API_GATEWAY.md` | Deep - routing, auth, rate limiting |

---

## Knowledge Base Consumption Pattern

When Decapod surfaces context to an agent for a specific engineering problem:

1. **Query Match**: Decapod matches the problem to relevant knowledge base leaves
2. **Context Carving**: Decapod extracts the specific section needed (not entire documents)
3. **Pre-Inference Payload**: Decapod formats the extracted context with:
   - Exact specifications or code patterns
   - Decision context (when to use this pattern)
   - Tradeoffs and anti-patterns
   - References to related patterns

Example: An agent asking about "how do I handle Kubernetes poddisruptionbudgets" would receive:
- The specific YAML structure with all available fields
- The exact semantics of `minAvailable` vs `maxUnavailable`
- Pod selector constraints and label requirements
- How it interacts with ClusterAutoscaler
- Common failure modes and how to debug them

---

## Density Standards for Leaf Articles

Each leaf article MUST provide:

1. **Exact Specifications**
   - Complete YAML/JSON/Proto schemas where applicable
   - Full HTTP request/response examples
   - Complete code snippets, not fragments

2. **Decision Frameworks**
   - Clear "when to use" criteria with specific thresholds
   - Tradeoff matrices with quantifiable tradeoffs
   - Comparison tables with specific attributes

3. **Production Patterns**
   - Working code/config examples that can be copy-pasted
   - Real-world failure modes with root causes
   - Debugging techniques and diagnostic queries

4. **Anti-Patterns with Specificity**
   - "Don't do X because [specific failure mode]"
   - Concrete examples of what breaks
   - The exact error messages or symptoms

5. **Implementation Breadth**
   - Cover the 80% case thoroughly (most common usage)
   - Document the edge cases explicitly
   - Note platform-specific variations when significant

---

## Cross-Cutting Concerns

These topics span multiple domains and are referenced from multiple leaves:

### Distributed Systems Fundamentals

Key texts:
- `architecture/CONSISTENCY.md` - CAP, PACELC, consensus algorithms
- `architecture/DISTRIBUTED_TRANSACTIONS.md` - 2PC, sagas, outbox patterns
- `architecture/CLOCKS.md` - Logical clocks, vector clocks, distributed ordering

### Error Handling Patterns

Key texts:
- `architecture/ERROR_HANDLING.md` - Retry, backoff, deadline propagation
- `architecture/BULKHEADS.md` - Isolation patterns, resource pools

### Performance Optimization

Key texts:
- `architecture/PERFORMANCE.md` - Profiling, optimization techniques
- `architecture/SCALING.md` - Horizontal vs vertical, sharding

---

## Navigation

- **Start here** for architecture decisions: `architecture/MICROSERVICES.md`
- **Start here** for API design: `architecture/API_DESIGN.md`
- **Start here** for infrastructure: `architecture/KUBERNETES.md`
- **Start here** for security: `architecture/AUTH.md`

---

## Maintaining This Knowledge Base

When updating leaf articles:
1. Ensure all code examples are tested and work out-of-the-box
2. Include version information for all dependencies
3. Document breaking changes explicitly
4. Add migration paths for updating existing systems
5. Mark deprecated patterns with clear upgrade paths

---

## Links

### Core Constitution
- `core/DECAPOD.md` - Core router and navigation charter
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards foundation
- `methodology/ARCHITECTURE.md` - Architecture decision methodology

### Constitution Authority
- `specs/INTENT.md` - Binding methodology contract
- `specs/SYSTEM.md` - System definition and authority doctrine

### Interfaces Layer
- `interfaces/CONTROL_PLANE.md` - Agent sequencing patterns
- `interfaces/TESTING.md` - Testing contract