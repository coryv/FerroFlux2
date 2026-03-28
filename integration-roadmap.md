# FerroFlux Integration Roadmap

## Context
FerroFlux currently has 5 platforms (core, GitHub, Open-Meteo, OpenAI, Resend). To compete with n8n (~400 integrations), Zapier (~6,000), and Make (~1,000), we need broad coverage across every major SaaS vertical. This document is the canonical prioritized integration list to work through using the `ferroflux-integration` skill.

**Tier 1 = Must-have to be taken seriously**
**Tier 2 = Needed for broad appeal**
**Tier 3 = Long-tail / power users**

Each entry lists:
- **Triggers** — polling or webhook-based events that start a workflow
- **Actions** — operations a workflow node can perform

---

## Already Built
- `core` — workflow primitives
- `github` — repos, issues
- `open-meteo` — weather/forecast
- `openai` — chat completions *(needs Chat Completion with Tools node added)*
- `resend` — transactional email

---

## Communication & Messaging

### Slack *(Tier 1)*
**Triggers:** New Message in Channel, New Direct Message, New Reaction Added, New Channel Created, App Mention, New File Shared
**Actions:** Send Message, Send Direct Message, Update Message, Delete Message, Add Reaction, Remove Reaction, Create Channel, Archive Channel, Invite User to Channel, Set Channel Topic, Upload File, Get User by ID, Get User by Email, List Channels, List Users, Create Reminder, Post to Incoming Webhook

### Discord *(Tier 1)*
**Triggers:** New Message in Channel, New Guild Member, New Reaction, Message Deleted
**Actions:** Send Message, Send Direct Message, Edit Message, Delete Message, Add Role to User, Remove Role from User, Create Channel, Delete Channel, Ban User, Kick User, Send Webhook Message, Pin Message

### Microsoft Teams *(Tier 1)*
**Triggers:** New Message in Channel, New Chat Message, New Meeting Created
**Actions:** Send Channel Message, Send Chat Message, Create Team, Create Channel, List Teams, List Channels, List Members, Upload File

### Telegram *(Tier 1)*
**Triggers:** New Message, New Command, New Callback Query, New Inline Query
**Actions:** Send Message, Send Photo, Send Document, Send Audio, Send Video, Edit Message, Delete Message, Pin Message, Ban User, Get Chat Info, Answer Callback Query, Send Poll

### Gmail *(Tier 1)*
**Triggers:** New Email, New Email Matching Search, New Label Added, New Attachment
**Actions:** Send Email, Reply to Email, Forward Email, Create Draft, Get Email, List Emails, Move to Label, Delete Email, Mark as Read, Mark as Unread, Add Label, Remove Label, Create Label, Get Attachment, Search Emails

### Microsoft Outlook *(Tier 1)*
**Triggers:** New Email, New Email Matching Filter, New Calendar Event
**Actions:** Send Email, Reply to Email, Forward Email, Create Draft, Get Email, Move Email, Delete Email, List Emails, Create Contact, Update Contact, Get Contact

### SMTP *(Tier 1)*
**Actions:** Send Email (plain text), Send Email (HTML), Send Email with Attachment

### Twilio *(Tier 1)*
**Triggers:** New SMS Received, New Call Received, New WhatsApp Message
**Actions:** Send SMS, Send MMS, Make Call, Send WhatsApp Message, Look Up Phone Number, Create Verification, Check Verification Code

### WhatsApp Business *(Tier 2)*
**Triggers:** New Message, Message Status Update
**Actions:** Send Text Message, Send Template Message, Send Image, Send Document, Send Audio

### Google Chat *(Tier 2)*
**Actions:** Send Message to Space, Create Space, List Spaces, Get Message

---

## Productivity & Project Management

### Notion *(Tier 1)*
**Triggers:** New Database Item, Updated Database Item, New Page in Database
**Actions:** Create Database Item, Update Database Item, Get Database Item, Delete Database Item, Query Database, Create Page, Get Page, Update Page, Archive Page, Create Block, Append Block Children, Get Block Children, Search, List Users, List Databases

