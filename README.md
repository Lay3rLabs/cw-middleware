# Getting Started

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
