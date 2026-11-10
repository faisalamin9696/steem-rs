# Security Policy

## Supported versions

The latest published `0.x` release receives security fixes. Because the crate
is pre-1.0, the API may change between minor versions.

## Reporting a vulnerability

Please do not open a public issue for a suspected vulnerability. Report it
privately to the maintainers through the repository's security advisory form
or by email to the address listed in the repository profile. Include:

- affected crate and version,
- a description of the issue,
- a minimal reproduction or test vector if possible.

We will acknowledge the report, assess impact, and coordinate a fix and
disclosure timeline.

## Scope

Security-sensitive areas include:

- private key parsing, zeroization and signing,
- transaction serialization and digest construction,
- memo encryption/decryption,
- authority weight evaluation,
- the RPC client's handling of untrusted responses.

The Rust crates never transmit private keys. The RPC crate only accepts
already-signed transactions.
