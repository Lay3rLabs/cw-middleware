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

_the first time this runs it will need to compile the binary, be patient_

Copy `.example.env` to `.env` and replace the values

## Building

#### Contracts
```
task contract:build-mocks
```

#### Components

```
task component:build-all
```

You can also just generate the bindings, to get errors in the IDE before building:

```
task component:bindings-all
```

## Testing

### Contracts 

*off-chain*

```
task contract:test-mocks-off-chain
```

*on-chain*

This requires spinning up a server, running tests, and then shutting down the server:

```
task backend:start-chains
task contract:test-mocks-on-chain
task backend:stop-chains
```

It may take a while for the chain to startup, recommendation is to leave it up while developing

If you run into errors with `Starship` or `helm` namespace being taken, try:

```bash
helm repo remove starship
helm delete cw-middleware
```

### Components 

Manually run `cargo test`, isolating on a specific package/contract as needed

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
task e2e-test
task backend:stop-all
```

It may take a while for the backend to startup, recommendation is to leave it up while developing

If you already have the chains running, then run `task backend:start-wavs` instead of `task backend:start-all`

### CLI

Sometimes it's useful to interact with the contracts after they're deployed with ad-hoc commands. Here's some examples:

```
task cli:query-service-handler-manager
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