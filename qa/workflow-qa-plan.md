# FerroFlux Workflow QA Plan

67 workflow test cases for validating the WAML workflow engine. Each entry describes a workflow that a QA agent should build as an actual `.yaml` WAML file, validate, and (where possible) dry-run.

---

## Category 1: Core Mechanics

Verify that each trigger type fires correctly and that simple linear node chains propagate data through edges.

### 1.1 Webhook Echo
Receives a POST webhook and immediately logs the raw body. The simplest possible workflow: one trigger, one action, one edge.
- **Nodes:** `core.trigger.webhook`, `core.action.log`
- **Tests:** Basic `body` edge wiring, minimal workflow structure.

### 1.2 Manual Trigger to HTTP GET
Fires manually, makes an HTTP GET request to a public API (e.g., httpbin.org/get), and logs the response.
- **Nodes:** `core.trigger.manual`, `core.action.http` (GET), `core.action.log`
- **Tests:** Manual trigger outputs, `output_var` propagation.

### 1.3 Schedule Trigger (Daily)
Runs on a daily schedule at a fixed time, fetches a JSON endpoint, and logs the result.
- **Nodes:** `core.trigger.schedule` (mode: daily), `core.action.http`, `core.action.log`
- **Tests:** Daily schedule config, timestamp output port.

### 1.4 Schedule Trigger (Interval)
Runs every 15 minutes, fetches a health-check endpoint, and logs the HTTP status code.
- **Nodes:** `core.trigger.schedule` (mode: interval, 15 minutes), `core.action.http`, `core.action.log`
- **Tests:** Interval-based scheduling distinct from calendar-based.

### 1.5 SSE Trigger to Log
Listens to an SSE endpoint and logs each received event's data, event_type, and id.
- **Nodes:** `core.trigger.sse`, `core.action.log`
- **Tests:** SSE trigger contract (`data`/`event_type`/`id` output ports).

### 1.6 Webhook to HTTP POST Chain
Receives a webhook, forwards the body via HTTP POST to an external API, and logs the response. A 3-node linear chain.
- **Nodes:** `core.trigger.webhook`, `core.action.http` (POST), `core.action.log`
- **Tests:** Multi-hop edge wiring, body data flow.

### 1.7 Webhook with Custom HTTP Headers
Receives a webhook POST, makes an HTTP GET with custom headers (Authorization bearer token), and logs the response body.
- **Nodes:** `core.trigger.webhook`, `core.action.http` (headers config), `core.action.log`
- **Tests:** Header configuration in HTTP nodes, secret template usage.

---

## Category 2: Control Flow

Test conditional routing, multi-branch switching, fan-out/fan-in iteration, and timed delays.

### 2.1 Binary Condition (If/Else)
Receives a webhook with a JSON body containing an `amount` field. Routes to a "high value" log if amount > 100, otherwise routes to a "low value" log.
- **Nodes:** `core.trigger.webhook`, `core.logic.condition` (operator: `>`), two `core.action.log`
- **Tests:** If/else condition, True/False output handles.

### 2.2 Multi-Way Switch
Receives a webhook with a `status` field. Routes to different log messages based on status being "success", "pending", "error", or default.
- **Nodes:** `core.trigger.webhook`, `core.action.switch` (4 rules incl. default), four `core.action.log`
- **Tests:** Switch node with multiple named output rules and a default fallback.

### 2.3 Split Array (Fan-Out Only)
Receives a webhook containing a JSON array of strings. Splits the array and logs each item individually via the `item` output port.
- **Nodes:** `core.trigger.webhook`, `core.action.split`, `core.action.log`
- **Tests:** Basic fan-out without aggregation, `item` edge handle.

### 2.4 Split and Aggregate Round-Trip
Receives a webhook with an array of numbers, splits them, multiplies each by 2 via a math node, aggregates the results, then logs the collected batch.
- **Nodes:** `core.action.split`, `core.utils.math` (mul), `core.action.aggregate` (batch_size: 0)
- **Tests:** Full fan-out/fan-in cycle, `item`/`batch` handles.

### 2.5 Delay Between Steps
Receives a webhook, logs "Starting", waits 3 seconds via delay, then logs "Completed".
- **Nodes:** `core.trigger.webhook`, `core.action.log`, `core.action.delay` (3000ms)
- **Tests:** Delay node pauses execution correctly, sequential edge flow.

