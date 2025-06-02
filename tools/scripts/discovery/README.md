# Discovery CLI Tool

A command-line tool for discovering and listing services in cloud environments, currently supporting AWS Lambda services.

## Features

- Authentication with Auth0 for secure access
- Support for AWS IAM Role assumption using web identity tokens
- Discovery of Lambda services in AWS regions
- Structured output of service configurations

## Changes Made

The following changes were made to fix errors and implement the Auth0 integration:

1. **Auth0 Integration**:
   - Integrated the Auth0 authentication from the identity package into the list command
   - Added token-based authentication flow for AWS operations

2. **Error Handling Improvements**:
   - Fixed error propagation in AWS service catalog functions
   - Added better error messaging throughout the codebase

3. **Command Structure**:
   - Improved the main command execution flow
   - Enhanced region handling with better error reporting

4. **Code Cleanup**:
   - Removed unused imports
   - Improved documentation

## Usage

```
./discovery list [region] [roleArn]
```

Where:
- `region`: AWS region (e.g., "US-EAST-1" or "ALL")
- `roleArn`: AWS IAM Role ARN to assume

## Authentication Flow

1. CLI triggers Auth0 authentication flow when you run the list command
2. Browser opens for you to authenticate with Auth0
3. After successful authentication, the token is used to assume the AWS role
4. Service discovery proceeds with the assumed role's permissions

## Supported AWS Services

Currently, the tool discovers:
- Lambda functions and their configurations

## Examples

Discover Lambda functions in US-EAST-1 region:
```
./discovery list US-EAST-1 arn:aws:iam::123456789012:role/YourRoleName
```

Discover Lambda functions in all supported regions:
```
./discovery list ALL arn:aws:iam::123456789012:role/YourRoleName
```