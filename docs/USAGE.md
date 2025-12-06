# Using the Middleware

The middleware CLI and compiled contracts are made available via a docker image at `ghcr.io/lay3rlabs/cw-middleware:{TAG}` where `{TAG}` is the version tag or `latest` for the most recent merged-to-main build.

All required contracts are pre-built and available in the image's `/wasm/built-in` directory

> _Tip:_ This package does not include any backend services or component tooling; it focuses purely on middleware functionality. See the [README.md](../README.md) for how to start a chain for on-chain testing.

Using the middleware usually consists of:

1. mounting a volume containing your `wavs.toml` and setting the corresponding guest path as `WAVS_HOME` in the ENV
2. specifying the docker-friendly chain key via the `CHAIN_KEY` environment variable (see the local [wavs.toml](../backend/wavs-home/wavs.toml) for an example)
3. providing your wallet mnemonic via the `CLI_MNEMONIC` environment variable
4. set the `FAUCET_URL` to tap the faucet (if needed)
5. running the command

When you're working with docker programmatically, you'll also typically pass a `--output-path /path/to/output.json` argument to commands that generate output (e.g. upload, instantiate, etc) so that the resulting addresses and code IDs can be captured for later use. Alternatively, you can set it in the `OUTPUT_PATH` environment variable. Either way, you'll need to make sure you mount that path as a volume in the docker run command, e.g.:

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    -v path/to/my/output:/output \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager upload --contract-kind mirror --output-path /output/service-manager-mirror.json
```

For the sake of convenience, we've put the typical env vars in a `.docker.env` and so the following commands just use that (and do not write to an output file)

## Examples

### Tap the faucet (assumes you've started a faucet e.g. via Starship)

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    faucet-tap
```

### Show wallet info

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    wallet show
```

### Upload the mirror service manager contract

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager upload --contract-kind mirror
```

### Upload the mirror stake registry contract

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    registry upload --contract-kind mirror_stake
```

### Instantiate the mirror stake registry contract

Replace `<CODE_ID>` and `<SERVICE_MANAGER_CODE_ID>` with the code IDs returned from the previous step.

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    registry instantiate-mirror-stake \
    --code-id <CODE_ID> \
    --service-manager-code-id <SERVICE_MANAGER_CODE_ID>
```

### Get the address of the mirror service manager contract from the registry

Replace `<ADDR>` with the address of the registry contract instantiated in the previous step.

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    registry get-service-manager \
    --contract-kind mirror_stake \
    --address juno1w27ekqvvtzfanfxnkw4jx2f8gdfeqwd3drkee3e64xat6phwjg0sgauq9a
```

### Upload the mirror service handler contract

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-handler upload --contract-kind mirror
```

### Upload the mirror quorum sync handler contract

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    contract upload --contract-kind mirror_quorum_sync_handler
```

### Upload the mirror operator sync handler contract

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    contract upload --contract-kind mirror_operator_sync_handler
```

### Instantiate the example mirror service handler contract

Replace `<CODE_ID>` with the code ID returned from the previous step.

Replace `<SERVICE_MANAGER_ADDR>` with the service manager address obtained from the registry in the previous step.

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-handler instantiate-mirror \
    --code-id 8 \
    --service-manager juno1m6g7dckc0ekvj82wk8899gj2j63hplcdk88ftgxlnzwwn3lp5pjsvxs3hp
```

### Set the Service URI on the mirror service manager contract

Replace `<ADDR>` with the service manager address obtained from the registry in the previous step.

Replace `<URI>` with the desired service URI.

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager set-service-uri \
    --address <ADDR> \
    --uri ipfs://example
```

### Get the Service URI from the mirror service manager contract

Replace `<ADDR>` with the service manager address obtained from the registry in the previous step.

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager get-service-uri \
    --address <ADDR> \
```

### Set Operator Signing Key

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    registry set-operator-signing-key \
    --address <REGISTRY_ADDR> \
    --operator <OPERATOR_EVM_ADDR> \
    --signing-key <SIGNING_KEY_EVM_ADDR> \
    --weight <WEIGHT>
```

# Local docker builds

If you want to build and test changes locally, you can build the docker image yourself:

```bash
docker build -t cw-middleware:local .
```
