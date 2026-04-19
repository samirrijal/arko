# api/

Rust Axum API server. Implements `specs/api/`.

**Status:** Not started. Will be scaffolded after the engine core is
solving real studies and the API spec has a first draft.

## Planned stack

- **Framework:** Axum 0.7+
- **Database access:** SQLx with compile-time checked queries against Postgres 16.
- **Auth:** Keycloak OIDC (self-hosted per C1).
- **Background jobs:** NATS JetStream (durable, self-hosted).
- **Storage:** MinIO (S3-compatible, self-hosted) for study blobs and result artifacts.
- **Observability:** OpenTelemetry traces, VictoriaMetrics metrics, structured logs.

## Why Axum

- **Tower middleware ecosystem** — auth, rate limiting, tracing are composable.
- **Compile-time type safety** from handler signatures through to the wire.
- **Async I/O without async surprises** — Tokio runtime is production-proven.
- **Small binary, fast startup** — matters for self-hosted multi-tenant deployment.
