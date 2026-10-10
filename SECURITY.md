# RustPdf — Security Policy & Hardening

Security and privacy are core architectural requirements of **RustPdf**.

---

## 1. Threat Model

PDF files are treated as **untrusted, hostile inputs**. Malicious or malformed PDFs may attempt:
- Decompression bombs (zip bombs / nested flate streams).
- Pathological dimensions or infinite recursion in cross-reference tables.
- Arbitrary code execution via embedded JavaScript.
- Path traversal through attachment extraction paths.

---

## 2. Security Mitigations

### Memory Safety & Pure Rust
- All core document parsing, object traversal, geometry math, and rendering are written in safe Rust.
- No linked C/C++ dependencies (zero vulnerabilities from buffer overflows or use-after-free).

### Bounded Resource Limits
- **Page Rasterization Caps**:
  - `MAX_SIDE = 8192.0` points.
  - `MAX_PIXELS = 64,000,000` pixels (~256 MB RGBA buffer).
  - Any scale or requested dimension exceeding these bounds is automatically clamped.
- **Decompression Bounds**: Stream filters limit decompressed size to prevent memory exhaustion bombs.
- **Cycle Prevention**: Object and outline traversals track visited ObjectIds with depth limits (max depth 32) to prevent infinite loops in cyclic trees.

### Script & Attachment Isolation
- **No JavaScript Execution**: Embedded JavaScript actions (`/JS`) are never executed.
- **No Automatic Launching**: Attachments and external launch actions (`/Launch`) are strictly forbidden from automatic execution.

### Privacy & Offline Guarantees
- **No Telemetry**: No tracking, phone-home beacons, or usage analytics.
- **No Cloud Processing**: All operations (rendering, search, combine, split, edit) occur strictly on the user's local CPU.
- **Zero Network Sockets**: The desktop application and CLI do not bind or listen on any network ports.

---

## 3. Reporting Vulnerabilities

If you discover a security issue or vulnerability in RustPdf, please report it via GitHub Security Advisories or contact the maintainers directly. Do not open public issues for unpatched vulnerabilities.
