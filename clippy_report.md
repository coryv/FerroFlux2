# Clippy Audit Report

## ferroflux-security

Found 1 issues.

| Level | Code | Description |
|-------|------|-------------|
| Warning | `unused_imports` | unused import: `Path` |

<details>
<summary>Detailed Output</summary>

```text
warning: unused import: `Path`
 --> crates/ferroflux-security/src/api_key.rs:4:17
  |
4 | use std::path::{Path, PathBuf};
  |                 ^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default


```
</details>

---

## ferroflux-integration

Found 2 issues.

| Level | Code | Description |
|-------|------|-------------|
| Warning | `clippy::collapsible_if` | this `if` statement can be collapsed |
| Warning | `clippy::collapsible_if` | this `if` statement can be collapsed |

<details>
<summary>Detailed Output</summary>

```text
warning: this `if` statement can be collapsed
   --> crates/ferroflux-integration/src/validation.rs:272:9
    |
272 | / ...   if let Some(obj) = content.as_object_mut() {
273 | | ...       if let Some(meta) = obj.get_mut("meta").and_then(|m| m...
274 | | ...           meta.remove("signature");
275 | | ...       }
276 | | ...   }
    | |_______^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
272 ~         if let Some(obj) = content.as_object_mut()
273 ~             && let Some(meta) = obj.get_mut("meta").and_then(|m| m.as_object_mut()) {
274 |                 meta.remove("signature");
275 ~             }
    |


warning: this `if` statement can be collapsed
   --> crates/ferroflux-integration/src/validation.rs:303:9
    |
303 | / ...   if step.tool == "http_client" {
304 | | ...       if let Some(url) = step.params.get("url").and_then(|v|...
305 | | ...           // If it's a hardcoded external URL (not using pla...
306 | | ...           if (url.starts_with("http://") || url.starts_with(...
...   |
323 | | ...   }
    | |_______^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#collapsible_if
help: collapse nested if block
    |
303 ~         if step.tool == "http_client"
304 ~             && let Some(url) = step.params.get("url").and_then(|v| v.as_str()) {
305 |                 // If it's a hardcoded external URL (not using platform.base_url)
...
321 |                 }
322 ~             }
    |


```
</details>

---

## FerroFlux-core

Found 4 issues.

| Level | Code | Description |
|-------|------|-------------|
| Warning | `unused_imports` | unused import: `std::collections::HashMap` |
| Warning | `clippy::type_complexity` | very complex type used. Consider factoring parts into `type` definitions |
| Warning | `clippy::redundant_pattern_matching` | redundant pattern matching, consider using `is_some()` |
| Warning | `clippy::needless_borrow` | this expression creates a reference which is immediately dereferenced by the compiler |

<details>
<summary>Detailed Output</summary>

```text
warning: unused import: `std::collections::HashMap`
 --> crates/FerroFlux-core/src/systems/pipeline/resolution.rs:6:5
  |
6 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default


warning: very complex type used. Consider factoring parts into `type` definitions
  --> crates/FerroFlux-core/src/resources.rs:66:20
   |
66 | ...y: std::collections::HashMap<Entity, Vec<(Option<String>, Entity, Option<String>)>>,
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#type_complexity
   = note: `#[warn(clippy::type_complexity)]` on by default


warning: redundant pattern matching, consider using `is_some()`
  --> crates/FerroFlux-core/src/systems/agent/prep.rs:53:23
   |