### 2.6 Condition with Delay on True Branch
Receives a webhook with a `priority` field. If priority equals "urgent", immediately logs an alert. If not urgent, delays 10 seconds then logs a deferred message.
- **Nodes:** `core.logic.condition`, `core.action.delay`, `core.action.log`
- **Tests:** Delay combined with conditional branching.

### 2.7 Nested Condition Chains
Receives a webhook with `type` and `value` fields. First condition checks if type equals "order". If true, a second condition checks if value > 500. Each terminal branch logs a distinct message.
- **Nodes:** Two `core.logic.condition` nodes in series, three `core.action.log`
- **Tests:** Chained/nested conditions, multi-level branching.

---

## Category 3: Data Transformation

Exercise every utility node type for parsing, transforming, and generating data.

### 3.1 JSON Query (JMESPath)
Receives a webhook with nested JSON. Extracts a deeply nested field using a JMESPath expression, then logs the extracted value.
- **Nodes:** `core.utils.json` (operation: query), `core.action.log`
- **Tests:** JMESPath expression evaluation, deep field access.

### 3.2 JSON Deep Merge
Receives a webhook with two JSON objects (base config and override). Deep-merges them and logs the result.
- **Nodes:** `core.utils.json` (operation: merge, `other` input)
- **Tests:** Merge operation, two data inputs to one node.

### 3.3 JSON Flatten
Receives a webhook with a deeply nested JSON object. Flattens it and logs the flat key-value structure.
- **Nodes:** `core.utils.json` (operation: flatten)
- **Tests:** Flatten operation, nested-to-flat conversion.

### 3.4 Text Regex Match and Replace
Receives a webhook with a text string. Extracts email addresses via regex, then redacts them. Logs both the matches and the redacted text.
- **Nodes:** Two `core.utils.text` (match, then replace), regex `pattern`
- **Tests:** Two-step text pipeline, regex pattern config.

### 3.5 Text Slugify and Truncate
Receives a webhook with a title string. Slugifies it, then truncates to 50 characters. Logs the final slug.
- **Nodes:** Two `core.utils.text` (slugify, truncate with `length: 50`)
- **Tests:** Chaining two text operations sequentially.

### 3.6 Math Operations Pipeline
Receives a webhook with `subtotal` and `tax_rate`. Multiplies to get tax amount, adds to subtotal for total. Logs the final total.
- **Nodes:** Two `core.utils.math` (mul, then add)
- **Tests:** Chained math with data edges between operations.

### 3.7 Date Parse and Diff
Receives a webhook with `start_date` and `end_date` strings. Parses both, calculates the diff, and logs the elapsed days.
- **Nodes:** `core.utils.date` (parse, then diff), date1/date2 inputs
- **Tests:** Date parsing and diff operations.

### 3.8 Crypto Hash and UUID
Receives a webhook body, generates a SHA-256 hash for integrity and a UUID v4 as correlation ID. Logs both.
- **Nodes:** Two `core.utils.crypto` (hash, uuid)
- **Tests:** Two crypto operations with different modes.

### 3.9 HTML Selector Extraction
Receives a webhook containing an HTML string. Extracts text from `h1` elements via CSS selector and logs it.
- **Nodes:** `core.utils.html` (selector, attr)
- **Tests:** HTML parsing, CSS selector config.

### 3.10 XML to JSON Pipeline
Receives a webhook with an XML string. Parses to JSON, extracts a field with JSON query, and logs it.
- **Nodes:** `core.utils.xml`, `core.utils.json` (query)
- **Tests:** XML-to-JSON conversion chained with JSON extraction.

### 3.11 Transform Field Mapping
Receives a webhook with user data in one schema (`first_name`, `last_name`). Remaps to a different schema (`customerName`, `customerEmail`). Logs the result.
- **Nodes:** `core.utils.transform` (template)
- **Tests:** Schema reshaping via template.

### 3.12 GraphQL Query
Fires manually, executes a GraphQL query against an endpoint with variables, and logs the response.
- **Nodes:** `core.trigger.manual`, `core.utils.graphql` (query, variables, operation_name)
- **Tests:** GraphQL utility with variables and named operation.

### 3.13 PDF Read and Summarize
Receives a webhook with a base64-encoded PDF. Extracts text via `pdf_read`, sends to an AI agent for summarization, logs the summary.
- **Nodes:** `core.utils.pdf_read`, `core.action.agent`
- **Tests:** Utility-to-AI pipeline, binary data handling.

### 3.14 PDF Write and Email
Receives a webhook with report data. Generates a PDF via `pdf_write`, sends it as an email via Resend.
- **Nodes:** `core.utils.pdf_write`, `resend.emails.send`
- **Tests:** PDF generation flowing into a platform action, binary data propagation.

