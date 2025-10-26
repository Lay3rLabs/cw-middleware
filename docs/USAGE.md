# Using the Middleware

The middleware CLI and compiled contracts are made available via a docker image at `ghcr.io/lay3rlabs/cw-middleware:{TAG}` where `{TAG}` is the version tag or `latest` for the most recent merged-to-main build.

All required contracts are pre-built and available in the image's `/wasm/built-in` directory

> _Tip:_ This package does not include any backend services or component tooling; it focuses purely on middleware functionality. See the [README.md](../README.md) for how to start a chain for on-chain testing.

Using the middleware usually consists of:

1. mounting a volume containing your `wavs.toml`
2. specifying the chain key via the `CHAIN_KEY` environment variable for a docker-friendly chain config
3. providing your wallet mnemonic via the `CLI_MNEMONIC` environment variable
4. set the `FAUCET_URL` to tap the faucet (if needed)
5. running the command

For the sake of convenience, we've put the typical env vars in a `.docker.env` and so the following commands just use that.

## Examples

#### Tap the faucet

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    faucet-tap
```

#### Show wallet info

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    wallet show
```

#### Upload the mock service manager contract

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    contract upload --wasm-file /wasm/built-in/cw_wavs_mock_service_manager.wasm
```

#### Instantiate the mock service manager contract (replace `<CODE_ID>` with the actual code ID from the upload step)

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager deploy --code-id <CODE_ID> --contract-kind mock
```

#### Upload the mock service handler contract

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    contract upload --wasm-file /wasm/built-in/cw_wavs_mock_service_handler.wasm
```

#### Instantiate the mock service manager contract (replace `<CODE_ID>` and `<SERVICE_MANAGER_ADDR>` from the previous steps)

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-handler deploy --code-id <CODE_ID> --service-manager <SERVICE_MANAGER_ADDR> --contract-kind mock
```


# Local docker builds

If you want to build and test changes locally, you can build the docker image yourself:

```bash
docker build -t cw-middleware:local .
