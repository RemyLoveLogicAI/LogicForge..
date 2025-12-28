# Agent Tasks & Testing Plan

## Comprehensive Task List

1. **Baseline audit & scoping**
   - Identify the primary product goals and non-goals.
   - Catalog critical paths, data flows, and user journeys.
   - Map dependencies (internal modules, external services, third-party libs).

2. **Quality gate definition**
   - Define acceptance criteria per feature (functional + non-functional).
   - Specify "100% system pass" as a blocking quality gate before any tests run.
   - Document required artifacts (coverage report, benchmarks, security scan).

3. **Architecture & risk review**
   - Identify high-risk modules (auth, data integrity, concurrency, I/O).
   - Create failure mode matrix (inputs, outputs, latency, consistency, recoverability).
   - Establish data classification and compliance expectations.

4. **Testing strategy finalization**
   - Specify test taxonomies: unit, integration, contract, E2E, property-based, fuzz, regression, performance, security, accessibility, chaos.
   - Define coverage goals for code, branch, and scenario coverage.

5. **Implementation milestones**
   - Prioritize fixes by risk and user impact.
   - Implement instrumentation needed for observability and testability.
   - Ensure deterministic test environment configuration.

6. **Verification & validation**
   - Run staged test suites after the 100% gate is met.
   - Capture and publish results in a standard, reproducible format.

7. **Post-verification innovation review**
   - Conduct semantic search for alternative or more innovative approaches.
   - Evaluate and document any superior approaches discovered.
   - Decide whether to pivot or iterate on the original plan.

---

## Boundary-Pushing Testing Plan

> **Mandatory gate:** Do not run any tests until the system passes **100%** of preflight checks and readiness criteria.

### A. Preflight / "100% System Pass" Criteria

- All build steps complete successfully with zero warnings.
- All dependency integrity checks pass (lockfile consistency, vulnerability scan baseline).
- All configuration validation checks pass (environment, secrets, schema checks).
- Observability hooks are functional (logging, metrics, tracing present).
- Test data generation deterministic and reproducible.

### B. Comprehensive Coverage Test (One-pass, full-scope)

- **Goal:** Maximum coverage across modules, features, and edge cases in a single coordinated sweep.
- **Scope:**
  - Unit tests for all public functions.
  - Integration tests for every module boundary.
  - Contract tests for external interfaces.
  - Scenario tests for all primary user workflows.
- **Coverage targets:**
  - 100% statement coverage on critical modules.
  - 95%+ branch coverage overall.
  - 100% business rule coverage for core flows.

### C. Complete Suite of Tests (Deep and exhaustive)

1. **Unit tests**
   - Exhaustive path testing, including error branches.
   - Property-based testing for invariants.

2. **Integration tests**
   - Cross-module correctness with realistic data.
   - Boundary and failure injection for network/file I/O.

3. **Contract tests**
   - API schemas verified across all client-server boundaries.
   - Backward compatibility checks.

4. **End-to-end tests**
   - Complete user journeys with real-world workflows.
   - Non-happy path validation (timeouts, cancellation, retries).

5. **Fuzz & adversarial tests**
   - Mutation fuzzing on parsers and user inputs.
   - Protocol-level fuzzing for external interfaces.

6. **Performance & stress tests**
   - Load testing at expected and peak traffic.
   - Soak testing for long-running stability.
   - Regression performance baselines.

7. **Security tests**
   - Static analysis (SAST) and dependency scanning.
   - Dynamic analysis (DAST) on runtime endpoints.
   - Secrets scanning and access control verification.

8. **Chaos & resilience tests**
   - Inject failures (latency, packet loss, partial outages).
   - Validate recovery and circuit-breaking.

9. **Accessibility & compliance tests**
   - Automated accessibility checks.
   - Compliance verification for data handling.

10. **Mutation testing**
   - Evaluate test suite strength by injecting faults.

### D. Reporting & Governance

- Produce a single report bundling:
  - Coverage metrics (statement/branch/function/scenario).
  - Performance baselines and regressions.
  - Security scan results.
  - Accessibility compliance summary.
- Require explicit sign-off before deployment.

---

## Semantic Search Directive (Post-testing)

After the system passes 100% and all tests complete, perform a semantic search across the codebase to identify newer, more innovative, or revolutionary approaches to achieve the defined tasks. Document findings and compare with the current plan.