### 3.15 Statistics with Outlier Detection
Receives a webhook with an array of items containing numeric `value` fields. Runs stats with outlier detection (z-score threshold: 2). Logs mean, stdev, and flagged outliers.
- **Nodes:** `core.manipulation.stats` (target_field, detect_outliers, threshold)
- **Tests:** Stats enrichment, outlier detection config.

---

## Category 4: State Management

Test set_var/get_var for workflow-wide state and context propagation.

### 4.1 Set and Get Variable Round-Trip
Receives a webhook, stores `user_id` via set_var, performs an HTTP GET, then retrieves the stored user_id with get_var and logs both values.
- **Nodes:** `core.action.set_var`, `core.action.get_var`, `core.action.http`
- **Tests:** Variable persistence across nodes, variable_name config.

### 4.2 Counter Accumulation Across Fan-Out
Receives a webhook with an array. Sets counter to 0. Splits the array, increments counter per item via get_var + math + set_var. After aggregation, logs the final count.
- **Nodes:** `core.action.set_var`, `core.action.get_var`, `core.utils.math`, `core.action.split`, `core.action.aggregate`
- **Tests:** Variable mutation within fan-out context, state across iteration.

### 4.3 Variable-Driven Conditional Routing
Receives a webhook, stores `role` as a variable. Retrieves it and routes via condition based on whether role equals "admin".
- **Nodes:** `core.action.set_var`, `core.action.get_var`, `core.logic.condition`
- **Tests:** get_var feeding into condition logic, variable-driven branching.

---

## Category 5: Error Handling

Test Error output edges, invalid input handling, and graceful failure paths.

### 5.1 HTTP Error Edge Handling
Receives a webhook and makes an HTTP GET to a URL that returns 404. Wires Error output to a failure log, Success output to a different log.
- **Nodes:** `core.action.http` (Error output), two `core.action.log`
- **Tests:** HTTP errors route through Error handle, Success vs Error path separation.

### 5.2 Agent Node Error Recovery
Receives a webhook, calls an AI agent with intentionally invalid provider config. Error output wires to a fallback log recording "AI unavailable".
- **Nodes:** `core.action.agent` (Error output), fallback `core.action.log`
- **Tests:** Agent node error propagation, Error edge wiring.

### 5.3 Script Node Runtime Error
Receives a webhook, runs a Rhai script that deliberately divides by zero. Error output wires to a failure log.
- **Nodes:** `core.action.script` (Error output), `core.action.log`
- **Tests:** Script node error handling, intentional runtime error in Rhai.

### 5.4 Condition with Missing Input Data
Receives a webhook where `amount` may or may not exist. Condition checks `amount > 100`. When missing, should route to False/Error. Logs outcome on all branches.
- **Nodes:** `core.logic.condition`, multiple `core.action.log`
- **Tests:** Missing field handling, defensive edge wiring.

### 5.5 Chained Error Propagation (Retry Pattern)
Receives a webhook, makes an HTTP call that fails. Error edge triggers a retry HTTP call to a different URL. If that also fails, logs "all retries exhausted".
- **Nodes:** Two `core.action.http`, cascading Error edges, final `core.action.log`
- **Tests:** Multi-hop error chains, retry-via-error-edge pattern.

---

## Category 6: Platform Integrations

Real-world scenarios using major platform nodes across multiple services.

### 6.1 GitHub Issue to Slack Notification
Receives a webhook (GitHub event), creates a GitHub issue, sends a Slack DM with the issue URL and title.
- **Nodes:** `core.trigger.webhook`, `github.issues.create`, `slack.messages.send_dm`
- **Tests:** Cross-platform data flow, GitHub + Slack integration.

### 6.2 Jira Issue Lifecycle
Receives a webhook with issue details. Creates a Jira issue, assigns it, and transitions to "In Progress". Logs the issue key.
- **Nodes:** `jira.issues.create`, `jira.issues.assign`, `jira.issues.transition`
- **Tests:** Sequential single-platform chain, Jira lifecycle operations.

### 6.3 Linear Issue from Slack Mention
Triggered by Slack mention. Extracts message text, creates a Linear issue, sends confirmation DM back.
- **Nodes:** `slack.messages.mention` (trigger), `linear.issues.create`, `slack.messages.send_dm`
- **Tests:** Trigger-to-action cross-platform, Slack trigger + Linear action.

