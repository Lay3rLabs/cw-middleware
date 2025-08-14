# What it is 

This repo is for Wavs Cosmwasm middleware

It's constructed so that developing and testing any combination of the following is as smooth as possible

**Contracts**

* Mock
* ECDSA
* BLS

**Chains**

* Any cosmos chain (e.g. Neutron, Juno, CosmosHub, etc.)

**Environment**

* Multitest / off-chain
* Local on-chain
* Remote on-chain

In other words: test functionality is shared between all these different requirements, making it painless to switch between e.g. mocks on-chain and ecdsa off-chain for common tests, while still supporting specific functionality as needed in isolated parts of the codebase

Additionally, a docker image is provided (TODO!) such that consumers can bring the middleware into their project with ease.

# What it is not

This is not a repo for developing Wavs/Cosmwasm components or non-middleware contracts. While we have components in this repo for testing, it's purely to fulfill that need, not meant for public consumption. Check [wavs-tools](https://github.com/Lay3rLabs/wavs-tools) for that.

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
* `CHAIN_NAME` will affect on-chain test commands

## Building

#### Contracts

```bash
task contracts:build
```

Some more contract building commands:

* `task contracts:build-all`: build all the different kinds of contracts
* `task contracts:build-service-handler`: build just the service handler for the current kind
* `task contracts:build-service-manager`: build just the service handler for the current kind
* `task contracts:build-mock-trigger`: build just the mock trigger (this is re-used for all kinds of tests) 


#### Components

```bash
task components:build-all
```

These components are the same regardless of whether we're targetting mock contracts, testnet chain, etc.

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
# Test all contract kinds (mock, bls, ecdsa, etc.)
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

Usually you just `cargo component test` as needed. However, for convenience, this will test all the component packages:

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

# Deploy an instance of the service manager to get an address
task cli:service-manager-deploy CODE_ID={value}

# Upload the service handler WASM to get a code id
task cli:service-handler-upload

# Deploy an instance of the service handler to get an address
task cli:service-handler-deploy CODE_ID={value} SERVICE_MANAGER_ADDR={value}

# Set the service uri on a service manager
task cli:service-manager-set-service-uri ADDR={value} URI={value}

# Get the service uri for a service manager
task cli:service-manager-get-service-uri ADDR={value}

# Get the service manager for a service handler
task cli:service-handler-get-manager ADDR={value}
```

### Architecture

#### Short version

Contract interaction is the same way everywhere:

1. `use utils::prelude::*`
2. call functions on your client

This works for off-chain and on-chain, whether it's wasi, binary, web, etc. etc.

Reusable functions are written by accepting trait arguments like `WavsClientExt`

For example:

```rust
use utils::prelude::*;

// this code can be run in any context
pub async fn run_sanity_tests(client: &impl WavsClientExt) {
    let addr = client.service_handler_querier().get_manager_address().await.unwrap();

    client.service_manager_exec().set_service_uri("http://example.com".to_string()).await.unwrap();
    let url = client.service_manager_querier().get_service_uri().await.unwrap();

    assert_eq!(url, "http://example.com");
}
```

There are more granular traits than `WavsClientExt` for all combinations of service handler/manager and query/exec, as well as specific to mock, ecdsa, and bls.

As long as you bring in the prelude with `use utils::prelude::*`, you have it available.

This is used in the CLI, for example, to not require a signing client when all we need is a query and also keeps things simple:

```rust
Command::ServiceManager(ServiceManagerArgs{command, address}) => {
    match command {
        ServiceManagerCommand::SetServiceUri { uri } => {
            let client = ctx.wavs_service_manager_signing_client(address).await.unwrap();
            let resp = client.set_service_uri(uri.to_string()).await.unwrap();
            println!("Set service URI TX hash: {}", resp.txhash);
        },
        ServiceManagerCommand::GetServiceUri => {
            let client = ctx.wavs_service_manager_query_client(address).await.unwrap();
            let uri = client.get_service_uri().await.unwrap();
            println!("Service URI: {}", uri);
        },
    }
},
Command::ServiceHandler(ServiceHandlerArgs{command, address}) => {
    match command {
        ServiceHandlerCommand::GetManager => {
            let client = ctx.wavs_service_handler_query_client(address).await.unwrap();
            let manager = client.get_manager_address().await.unwrap();
            println!("Service Manager: {}", manager);
        }
    }
},
```

#### Constructing a client

Although the core structs and traits are all implemented for you, constructing a client happens per-package. This allows flexibility, for example, in configuring the client on the fly to hit an arbitrary endpoint or wrapping it in a higher-level client that can also talk to the wavs node.

#### Adding functionality

Wavs-types interfaces are in exactly one place: [packages/utils/src/contract_client/functionality.rs](packages/utils/src/contract_client/functionality.rs).

Changes made to that file will propogate to all wavs-types-aware clients everywhere.

More specific interfaces (mock, ecdsa, bls) are also in exactly one place: [packages/utils/src/contract_client/functionality/](packages/utils/src/contract_client/functionality/).

Changes made to that file will likewise propogate to all of those clients everywhere.

#### More detail

The way this works is through a trait system. 

Clients must implement a very minimal set of extension traits defined in [packages/utils/src/contract_client/ext.rs](packages/utils/src/contract_client/ext.rs). This is already done for both [on-chain](packages/utils/src/contract_client/on_chain.rs) and [off-chain](packages/tests/off-chain/src/client.rs). Importantly, these requirements are just basic and generic, _and because it's all already setup for you, there's no need to edit anything here_.

Through blanket implementations, the functionality defined in `functionality.rs` gets mixed in automagically.

The same mechanism happens for the bls, ecdsa, and mock extensions (it's automagically mixed in, and their specific functionality is made available everywhere)

As mentioned above, the traits are actually split up, and so it's possible to get just a service handler querier (take a `impl WavsServiceHandlerQueryClientExt`) or just a service manager executor (take a `impl WavsServiceManagerExecClientExt`). There are more subdivisions defined in [packages/utils/src/contract_client/ext/](packages/utils/src/contract_client/ext.rs) and further in [packages/utils/src/contract_client/ext/](packages/utils/src/contract_client/ext/) 