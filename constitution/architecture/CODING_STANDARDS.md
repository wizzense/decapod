# CODING_STANDARDS.md - Industry Coding Standards Reference

**Authority:** constitution (multi-level coding principles and patterns)
**Layer:** Architecture
**Binding:** mixed (see per-principle designation)
**Scope:** coding and architectural standards drawn from canonical industry references

This document codifies binding and advisory engineering principles drawn from canonical industry texts. Principles marked **BINDING** must be followed unless an explicit `.decapod/OVERRIDE.md` entry documents the deviation and its justification. Principles marked **ADVISORY** are strongly recommended defaults that apply unless user intent explicitly indicates otherwise.

---

## 1. Uncle Bob Martin (Clean Code / SOLID)

*Source: "Clean Code" (2008), "Agile Software Development" (2003)*

### 1.1 SOLID Principles (BINDING for public APIs and shared libraries)

| Principle | Description | When Applicable |
|-----------|-------------|-----------------|
| **S**ingle Responsibility | A module should have one, and only one, reason to change. | All modules and classes |
| **O**pen/Closed | Open for extension, closed for modification. | Public APIs, library code |
| **L**iskov Substitution | Objects should be replaceable with subtypes without altering correctness. | Type hierarchies |
| **I**nterface Segregation | Prefer small, client-specific interfaces over large general-purpose ones. | Public API design |
| **D**ependency Inversion | Depend on abstractions, not concretions. | Module coupling |

### 1.2 Clean Code Guidelines (ADVISORY)

- **Meaningful names:** Variables, functions, and classes must reveal intent. If the name requires a comment to explain, rename it.
- **Functions should be small and do one thing:** If a function does multiple things, break it apart.
- **Comments should explain *why*, not *what*:** Code that needs comments to explain what it does is poorly written.
- **Error handling is one thing:** Functions that handle errors should not do anything else.
- **Prefer exceptions over error codes:** Clean, localized error propagation.
- **Don't return null:** Null object pattern or Optional instead of null returns.

**Exception to binding:** SOL ID principles are **ADVISORY** for:
-throwaway scripts, prototypes, and one-off automation
- Code under active initial development (< 24h old) where API surfaces are not yet stabilized
- Explicit user direction to prioritize velocity over structure

---

## 2. Martin Fowler (Refactoring / Patterns)

*Source: "Refactoring" (1999, 2018), "Patterns of Enterprise Application Architecture" (2002), "Enterprise Integration Patterns" (2003)*

### 2.1 Refactoring Principles (ADVISORY)

- **Refactor before adding features:** If you need to add a feature to a system that is not nicely structured, refactor first.
- **Small refactorings, frequently applied:** Continuous refactoring prevents accumulation of technical debt.
- **Never refactor and add features simultaneously:** Separate commits for refactoring vs. functional changes.
- **Maintain tests during refactoring:** Tests are the safety net that makes refactoring safe.

### 2.2 Key Patterns (ADVISORY for architecture, BINDING for consistency)

| Pattern | Use Case | Binding Level |
|---------|----------|---------------|
| Strategy | Varying algorithms selectable at runtime | ADVISORY |
| Observer | Event propagation to dependents | ADVISORY |
| Composite | Tree structures treated uniformly | ADVISORY |
| Decorator | Attach responsibilities dynamically | ADVISORY |
| Factory | Object creation abstraction | ADVISORY |
| Repository | Collection-oriented data access | ADVISORY |
| Unit of Work | Atomic state changes | ADVISORY |
| Lazy Load | Defer expensive object creation | ADVISORY |

**Exception:** When the codebase already uses a pattern consistently, continue that pattern. Mixing equivalent patterns without cause is a violation.

---

## 3. Pragmatic Programmer (Pragmatic Engineering)

*Source: "The Pragmatic Programmer" (1999, 2020) - Hunt & Thomas*

### 3.1 Core Tips (BINDING for critical workflows)

| Tip | Principle | Applicability |
|-----|-----------|---------------|
| **Tip 1: Don't Repeat Yourself** | Every piece of knowledge must have a single, authoritative representation. | BINDING - see Section 5 |
| **Tip 2: Orthogonality** | Design components that are independent; changes don't propagate. | BINDING for architecture |
| **Tip 3: Traceability** | Good enough architecture; tracer bullets over big upfront design. | ADVISORY |
| **Tip 4: Prototype** | Prototype to learn; throw away prototype code, not production instincts. | ADVISORY |
| **Tip 5: Property-Based Testing** | Test invariants, not just examples. | ADVISORY |
| **Tip 6: Domain Languages** | Build languages suited to the domain. | ADVISORY |
| **Tip 7: Mindful Programming** | Program deliberately, not by accident or coincidence. | BINDING for reviewed code |
| **Tip 8: Elegance** | Simple, expressive, minimal. Avoid clever clever. | ADVISORY |
| **Tip 9: Automate** | Automate repetitive tasks. | BINDING for CI/CD |
| **Tip 10: Debugging** | Fix the symptom, not the cause. Find root causes. | BINDING for bug fixes |

### 3.2 Orthogonality (BINDING for system design)

- Changes in one component should not affect others.
- Each module should be independent: know nothing of other modules' internals.
- Orthogonal systems are easier to test, debug, and extend.

---

## 4. Gang of Four (Design Patterns)

*Source: "Design Patterns: Elements of Reusable Object-Oriented Software" (1994) - Gamma, Helm, Johnson, Vlissides*

### 4.1 Creational Patterns (ADVISORY)

