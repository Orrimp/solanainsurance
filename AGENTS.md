### Context

## Use README.md file to understand the project. 

# AI Agent Development Guidelines

This document outlines key principles and best practices for developing AI agents that interact with this Rust codebase. Adhering to these guidelines will improve agent effectiveness, code quality, and overall project maintainability.

## Core Principle: Design for AI

The fundamental principle is to design APIs and code that are easy for both humans and AI agents to understand and use. Rust's strong type system is a significant advantage, as it allows the compiler to catch errors that might arise from an agent's incomplete understanding of the code.

To make the codebase more AI-friendly, please follow these specific guidelines:

### 1. Follow Idiomatic Rust Patterns

- **Consistency is Key**: Ensure that all APIs, whether public or internal, follow standard Rust idioms and conventions.
- **Adhere to Rust API Guidelines**: Follow the official [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html) to maintain a consistent and predictable codebase.
- **Document Framework Decisions**: Clearly indicate whether code uses native Solana, Anchor, or other frameworks to avoid confusion during refactoring.

### 2. Provide Comprehensive Documentation

- **Document Everything**: All public items, including modules, functions, and types, must be thoroughly documented.
- **Assume a Knowledgeable Reader**: Write documentation with the assumption that the reader is familiar with Rust and its standard library but may not be an expert in this specific codebase.
- **Include Essential Sections**: Documentation should include failure conditions ([C-FAILURE](https://rust-lang.github.io/api-guidelines/checklist.html#c-failure)), links to related items ([C-LINK](https://rust-lang.github.io/api-guidelines/checklist.html#c-link)), and canonical sections as described in our internal guidelines.
- **Account Requirements**: For Solana programs, document expected accounts, their mutability, and ownership requirements in instruction handlers.

### 3. Include Practical Examples

- **Show, Don't Just Tell**: Provide clear and usable code examples within the documentation.
- **Elaborate in the Repository**: For more complex scenarios, include detailed examples in the project's repository.
- **Use `?` in Examples**: Follow the convention of using the `?` operator in examples to demonstrate idiomatic error handling ([C-QUESTION-MARK](https://rust-lang.github.io/api-guidelines/checklist.html#c-question-mark)).
- **Test-Driven Examples**: Include examples that can be run as tests to ensure they remain accurate.

### 4. Use Strong, Expressive Types

- **Avoid Primitive Obsession**: Instead of using primitive types like `String` or `u64` for domain-specific concepts, create new types with well-defined semantics. This helps prevent incorrect usage.
- **Leverage the Newtype Pattern**: Use the newtype pattern ([C-NEWTYPE](https://rust-lang.github.io/api-guidelines/checklist.html#c-newtype)) to wrap primitive types and enforce invariants at the type level.
- **Solana-Specific Types**: Consider creating domain types like `Lamports(u64)`, `UnixTimestamp(i64)`, or `AccountSize(usize)` for Solana programs.

### 5. Design for Testability

- **Enable Unit Testing**: APIs should be designed in a way that allows for easy unit testing. This may involve providing mocks, fakes, or using Cargo features to enable test-specific functionality.
- **Facilitate Rapid Iteration**: A testable design allows AI agents to quickly verify that their generated code is correct and functions as expected.

### 6. Ensure Thorough Test Coverage

- **Test Observable Behavior**: The codebase should have a high level of test coverage, focusing on the observable behavior of the system.
- **Enable Safe Refactoring**: Good test coverage gives agents the confidence to perform refactoring tasks with minimal human supervision, knowing that the tests will catch any regressions.

## AI Agent-Specific Guidelines (Written by AI)

### 7. Maintain Dependency Clarity

- **Document Framework Choices**: Clearly document in README or comments whether the project uses Anchor, native Solana, or hybrid approaches.
- **Consistent Dependencies**: Ensure `Cargo.toml` dependencies match the code's actual framework usage to avoid confusion.
- **Migration Paths**: When changing frameworks, document the migration strategy and intermediate states.

### 8. Provide Context for Large Changes

- **Strategy Documentation**: For major refactoring (like Anchor to native conversion), document the reasoning and approach.
- **Progress Tracking**: Use clear file organization and naming to help agents understand conversion states.
- **Reference Implementation**: Keep working examples or reference implementations accessible during large changes.

### 9. Error Handling Patterns

- **Consistent Error Types**: Use consistent error handling patterns throughout the codebase (`thiserror` for custom errors, proper error propagation).
- **Domain-Specific Errors**: Create meaningful error types that clearly indicate what went wrong and why.
- **Error Context**: Include sufficient context in error messages to help with debugging.

### 10. File Organization Standards

- **Follow Official Patterns**: For Solana programs, follow the official file structure:
  - `entrypoint.rs`: Program entry point and instruction routing
  - `state.rs`: Account data structures  
  - `instructions.rs`: Instruction definitions
  - `processor.rs`: Instruction handler implementations
  - `errors.rs`: Custom error types
- **Separation of Concerns**: Keep business logic separate from framework-specific code.
- **Module Documentation**: Document the purpose and scope of each module at the file level.

### 11. Modular Program & Client Structuring (New Learnings)

This project evolved to separate on-chain logic from client orchestration in a clean, self-descriptive way. These patterns should be treated as baseline architecture going forward.

#### Layered Architecture
1. On-chain core (program crate)
   - `entrypoint.rs` routes to `processor.rs`.
   - `instructions.rs` defines the canonical instruction enum (command layer).
   - `state.rs` defines serialized account data models.
   - `errors.rs` contains domain error types.
2. Off-chain SDK / Client helpers (example or future `sdk` feature)
   - `client/instructions.rs`: pure instruction builders (no side effects, no keypairs).
   - `client/sender.rs`: transaction assembly & submission (imperative layer).
   - `client/solana_ctx.rs`: environment, RPC client factory, funding utilities.
   - `client/client.rs`: orchestration / demo flow (compose builders + send). 
3. Optional façade (future): `InsuranceClient` struct with high-level domain methods (`initialize_pensioner`, `mark_deceased`, `due_payment_view`).

This separation minimizes coupling and makes each layer easier for agents to reason about: build instructions → send them → observe state.

#### Instruction Builder Pattern
- Builders must be deterministic, side-effect free, and reflect processor account ordering exactly.
- Never inline account meta assembly at call sites; centralize in `client/instructions.rs`.
- Serialize using a single agreed format (currently Borsh). If multi-language support is required, add an IDL JSON export.

#### REST-Like Semantics Mapping
| REST Concept | Solana Mapping |
|--------------|----------------|
| POST /resource | Transaction with instruction(s) |
| GET /resource/{id} | RPC getAccountData + deserialize |
| GET /resource/{id}/computed | Local read + deterministic computation (avoid on-chain cost unless auditing) |

Commands mutate; queries read. Keep them logically distinct—do not design instructions solely for information retrieval unless needed for verifiability/logging.

#### Deterministic Account Addresses (PDAs)
Adopt PDAs for discoverability & idempotency:

```rust
// PDA derivation example (documentation only)
let (pension_pda, bump) = Pubkey::find_program_address(&[b"pension", pensioner_pubkey.as_ref()], program_id);
```
Guidelines:
- Prefer PDAs over random Keypairs for state accounts.
- Include seeds documentation in `instructions.rs` and `state.rs`.
- Use `invoke_signed` in processor when creating PDA-owned accounts.

#### Versioning & Forward Compatibility
- Stabilize instruction discriminants: use explicit `u8` tags or enum variants with version suffixes (`InitializePensionerV1`).
- Reserve extension bytes in state (`reserved: [u8; N]`) for future flags to avoid migrations.
- Document upgrade strategy (add new variants; keep old ones available until deprecation window ends).

#### Error Mapping Strategy
- Domain errors (`PensionError`) must be exhaustive and stable.
- Client layer should translate `ProgramError` + logs → domain errors.
- For multi-instruction transactions, surface which index failed.

#### Transaction Assembly
Rules:
- Payer first; additional signers appended uniquely.
- Reuse `send_instructions` helper for consistency (no ad-hoc signing code elsewhere).
- Batch logically related instructions (e.g., create + initialize) while keeping failure boundaries clear.

#### Observability & Logging
- Use structured log prefixes: `msg!("[insurance][initialize] pensioner={} payment={}", pensioner_pubkey, monthly_payment);`
- Provide compute unit logging behind a feature (`profile-cu`) for profiling heavy flows.

#### Testing Pyramid
1. Unit: pure functions (instruction builders, state helpers).
2. Program logic: processor tests with LiteSVM (fast, deterministic).
3. Integration: example client flows against local validator (full stack). 
4. Property tests (optional): invariants (e.g., payment calculation monotonicity).

#### Deprecation Management
- Track SDK deprecations (e.g., `system_program` → `solana_system_interface::program::ID`).
- Introduce features (`legacy-sdk`) to toggle legacy paths without invasive diffs.
- Periodic dependency review (`cargo update -p <crate>` + clippy) to surface new warnings.

#### Security & Safety Considerations
- Validate account ownership before mutation (`pension_account.owner == program_id`).
- Enforce signer checks early; fail fast.
- Idempotent design: repeated initialize on existing PDA either no-op or returns a domain error—decide & document.
- Consider authority transfer instruction with two-phase confirmation for production.

#### Clean Naming Conventions
- Use verb_noun for instruction builders: `initialize_pensioner`, `mark_deceased`.
- Include units in field names (`monthly_payment_lamports`).
- Favor domain newtypes later (`Lamports(u64)`, `Months(u64)`), to prevent parameter mix-ups.

#### Migration Path to Dedicated SDK Crate
Steps:
1. Create `sdk/` with `Cargo.toml` depending on program crate via path.
2. Move `instructions.rs` & `sender.rs` as public API (re-export types from program crate).
3. Add feature flags for optional utilities (airdrop, profiling).
4. Publish internal docs & IDL for external integrators.

#### Agent Implementation Checklist (When Adding Features)
- [ ] Define/extend instruction enum with explicit docs.
- [ ] Add builder in `client/instructions.rs` (or SDK crate).
- [ ] Update processor with validation & domain errors.
- [ ] Add unit + LiteSVM tests for new flow.
- [ ] Update AGENTS.md and README with usage example.
- [ ] Run clippy & ensure no new warnings beyond accepted deprecations.

#### Example High-Level Flow (Pseudocode)
```rust
// High-level client usage (pseudocode)
let client = InsuranceClient::new(rpc_url, program_id, authority_keypair);
client.initialize_pensioner(pensioner_pubkey, Lamports(100_000_000))?;
let due = client.view_due_payment(pensioner_pubkey)?; // local compute
client.mark_deceased(pensioner_pubkey)?;
```

---
**Summary:** Decoupling instruction construction, transaction sending, and environment setup produced a clearer, more maintainable interface akin to a RESTful service architecture. Future agents MUST follow this separation to keep cognitive load low and reliability high.