### Airtable *(Tier 1)*
**Triggers:** New Record, Updated Record, New/Updated Record
**Actions:** Create Record, Update Record, Get Record, Delete Record, List Records, Search Records, Create Table, List Tables, Get Table Schema, Upload Attachment

### Trello *(Tier 1)*
**Triggers:** New Card, Card Moved, Card Updated, New Comment, New Checklist Item Completed
**Actions:** Create Card, Update Card, Delete Card, Move Card, Add Comment, Get Card, List Cards, Create Board, Create List, Archive Card, Add Label, Add Attachment, Add Checklist, Add Checklist Item, Mark Checklist Item Complete

### Asana *(Tier 1)*
**Triggers:** New Task, Updated Task, New Project, Task Completed
**Actions:** Create Task, Update Task, Delete Task, Get Task, List Tasks, Complete Task, Add Subtask, Add Comment, Add Follower, Create Project, Get Project, List Projects, Create Section, Move Task to Section

### Jira *(Tier 1)*
**Triggers:** New Issue, Updated Issue, Issue Status Changed, New Comment, New Sprint Started
**Actions:** Create Issue, Update Issue, Delete Issue, Get Issue, Transition Issue, Add Comment, Get Comments, Assign Issue, Search Issues (JQL), Create Sprint, Start Sprint, Get Sprint, List Projects, Create Project, Get User, Attach File

### Linear *(Tier 1)*
**Triggers:** New Issue, Updated Issue, Issue State Changed, New Comment
**Actions:** Create Issue, Update Issue, Archive Issue, Get Issue, Search Issues, Create Comment, Update Issue Status, Assign Issue, Create Label, List Teams, List Projects, Create Project

### Monday.com *(Tier 1)*
**Triggers:** New Item, Updated Item, Status Changed, New Update (comment)
**Actions:** Create Item, Update Item, Delete Item, Get Item, List Items, Change Column Value, Create Update, Get Board, List Boards, Create Board, Create Column, Move Item to Group

### ClickUp *(Tier 1)*
**Triggers:** New Task, Updated Task, Task Status Changed, New Comment
**Actions:** Create Task, Update Task, Delete Task, Get Task, List Tasks, Move Task, Create Comment, Get Comments, Create List, Create Space, Get Space, List Spaces, Get Folder, Set Task Status

### Todoist *(Tier 2)*
**Triggers:** New Task, Completed Task, New Project
**Actions:** Create Task, Update Task, Complete Task, Delete Task, Get Task, List Tasks, Create Project, Get Project, List Projects, Add Comment, List Labels

### Basecamp *(Tier 2)*
**Triggers:** New To-Do, New Message
**Actions:** Create To-Do, Complete To-Do, Create Message, Post Comment, List Projects, Get Project

---

## Calendar & Scheduling

### Google Calendar *(Tier 1)*
**Triggers:** New Event, Event Starting, Event Updated, Event Deleted
**Actions:** Create Event, Update Event, Delete Event, Get Event, List Events, Search Events, Create Calendar, List Calendars, Add Attendee, Get Free/Busy Info

### Microsoft Outlook Calendar *(Tier 1)*
**Triggers:** New Event, Event Starting, Event Updated
**Actions:** Create Event, Update Event, Delete Event, Get Event, List Events, Accept Event, Decline Event, List Calendars

### Calendly *(Tier 2)*
**Triggers:** Invitee Created, Invitee Cancelled
**Actions:** Get User, List Event Types, Get Scheduled Event, List Scheduled Events, Cancel Event, Create Single-Use Scheduling Link

### Cal.com *(Tier 2)*
**Triggers:** Booking Created, Booking Cancelled, Booking Rescheduled
**Actions:** Get Booking, List Bookings, Create Booking, Cancel Booking, List Event Types

---

## CRM & Sales

