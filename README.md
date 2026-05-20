# What it is

> **Status (v0.3.0, 2026-05-11)**: Audit-remediation work from the [2026-05-09 internal audit](./audits/2026-05-09-internal-audit.md) is complete — all Critical, High, and 4 of 5 Medium findings closed (see [`audits/2026-05-09-fixes.md`](./audits/2026-05-09-fixes.md) for the finding-to-commit map and [`audits/2026-05-09-handoff.md`](./audits/2026-05-09-handoff.md) for the v0.3.0 readiness summary). ECDSA, BLS, and Mirror families ship real cryptography; the Mock variant was removed. External audit and the comprehensive ECDSA/BLS test buildout are tracked as v0.3.x follow-up.

This repo is for Wavs Cosmwasm middleware

It's constructed so that developing and testing any combination of the following is as smooth as possible

This README is focused on the testing/development story, for information on how to use the middleware in your own project via the docker image, see [USAGE.md](docs/USAGE.md)

**Contracts**

* ECDSA (Service Handler and Service Manager) — standalone POA-shape stake registry; secp256k1/EIP-191 signatures
* BLS (Service Handler and Service Manager) — standalone POA-shape stake registry; BLS12-381 aggregate signatures via cosmwasm-crypto host calls
* Mirror (Service Handler, Service Manager, Stake Registry, Operator Sync Handler, Quorum Sync Handler) — bridges an EVM-side operator set into a CosmWasm chain via signed envelopes
* Trigger (Simple) — optional `allowed_pushers` allowlist at instantiate

**Chains**

* Any cosmos chain (e.g. Neutron, Juno, CosmosHub, etc.)

**Environment**

* Multitest / off-chain
* Local on-chain
* Remote on-chain

Code is generally shared between all these different requirements. So, for example, it's very easy to switch between ecdsa and bls flavors for common tests that hit "wavs service" code.

Functionality can also be shared between tests and non-tests, such as CLI and components

Additionally, a docker image is provided (TODO!) such that consumers can bring the middleware into their project with ease.

# What it is not

This is not a repo for developing Wavs/Cosmwasm components, triggers, or other non-middleware contracts. While we have extra goodies in this repo for testing, it's purely to fulfill that need, not meant for public consumption. Check [wavs-tools](https://github.com/Lay3rLabs/wavs-tools) for that.

# Getting Started

## Prerequisites

