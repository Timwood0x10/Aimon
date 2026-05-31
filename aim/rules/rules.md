# rust code standars and best practices

1. The code length of a single file should not exceed 1000 lines (including comments and test cases). Files exceeding this limit can be designed as a module. Ideally, it should be between 500 and 1000 lines for best readability.
2. Acceptance criterion: `make check` should return 0 errors.
3. After each modification, ensure `make fmt` is executed.
4. Warnings displayed by `make check` can be ignored initially.
5. When fixing warnings, do not use `#` followed by `[allow(dead_code)]`.
6. Do not use any `git && rm ` commands.
7. When writing tests, prioritize detecting hidden bugs, rather than superficially asserting. This is irresponsible. Avoid shoddy work; write test cases that conform to the module's functionality.

8. Test module by module, completing one before moving on to the next.

9. Do not perform coverage tests (because it wastes time and resources).

10. Have a complete error module and a logging system (tracing). When an error occurs, report the module and method name to facilitate troubleshooting and analysis.
<br />

# Tier-1 Testing Rigor & Standards

## I. Core Testing Philosophy

- **Proof, Not Prayer**: Testing is not about proving that the code can run, but about proving that the code will not crash under extreme conditions.。
- **Deterministic Safety**: All unsafe code paths must implement $100\\%$ overriding.
- **Zero-Cost Verification**: Tests themselves should not pollute production code (using `#[cfg(test)]`).。

***

## II. Strict Test Case Requirements

### 1. The "Golden Trio" of Coverage

Each set of core logic (especially those involving memory allocation, atomic counting, and double-lifecycle structures) must include:

- **Positive Tests (Happy Path)**: Verify the expected behavior under standard input.。
- **Negative Tests (Edge Cases)**: You must verify null pointers, zero-length allocations, capacity overflows, and invalid memory alignments.。
- **Stress/Concurrency Tests**: Logic involving `Atomic` or `Lock-free` must be tested using `loom` or at least 50 concurrent threads.

### 2. Implementation Rules (Mandatory)

- **No** `println!`: Test output must use `tracing` with `test_subscriber`.

- **Expect meaningful context**: All assertions `assert!` must include a specific error message.

  Rust
  ```
  // ✅ GOOD
  assert_eq!(tracker.count(), 1, "Tracker should record exactly one allocation after push");
  // ❌ BAD
  assert_eq!(tracker.count(), 1);
  
  ```
- **Panic Testing**: For places where `Result::Err` should logically be triggered, the type of error returned must be verified; for contract violations, use `#[should_panic]`.

***

## III. Advanced Verification Standards

### 1. Memory Leak Detection (The "Miriri" Rule)

All modules involving `Raw Pointers` or manual memory management must pass the `Miri` check.

- **Requirement**: The `cargo miri test` must return 0 errors.

- **Focus**: Checks for data races, invalid pointer dereferences, and memory leaks.

### 2. Concurrency Invariant Testing (Loom)

I am pursuing lock-free operation, and I must use the `loom` library to simulate all possible combinations of thread scheduling.

Rust

```
#[test]
fn test_concurrent_allocation_logic() {
    loom::model(|| {
        let tracker = Tracker::new();
        let t1 = thread::spawn(move || tracker.track(0x1, 100));
        let t2 = thread::spawn(move || tracker.track(0x2, 200));
        t1.join().unwrap();
        t2.join().unwrap();
        // Verify atomicity consistency
    });
}

```

### 3. Fuzzing (AFL++ / libFuzzer)

Functions that parse memory metadata must include fuzz testing.

- **Goal:** Prove that a random input stream will not cause memory overflow or system crashes.

***

## IV. Documentation of Test Cases

### 1. The 7:3 Ratio in Tests

Even test code must follow your guidelines: **70% code, 30% comments**. Each test case must be clearly marked with:

- **Objective**: What does this test verify?

- **Invariants**: The state constraints before and after running this test.

Rust

```
/// Objective: Verify that the tracker correctly handles rapid allocation/deallocation
/// Invariants: Total tracked bytes must return to zero after all blocks are freed.
#[test]
fn test_balance_invariants() {
    // ... code ...
}

```

***

## V. Quality Gates (Pre-merge)

To improve coverage, any pull request (PR) must meet the following hard metrics:

1. **Line Coverage**: > 90%

