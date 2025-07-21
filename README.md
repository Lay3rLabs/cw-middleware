# WAVS CosmWasm Middleware

WAVS CosmWasm middleware contracts needed to submit to Cosmos smart contracts, as well as packages, macros, and examples.

## Contracts

### Examples

- [`wavs-counter-handler`](./contracts/examples/wavs-counter-handler): A simple service handler that counts the number of times a service is run and successfully validated by WAVS.

## Prerequisites

### Rust

https://www.rust-lang.org/tools/install

### Just

https://github.com/casey/just

```sh
cargo install just
```

## Development

### Linting

```sh
just lint
```

### Formatting

```sh
just format
```

### Testing

```sh
just test
```

### Building production contracts

```sh
just optimize
```
