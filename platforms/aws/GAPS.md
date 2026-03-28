# AWS — Integration Gaps
## All Services (S3, Lambda, SES, SQS, SNS, DynamoDB, Secrets Manager)
- **Why omitted:** AWS strictly relies on SigV4 Request Authentication, which requires complex cryptographic signing of headers and canonical bodies iteratively. Our standard HTTP primitive cannot magically sign these requests via raw configuration natively.
- **Value:** Very High.
- **Unblocked by:** Creation of an `aws_client` primitive or a `sigv4_sign` built-in tool step.