1. The usual stuff (Rust, Docker, NPM, etc.)
2. [Taskfile](https://taskfile.dev/installation)
3. [Install Starship v1](https://docs.hyperweb.io/starship#quick-start-guide)

## Secrets

You'll need wallets and signing keys along with their chain-specific addresses:

```bash
task cli:generate-env
```

_the first time the CLI runs it will need to compile the binary, be patient.. subsequent runs will be faster_

Copy `.example.env` to `.env` and replace the values

## ContractKind and ChainName

These are set in [.env](.example.env) to support local development workflows without committing to the repo.

* `CONTRACT_KIND` will affect build and test commands
* `CHAIN_KEY` will affect on-chain test commands

## Building

#### Contracts

```bash
task contracts:build
```

Some more contract building commands:

* `task contracts:build-all`: build all the different kinds of contracts
* `task contracts:build-service-handler`: build just the service handler for the current kind
* `task contracts:build-service-manager`: build just the service handler for the current kind
* `task contracts:build-trigger-simple`: build just the simple trigger (this is re-used for all kinds of tests)


#### Components

```bash
task components:build-all
```

These components are the same regardless of which contract kind or chain we're targeting.

Some more component building commands:

* `task components:bindings-all`: generate the bindings for all components
* `task components:build-echo-with-id`: build just the echo-with-id component
* `task components:bindings-echo-with-id`: generate bindings for just the echo-with-id component


## Testing


### Contracts

*off-chain*

```bash
# Test the currently configured CONTRACT_KIND
task test:contracts-off-chain
# Test all contract kinds (ecdsa, bls, mirror)
task test:all-off-chain
```

*on-chain*

This requires first start the chains, running the tests, and then remembering to shut it down:

```bash
task backend:start-chains
task test:contracts-on-chain
task backend:stop-chains
```

It may take a while for the chain to startup, recommendation is to leave it up while developing

If you run into errors with `Starship` or `helm` namespace being taken, try:

```bash
helm repo remove starship
helm delete cw-middleware
```

### Components

Usually you just `cargo test` as needed. However, for convenience, this will test all the component packages:

```bash
task test:components
```

You can also execute a specific component to test it with some data:

```bash
task components:exec-echo-with-id -- "Hello World"
```


### End-to-end services

The flow is similar to on-chain tests, and assumes the contracts are already built

```bash
task backend:start-all
task test:e2e
task backend:stop-all
```

It may take a while for the backend to startup, recommendation is to leave it up while developing

If you already have the chains running, then run `task backend:start-wavs` instead of `task backend:start-all`

Jaeger UI is at [http://localhost:16686/](http://localhost:16686/)
Prometheus is at [http://localhost:9090/](http://localhost:9090/)

### Testing all

This will literally run all tests, usually it's not what you want... but, it's doable:

TODO: currently blocked on https://github.com/Lay3rLabs/cw-middleware/issues/26

```bash
task test:all
```

### Multiple operators

```bash
# When starting the whole backend
task backend:start-all OPERATORS=N

# If just starting wavs, not chain
task backend:start-wavs OPERATORS=N
```

Make sure you have that number of submission wallets in your `.env`

### CLI

Sometimes it's useful to interact with the backend and contracts with ad-hoc commands. For example:

```bash
# Tap the faucet for CLI wallet
task cli:tap-faucet

# Upload the service manager WASM to get a code id
task cli:service-manager-upload

# Instantiate an instance of the service manager to get an address
task cli:service-manager-instantiate CODE_ID={value}

# Upload the service handler WASM to get a code id
task cli:service-handler-upload

# Instantiate an instance of the service handler to get an address
task cli:service-handler-instantiate CODE_ID={value} SERVICE_MANAGER_ADDR={value}

# Set the service uri on a service manager
task cli:service-manager-set-service-uri ADDR={value} URI={value}

# Get the service uri for a service manager
task cli:service-manager-get-service-uri ADDR={value}

# Get the service manager for a service handler
task cli:service-handler-get-manager ADDR={value}
```

## Deploying

For the docker-based deploy story, see [docs/USAGE.md](docs/USAGE.md). The notes below summarize what the v0.3.0 ECDSA / BLS / Mirror flows expect.

### ECDSA and BLS — explicit owner/admin at instantiate

ECDSA and BLS service-managers split the operator-set role (OWNER) from the operational-config role (ADMIN); both addresses are required at instantiate. An admin-key compromise cannot reshape the operator set, and vice versa.

```bash
# ECDSA
docker run ... ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager instantiate-ecdsa \
    --code-id <CODE_ID> \
    --owner <OWNER_ADDR> \
    --admin <ADMIN_ADDR> \
    [--quorum-numerator <N>] [--quorum-denominator <D>]   # default 2/3

# BLS (same shape)
docker run ... ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager instantiate-bls \
    --code-id <CODE_ID> \
    --owner <OWNER_ADDR> \
    --admin <ADMIN_ADDR>
```

Ownership and admin transfers on ECDSA / BLS are **two-step** (`TransferOwnership` → `AcceptOwnership`, `SetAdmin` → `AcceptAdmin`). The Pause / Unpause messages are owner-only; while paused, weight-mutating writes and validation queries reject.

### Mirror — post-deploy ownership handoff (audit C-5)

The Mirror flavor uses sync-handlers (`mirror-operator-sync-handler`, `mirror-quorum-sync-handler`) to bridge an EVM-side operator set onto the Cosmos chain. Because these handlers are deployed *after* the stake-registry and service-manager, the deploy flow ends with two handoff steps that transfer control to the handlers:

```bash
# After uploading + instantiating stake-registry, service-manager, and both sync-handlers:

# Hand the stake-registry OWNER role to the operator-sync-handler
docker run ... ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    registry transfer-ownership \
    --address <STAKE_REGISTRY_ADDR> \
    --new-owner <OPERATOR_SYNC_HANDLER_ADDR>

# Hand the service-manager ADMIN role to the quorum-sync-handler
docker run ... ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager set-mirror-admin \
    --address <SERVICE_MANAGER_ADDR> \
    --new-admin <QUORUM_SYNC_HANDLER_ADDR>
```

Mirror uses **single-step** TransferOwnership / SetAdmin (no accept) because the typical handoff target is a contract address that can't sign an `accept` message. Without these two steps, the sync-handlers cannot authorize their downstream calls.

### Architecture

#### SDK

Core clients are defined in [packages/sdk](packages/sdk)

These clients are feature-gated so that they work in wasm, binaries, multi-test, etc.

The sdk provides structs that wrap the client with whatever functionlity is needed. These structs generally form a hierarchy, where helper methods make their way through at each level.

So, for example, at the root level theres structs for WAVS ServiceHandler/ServiceManager, which wrap Query and/or Execute clients. Then at deeper levels there's structs for BLS/ECDSA/Mirror contracts that extend these. Yet, at each level, `.querier()` will return the core querier client.

#### Implementations

With the SDK available and working everywhere, we simply need to construct what we need, depending on whether we have a SigningPool, .wasm file, off-chain multitest code in memory, etc. This happens per-project and does nothing other than create the client. For example, off-chain tests create sdk clients via cw-multitest, while on-chain tests do so by deploying contracts on-chain.

Importantly - functionality remains in the SDK, and so code can be easily shared everywhere, by taking the SDK structs as parameters.
