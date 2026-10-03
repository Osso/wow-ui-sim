# rilua secret-string transform review

**Verdict: ACCEPT.** Low incremental merge risk; no actionable defect found in commit `70625a96e1556c45845fb4d030f37af3ef896877` against `6044544`.

Reviewed `/home/osso-test/.worktrees/rilua-secret-string-transform`; HEAD matched the requested commit. Git status was clean before and after checks. No source changes, commits, pushes, delegation, or model CLIs performed.

## Safety and security findings

1. **GC safety — no defect found.** `src/table_security.rs:124–138` validates arena references and copies the input bytes into an owned `Vec` before invoking the callback. The callback therefore cannot observe freed arena memory through its supplied slice. The callback accepts no state reference; safe Rust cannot reborrow the exclusively borrowed state through it. Rust heap allocations during copying/callback execution do not run this VM's collector. `intern_string` (`src/vm/state.rs:248–261`), `alloc_userdata` (`src/vm/state.rs:359–366`), and allocation accounting (`src/vm/gc/collector.rs:267–272`) do not collect or invoke Lua. The new payload is pushed at `src/table_security.rs:142` before wrapper allocation and popped at line 144. Stack growth itself does not collect (`src/vm/state.rs:931–948`). No GC safe point occurs between interning and rooting. The only returned error occurs at line 137, before any push; no fallible operation follows the push. A callback panic also precedes the push. Wrapper marking traces its private payload (`src/vm/gc/collector.rs:361–380`). The returned wrapper remains caller-rooted by contract, like the sibling constructors.

2. **Authenticity / Lua exposure — no new bypass.** Input must be `Val::Userdata`, resolve to a live generation-checked arena entry, contain the private secret payload, and hold an actual `Val::Str` (`src/table_security.rs:124–137`; `src/vm/gc/arena.rs:290–300`). Ordinary userdata has `secret_value: None`; metatable or environment changes cannot create that field. Its constructor/accessor are crate-private (`src/vm/value.rs:64–119`). A table-shaped impostor cannot pass the enum match. Registration remains only `settablesecurity`, `secretwrap`, `secretunwrap`, and `issecretvalue` (`src/table_security.rs:21–27`); the new helper has no production Lua registration or caller. Existing secure Lua unwrapping remains available as before, but tainted callers gain no plaintext access. Trusted Rust explicitly receives plaintext bytes; exposing or leaking those bytes through a separately written host callback would be the host's responsibility, not a new Lua path in this commit.

3. **Taint / hooks — unchanged.** The helper neither writes taint nor modifies hook configuration, invokes Lua, or runs GC/finalizers. Its only VM mutations are allocation accounting, string/userdata allocation, and a balanced temporary stack root. Tainted execution succeeds without granting guarded unwrap authority. Hooks are established by source inspection, not a dedicated new hook test.

## Behavioral proof and limits

The five new tests assert transformed bytes, a fresh wrapper, unchanged input identity/payload, unchanged stack top, preserved taint, blocked tainted unwrap/string operations, rejection of invalid input without invoking the callback, binary/NUL/empty output, and survival through two full collections after dropping the input.

Returning the input fails fresh-identity and changed-payload assertions (`tests/helpers/secret_string_transform.rs:43–47`). Returning an ordinary string fails the secret-wrapper assertion (`:19–23`) and Lua secrecy assertions. Tests assert observable behavior, not source construction. Mutation variants were not executed because source edits were prohibited.

Coverage limits: full GC is exercised after the result is rooted, not injected between allocations; no dedicated hook-state assertion, stale-reference rejection test, or direct stack-top assertion on an error return was added. Those boundaries were inspected in source. They do not expose an observed defect in the current implementation.

## API / documentation / scope

The signature, error text, authentic-input requirement, binary-byte behavior, fresh-result behavior, taint preservation, host-only boundary, and caller-rooting requirement agree across `src/table_security.rs:114–146`, `docs/specs/table-security.md:12`, and `docs/src/api.md:544–561`. Unlike the string constructor's `&str` input, the transform intentionally accepts/returns arbitrary bytes. The explicit temporary payload root is compatible with the sibling helpers; their lower-level allocations currently do not trigger collection.

Exactly six files changed, 223 insertions and no deletions: the helper, five new tests, integration-module wiring, changelog, security spec, and API documentation. No unrelated runtime changes.

## Proof ledger / observed test counts

Both commands ran once against the requested HEAD; no subsequent source changes invalidated their evidence.

- `cargo test -j 4 --test integration secret_string_transform` — **exit 0; 5 passed, 0 failed, 0 ignored, 0 measured, 479 filtered out**.
- `cargo test -j 4 --test integration table_security` — **exit 0; 24 passed, 0 failed, 0 ignored, 0 measured, 460 filtered out**.

**Total: 29/29 selected tests passed.** No broader-suite, sanitizer, or mutation-test claim. Defects requiring file:line remediation: **none found**.