### HubSpot *(Tier 1)*
**Triggers:** New Contact, Updated Contact, New Deal, Deal Stage Changed, New Company, Form Submitted, New Ticket
**Actions:** Create Contact, Update Contact, Delete Contact, Get Contact, Search Contacts, Create Deal, Update Deal, Get Deal, List Deals, Create Company, Get Company, Create Ticket, Update Ticket, Add Note, Create Engagement, Associate Objects, List Properties, Create Property

### Salesforce *(Tier 1)*
**Triggers:** New Record, Updated Record (any object), New Lead, New Opportunity, Opportunity Stage Changed
**Actions:** Create Record, Update Record, Delete Record, Get Record, Query Records (SOQL), Search Records, Create Lead, Convert Lead, Create Opportunity, Create Case, Add Note, Upload File, Get Fields

### Pipedrive *(Tier 1)*
**Triggers:** New Deal, Updated Deal, Deal Stage Changed, New Person, New Organization, New Activity
**Actions:** Create Deal, Update Deal, Delete Deal, Get Deal, List Deals, Create Person, Update Person, Get Person, Create Organization, Create Activity, Get Activity, Add Note, Search Deals, Search Persons

### Zoho CRM *(Tier 2)*
**Triggers:** New Contact, New Lead, Updated Record
**Actions:** Create Record, Update Record, Delete Record, Get Record, Search Records, Convert Lead, Add Note

### Close *(Tier 2)*
**Triggers:** New Lead, Updated Lead, New Activity
**Actions:** Create Lead, Update Lead, Get Lead, List Leads, Create Contact, Create Note, Create Task, Send Email, List Activities

---

## Developer Tools & DevOps

### GitLab *(Tier 1)*
**Triggers:** New Push, New Merge Request, New Issue, Pipeline Status Changed, New Comment
**Actions:** Create Issue, Update Issue, Get Issue, List Issues, Create Merge Request, Merge a MR, Create Note, List Pipelines, Trigger Pipeline, Get Pipeline, Create Repository File, Get Repository File, List Branches, Create Tag, Get Project

### Bitbucket *(Tier 1)*
**Triggers:** New Push, New Pull Request, Pull Request Merged, New Issue
**Actions:** Create Issue, Get Issue, List Issues, Create Pull Request, Merge Pull Request, List Repos, Get Repository, List Branches, Create Branch, Get File Contents

### Vercel *(Tier 2)*
**Triggers:** Deployment Created, Deployment Succeeded, Deployment Failed
**Actions:** List Deployments, Get Deployment, Delete Deployment, List Projects, Get Project, Create Deployment, List Domains, Add Domain

### Netlify *(Tier 2)*
**Triggers:** Deploy Succeeded, Deploy Failed, Form Submission
**Actions:** List Sites, Get Site, Create Deploy, List Deploys, Get Deploy, Restore Deploy, Lock Deploy, List Forms, Get Form Submissions

### CircleCI *(Tier 2)*
**Triggers:** Workflow Completed, Job Completed
**Actions:** Trigger Pipeline, Get Pipeline, List Pipelines, Get Workflow, Cancel Workflow, Get Job, List Jobs, Get Artifacts

### Sentry *(Tier 2)*
**Triggers:** New Issue, Issue Resolved, New Error
**Actions:** Create Issue, Resolve Issue, Assign Issue, Get Issue, List Issues, List Projects, Get Project, Get Organization

### PagerDuty *(Tier 2)*
**Triggers:** New Incident, Incident Acknowledged, Incident Resolved, New Alert
**Actions:** Create Incident, Acknowledge Incident, Resolve Incident, Get Incident, List Incidents, Create Note, Get User, List Services, Create Override

### Datadog *(Tier 2)*
**Triggers:** New Monitor Alert, New Log Event
**Actions:** Post Metric, Create Event, Query Metrics, Get Monitor, Create Monitor, Mute Monitor, List Monitors, Search Logs

---

## Databases

### PostgreSQL *(Tier 1)*
**Actions:** Execute Query, Insert Row, Update Row, Delete Row, Select Rows, Execute Stored Procedure, Begin Transaction, Commit Transaction, Rollback Transaction

