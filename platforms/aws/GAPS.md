# AWS — Integration Gaps

## Resolved
- **SigV4 Authentication:** Now supported natively by the `http_client` tool primitive using the `aws_sigv4` helper.
- **S3 Operations:** Basic S3 operations (List Objects) available as YAML nodes.
- **Lambda Operations:** Invoke Lambda available as a YAML node.

## Remaining Gaps
- **IAM / Role Switching:** No support for `AssumeRole` or cross-account access via the UI.
- **Binary S3 Uploads:** `http_client` needs to better handle binary DataRefs for S3 PUT operations.
- **Broad Service Coverage:** Many AWS services (SQS, SNS, DynamoDB) still need their specific YAML node mappings.

## System-Level Gaps
- Refer to `SYSTEM_GAPS.md` for global engine limitations.
