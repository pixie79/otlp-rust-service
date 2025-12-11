# Security Policy

## Supported Versions

We provide security updates for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.4.x   | :white_check_mark: |
| < 0.4.0 | :x:                |

## Security Model

The OTLP Rust Service is designed with security as a core principle. Our security model includes:

### Authentication & Authorization

- **Credential Storage**: All authentication credentials are stored using `secrecy::SecretString`, which ensures:
  - Credentials are zeroed in memory when dropped
  - Credentials never appear in `Debug` or `Display` implementations
  - Credentials are not exposed in logs, error messages, or memory dumps

- **Authentication Methods**: Supports API key, bearer token, and basic authentication for remote forwarding
- **No Built-in Authorization**: The service does not implement user-level authorization. Access control should be handled at the network/infrastructure level

### Network Security

- **Dashboard Server**: By default, binds to `127.0.0.1` (localhost only) to prevent network exposure
- **gRPC Servers**: Bind to configured addresses (default: all interfaces for gRPC protocols)
- **Path Traversal Protection**: Comprehensive path validation prevents directory traversal attacks:
  - Rejects absolute paths
  - Rejects paths with `..` components
  - Rejects Windows UNC paths
  - Normalizes paths and resolves symlinks safely
  - Verifies canonical paths stay within allowed directories

### Data Protection

- **In-Memory Security**: Credentials use secure string types that zero memory on drop
- **Buffer Limits**: Configurable buffer size limits prevent unbounded memory growth
- **Input Validation**: Comprehensive URL and input validation using standard libraries (`url` crate)

### HTTP Security Headers

The dashboard HTTP server includes security headers:
- `Content-Security-Policy: default-src 'self'` - Prevents XSS attacks
- `X-Frame-Options: DENY` - Prevents clickjacking (configurable)
- `X-Content-Type-Options: nosniff` - Prevents MIME type sniffing
- `X-XSS-Protection: 1; mode=block` - Additional XSS protection

## Threat Model

### Threats Addressed

1. **Credential Exposure**
   - **Threat**: Credentials exposed in logs, errors, or memory dumps
   - **Mitigation**: `SecretString` ensures credentials are never exposed

2. **Path Traversal Attacks**
   - **Threat**: Accessing files outside intended directories via path manipulation
   - **Mitigation**: Comprehensive path validation and canonicalization

3. **Memory Exhaustion**
   - **Threat**: Unbounded memory growth leading to denial of service
   - **Mitigation**: Configurable buffer size limits with backpressure

4. **XSS Attacks**
   - **Threat**: Cross-site scripting attacks via dashboard
   - **Mitigation**: Content-Security-Policy and X-XSS-Protection headers

5. **Clickjacking**
   - **Threat**: Embedding dashboard in malicious iframes
   - **Mitigation**: X-Frame-Options header (configurable)

6. **MIME Type Sniffing**
   - **Threat**: Browser incorrectly interpreting content types
   - **Mitigation**: X-Content-Type-Options header

### Threats Not Addressed

1. **Network-Level Attacks**: The service does not implement DDoS protection, rate limiting, or network-level encryption. These should be handled by infrastructure (load balancers, firewalls, TLS termination)

2. **User-Level Authorization**: The service does not implement user authentication or authorization. Access control should be handled at the network/infrastructure level

3. **Data Encryption at Rest**: Arrow IPC files are stored unencrypted. If encryption is required, use filesystem-level encryption or encrypt files before storage

4. **Audit Logging**: The service logs operations but does not provide comprehensive audit trails. For compliance requirements, integrate with external logging/audit systems

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security vulnerability, please follow these steps:

### Reporting Process

1. **DO NOT** open a public GitHub issue for security vulnerabilities
2. **DO** email security concerns to: [security@your-org.com] (replace with actual security contact)
3. Include the following information:
   - Description of the vulnerability
   - Steps to reproduce (if applicable)
   - Potential impact
   - Suggested fix (if available)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity:
  - **Critical**: Fix within 7 days
  - **High**: Fix within 30 days
  - **Medium/Low**: Fix in next release cycle

### Disclosure Policy

- We will acknowledge receipt of your report within 48 hours
- We will keep you informed of the progress toward resolving the issue
- We will credit you in the security advisory (unless you prefer to remain anonymous)
- We will coordinate public disclosure after a fix is available

## Security Best Practices

### Configuration

1. **Dashboard Access**: 
   - Keep dashboard disabled in production unless needed
   - If enabled, bind to `127.0.0.1` (localhost) or use a reverse proxy with authentication
   - Never expose dashboard to public internet without proper authentication

2. **Buffer Limits**:
   - Configure `max_trace_buffer_size` and `max_metric_buffer_size` based on available memory
   - Monitor memory usage and adjust limits as needed
   - Default limits (10,000) are reasonable for most use cases

3. **Output Directory**:
   - Use a dedicated directory with appropriate permissions
   - Ensure the directory is not accessible via web servers
   - Consider filesystem-level encryption for sensitive data

4. **Authentication Credentials**:
   - Store credentials in environment variables or secure secret management systems
   - Never commit credentials to version control
   - Rotate credentials regularly

5. **Network Security**:
   - Use TLS/HTTPS for remote forwarding endpoints
   - Restrict network access using firewalls
   - Use VPN or private networks for internal communication

### Runtime Security

1. **Principle of Least Privilege**: Run the service with minimal required permissions
2. **Resource Limits**: Set appropriate memory and CPU limits
3. **Monitoring**: Monitor for:
   - Buffer full errors (may indicate need for higher limits or faster writes)
   - Circuit breaker state changes (may indicate remote endpoint issues)
   - Path validation failures (may indicate attack attempts)

### Development Security

1. **Dependencies**: Keep dependencies up to date
2. **Code Review**: All security-related changes require review
3. **Testing**: Run security tests before releases
4. **Documentation**: Keep security documentation up to date

## Known Security Considerations

1. **Python Bindings**: The Python bindings use PyO3, which requires careful memory management. We've implemented proper reference counting and GIL handling, but segfaults may still occur if Python objects are misused. Always use the library through the provided Python API.

2. **Arrow IPC Files**: Files are stored in plain Arrow IPC format. If encryption is required, implement filesystem-level encryption or encrypt files before storage.

3. **Dashboard**: The dashboard serves static files and Arrow IPC files. Ensure the output directory contains only intended data and is not accessible via other means.

4. **Remote Forwarding**: When forwarding to remote endpoints, ensure:
   - Endpoints use TLS/HTTPS
   - Authentication credentials are properly secured
   - Circuit breaker is configured appropriately to prevent cascading failures

## Security Updates

Security updates are released as needed. Critical security fixes may be backported to previous versions. Check the [CHANGELOG.md](CHANGELOG.md) for security-related changes.

## Security Contact

For security concerns, please contact: [security@your-org.com] (replace with actual security contact)

---

**Last Updated**: 2025-01-27