### MySQL *(Tier 1)*
**Actions:** Execute Query, Insert Row, Update Row, Delete Row, Select Rows, Execute Stored Procedure

### MongoDB *(Tier 1)*
**Actions:** Insert Document, Update Document, Delete Document, Find Documents, Find One, Aggregate, Count Documents, Create Index, Drop Collection

### Redis *(Tier 2)*
**Actions:** Get, Set, Delete, Expire, List Push, List Pop, List Range, Hash Get, Hash Set, Publish, Increment, Decrement

### Supabase *(Tier 1)*
**Triggers:** Database Row Inserted, Row Updated, Row Deleted (via Realtime)
**Actions:** Insert Row, Update Row, Delete Row, Select Rows, Execute RPC, Upload File (Storage), Download File, Delete File, List Files, Sign In User, Create User, Delete User

### Elasticsearch *(Tier 2)*
**Actions:** Index Document, Update Document, Delete Document, Get Document, Search, Count, Bulk Operations, Create Index, Delete Index, Get Mapping

---

## Cloud Storage & Files

### Google Drive *(Tier 1)*
**Triggers:** New File, New File in Folder, File Updated, File Deleted
**Actions:** Upload File, Download File, Create Folder, Move File, Copy File, Delete File, Get File Metadata, List Files, Search Files, Share File, Get File Permission, Create Google Doc, Create Google Sheet

### Dropbox *(Tier 1)*
**Triggers:** New File, Updated File, Deleted File
**Actions:** Upload File, Download File, Create Folder, Move File, Copy File, Delete File, Get File Metadata, List Folder, Search Files, Share Folder, Create Shared Link

### AWS S3 *(Tier 1)*
**Actions:** Upload Object, Download Object, Delete Object, Copy Object, List Objects, Get Object Metadata, Create Bucket, Delete Bucket, List Buckets, Generate Presigned URL, Set Object ACL

### Microsoft OneDrive *(Tier 1)*
**Actions:** Upload File, Download File, Create Folder, Move File, Delete File, Get File Metadata, List Files, Search Files, Share File

### Cloudinary *(Tier 2)*
**Actions:** Upload Image, Upload Video, Get Resource, Transform Image (resize/crop/format), Delete Resource, Create Folder, List Resources, Generate URL

---

## Spreadsheets

### Google Sheets *(Tier 1)*
**Triggers:** New Row, Updated Row, New Spreadsheet
**Actions:** Append Row, Get Row, Update Row, Delete Row, Clear Row, Get Spreadsheet, Create Spreadsheet, Add Sheet, Get Sheet Values, Update Sheet Values, Clear Sheet, Format Cells, Lookup Row

### Microsoft Excel 365 *(Tier 1)*
**Triggers:** New Row in Table, Updated Row in Table
**Actions:** Add Row to Table, Get Row, Update Row, Delete Row, List Rows, Create Workbook, Add Worksheet, Get Range, Update Range

---

## E-Commerce & Payments

### Stripe *(Tier 1)*
**Triggers:** Payment Succeeded, Payment Failed, Subscription Created, Subscription Updated, Subscription Cancelled, Customer Created, Invoice Created, Invoice Paid, Checkout Session Completed, Refund Created
**Actions:** Create Customer, Update Customer, Get Customer, Delete Customer, Create Charge, Capture Charge, Refund Charge, Create Payment Intent, Confirm Payment Intent, Create Subscription, Update Subscription, Cancel Subscription, Create Invoice, Finalize Invoice, Pay Invoice, Create Price, Create Product, List Customers, List Charges, List Subscriptions, Create Checkout Session, Retrieve Balance

### PayPal *(Tier 1)*
**Triggers:** Payment Completed, Subscription Activated, Subscription Cancelled
**Actions:** Create Order, Capture Order, Get Order, Create Subscription, Suspend Subscription, Cancel Subscription, List Subscriptions, Get Subscription, Create Invoice, Send Invoice, List Payments