2. **Unsafe Block Coverage**: **Must be 100%**

3. **Property-Based Testing**: Validate at least 1000 sets of randomly generated memory addresses and sizes using `proptest`.

4. **Static Check**: `cargo clippy --tests` must pass all tests.

***



## Rust Standard Coding Guidelines

### 1. Naming Conventions

Rust follows strict naming conventions, and violating these conventions will usually trigger a warning from `rustc`.

| 项目                              | 格式                        | 范例               |
| :------------------------------ | :------------------------ | :--------------- |
| **Crates / Modules**            | `snake_case`              | `data_processor` |
| **Types (Struct, Enum, Trait)** | `UpperCamelCase`          | `UserRecord`     |
| **Functions / Methods**         | `snake_case`              | `get_user_id()`  |
| **Variables / Parameters**      | `snake_case`              | `total_count`    |
| **Constants / Statics**         | `SCREAMING_SNAKE_CASE`    | `MAX_TIMEOUT`    |
| **Type Parameters**             | `UpperCamelCase`  | `T`, `U`, `Item` |

***

### 2. 注释规范 (Commenting)

Good comments explain more than just "what was done," they explain "why it was done."

- **Regular code comments:** Use `//` for inline explanation.

- **Documentation comments (Outer):** Use `///` to generate documentation for the following item (functions, structs, etc.).

- **Documentation comments (Inner):** Use `//!` to write module-level documentation for the current item (usually at the top of `lib.rs` or `mod.rs`).

- The code-to-comments ratio should be 7:3, and comments must be in English.

#### Best practices for document annotation：

````rust
//! # Module Title

//! This describes the functionality of the entire crate or module.

/// A brief description of the function (capitalized first letter, ending with a period).

///
/// # Examples

///
/// ```
/// let result = my_crate::add(1, 2);

/// assert_eq!(result, 3);

/// ```
///
/// # Errors

///
/// Lists the possible `Err` errors that this function might return.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
````

***

### 3. Documentation

