# Using the Middleware

The middleware CLI and compiled contracts are made available via a docker image at `lay3rlabs/cw-middleware:latest`

Built-in contracts are available in the image's `/wasm/built-in` directory

> _Tip:_ This package does not include any backend services or component tooling; it focuses purely on middleware functionality. See the [README.md](../README.md) for how to start a chain for on-chain testing.

Using the middleware usually consists of mounting a volume containing your `wavs.toml` and running commands like so:

```bash
# Example: Running the middleware to see the help menu
# TODO!
```