### Shopify *(Tier 1)*
**Triggers:** New Order, Order Updated, Order Paid, Order Fulfilled, Order Cancelled, New Customer, Customer Updated, New Product, Product Updated, New Refund, Abandoned Checkout, New Draft Order
**Actions:** Create Order, Update Order, Cancel Order, Fulfill Order, Create Product, Update Product, Delete Product, Get Product, List Products, Create Customer, Update Customer, Get Customer, List Customers, Create Discount, Apply Discount, Get Inventory Level, Adjust Inventory, Create Metafield

### WooCommerce *(Tier 1)*
**Triggers:** New Order, Order Status Updated, New Customer, New Product
**Actions:** Create Order, Update Order, Get Order, List Orders, Create Product, Update Product, Delete Product, Get Product, Create Customer, Update Customer, Get Customer, List Customers, Get Order Item

### Square *(Tier 2)*
**Triggers:** New Payment, New Order, New Customer
**Actions:** Create Payment, Get Payment, List Payments, Create Customer, Update Customer, Get Customer, List Customers, Create Order, Update Order, Get Catalog Item, List Catalog Items, Create Invoice, Publish Invoice

---

## Marketing

### Mailchimp *(Tier 1)*
**Triggers:** New Subscriber, Unsubscribe, Profile Updated, Campaign Sent
**Actions:** Add/Update Subscriber, Remove Subscriber, Get Subscriber, List Subscribers, Create Campaign, Send Campaign, Schedule Campaign, Add Tag, Remove Tag, Create List, List Audiences, Add Note, Get Campaign Report

### SendGrid *(Tier 1)*
**Triggers:** Email Bounced, Email Opened, Email Clicked, Unsubscribe
**Actions:** Send Email, Send Template Email, Add Contact, Update Contact, Delete Contact, List Contacts, Add to List, Remove from List, Create List, Get Contact, Search Contacts

### ActiveCampaign *(Tier 2)*
**Triggers:** New Contact, Contact Tag Added, Automation Completed
**Actions:** Create Contact, Update Contact, Get Contact, Add Tag, Remove Tag, Create Deal, Update Deal, Add to List, Remove from List, Subscribe to Automation, Create Note

### ConvertKit *(Tier 2)*
**Triggers:** New Subscriber, Tag Added to Subscriber, Form Subscription
**Actions:** Add Subscriber, Update Subscriber, Get Subscriber, Tag Subscriber, Remove Tag, Add to Sequence, List Forms, List Tags, List Sequences, Broadcast Email

### Klaviyo *(Tier 2)*
**Triggers:** New Profile, Profile Updated, New Event
**Actions:** Create Profile, Update Profile, Get Profile, Track Event, Add to List, Remove from List, List Lists, List Profiles in List, Send Campaign, Create Campaign

---

## Customer Support

### Zendesk *(Tier 1)*
**Triggers:** New Ticket, Ticket Updated, Ticket Status Changed, New Comment
**Actions:** Create Ticket, Update Ticket, Delete Ticket, Get Ticket, List Tickets, Add Comment, Get Comments, Assign Ticket, Set Ticket Status, Create User, Get User, List Organizations

### Intercom *(Tier 1)*
**Triggers:** New Conversation, New Message, New User, User Event Triggered
**Actions:** Create Conversation, Reply to Conversation, Assign Conversation, Close Conversation, Snooze Conversation, Create Note, Create User (Contact), Update User, Delete User, Get User, List Users, Tag User, Untag User, Send Event

### Freshdesk *(Tier 1)*
**Triggers:** New Ticket, Ticket Updated, New Reply
**Actions:** Create Ticket, Update Ticket, Delete Ticket, Get Ticket, List Tickets, Add Reply, Add Note, Create Contact, Get Contact, List Contacts, List Agents

---

## Finance & Accounting

### QuickBooks *(Tier 2)*
**Triggers:** New Invoice, New Payment, New Customer, New Expense
**Actions:** Create Invoice, Send Invoice, Get Invoice, List Invoices, Create Customer, Get Customer, List Customers, Create Payment, Create Expense, Create Bill, Create Item, Get Item, List Items