### 6.4 Stripe Payment to Google Sheets Log
Triggered by Stripe payment succeeded. Extracts amount, email, timestamp. Appends a row to Google Sheets.
- **Nodes:** `stripe.payments.succeeded` (trigger), `google_sheets.values.append`
- **Tests:** Webhook trigger integration, payment data extraction.

### 6.5 Gmail Attachment to Google Drive
Triggered by Gmail attachment arrival. Downloads attachment, uploads to Google Drive, sends Slack notification.
- **Nodes:** `gmail.emails.new_attachment` (trigger), `gmail.emails.get_attachment`, `google_drive.files.upload`, `slack.messages.send_dm`
- **Tests:** Multi-platform chain (Gmail + Drive + Slack), binary data flow.

### 6.6 Shopify Order Fulfillment with Inventory Check
Triggered by new Shopify order. Checks inventory; if > 0, creates fulfillment. If out of stock, sends Telegram alert to ops.
- **Nodes:** `shopify.orders.new` (trigger), `shopify.inventory.get`, `core.logic.condition`, `shopify.orders.fulfill`, `telegram.messages.send`
- **Tests:** Conditional platform routing, e-commerce workflow.

### 6.7 PostgreSQL Daily Report to Slack
Daily schedule trigger. Executes a PostgreSQL SELECT, formats results via transform, sends as Slack DM.
- **Nodes:** `core.trigger.schedule`, `postgresql.query.execute`, `core.utils.transform`, `slack.messages.send_dm`
- **Tests:** Database-to-messaging pipeline, scheduled data reporting.

### 6.8 Redis Cache-Aside Pattern
Receives a webhook with a cache key. GETs from Redis; if cached, logs and returns. If miss, fetches via HTTP, SETs in Redis, logs fresh value.
- **Nodes:** `redis.key.get`, `core.logic.condition`, `core.action.http`, `redis.key.set`
- **Tests:** Cache-aside pattern, conditional fetch-and-store.

### 6.9 Notion Weekly Digest via Email
Weekly schedule. Queries Notion database for items updated in last 7 days. Formats and sends via Resend.
- **Nodes:** `core.trigger.schedule` (weekly), `notion.databases.query`, `core.utils.transform`, `resend.emails.send`
- **Tests:** Scheduled data pull, Notion + email integration.

### 6.10 HubSpot to Salesforce Contact Sync
Triggered by new HubSpot contact. Searches Salesforce by email. If not found, creates record. If found, updates. Logs outcome.
- **Nodes:** `hubspot.contacts.new` (trigger), `salesforce.records.search`, `core.logic.condition`, `salesforce.records.create`, `salesforce.records.update`
- **Tests:** CRM sync pattern, conditional create-or-update.

### 6.11 AI Telegram Bot
Triggered by Telegram message. Sends text to Claude via agent node. Sends AI response back via Telegram.
- **Nodes:** `telegram.messages.new` (trigger), `core.action.agent` (anthropic), `telegram.messages.send`
- **Tests:** AI chatbot pattern, Telegram round-trip.

### 6.12 Sentry Alert Triage to Jira
Receives Sentry webhook. Switch routes by severity: fatal creates high-priority Jira issue, error creates normal Jira issue, warning logs only.
- **Nodes:** `core.trigger.webhook`, `core.action.switch` (3 rules), two `jira.issues.create`, `core.action.log`
- **Tests:** Alert triage pattern, severity-based routing.

---

## Category 7: Complex Patterns

Multi-step orchestrations combining advanced features.

### 7.1 Fan-Out HTTP Enrichment with Aggregate and Email
Receives a webhook with an array of user IDs. Splits, fetches each user profile via HTTP, aggregates all profiles, transforms into summary, emails via Resend.
- **Nodes:** `core.action.split`, `core.action.http`, `core.action.aggregate`, `core.utils.transform`, `resend.emails.send`
- **Tests:** Full fan-out/fan-in with downstream processing, HTTP-per-item pattern.

### 7.2 Subflow: Reusable User Enrichment
Parent workflow receives a webhook and calls a subflow. The subflow (separate WAML file) uses `core.trigger.subflow`, fetches user via HTTP, returns via `core.action.subflow_output`. Parent logs the result. **Produces two WAML files.**
- **Nodes:** `core.trigger.subflow`, `core.action.subflow_output`, parent workflow with subflow call
- **Tests:** Parent-child workflow pattern, subflow trigger/output contract.

