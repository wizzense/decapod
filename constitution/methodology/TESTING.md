# TESTING.md - Testing Practice Guide

**Authority:** guidance (testing discipline and execution workflow)
**Layer:** Guides
**Binding:** No
**Scope:** practical testing habits for reliable delivery
**Non-goals:** replacing binding test contracts

---

## Table of Contents

1. [Testing Mission](#1-testing-mission)
2. [The Test Pyramid in Practice](#2-the-test-pyramid-in-practice)
3. [Unit Testing Practices](#3-unit-testing-practices)
4. [Integration Testing Practices](#4-integration-testing-practices)
5. [End-to-End Testing Practices](#5-end-to-end-testing-practices)
6. [Change-Coupled Testing](#6-change-coupled-testing)
7. [Test Quality Guidelines](#7-test-quality-guidelines)
8. [Failure-First Debug Loop](#8-failure-first-debug-loop)
9. [Test Maintenance](#9-test-maintenance)
10. [Evidence and Reporting](#10-evidence-and-reporting)
11. [Anti-Patterns](#11-anti-patterns)
12. [Test Naming Conventions](#12-test-naming-conventions)

---

## 1. Testing Mission

Testing exists to reduce avoidable regressions and accelerate safe iteration.

**Primary outcomes:**
- Fast feedback on intended behavior
- Confidence to refactor
- Clear failure signals for rollbacks

A test suite is not a safety net — it is an executable specification of what the system must do. The following principles define how to build one that is worth trusting.

### 1.1 Core Testing Principles

**Test velocity is delivery velocity.**
You cannot ship faster than you can verify. A slow or flaky test suite directly limits how often code can be merged and deployed. Fast, deterministic tests are the engine of rapid delivery — not optional infrastructure.

**Test invariants, not coverage.**
100% line coverage is a vanity metric. 100% invariant coverage — proving that every documented behavioral guarantee holds — is engineering excellence. Focus test effort on behavior that, if broken, would cause a failure in production.

**Flaky tests are broken tests.**
A test that occasionally fails is worse than no test. It trains engineers to dismiss failure signals. Flaky tests must be quarantined and stabilized on the same timeline as production bugs. They do not belong on the main branch.

**Shift left on all failure modes.**
A bug found in production costs two orders of magnitude more to fix than a bug found locally. Security, performance, and integration failures should be caught as early in the pipeline as possible — ideally before the PR is merged.

**Hard-to-test code is poorly designed code.**
If a component requires extensive mocking infrastructure to unit test, it has too many implicit dependencies. Testing friction is a design signal. Listen to it and decouple before adding the mocking scaffolding.

**Integration coverage over unit volume.**
In distributed and concurrent systems, the majority of real failures occur at boundaries — between services, between async components, between schema and code. The test suite should reflect where failures actually happen, not where they are easiest to write.

**Tests must own their state.**
No test may depend on external mutable state or the execution order of other tests. Every test sets up the state it needs, executes, and tears down cleanly. Shared database state and global mocks are defects in the test design.

**Test names are behavioral specifications.**
A new engineer reading a test file should understand what the component guarantees and what edge cases are explicitly handled. Test names that describe behavior (`returns_empty_list_when_store_is_uninitialized`) are documentation. Test names that describe implementation (`test_init_path_2`) are noise.

### 1.2 Relationship to Binding Contracts

This file is guidance-only. Binding testing requirements live in:
- `interfaces/TESTING.md` — Machine-readable testing interface definitions
- `plugins/VERIFY.md` — Validation subsystem proof surfaces
- `core/INTERFACES.md` — Interface contracts index

---

## 2. The Test Pyramid in Practice

### 2.1 Pyramid Structure

```
           ┌─────────────────────────┐
           │                         │
           │      E2E Tests          │  ← Few, slow, high confidence
           │   (Critical journeys)   │
           │                         │
           ├─────────────────────────┤
           │                         │
           │   Integration Tests     │  ← Medium count, medium speed
           │  (Component boundaries) │
           │                         │
           ├─────────────────────────┤
           │                         │
           │      Unit Tests         │  ← Many, fast, isolated
           │  (Local behavior)       │
           │                         │
           └─────────────────────────┘
```

### 2.2 Default Emphasis

1. **Unit tests** for local behavior and edge cases
2. **Service/component tests** for boundaries and integration seams
3. **End-to-end tests** for critical user journeys only

**Avoid over-indexing on slow E2E suites** when cheaper lower-level proof can catch the same class of failures.

### 2.3 When to Add Tests at Each Level

| Test Level | When to Add | Example |
|------------|-------------|---------|
| **Unit** | Testing isolated logic, edge cases, algorithm correctness | "Does this function handle null inputs correctly?" |
| **Integration** | Testing component interactions, API contracts, data flow | "Does the store correctly persist and retrieve?" |
| **E2E** | Testing critical user journeys, full system correctness | "Can user complete checkout end-to-end?" |

---

## 3. Unit Testing Practices

### 3.1 What Makes a Good Unit Test

A good unit test has these properties:
- **Fast**: Runs in milliseconds
- **Isolated**: No dependencies on external systems or other tests
- **Deterministic**: Same result every time
- **Readable**: Test name describes the behavior being tested
- **Maintainable**: Easy to update when requirements change

### 3.2 Unit Test Structure (Arrange-Act-Assert)

```rust
#[test]
fn returns_err_when_store_is_uninitialized() {
    // Arrange: Set up the test fixture
    let store = UninitializedStore::new();
    let expected_error = StoreError::NotInitialized;

    // Act: Execute the behavior under test
    let result = store.get(key);

    // Assert: Verify the expected outcome
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), expected_error);
}
```

### 3.3 What to Test in Units

**Test behaviors, not implementation:**
- Public method contracts
- Edge cases and error conditions
- Boundary conditions (empty, full, one item)
- Invalid inputs
- State transitions

**Do not test:**
- Private implementation details
- Framework behavior
- Trivial code (getters/setters with no logic)

### 3.4 Common Unit Test Mistakes

**Testing implementation instead of behavior:**
```rust
// BAD: Tests implementation
#[test]
fn test_internal_counter_increments() {
    let sut = Counter::new();
    assert_eq!(sut.count, 0);
    sut.increment();
    assert_eq!(sut.count, 1); // Tests internal state
}

// GOOD: Tests behavior
#[test]
fn incrementing_returns_next_count() {
    let sut = Counter::new();
    assert_eq!(sut.next(), 0);
    assert_eq!(sut.next(), 1); // Tests observable behavior
}
```

---

## 4. Integration Testing Practices

### 4.1 What Makes a Good Integration Test

A good integration test:
- **Tests component boundaries**: Verifies components work together
- **Uses real dependencies**: Where practical, use real implementations
- **Isolates from external systems**: Uses test doubles for external services
- **Is deterministic**: Same result every time
- **Covers contract compliance**: Verifies API contracts are honored

### 4.2 Integration Test Scope

Integration tests typically verify:
- Database operations (CRUD, migrations, transactions)
- API calls between services
- Message queue publishing and consumption
- File system operations
- Authentication and authorization flows

### 4.3 Test Fixtures and Setup

Use shared fixtures for expensive setup:

```rust
// Shared test database for integration tests
pub struct TestDatabase {
    connection: TestConnection,
}

impl TestDatabase {
    pub fn new() -> Self {
        let connection = TestConnection::in_memory();
        run_migrations(&connection);
        TestDatabase { connection }
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}
```

### 4.4 Contract Testing

When services communicate, verify contract compliance:

```rust
#[test]
fn store_api_returns_correct_json_schema() {
    let store = create_test_store();
    let result = store.get_json(key);
    
    // Verify schema compliance
    assert_valid_schema(&result, " StoreResponse");
}
```

---

## 5. End-to-End Testing Practices

### 5.1 When to Write E2E Tests

E2E tests are appropriate when:
- Testing critical user journeys (checkout, signup, login)
- Verifying system integration in production-like environment
- Testing security-critical paths
- Validating regulatory compliance

**E2E tests are expensive.** Only write E2E tests when lower-level tests cannot catch the same failures.

### 5.2 E2E Test Design Principles

1. **Minimize the surface area**: Only critical paths, not every possible flow
2. **Use realistic data**: Test with data that mirrors production
3. **Isolate tests**: Each E2E test should be independent
4. **Keep tests focused**: One assertion per test is often appropriate
5. **Maintain the suite**: E2E tests rot quickly if not maintained

### 5.3 E2E Test Example

```rust
#[test]
fn user_can_complete_checkout_with_valid_payment() {
    // Launch browser/app in test environment
    let browser = Browser::new_test_browser();
    let mut context = browser.new_context();
    
    // Add items to cart
    let page = context.new_page();
    page.goto("/products/widget");
    page.click("#add-to-cart");
    
    // Proceed to checkout
    page.click("#checkout");
    page.fill("#card-number", TEST_CARD);
    page.fill("#expiry", "12/28");
    page.fill("#cvv", "123");
    
    // Complete purchase
    page.click("#pay-now");
    
    // Verify success
    assert!(page.is_visible("#order-confirmation"));
    assert!(page.text_content("#order-number").starts_with("ORD-"));
}
```

---

## 6. Change-Coupled Testing

### 6.1 The Change-Coupled Testing Rule

For each code change, ask:
1. What behavior changed?
2. Which invariant might regress?
3. What is the smallest test that fails when regression appears?

**Ship only when at least one changed behavior is covered by a falsifiable check.**

### 6.2 Change Impact Analysis

Before writing tests, analyze what your change affects:

```
Code Change: Modify store.get() to return cached values

Impact Analysis:
├── What changed: get() behavior (cache lookup before DB)
├── Invariants at risk:
│   ├── Same value returned for same key
│   ├── Cache invalidation on update
│   └── Stale data prevention
└── Tests needed:
    ├── returns_cached_value_when_available
    ├── falls_back_to_db_when_cache_miss
    ├── invalidates_cache_on_update
    └── returns_fresh_after_invalidation
```

### 6.3 Minimal Test Set

Write the minimum tests that would catch regressions:

| Change Type | Minimum Test |
|-------------|-------------|
| Add new feature | Happy path, error path, edge cases |
| Modify existing feature | Old behavior regression, new behavior verification |
| Performance change | Baseline performance test |
| Security change | Security test for the vulnerability |
| Refactoring | Same tests as before (behavior should not change) |

---

## 7. Test Quality Guidelines

### 7.1 Test Completeness Checklist

Before considering a feature tested:

- [ ] Happy path works
- [ ] Error paths handled correctly
- [ ] Edge cases covered (empty, one item, many items)
- [ ] Invalid inputs rejected with clear errors
- [ ] Concurrent access handled correctly
- [ ] Performance acceptable under load
- [ ] Security requirements met
- [ ] Integration points tested

### 7.2 Test Readability Guidelines

**Good test names:**
- `validates_card_number_using_luhn_algorithm`
- `rejects_negative_quantities`
- `returns_err_when_item_not_found`
- `notifies_observers_on_state_change`

**Bad test names:**
- `test1`
- `test_card`
- `check_valid`
- `handle_error_case`

### 7.3 Test Isolation Rules

1. **No shared mutable state** between tests
2. **No dependency on test execution order**
3. **No external network calls** in unit tests
4. **No file system operations** in unit tests (use test doubles)
5. **Each test sets up its own fixtures**

---

## 8. Failure-First Debug Loop

### 8.1 The Failure-First Principle

When a test fails:
1. **Reproduce deterministically** — Ensure the failure is consistent
2. **Minimize input to isolate fault** — Find the smallest failing case
3. **Fix root cause, not assertion symptom** — Don't just make the test pass
4. **Re-run closest tests first, then broaden** — Test the affected code first

### 8.2 Debugging Steps

```bash
# Step 1: Run the failing test in isolation
cargo test failing_test_name -- --nocapture

# Step 2: Verify the test fails consistently
cargo test failing_test_name -- --test-threads=1

# Step 3: Run tests in the same file
cargo test --package <package> --lib <module>

# Step 4: Run the broader test suite
cargo test --package <package>

# Step 5: Run validation to check doc compatibility
decapod validate
```

### 8.3 Common Failure Modes

| Failure Type | Common Cause | Fix |
|--------------|-------------|-----|
| Flaky test | Race condition, timing dependency | Isolate, add retry logic, fix root cause |
| Wrong assertion | Test doesn't match expected behavior | Fix test or fix code |
| Missing setup | Fixture not initialized | Add arrange step |
| External dependency | Network, database not available | Mock or provide test environment |
| Mutation sharing | Tests pollute shared state | Reset state between tests |

---

## 9. Test Maintenance

### 9.1 When to Update Tests

Update tests when:
- Requirements change
- Bug fixes require test updates
- Code refactoring changes behavior (intentionally)
- Tests are flaky or brittle
- New edge cases are discovered

Do not update tests when:
- Refactoring preserves behavior (tests should pass unchanged)
- Tests are correct and code is wrong

### 9.2 Test Debt

Test debt accumulates when:
- Tests are commented out
- Tests are marked `#[ignore]`
- Flaky tests are normalized
- New features ship without tests

**Treat test debt like technical debt.** Allocate time to address it.

### 9.3 Test Review Checklist

When reviewing tests:
- [ ] Test names describe behavior, not implementation
- [ ] Each test has one assertion focus
- [ ] Edge cases are covered
- [ ] Error cases are tested
- [ ] No shared mutable state
- [ ] Tests are deterministic
- [ ] No unnecessary mocking
- [ ] Fixtures are reusable and clear

---

## 10. Evidence and Reporting

### 10.1 Proof Reporting Requirements

For every test run, capture:
- Command executed
- Pass/fail status
- Scope covered (which tests ran)
- Known gaps (what is not covered)

### 10.2 Evidence Format

```markdown
## Test Evidence

**Command:** `cargo test --package decapod --lib`

**Results:**
- Total: 142 tests
- Passed: 140
- Failed: 2
- Skipped: 0

**Failures:**
1. `test_store_returns_err_when_uninitialized` - FAILED
   - Error: assert_eq failed: expected StoreError::NotInitialized, got NotFound
   - Root cause: Incorrect error type in error handling path

2. `test_cache_invalidates_on_update` - FAILED  
   - Error: Assertion failed: cache.get(key) == value (got stale)
   - Root cause: Invalidation not triggered in concurrent update path

**Coverage:**
- Unit tests: 95% line coverage
- Integration tests: 12 tests covering store API
- E2E tests: 4 critical journeys

**Gaps:**
- No concurrent access tests for store
- No tests for partial network failure recovery
```

### 10.3 When Proof Cannot Run

When proof cannot run, state this explicitly:

```markdown
## Test Evidence: UNABLE TO RUN

**Blocker:** Test environment unavailable (database connection timeout)

**Workarounds attempted:**
- Verified code compiles: YES
- Ran unit tests locally: YES (all passed)
- Ran integration tests: BLOCKED (requires DB)

**Mitigation:**
- Manual code review completed
- Additional logging added to trace execution
- Scheduled follow-up run for [DATE]
```

---

## 11. Anti-Patterns

### 11.1 Test Anti-Patterns

**The Slow Test Suite**
- Tests that hit the database, network, or file system unnecessarily
- Tests that don't clean up after themselves
- Tests that run sequentially when they could run in parallel

**The Brittle Test**
- Tests that break when implementation changes but behavior doesn't
- Tests that check internal state instead of observable behavior
- Tests with hard-coded dates, UUIDs, or other volatile data

**The Mock Overload**
- So many mocks that the test doesn't test anything real
- Mocks that don't reflect actual dependency behavior
- Mock setup that's longer than the test itself

**The God Test**
- One test that tries to test everything
- Tests with 50 assertions
- Tests that require a PhD to understand

**The Copy-Paste Test**
- Duplicated test code with minor variations
- Tests that don't follow DRY principles
- Same assertion logic repeated 20 times

### 11.2 How to Fix Anti-Patterns

| Anti-Pattern | Fix |
|--------------|-----|
| Slow suite | Move to proper level (unit vs integration), parallelize |
| Brittle tests | Test behavior, not implementation; use test factories |
| Mock overload | Redesign for testability; reduce coupling |
| God test | Split into focused tests |
| Copy-paste tests | Extract shared helper functions, use parameterized tests |

---

## 12. Test Naming Conventions

### 12.1 Naming Pattern

Use the pattern: `<subject>_<condition>_<expected_result>`

**Examples:**
- `store_returns_err_when_key_not_found`
- `cache_invalidates_on_delete`
- `payment_rejects_expired_card`
- `user_authentication_succeeds_with_valid_credentials`

### 12.2 Consistency

Be consistent within your codebase. If one test file uses `returns_err_when`, don't use `err_returns_when` in another.

### 12.3 Documentation Names

For tests that document behavior:
- `does_not_panic_on_null_input`
- `handles_concurrent_access_safely`
- `preserves_order_of_messages`

---

## Links

### Core Router
- [core/DECAPOD.md](../../core/DECAPOD.md) - **Router and navigation charter (START HERE)**
- [core/ENGINEERING_EXCELLENCE.md](../../core/ENGINEERING_EXCELLENCE.md) - **Oracle for Engineering Standards**
- [core/GAPS.md](../../core/GAPS.md) - Gap analysis methodology

### Authority (Constitution Layer)
- [specs/INTENT.md](../specs/INTENT.md) - **Methodology contract (READ FIRST)**
- [specs/SYSTEM.md](../specs/SYSTEM.md) - System definition and authority doctrine
- [specs/SECURITY.md](../specs/SECURITY.md) - Security contract

### Registry (Core Indices)
- [core/PLUGINS.md](../../core/PLUGINS.md) - Subsystem registry
- [core/METHODOLOGY.md](../../core/METHODOLOGY.md) - Methodology guides index
- [core/INTERFACES.md](../../core/INTERFACES.md) - Interface contracts index

### Contracts (Interfaces Layer)
- [interfaces/TESTING.md](../../interfaces/TESTING.md) - **Testing contract (BINDING)**
- [interfaces/CONTROL_PLANE.md](../../interfaces/CONTROL_PLANE.md) - Sequencing patterns
- [interfaces/CLAIMS.md](../../interfaces/CLAIMS.md) - Promises ledger
- [interfaces/GLOSSARY.md](../../interfaces/GLOSSARY.md) - Term definitions

### Practice (Methodology Layer - This Document)
- [methodology/ARCHITECTURE.md](./ARCHITECTURE.md) - Architecture practice
- [methodology/SOUL.md](./SOUL.md) - Agent identity
- [methodology/KNOWLEDGE.md](./KNOWLEDGE.md) - Knowledge curation
- [methodology/MEMORY.md](./MEMORY.md) - Memory and learning
- [methodology/CI_CD.md](./CI_CD.md) - CI/CD practice

### Architecture
- [architecture/TESTING_STRATEGY.md](../../architecture/TESTING_STRATEGY.md) - Testing strategy patterns

### Operations (Plugins Layer)
- [plugins/TODO.md](../plugins/TODO.md) - Work tracking
- [plugins/VERIFY.md](../plugins/VERIFY.md) - **Validation subsystem (PROOF SURFACES)**