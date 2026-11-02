# Verified synthetic WASM fixture (OP-5756)

This is an internal, non-publishable fixture, not a customer connector. The only
functional action, `echo_contract`, deterministically returns its fixed JSON input
plus `environment_count: 0`. Input/output contracts are in `schemas/`: a nonempty
reference, and up to eight boolean, null or safe-integer values. There are no
credentials, accounts, triggers, external services or platform imports.

Three additional diagnostic actions intentionally attempt forbidden HTTP, consume
CPU indefinitely, or exhaust memory. **Never load this fixture on the host or in
a platform App/Version.** Even schema extraction executes WASM. Run only through
the restricted verifier below.

## Reproduce

Requires existing Docker and Python 3; image construction downloads public Rust
and Ruby dependencies. Commit the source first (development runs may explicitly
use `--allow-dirty`, but their evidence is not accepted as final).

```sh
docker build -f verification/build.Dockerfile -t lg1365-wasm-builder:1.93.1 verification
docker build -f verification/runtime.Dockerfile -t lg1365-wasm-runtime:5.0.0 verification
python3 verification/verify.py --output .verification-output/final
```

The verifier stages only fixture source, schemas and the repository's v5 WIT,
then performs one locked release build. It records image IDs and uses the same
read-only WASM bytes for every test in real App Bridge 5.0.0. No mock adapter or
platform database is used. Contract discovery, deterministic output, false/null,
invalid input, unknown action, denied real WASM HTTP, CPU deadline, memory OOM,
and rejection of a wrong digest before loading are checked. An offline copy is
also tampered with to verify substitution rejection without executing it.

Each execution gets a separate disposable non-root container with no network,
read-only root and binds, all capabilities dropped, no-new-privileges, bounded
memory/swap, CPU and PIDs, and an external process deadline. Docker settings are
inspected before starting; the worker requires no active non-loopback interface
or external IPv4 route. Inactive Docker Desktop tunnel devices are permitted.
The network test additionally requires a host socket to fail as unreachable and
the actual App Bridge HTTP import to fail. No privileged container, host socket,
customer environment or app environment is mounted. The runtime entrypoint clears
inherited environment; WASM receives an explicit empty environment. Ruby Timeout
alone cannot reliably interrupt native WASM; the container/process boundary does.
Only verifier-created containers are removed. Failed runs retain failed evidence
and never produce a passing manifest.

## Bundle contract

`manifest.json` (`integrationer.verified-wasm-fixture.v1`) binds source commit,
artifact SHA-256/size, canonical JSON contract digest and test-result file digest.
`tests.json` records real outcomes, resource exits and isolation settings.
`connector.wasm`, `contract.json` and `build.log` are included. `source_dirty` must
be false for final evidence; `test_only` is true and `publishable` is false.

Offline consumers can check a selected bundle without loading its WASM:

```sh
python3 verification/check_bundle.py .verification-output/final \
  --expected-artifact <independently-selected-sha256> --expected-commit <source-commit>
```

These are **unsigned integrity checks**, not authentication of source, builder,
runtime or test author. A malicious party can replace both content and unsigned
manifest; trusted provenance/signing and release policy remain separate work.
The reviewed App Bridge source commit is contextual evidence, not a claim that
the public gem was reproducibly built from that commit. Container image IDs bind
the actual local runtime used. Tool versions are pinned, but base image tags and
transitive Ruby dependencies are not a cross-platform reproducible-build promise;
ARM and x86 builds may differ. The guarantee here is testing the same built bytes
within a run, with a repeatable verification procedure.

## CI and remaining scope

`verify-synthetic.yml` builds/verifies and retains a test-only Actions artifact for
14 days, with read-only repository permission and no checkout credential retained.
It does not publish a GitHub release, platform connector, version or `latest`.
The existing release workflow is unchanged and still builds separately: this slice
is **not yet a release gate** and does not complete all OP-5756 acceptance criteria.
OP-5672's shared-workflow/public-private access question remains unresolved.

Later staged import must not use real App/Version schema inspection as read-only:
it executes WASM with app environment variables, and current app-key/version-name
cache identity is not a content digest. Also account for App creation producing
1.0.0 with `latest: true` and controller promotion behavior. This PR changes none
of those semantics. No signing keys, new grants, platform publication or release
workflow changes are required for this isolated artifact slice.