53 |             while let Some(_) = inbox.queue.pop_front() {
   |             ----------^^^^^^^-------------------------- help: try: `while inbox.queue.pop_front().is_some()`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#redundant_pattern_matching
   = note: `#[warn(clippy::redundant_pattern_matching)]` on by default


warning: this expression creates a reference which is immediately dereferenced by the compiler
   --> crates/FerroFlux-core/src/systems/pipeline/execution.rs:161:37
    |
161 | ...   let res = resolve_recursive(&v, &h_ctx_temp, &handlebars, st...
    |                                   ^^ help: change this to: `v`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#needless_borrow
    = note: `#[warn(clippy::needless_borrow)]` on by default


```
</details>

---

## ferroflux-connectors

Found 5 issues.

| Level | Code | Description |
|-------|------|-------------|
| Error | `E0609` | error[E0609]: no field `metadata` on type `(std::option::Option<std::string::String>, ferroflux_types::SecureTicket)` |
| Error | `E0609` | error[E0609]: no field `metadata` on type `(std::option::Option<std::string::String>, ferroflux_types::SecureTicket)` |
| Error | `E0609` | error[E0609]: no field `metadata` on type `(std::option::Option<std::string::String>, ferroflux_types::SecureTicket)` |
| Error | `E0308` | error[E0308]: mismatched types |
| Error | `E0609` | error[E0609]: no field `metadata` on type `(std::option::Option<std::string::String>, ferroflux_types::SecureTicket)` |

<details>
<summary>Detailed Output</summary>

```text
error[E0609]: no field `metadata` on type `(std::option::Option<std::string::String>, ferroflux_types::SecureTicket)`
  --> crates/ferroflux-connectors/src/systems/ftp.rs:68:57
   |
68 | ...                   t.metadata = ticket.metadata.clone();
   |                                           ^^^^^^^^ unknown field
   |
help: one of the expressions' fields has a field of the same name
   |
68 |                                     t.metadata = ticket.1.metadata.clone();
   |                                                         ++


error[E0609]: no field `metadata` on type `(std::option::Option<std::string::String>, ferroflux_types::SecureTicket)`
  --> crates/ferroflux-connectors/src/systems/ssh.rs:85:45
   |
85 |                         t.metadata = ticket.metadata.clone();
   |                                             ^^^^^^^^ unknown field
   |
help: one of the expressions' fields has a field of the same name
   |
85 |                         t.metadata = ticket.1.metadata.clone();
   |                                             ++


error[E0609]: no field `metadata` on type `(std::option::Option<std::string::String>, ferroflux_types::SecureTicket)`
  --> crates/ferroflux-connectors/src/systems/xml.rs:21:18
   |
21 |                 .metadata
   |                  ^^^^^^^^ unknown field
   |
help: one of the expressions' fields has a field of the same name
   |
21 |                 .1.metadata
   |                  ++


error[E0308]: mismatched types
   --> crates/ferroflux-connectors/src/systems/xml.rs:26:51
    |
 26 |             let payload_bytes = match store.claim(&ticket) {
    |                                             ----- ^^^^^^^ expected `&SecureTicket`, found `&(Option<String>, SecureTicket)`
    |                                             |
    |                                             arguments to this method are incorrect
    |
    = note: expected reference `&ferroflux_types::SecureTicket`
               found reference `&(std::option::Option<std::string::String>, ferroflux_types::SecureTicket)`
note: method defined here
   --> /Users/coryvogan/FerroFlux2/crates/ferroflux-types/src/blob.rs:170:12
    |
170 |     pub fn claim(&self, ticket: &SecureTicket) -> anyhow::Result<Vec<u8>> {
    |            ^^^^^


error[E0609]: no field `metadata` on type `(std::option::Option<std::string::String>, ferroflux_types::SecureTicket)`
  --> crates/ferroflux-connectors/src/systems/xml.rs:58:58
   |
58 | ...                   new_ticket.metadata = ticket.metadata.clone();
   |                                                    ^^^^^^^^ unknown field
   |
help: one of the expressions' fields has a field of the same name
   |
58 |                             new_ticket.metadata = ticket.1.metadata.clone();
   |                                                          ++


```
</details>

---

## ferroflux-tools

Found 6 issues.

| Level | Code | Description |
|-------|------|-------------|
| Warning | `clippy::collapsible_if` | this `if` statement can be collapsed |
| Warning | `clippy::collapsible_if` | this `if` statement can be collapsed |
| Warning | `clippy::collapsible_if` | this `if` statement can be collapsed |
| Warning | `clippy::collapsible_if` | this `if` statement can be collapsed |
| Warning | `clippy::collapsible_if` | this `if` statement can be collapsed |
| Warning | `clippy::too_many_arguments` | this function has too many arguments (8/7) |

<details>
<summary>Detailed Output</summary>

```text
warning: this `if` statement can be collapsed
   --> crates/ferroflux-tools/src/primitives/http_client.rs:715:9
    |
715 | /         if aws_config.is_none() {
716 | |             if let Some(slug) = connection_slug {
717 | |                 if let Some(resolver) = context.secrets {
...   |
736 | |         }
    | |_________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#collapsible_if
    = note: `#[warn(clippy::collapsible_if)]` on by default
help: collapse nested if block
    |
715 ~         if aws_config.is_none()
716 ~             && let Some(slug) = connection_slug {
717 |                 if let Some(resolver) = context.secrets {
...
734 |                 }
735 ~             }
    |


warning: this `if` statement can be collapsed
   --> crates/ferroflux-tools/src/primitives/http_client.rs:716:13
    |
716 | /             if let Some(slug) = connection_slug {
717 | |                 if let Some(resolver) = context.secrets {
718 | |                     // Note: using hardcoded default_tenant to match resolve_connection_auth helper internal logic
719 | |                     let tenant = ferroflux_iam::TenantId::from("default_tenant");
...   |
735 | |             }
    | |_____________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#collapsible_if
help: collapse nested if block
    |
716 ~             if let Some(slug) = connection_slug
717 ~                 && let Some(resolver) = context.secrets {
718 |                     // Note: using hardcoded default_tenant to match resolve_connection_auth helper internal logic
...
733 |                     }
734 ~                 }
    |


warning: this `if` statement can be collapsed
   --> crates/ferroflux-tools/src/primitives/http_client.rs:720:21
    |
720 | /                     if let Ok(conn_data) = resolver.resolve_connection(&tenant, slug) {
721 | |                         if let Some("aws_sigv4") = conn_data.get("auth_type").and_then(|v| v.as_str()) {
722 | |                             let service = params.get("aws_service").and_then(|v| v.as_str()).unwrap_or("s3").to_string();
723 | |                             let region = conn_data.get("region").and_then(|v| v.as_str())
...   |
733 | |                     }
    | |_____________________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#collapsible_if
help: collapse nested if block
    |
720 ~                     if let Ok(conn_data) = resolver.resolve_connection(&tenant, slug)
721 ~                         && let Some("aws_sigv4") = conn_data.get("auth_type").and_then(|v| v.as_str()) {
722 |                             let service = params.get("aws_service").and_then(|v| v.as_str()).unwrap_or("s3").to_string();
...
731 |                             }
732 ~                         }
    |


warning: this `if` statement can be collapsed
   --> crates/ferroflux-tools/src/primitives/request.rs:323:17
    |
323 | /                 if let Some(parent) = stack.last_mut() {
324 | |                     if let Some(obj) = parent.as_object_mut() {
325 | |                         obj.insert("text".to_string(), Value::String(text));
326 | |                     }
327 | |                 }
    | |_________________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#collapsible_if
help: collapse nested if block
    |
323 ~                 if let Some(parent) = stack.last_mut()
324 ~                     && let Some(obj) = parent.as_object_mut() {
325 |                         obj.insert("text".to_string(), Value::String(text));
326 ~                     }
    |


warning: this `if` statement can be collapsed
   --> crates/ferroflux-tools/src/primitives/request.rs:332:17
    |
332 | /                 if let Some(parent) = stack.last_mut() {
333 | |                     if let Some(obj) = parent.as_object_mut() {
334 | |                         let content = obj.get_mut("content").and_then(|v| v.as_object_mut()).unwrap();
335 | |                         let current = content.get(&name);
...   |
350 | |                 }
    | |_________________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#collapsible_if
help: collapse nested if block
    |
332 ~                 if let Some(parent) = stack.last_mut()
333 ~                     && let Some(obj) = parent.as_object_mut() {
334 |                         let content = obj.get_mut("content").and_then(|v| v.as_object_mut()).unwrap();
...
348 |                         }
349 ~                     }
    |


warning: this function has too many arguments (8/7)
   --> crates/ferroflux-tools/src/primitives/request.rs:709:1
    |
709 | / pub fn execute_binary_request(
710 | |     client: &reqwest::blocking::Client,
711 | |     url: &str,
712 | |     method: &str,
...   |
717 | |     aws_config: Option<&AwsSigV4Config>,
718 | | ) -> Result<(u16, HashMap<String, String>, Value)> {
    | |__________________________________________________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#too_many_arguments
    = note: `#[warn(clippy::too_many_arguments)]` on by default


```
</details>

---

