# AWS S3 Integration Guide

Connects to Amazon S3 (Simple Storage Service) for managing buckets and objects.

## Setup & Authentication
1. Generate an Access Key and Secret Key in the [AWS Console (IAM)](https://console.aws.amazon.com/iam).
2. In FerroFlux, create a new Connection with the following configuration:
    - `access_key_id`: `YOUR_ACCESS_KEY`
    - `secret_access_key`: `YOUR_SECRET_KEY`
    - `region`: `YOUR_AWS_REGION` (e.g., `us-east-1`).
    - `endpoint`: (Optional) Custom S3-compatible endpoint (e.g., MinIO, DigitalOcean).

## Available Actions

### `objects.upload`
Uploads a file or content to an S3 bucket.
- **Key Inputs**: 
    - `bucket`: Bucket name.
    - `key`: Object key (file path in bucket).
    - `content`: Base64-encoded file content or text.
    - `content_type`: (Optional) MIME type.

### `objects.download`
Retrieves an object from an S3 bucket.
- **Key Inputs**: 
    - `bucket`, `key`.
- **Outputs**: 
    - `content`: Base64-encoded file content.

### `objects.list`
Lists objects in an S3 bucket.
- **Key Inputs**: 
    - `bucket`, `prefix` (Optional folder path).
- **Outputs**: 
    - `objects`: An array of object metadata.

### `objects.delete`
Deletes an object from an S3 bucket.

### `buckets.create`
Creates a new S3 bucket in a specified region.

## Examples (WAML)

### Uploading a Report
```waml
- step: upload_report
  call: s3.objects.upload
  with:
    bucket: "reports-bucket"
    key: "daily-report-2024.pdf"
    content: steps.generate_pdf.content
    content_type: "application/pdf"
```

### Listing Files in a Folder
```waml
- step: list_images
  call: s3.objects.list
  with:
    bucket: "assets-bucket"
    prefix: "images/"
```
```