### Xero *(Tier 2)*
**Triggers:** New Invoice, New Contact, New Payment
**Actions:** Create Invoice, Update Invoice, Get Invoice, List Invoices, Create Contact, Update Contact, Get Contact, List Contacts, Create Payment, Create Credit Note, Get Account, List Accounts

### Stripe (Billing) — see Stripe above

---

## AI & Machine Learning

### Anthropic Claude *(Tier 1)*
**Actions:** Messages (single-turn), Messages with Tools, Messages with Vision, Multi-turn Conversation, Count Tokens, Streaming Messages

### Azure OpenAI *(Tier 1 — Enterprise)*
**Note:** Uses per-customer resource URLs (`https://<resource>.openai.azure.com/openai/deployments/<deployment>`) and `api-key` header auth. Deployment name is a node-level setting.
**Actions:** Chat Completion, Chat Completion with Tools (function calling), Chat Completion with Vision, Create Embeddings, List Deployments

### Google Gemini *(Tier 1)*
**Actions:** Generate Content, Generate Content with Tools (function calling), Chat, Generate with Vision, Embed Text, Count Tokens, List Models

### Mistral *(Tier 2)*
**Actions:** Chat Completion, Chat Completion with Tools (function calling), Embedding, List Models

### Groq *(Tier 2)*
**Actions:** Chat Completion, Chat Completion with Tools (function calling), Transcribe Audio (Whisper)

### Cohere *(Tier 2)*
**Actions:** Generate Text, Chat with Tools (function calling), Embed Text, Rerank Documents, Classify Text, Detect Language, Summarize Text

### Hugging Face *(Tier 2)*
**Actions:** Text Generation, Text Classification, Token Classification, Question Answering, Summarization, Translation, Image Classification, Image-to-Text, Feature Extraction (embeddings)

### Replicate *(Tier 2)*
**Actions:** Run Model, Get Prediction, Cancel Prediction, List Models, Search Models

### Stability AI *(Tier 2)*
**Actions:** Text to Image, Image to Image, Image Upscale, Inpainting, Image to Video

### ElevenLabs *(Tier 2)*
**Actions:** Text to Speech, Speech to Speech, Get Voices, Add Voice, Delete Voice, Clone Voice, Get Models

### AssemblyAI *(Tier 2)*
**Triggers:** Transcription Completed
**Actions:** Transcribe Audio, Get Transcript, List Transcripts, Summarize Transcript, Detect Entities, Sentiment Analysis, Chapter Detection, Question & Answer

### Pinecone *(Tier 2)*
**Actions:** Create Index, Delete Index, List Indexes, Upsert Vectors, Query Vectors, Fetch Vectors, Delete Vectors, Describe Index Stats

### Weaviate *(Tier 2)*
**Actions:** Create Object, Get Object, Delete Object, Query (GraphQL), Near Text Search, Near Vector Search, Batch Import

---

## Analytics

### Google Analytics 4 *(Tier 1)*
**Actions:** Run Report, Run Realtime Report, Get Property, List Properties, Get Audience List, Create Audience List

### Mixpanel *(Tier 2)*
**Actions:** Track Event, Set User Properties, Increment Property, Alias User, Get Event Data, Query JQL, Export Events

### Amplitude *(Tier 2)*
**Actions:** Track Event, Identify User, Set User Properties, Get Cohort, Export Events

### Segment *(Tier 2)*
**Actions:** Track Event, Identify User, Group User, Alias User, Page View, Screen View

### PostHog *(Tier 2)*
**Actions:** Capture Event, Identify User, Group Identify, Feature Flag Enabled, Get Feature Flag, List Feature Flags

---

## Identity & Auth

### Auth0 *(Tier 2)*
**Actions:** Create User, Update User, Delete User, Get User, List Users, Assign Roles, Remove Roles, List Roles, Block User, Unblock User, Send Password Reset Email, Create Application, Get Application