### 7.3 Multi-Platform Alert Router
Receives a webhook with `channel` (slack/discord/telegram/email) and `message`. Switch routes to the correct platform action.
- **Nodes:** `core.action.switch` (4 branches), `slack.messages.send_dm`, `discord.messages.send`, `telegram.messages.send`, `resend.emails.send`
- **Tests:** Single workflow dispatching to four platforms, wide switch routing.

### 7.4 ETL Pipeline: API to Database
Schedule trigger. Fetches data via HTTP, transforms JSON to match target schema, inserts into PostgreSQL, logs row count.
- **Nodes:** `core.trigger.schedule`, `core.action.http`, `core.utils.json` (query), `core.utils.transform`, `postgresql.rows.insert`
- **Tests:** Classic ETL pattern, API-to-database pipeline.

### 7.5 Rhai Script for Complex Business Logic
Receives a webhook with an order payload. Rhai script applies tiered pricing, coupon validation, and minimum order checks. Logs calculated price.
- **Nodes:** `core.trigger.webhook`, `core.action.script` (multi-line Rhai), `core.action.log`
- **Tests:** Complex Rhai scripting with conditionals and arithmetic.

### 7.6 Webhook Signature Verification
Receives a webhook with `X-Hub-Signature-256` header. Computes HMAC-SHA256 of the raw body. Condition compares computed vs. received signature. Match processes normally; mismatch logs "Rejected".
- **Nodes:** `core.utils.crypto` (hmac), `core.logic.condition`, `core.action.log`
- **Tests:** Webhook security pattern, HMAC verification flow.

### 7.7 MongoDB Transaction Workflow
Receives a webhook with transfer data. Begins MongoDB transaction, debits sender, credits receiver, commits. On Error, aborts transaction and logs failure.
- **Nodes:** `mongodb.transaction.begin`, `mongodb.document.update` (x2), `mongodb.transaction.commit`, `mongodb.transaction.abort`
- **Tests:** Transactional error edge handling, multi-step database transaction.

### 7.8 Multi-Step AI Content Pipeline
Receives a webhook with a topic. AI agent 1 generates an outline. AI agent 2 expands into full content. Text node truncates to 2000 chars. AI agent 3 generates a social summary. Summary sent to Slack.
- **Nodes:** Three `core.action.agent`, `core.utils.text` (truncate), `slack.messages.send_dm`
- **Tests:** Multi-stage LLM pipeline, chained AI with intermediate processing.

### 7.9 Parallel Branch Merge
Receives a webhook, sends two parallel HTTP requests (two edges from trigger). Each response stored via set_var. Downstream node retrieves both and logs combined result.
- **Nodes:** Two parallel `core.action.http`, two `core.action.set_var`, `core.action.get_var`
- **Tests:** Parallel execution paths converging, dual variable storage.

### 7.10 Trello Card Lifecycle with Checklist
Receives a webhook with task details. Creates a Trello card, adds a checklist, adds items, moves card to "In Progress". Logs final card URL.
- **Nodes:** `trello.cards.create`, `trello.checklists.create`, `trello.checklists.addItem`, `trello.cards.move`
- **Tests:** Multi-step single-platform orchestration.

---

## Category 8: Edge Cases

Boundary conditions, unusual inputs, and structural limits.

### 8.1 Empty Array Split
Receives a webhook with `[]`. Split fires `Done` immediately with no items. Aggregate receives empty batch.
- **Nodes:** `core.action.split`, `core.action.aggregate`
- **Tests:** Zero-iteration behavior, Done handle on empty input.

### 8.2 Single-Item Array Split
Receives a webhook with `[42]`. Splits, processes the one item, aggregates. Tests single-element boundary.
- **Nodes:** `core.action.split`, `core.action.aggregate`
- **Tests:** Off-by-one boundary, single-item fan-out/fan-in.

### 8.3 Large Payload Propagation
Receives a webhook with a large JSON body (many fields, deep nesting, 100+ item arrays). Passes through JSON flatten, transform, and log.
- **Nodes:** `core.utils.json` (flatten), `core.utils.transform`, `core.action.log`
- **Tests:** Large payload survival across multi-hop edges without truncation.

### 8.4 Deeply Nested JSON Extraction
Receives a webhook with 5-level nested JSON. JSON query extracts `a.b.c.d.e` leaf value. Logs result and verifies non-null.
- **Nodes:** `core.utils.json` (query, deep expression)
- **Tests:** Deep JMESPath access, nested data extraction.

### 8.5 Many-Branch Switch (10+ Outputs)
Receives a webhook with `event_type`. Switch has 10 rules plus default, each routing to a separate log.
- **Nodes:** `core.action.switch` (11 rules), 11 `core.action.log`
- **Tests:** High branch count, wide fan-out from single switch.