| Pattern | Intent | When to Use |
|---------|--------|--------------|
| Abstract Factory | Create families of related objects | When system should be independent of creation |
| Builder | Construct complex objects step by step | When construction involves multiple steps |
| Factory Method | Defer instantiation to subclasses | When class doesn't know which subclass to create |
| Prototype | Clone pre-existing objects | When instantiation is expensive |
| Singleton | Single global instance | When exactly one instance is needed (use sparingly) |

### 4.2 Structural Patterns (ADVISORY)

| Pattern | Intent | When to Use |
|---------|--------|--------------|
| Adapter | Convert interface to another | When integrating incompatible interfaces |
| Bridge | Decouple abstraction from implementation | When both may vary independently |
| Composite | Treat individual and compositions uniformly | When tree structures appear |
| Decorator | Attach responsibilities dynamically | When extension via subclassing is impractical |
| Facade | Simple unified interface to subsystem | When simplifying complex subsystem usage |
| Flyweight | Share common state | When many objects share state |
| Proxy | Placeholder for another object | When lazy initialization or access control needed |

### 4.3 Behavioral Patterns (ADVISORY)

| Pattern | Intent | When to Use |
|---------|--------|--------------|
| Chain of Responsibility | Pass request along chain until handled | When multiple handlers possible |
| Command | Encapsulate request as object | When undo/redo needed, or queuing |
| Iterator | Access elements sequentially | When abstraction over collection needed |
| Mediator | Centralized communication | When direct communication causes coupling |
| Memento | Capture and restore state | When snapshot/restore needed |
| Observer | Notify dependents of state change | When change propagation needed |
| State | Alter behavior when state changes | When behavior depends on state |
| Strategy | Vary algorithm at runtime | When multiple algorithms possible |
| Template Method | Define skeleton, defer steps | When invariance exists across subclasses |
| Visitor | Separate algorithm from object structure | When operations on mixed types needed |

**Binding note:** GoF patterns are ADVISORY. However, once a pattern is adopted in a codebase, consistency is BINDING - do not reimplement equivalent functionality with a different pattern without cause.

---

## 5. Don't Repeat Yourself (DRY)

*Source: "The Pragmatic Programmer" - Hunt & Thomas*

### 5.1 Definition (BINDING)

**DRY Principle:** Every piece of knowledge must have a single, unambiguous, authoritative representation within a system.

### 5.2 Violations

| Violation | Anti-pattern | Remedy |
|-----------|--------------|--------|
| Copy-paste code | Identical logic in multiple places | Extract to function/module |
| Shared knowledge | Same information encoded in multiple places | Single source of truth |
| Schema duplication | DB schema and code types drift | Generate from single source |
| Documentation drift | Comments don't match code | Comments explain *why*, code is authoritative |
| Configuration scatter | Same config in multiple places | Centralize configuration |

### 5.3 Exceptions (ADVISORY - allowed when documented)

- Intentional denormalization for performance (documented in code)
- Bridging between incompatible abstractions (documented rationale)
- Test fixtures that must remain independent (isolation requirement)
- `.decapod/OVERRIDE.md` entries that override DRY for specific contexts

---

## 6. Unix Philosophy

*Source: "The Art of Unix Programming" (2003) - Eric Raymond*

### 6.1 Core Principles (BINDING for system design, ADVISORY for application code)

| Principle | Description | Applicability |
|-----------|-------------|---------------|
| **Do One Thing Well** | Each program should do one thing and do it completely. | BINDING for CLI tools, system utilities |
| **Composability** | Programs should communicate via clean interfaces (stdin/stdout, files, pipes). | BINDING for CLI tools |
| **Small is Beautiful** | Write programs that do one thing, and do it well. Prefer smaller components. | ADVISORY for application architecture |
| **Data Transformation** | Programs should read from stdin, transform, write to stdout. | BINDING for new CLI utilities |
| **Text Stream Interface** | Use text (not binary) for universal interface. | ADVISORY, BINDING for public APIs |
| **Reuse Programs** | Build on existing programs rather than reinvent. | ADVISORY |
| **Silence is Golden** | Only produce output that matters. | ADVISORY |
| **Optimization** | Profile before optimizing. Make it work, then make it fast. | ADVISORY |

### 6.2 Application to Decapod

For Decapod's architecture:
- Each CLI command should perform one logical operation
- Internal modules should be composable and testable independently
- Workspace isolation enables Unix-style pipeline thinking across the tool suite

---

## 7. Standards Interaction Matrix

| Standard | Binding When | Advisory When |
|----------|--------------|---------------|
| Uncle Bob Martin (SOLID) | Public APIs, shared libraries | Prototypes, throwaway code |
| Martin Fowler (Patterns) | Consistency within codebase | Greenfield design |
| Pragmatic Engineering | CI/CD automation, bug fixes | Early-stage development |
| Gang of Four | Consistency after adoption | Initial design decisions |
| DRY | All production code | Explicitly documented exceptions |
| Unix Philosophy | CLI tools, system utilities | Application business logic |

---

## Links

### Core Router
- `core/DECAPOD.md` - Router and navigation charter (START HERE)
- `core/ENGINEERING_EXCELLENCE.md` - Engineering quality standards

### Practice (Methodology Layer)
- `methodology/ARCHITECTURE.md` - Architecture practice
- `methodology/TESTING.md` - Testing practice

### Architecture Patterns
- `architecture/ALGORITHMS.md` - Algorithm selection
- `architecture/API_DESIGN.md` - API design standards
- `architecture/CONCURRENCY.md` - Concurrency architecture