- **Markdown Support:** Rustdoc natively supports Markdown. Use backticks (\`) to enclose code and use headings for categorization.

- **Common Sections:**

- `# Examples`: Required. This serves as both documentation and **documentation tests**.

- `# Panics`: Must be specified if a function will crash under certain conditions.

- `# Safety`: If it's an `unsafe fn`, the contract that the caller must adhere to must be stated.

- **Hidden Code:** In documentation tests, `#` can be used to hide auxiliary setup code, showing only the core logic.

***

### 4. Testing Standards

Rust tests are divided into unit tests and integration tests.

#### 4.1 Unit Tests

- **Location**: Placed at the end of the source file using `mod tests`.

- **Annotation**: Use the `#[cfg(test)]` attribute to ensure compilation only occurs when `cargo test` is called.

```rust
#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn test_add_function() {
        assert_eq!(add(2, 2), 4);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_divide_by_zero() {
        // Tests can cause crashes
    }
}
```

#### 4.2 Integration Tests

- **Location**: Place them in the `tests/` folder under the project root directory.

- **Purpose**: Enables you to call your public API like an external user.

#### 4.3 Documentation Tests

- Code blocks written within `///` will run automatically. This is the best way to ensure that documentation examples never become outdated.

***

### 5. Code Style and Tools

- **Formatting**: Always use `cargo fmt` for automatic code alignment.

- **Static Analysis**: Always run `cargo clippy`. It not only checks for errors but also provides suggestions for "more Rust-like" code.

- **Error Handling**:

- Prioritize using errors built into memscope-rs.

- Use the `?` operator to propagate errors.

- Avoid using `unwrap()` in library code unless you can prove it will never fail (and provide a comment explaining why).

***

### 6. Core Design Principles

- **Ownership First:** When designing APIs, consider whether parameters should be transferred ownership (`T`), borrowed (`&T`), or mutable borrowed (`&mut T`).

- **Composition Over Inheritance:** Use `Traits` to implement polymorphism.

- **Zero-Cost Abstraction:** Don't sacrifice performance for code simplicity unless the overhead is necessary.
***

## Advanced Coding Standards and Best Practices

### 1. Variable & Type Handling

- **Shadowing**：
  Variable masking is encouraged to change the type or mutability of variables, rather than creating redundant names like `data_str` and `data_int`.
  ```rust
  let data = " 42 ";
  let data: i32 = data.trim().parse()?; 
  ```
- **Temporary Variable Mutability:**

Try to keep variables immutable (default `let`). Only use `mut` when in-place modification is absolutely necessary.

- **Use `new()` and `Default`:**
- The constructor for a struct should be named `new`.
- If all fields of the struct have default values, be sure to implement the `Default` trait so that users can use the `..Default::default()` syntax.

***

### 2. Function Signatures

- **Deref Coercions**：
  Function parameters should preferably use slices rather than collection containers.

- **Bad**: `fn process(s: &String)` or `fn process(v: &Vec<u32>)`

- **Good**: `fn process(s: &str)` or `fn process(v: &[u32])`

- *Reason*: `&String` can only accept String, while `&str` can accept String and string literals.

- **Avoid Overuse** **`Clone`:**

Before calling `.clone()` inside a function, consider whether it can be solved by borrowing (`&T`). If ownership is necessary, let the caller decide whether to clone.

- **Returns** **`impl Trait`:**

Use `fn iter_elements(&self) -> impl Iterator<Item = &u32>` to hide complex internal types when returning closures or complex iterators.

***

### 3. Error Handling - In-Depth Specification

- **Library vs. Binary**:

- **Library Code**: Define your own `Error` enumeration and implement `std::fmt::Display` and `std::error::Error` (using the `thiserror` crate is recommended).

- **Application Code**: Use the `anyhow` crate to handle errors from various sources; it supports error context injection.

- **Boundaries of Unrecoverable Errors**:

- Use `panic!` / `unwrap()` only in the following situations:

1. Logically "absolutely impossible" to occur (e.g., hardcoded core configuration failing to load).

2. In test code.

3. In example code (for brevity).

- Always return `Result` in other cases.

***

### 4. Control Flow

- **if let vs match**：
  - If  only care about one pattern, use `if let`.

- If there are multiple branches or you need to exhaustively search, always use `match`.

- **Early Return**:

Reduce nesting depth. Handle erroneous branches first and return.

```rust

// Recommended Practice

let Some(user) = get_user() else { return Err(Error::NotFound) };

```
- **Iterators over Loops**:

Prefer using iterator methods like `.map()`, `.filter()`, and `.collect()` instead of explicit `for` loops. They are not only more concise but also often trigger compiler optimizations (such as boundary check elimination).

***

### 5. Trait Design Guidelines

- **Orphan Rule**:

Remember: You must either own the Trait or own the type; otherwise, you cannot implement the Trait for the type.

- **Associative Types vs. Generics**:

- If a type can only have one implementation of a Trait (e.g., the result of `Add`), use **associative types**.

- If multiple implementations are required (e.g., `From<T>` can have multiple Ts), use **generics**.

- **Sealed Traits**:

If you don't want external users to implement your Trait, you can use the private module pattern:
  ```rust
  mod private {
      pub trait Sealed {}
  }
  pub trait MyPublicTrait: private::Sealed { ... }
  ```

***

### 6. Concurrency

- **Send & Sync**:

Understand these two marker traits. Most types default to `Send` + `Sync`, but `Rc`, `Cell`, and `RefCell` are not.

- **Lock Granularity**:

Minimize the scope of `Mutex` locks. Use curly braces `{}` to explicitly control the lock's lifecycle, preventing deadlocks and improving concurrency.

- **Atomic Operations**:

For simple counters, prefer `std::sync::atomic` over `Mutex<i32>`.


***

### 7. Cargo.toml Project Structure

- **Explicit Version Declarations:** Always fix the major version number of dependencies in `Cargo.toml`.

- **Feature Gates:**

If your library has many features, use `[features]` to make non-core features optional to optimize compilation speed and binary size for downstream users.

- **Hierarchical Structure:**

  ```text
  src/
    lib.rs       # export API
    error.rs     # Unified error definition
    models/      # Data Structures
      mod.rs
      user.rs
  ```

***

### 8. Performance Fine-tuning Techniques

- **Pre-allocate Space:**

If you know the size of the collection, use `Vec::with_capacity(n)`.

- **Avoid Frequent Allocation:**

Reuse `String` or `Vec` buffers as much as possible inside loops, using `.clear()` instead of recreating them.

***

**Recommended Toolchain:**

1. **`cargo clippy`:** Its recommendations are almost the "standard answer".

2. **`cargo deny`:** Used to check dependency licenses and vulnerabilities.

3. **`cargo bloat`:** Check who is occupying space in the binary.

4. `make check` Ensure 0 errors.



### Compilation Speed Optimization Guide

This document describes the compilation speed optimizations configured for OmniScope-rs.

### Optimizations Enabled

#### 1. **sccache** - Compilation Cache
- **What**: Caches compiled artifacts across builds
- **Speedup**: 2-10x for incremental builds
- **Configured in**: `.cargo/config.toml`
- **Usage**:
  ```bash
  # Install sccache
  cargo install sccache
  
  # Add to shell profile (.bashrc or .zshrc)
  export RUSTC_WRAPPER=sccache
  
  # View cache statistics
  make cache-stats
  
  # Clear cache
  make cache-clear
  ```

#### 2. **Fast Linkers**
- **macOS**: `zld` (Apple's linker replacement)
  - Install: `brew install zld`
  - Speedup: 2-3x faster linking

- **Linux**: `mold` (Modern linker)
  - Install: `sudo apt install mold` or build from source
  - Speedup: 3-5x faster linking

- **Configured in**: `.cargo/config.toml`

#### 3. **Incremental Compilation**
- **What**: Only recompile changed files
- **Speedup**: 5-20x for small changes
- **Configured in**: `.cargo/config.toml` and `Cargo.toml`

#### 4. **Parallel Compilation**
- **What**: Compile multiple crates in parallel
- **Default**: Uses all available CPU cores
- **Customize**: `CARGO_BUILD_JOBS=8` in `.cargo/config.toml`

#### 5. **LTO (Link Time Optimization)**
- **What**: Whole-program optimization
- **Profile**: Release builds only
- **Configured in**: `Cargo.toml`
  ```toml
  [profile.release]
  lto = "fat"
  codegen-units = 1
  ```

### Build Commands

#### Fast Development Builds
```bash
# Quick debug build (fastest)
make build

# Quick test run
make test

# Development workflow (fmt + check + test)
make dev
```

#### Optimized Release Builds
```bash
# Full release build with all optimizations
make release

# Build with timing information
make build-timed

# Full optimization pipeline
make optimize
```

#### Cache Management
```bash
# View cache statistics
make cache-stats

# Clear cache
make cache-clear
```

### Performance Tips

#### 1. **Use Check Instead of Build**
```bash
# Faster than build (skips codegen and linking)
cargo check
```

#### 2. **Limit Dependencies**
- Use `cargo tree` to analyze dependencies
- Remove unused dependencies with `cargo udeps`

#### 3. **Use Workspace Caching**
- Workspace structure already configured
- Shared dependencies across crates

#### 4. **Monitor Build Times**
```bash
# Generate build timing report
cargo build --timings

# View in browser
open target/cargo-timings/cargo-timing.html
```

#### 5. **Reduce Codegen Units for Release**
- Already configured: `codegen-units = 1`
- Better optimization, slower build
- Only for release builds

### Expected Performance

#### Initial Build (Cold Cache)
- **Without optimizations**: ~5-10 minutes
- **With optimizations**: ~2-5 minutes

#### Incremental Build (Warm Cache)
- **Without optimizations**: ~30-60 seconds
- **With optimizations**: ~5-15 seconds

#### Link Time
- **Default linker**: ~10-30 seconds
- **Fast linker (zld/mold)**: ~2-10 seconds

## Troubleshooting

#### sccache Not Working
```bash
# Check if sccache is running
sccache --show-stats

# Restart sccache
sccache --stop-server
sccache --start-server
```



#### Slow Builds
1. Check cache hit rate: `make cache-stats`
2. Verify linker is being used: `cargo build -vv`
3. Check for dependency issues: `cargo tree`
4. Use `cargo check` instead of `cargo build`

### Additional Tools

#### cargo-watch (Auto-rebuild on changes)
```bash
cargo install cargo-watch
cargo watch -x check
```

#### cargo-hack (Feature checking)
```bash
cargo install cargo-hack
make check-features
```

#### flamegraph (Profiling)
```bash
cargo install cargo-flamegraph
make profile
```

### Configuration Files

- `.cargo/config.toml` - Cargo configuration
- `rust-toolchain.toml` - Rust version pinning
- `Cargo.toml` - Build profiles and dependencies
- `Makefile` - Build commands and shortcuts
