# AWS Integration Guide

Connects to Amazon Web Services (AWS) to trigger Lambda functions and list S3 objects.

## Setup & Authentication
1. Generate an Access Key and Secret Key in the [AWS Console (IAM)](https://console.aws.amazon.com/iam).
2. Attach the required policies: `lambda:InvokeFunction`, `s3:ListBucket`.
3. In FerroFlux, create a new Connection with the following configuration:
    - `access_key_id`: `YOUR_ACCESS_KEY`
    - `secret_access_key`: `YOUR_SECRET_KEY`
    - `region`: `YOUR_AWS_REGION` (e.g., `us-east-1`).

## Available Actions

### `lambda.invoke`
Invokes an AWS Lambda function synchronously.
- **Key Inputs**: 
    - `function_name`: The name or ARN of the Lambda function.
    - `payload`: (Optional) A JSON object to pass to the function.
- **Outputs**: 
    - `status_code`: HTTP status code from Lambda.
    - `payload`: The function response.

### `s3.list_objects`
Lists objects in an S3 bucket (V2).
- **Key Inputs**: 
    - `bucket`: Bucket name.
    - `prefix`: (Optional) Folder path.
- **Outputs**: 
    - `objects`: An array of object metadata.

## Examples (WAML)

### Invoking a Lambda Function
```waml
- step: process_data
  call: aws.lambda.invoke
  with:
    function_name: "my-data-processor"
    payload:
      user_id: inputs.user_id
      source: "ferroflux"
```

### Listing Bucket Folders
```waml
- step: list_logs
  call: aws.s3.list_objects
  with:
    bucket: "my-app-logs"
    prefix: "2024/"
```
```
