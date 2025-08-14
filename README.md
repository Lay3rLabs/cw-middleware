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

## Building

#### Contracts

```
task contract:build
```

The contract `kind` to build is specified in [.env](.example.env) to support local development workflows without committing to the repo.

Some more contract building commands:

* `task contract:build-all`: build all the different kinds of contracts
* `task contract:build-service-handler`: build just the service handler for the current kind
* `task contract:build-service-manager`: build just the service handler for the current kind


#### Components

```
task component:build-all
```

Some more component building commands:

* `task component:bindings-all`: generate the bindings for all components
* `task component:build-echo-with-id`: build just the echo-with-id component
* `task component:bindings-echo-with-id`: generate bindings for just the echo-with-id component



## Testing

### All

```bash
task test:all
```

### Contracts 

*off-chain*

```
task test:contract-mocks-off-chain
```

*on-chain*

This requires spinning up a server, running tests, and then shutting down the server:

```
task backend:start-chains
task test:contract-mocks-on-chain
task backend:stop-chains
```

It may take a while for the chain to startup, recommendation is to leave it up while developing

If you run into errors with `Starship` or `helm` namespace being taken, try:

```bash
helm repo remove starship
helm delete cw-middleware
```

### Components 

Manually run `cargo component test`, isolating on a specific package/contract as needed

For convenience, this will test all the component packages:

```
task component:test-all
```

You can also execute a specific component to test it with some data:

```
task component:exec-echo-with-id -- "Hello World"
```


### End-to-end services 

```
task backend:start-all
task test:e2e-mocks
task backend:stop-all
```

It may take a while for the backend to startup, recommendation is to leave it up while developing

If you already have the chains running, then run `task backend:start-wavs` instead of `task backend:start-all`

Jaeger UI is at [http://localhost:16686/](http://localhost:16686/)
Prometheus is at [http://localhost:9090/](http://localhost:9090/)

### Multiple operators

Multiple operators can be launched by passing `OPERATORS=N` to either `backend:start-all` or `backend:start-wavs`, just make sure you have that number of submission wallets in your `.env`

### CLI

Sometimes it's useful to interact with the backend and contracts with ad-hoc commands. For example:

```bash
# Tap the faucet for CLI wallet
task cli:tap-faucet

# Upload the mock service manager WASM to get a code id
task cli:mock-service-manager-upload

# Deploy an instance of the mock service manager to get an address
task cli:mock-service-manager-deploy CODE_ID={value}

# Upload the mock service handler WASM to get a code id
task cli:mock-service-handler-upload

# Deploy an instance of the mock service handler to get an address
task cli:mock-service-handler-deploy CODE_ID={value} SERVICE_MANAGER_ADDR={value}

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
2. bring `WavsClientExt` into scope or take it as an argument
3. call functions on your client

This works for off-chain and on-chain, whether it's wasi, binary, web, etc. etc.

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

There are more granular traits than `WavsClientExt` for all combinations of service handler/manager and query/exec.

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

#### Getting a client

All on-chain clients use the one(s) from [on-chain utils](packages/utils/src/client/on_chain.rs)

For off-chain, it's local to the [off-chain package](packages/tests/off-chain/src/client.rs)

#### Adding functionality

It's all in exactly one place: [packages/utils/src/client/functionality.rs](packages/utils/src/client/functionality.rs). 

Changes made to that file will propogate to all clients everywhere. 

#### More detail

The way this works is through a trait system. 

Clients must implement a very minimal set of extension traits defined in [packages/utils/src/client/ext.rs](packages/utils/src/client/ext.rs). This is already done for both [on-chain](packages/utils/src/client/on_chain.rs) and [off-chain](packages/tests/off-chain/src/client.rs). Importantly, these requirements are just basic and generic, _and because it's all already setup for you, there's no need to edit anything here_.

Through blanket implementations, the functionality defined in `functionality.rs` gets mixed in automagically.

The on-chain client is used in multiple scenarios, and so the module actually has several clients and is shared from the `utils` crate - e.g. CLI uses a regular signing client, tests use a signing client pool, sometimes we only need a querier, etc. The off-chain client is just local to multitest.

Lastly, the traits are actually split up, and so it's possible to get just a service handler querier (take a `impl WavsServiceHandlerQueryClientExt`) or just a service manager executor (take a `impl WavsServiceManagerExecClientExt`). There are more subdivisions defined in [ext.rs](packages/utils/src/client/ext.rs)