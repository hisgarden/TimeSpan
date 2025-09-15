# Security Policy

## Software Bill of Materials (SBOM)

TimeSpan is built with security in mind, using carefully selected dependencies with active maintenance and security monitoring.

### Core Dependencies

| Component | Version | Purpose | CVE Status | Last Updated |
|-----------|---------|---------|------------|--------------|
| **Rust Core** | 1.75+ | Language Runtime | ✅ No known CVEs | Active |
| **tokio** | 1.47.1 | Async Runtime | ✅ No known CVEs | Active |
| **clap** | 4.5.47 | CLI Parsing | ✅ No known CVEs | Active |
| **rusqlite** | 0.30.0 | SQLite Database | ✅ No known CVEs | Active |
| **chrono** | 0.4.42 | Date/Time Handling | ⚠️  Monitor | Active |
| **serde** | 1.0.223 | Serialization | ✅ No known CVEs | Active |
| **git2** | 0.18.3 | Git Integration | ⚠️  Monitor | Active |
| **uuid** | 1.18.1 | UUID Generation | ✅ No known CVEs | Active |
| **thiserror** | 1.0.69 | Error Handling | ✅ No known CVEs | Active |
| **anyhow** | 1.0.99 | Error Context | ✅ No known CVEs | Active |

### System Dependencies

- **SQLite** (bundled via rusqlite)
- **OpenSSL** (system or bundled)
- **Git libraries** (libgit2)

### Security Considerations

#### ✅ **Low Risk Areas**
- **Local-only data**: All data stored locally, no network transmission
- **No external APIs**: No remote service dependencies
- **Minimal attack surface**: CLI application with file system access only
- **Memory safety**: Rust prevents buffer overflows and memory corruption
- **Type safety**: Compile-time prevention of many vulnerability classes

#### ⚠️ **Areas to Monitor**

1. **File System Access**
   - TimeSpan reads/writes local SQLite database
   - Git integration accesses local repositories
   - Mitigation: Input validation, path sanitization

2. **Git Integration** 
   - Uses libgit2 through git2 crate
   - Could expose to malicious git repositories
   - Mitigation: Read-only operations, path validation

3. **CLI Input Processing**
   - Command-line argument parsing via clap
   - User input validation required
   - Mitigation: Input sanitization, type checking

## Vulnerability Monitoring

### Automated Checks
- **Dependabot**: GitHub automated dependency updates
- **Cargo Audit**: Regular security audits of Rust dependencies
- **RUSTSEC**: Rust Security Advisory Database monitoring

### Manual Reviews
- Monthly dependency version reviews
- Quarterly security assessment
- Input validation testing

## Reporting Security Issues

If you discover a security vulnerability in TimeSpan:

1. **DO NOT** create a public GitHub issue
2. Email: [jin.wen@hisgarden.org](mailto:jin.wen@hisgarden.org)
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact assessment
   - Suggested fixes (if any)

Response time: 48 hours for acknowledgment, 7 days for assessment.

## Security Hardening

### Build Security
- Reproducible builds
- Dependency pinning
- Supply chain verification

### Runtime Security  
- Minimal privileges required
- No network access needed
- Sandboxing compatible

### Data Security
- Local SQLite database only
- No sensitive data transmission
- User controls all data

---

**Last Updated**: 2025-09-15  
**Next Review**: 2025-12-15