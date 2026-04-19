# API specification

**Status:** Not started.

Will cover the public HTTP/JSON API surface:

- Resource model (workspaces, studies, models, libraries, results).
- Authentication (Keycloak OIDC).
- Authorization (workspace membership + row-level security).
- Versioning and deprecation policy.
- Rate limits and quota semantics.
- Webhook model for long-running calculations.
- OpenAPI 3.1 machine-readable document.
- SDK generation contract (TypeScript, Python, Rust clients).

## Guiding principle

**The UI has no private endpoints.** Every action the Arko web app performs
is a documented, callable API call. This is commitment **C6**.

Target ratification: alongside first public API release (beta).