### Okta *(Tier 2)*
**Actions:** Create User, Update User, Deactivate User, Delete User, Get User, List Users, Assign to Group, Remove from Group, Create Group, List Groups, Assign to Application

---

## Forms & Surveys

### Typeform *(Tier 2)*
**Triggers:** New Form Response
**Actions:** Get Form, List Forms, Get Response, List Responses, Delete Response, Create Form, Update Form, Create Webhook

### Jotform *(Tier 2)*
**Triggers:** New Form Submission
**Actions:** Get Form, List Forms, Get Submission, List Submissions, Delete Submission, Get Form Questions

### Tally *(Tier 2)*
**Triggers:** New Form Submission
**Actions:** List Forms, Get Form, Get Submissions

---

## Documents & Signatures

### Google Docs *(Tier 1)*
**Actions:** Create Document, Get Document, Update Document (insert/replace text), Delete Document, Export Document (PDF/DOCX)

### Docusign *(Tier 2)*
**Triggers:** Envelope Signed, Envelope Declined, Envelope Voided
**Actions:** Create Envelope (send for signature), Get Envelope, List Envelopes, Get Document from Envelope, Void Envelope, Create Template, Send Template

### HelloSign / Dropbox Sign *(Tier 2)*
**Triggers:** Signature Request Signed
**Actions:** Create Signature Request, Get Signature Request, List Signature Requests, Cancel Signature Request, Download Files

### PandaDoc *(Tier 2)*
**Triggers:** Document Completed, Document Viewed
**Actions:** Create Document, Send Document, Get Document, List Documents, Download Document, Create Contact, Delete Document

---

## HR & People Ops

### BambooHR *(Tier 2)*
**Triggers:** New Employee, Employee Updated, Time-Off Request
**Actions:** Get Employee, List Employees, Create Employee, Update Employee, Get Time-Off Requests, Approve Time-Off, Deny Time-Off, Get Job Titles

### Greenhouse *(Tier 2)*
**Triggers:** New Application, Application Advanced, New Candidate
**Actions:** Create Candidate, Get Candidate, List Candidates, Create Application, Update Application, Move Application Stage, Add Note, List Jobs, Get Job

### Gusto *(Tier 2)*
**Actions:** List Employees, Get Employee, Create Employee, List Pay Schedules, Get Company, List Locations

---

## Cloud Provider Services

### AWS *(Tier 1)*
**Services & Actions:**
- **S3**: Upload Object, Download Object, Delete Object, List Objects, Create Presigned URL, Create Bucket
- **Lambda**: Invoke Function, List Functions, Get Function
- **SES**: Send Email, Send Template Email, Verify Email, List Identities
- **SNS**: Publish Message, Create Topic, Subscribe, List Topics, List Subscriptions
- **SQS**: Send Message, Receive Message, Delete Message, Create Queue, List Queues
- **DynamoDB**: Put Item, Get Item, Update Item, Delete Item, Query, Scan, Batch Write
- **Secrets Manager**: Get Secret, Create Secret, Update Secret, Delete Secret

### Google Cloud *(Tier 2)*
**Services & Actions:**
- **Pub/Sub**: Publish Message, Create Subscription, Acknowledge Message, Pull Messages
- **Cloud Functions**: Invoke Function, List Functions
- **BigQuery**: Run Query, Insert Rows, Create Dataset, Create Table, List Tables
- **Firestore**: Add Document, Get Document, Update Document, Delete Document, Query Documents

### Azure *(Tier 2)*
**Services & Actions:**
- **Service Bus**: Send Message, Receive Message, Delete Message, Create Queue, Create Topic
- **Blob Storage**: Upload Blob, Download Blob, Delete Blob, List Blobs, Create Container

---

## CMS & Content

### WordPress *(Tier 2)*
**Triggers:** New Post, New Comment, New User
**Actions:** Create Post, Update Post, Delete Post, Get Post, List Posts, Create Page, Get Page, Create Comment, Get Comment, Create User, Get User, Upload Media