### 8.6 Unicode and Special Characters
Receives a webhook with emoji, CJK characters, RTL text, null bytes, and newlines. Passes through text regex, transform, and log.
- **Nodes:** `core.utils.text` (match), `core.utils.transform`, `core.action.log`
- **Tests:** Non-ASCII data survival, encoding resilience.

### 8.7 Split with Large Array (500 Items)
Receives a webhook with 500 simple objects. Splits, applies lightweight transform to each, aggregates all 500, logs count.
- **Nodes:** `core.action.split`, `core.utils.transform`, `core.action.aggregate` (batch_size: 500)
- **Tests:** Fan-out/fan-in at scale, high-cardinality iteration.

### 8.8 Null and Missing Field Handling
Receives a webhook where expected fields are null or missing. Passes through JSON query (missing paths), condition (null comparison), and transform. Logs each step.
- **Nodes:** `core.utils.json` (query), `core.logic.condition`, `core.utils.transform`
- **Tests:** Null propagation behavior, undefined field resilience.

---

## Coverage Matrices

### Node Type Coverage

| Node Type | Workflow IDs |
|-----------|-------------|
| `core.trigger.webhook` | 1.1, 1.6, 1.7, 2.1-2.7, 3.x, 4.x, 5.x, 7.x, 8.x |
| `core.trigger.manual` | 1.2, 3.12 |
| `core.trigger.schedule` | 1.3, 1.4, 6.7, 6.9, 7.4 |
| `core.trigger.sse` | 1.5 |
| `core.trigger.subflow` | 7.2 |
| `core.action.http` | 1.2, 1.6, 1.7, 5.1, 5.5, 6.8, 7.1, 7.4, 7.9 |
| `core.action.agent` | 3.13, 5.2, 6.11, 7.8 |
| `core.action.log` | Nearly all |
| `core.action.switch` | 2.2, 6.12, 7.3, 8.5 |
| `core.logic.condition` | 2.1, 2.6, 2.7, 4.3, 5.4, 6.6, 6.8, 6.10, 7.6, 8.8 |
| `core.action.split` | 2.3, 2.4, 4.2, 7.1, 8.1, 8.2, 8.7 |
| `core.action.aggregate` | 2.4, 4.2, 7.1, 8.1, 8.2, 8.7 |
| `core.action.delay` | 2.5, 2.6 |
| `core.action.set_var` | 4.1, 4.2, 4.3, 7.9 |
| `core.action.get_var` | 4.1, 4.2, 4.3, 7.9 |
| `core.action.script` | 5.3, 7.5 |
| `core.action.subflow_output` | 7.2 |
| `core.utils.json` | 3.1, 3.2, 3.3, 3.10, 7.4, 8.3, 8.4, 8.8 |
| `core.utils.text` | 3.4, 3.5, 7.8, 8.6 |
| `core.utils.math` | 2.4, 3.6, 4.2 |
| `core.utils.date` | 3.7 |
| `core.utils.crypto` | 3.8, 7.6 |
| `core.utils.html` | 3.9 |
| `core.utils.xml` | 3.10 |
| `core.utils.transform` | 3.11, 6.7, 6.9, 7.1, 7.4, 8.3, 8.6, 8.7 |
| `core.utils.graphql` | 3.12 |
| `core.utils.pdf_read` | 3.13 |
| `core.utils.pdf_write` | 3.14 |
| `core.manipulation.stats` | 3.15 |

### Platform Coverage

| Platform | Workflow IDs |
|----------|-------------|
| Slack | 6.1, 6.3, 6.5, 6.7, 7.3, 7.8 |
| GitHub | 6.1 |
| Jira | 6.2, 6.12 |
| Linear | 6.3 |
| Stripe | 6.4 |
| Google Sheets | 6.4 |
| Gmail | 6.5 |
| Google Drive | 6.5 |
| Shopify | 6.6 |
| Telegram | 6.6, 6.11, 7.3 |
| PostgreSQL | 6.7, 7.4 |
| Redis | 6.8 |
| Notion | 6.9 |
| Resend | 3.14, 6.9, 7.1, 7.3 |
| HubSpot | 6.10 |
| Salesforce | 6.10 |
| Anthropic (via agent) | 6.11, 7.8 |
| Discord | 7.3 |
| MongoDB | 7.7 |
| Sentry | 6.12 |
| Trello | 7.10 |