### Webflow *(Tier 2)*
**Triggers:** Form Submission, Site Published
**Actions:** Create CMS Item, Update CMS Item, Delete CMS Item, Get CMS Item, List CMS Items, List Collections, Publish Site, Create Order

### Ghost *(Tier 2)*
**Triggers:** Post Published
**Actions:** Create Post, Update Post, Delete Post, Get Post, List Posts, Create Member, Update Member, Delete Member, Get Member, List Members

### Contentful *(Tier 2)*
**Triggers:** Entry Published, Asset Published
**Actions:** Create Entry, Update Entry, Publish Entry, Archive Entry, Delete Entry, Get Entry, Search Entries, Create Asset, Publish Asset, Get Asset

### Strapi *(Tier 2)*
**Actions:** Create Entry, Update Entry, Delete Entry, Get Entry, List Entries, Upload Media, Get Media

---

## Monitoring & Observability

### Sentry *(Tier 2)* — see Developer Tools above

### New Relic *(Tier 2)*
**Triggers:** New Alert Incident
**Actions:** Create Deployment Marker, Get APM Application, List Applications, Query NRQL, Create Custom Event

### Datadog *(Tier 2)* — see Developer Tools above

### PagerDuty *(Tier 2)* — see Developer Tools above

---

## Utilities & Data

### RSS / Atom Feeds *(Tier 2)*
**Triggers:** New Feed Item
**Actions:** Get Feed, Parse Feed (returns items)

### HTML Parser / Web Scraper *(Tier 2)*
**Actions:** Extract from HTML (CSS selector), Extract Links, Extract Images, Get Page HTML

### XML Parser *(Tier 1)*
**Actions:** Parse XML to Object, Object to XML, XPath Query

### PDF *(Tier 2)*
**Actions:** Extract Text, Merge PDFs, Split PDF, Convert HTML to PDF

### JSON Utilities *(Tier 1)*
**Actions:** JSONata Transform, JSON Schema Validate, Flatten Object, Deep Merge Objects

### Crypto / Hashing *(Tier 2)*
**Actions:** Hash (MD5/SHA1/SHA256/SHA512), HMAC, Base64 Encode/Decode, UUID Generate, Generate Random String

### Date / Time *(Tier 1)*
**Actions:** Format Date, Parse Date, Add/Subtract Duration, Convert Timezone, Diff Dates, Is Before/After, Start/End of Day/Week/Month

### Text *(Tier 1)*
**Actions:** Trim, Split, Replace, Substring, Regex Match, Regex Replace, Lowercase, Uppercase, Truncate, Slugify, Count Words, Pad String

---

## Webhooks & APIs

### Generic Webhook *(Tier 1)*
**Triggers:** Receive Webhook (any HTTP method), Receive & Validate (HMAC signature)
**Actions:** — (handled by core HTTP action)

### GraphQL Client *(Tier 1)*
**Actions:** Execute Query, Execute Mutation, Execute Subscription (streaming), Introspect Schema

### WebSocket Client *(Tier 2)*
**Actions:** Connect, Send Message, Disconnect
**Triggers:** Message Received

---

## Summary

| Tier | Platforms | Priority                                            |
| ---- | --------- | --------------------------------------------------- |
| 1    | ~40       | Build first — needed for initial launch credibility |
| 2    | ~90       | Needed for competitive feature parity               |
| 3    | ~50       | Nice-to-have for long-tail users                    |

## Recommended Build Order (Phase 1 MVP)

1. Slack
2. Gmail
3. Google Sheets
4. Notion
5. Airtable
6. Stripe
7. HubSpot
8. PostgreSQL
9. MySQL
10. MongoDB
11. Supabase
12. Google Drive
13. Anthropic Claude
14. Jira
15. Trello
16. Asana
17. Linear
18. Google Calendar
19. Twilio
20. Discord
21. Microsoft Teams
22. Google Docs
23. AWS (S3 + Lambda + SES + SQS + SNS + DynamoDB)
24. GitLab
25. Bitbucket
26. XML Parser
27. JSON Utilities
28. Date/Time
29. Text Utilities
30. GraphQL Client
