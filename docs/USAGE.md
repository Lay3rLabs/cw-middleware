# Using the Middleware

The middleware CLI and compiled contracts are made available via a docker image at `ghcr.io/lay3rlabs/cw-middleware:{TAG}` where `{TAG}` is the version tag or `latest` for the most recent merged-to-main build.

All required contracts are pre-built and available in the image's `/wasm/built-in` directory

> _Tip:_ This package does not include any backend services or component tooling; it focuses purely on middleware functionality. See the [README.md](../README.md) for how to start a chain for on-chain testing.

> _v0.3.0 note:_ The Mock contract family has been removed. ECDSA and BLS service-managers now require explicit `--owner` and `--admin` at instantiate, and Mirror deploys end with two ownership-handoff calls (see [Mirror ownership handoff](#mirror-ownership-handoff) at the bottom of this doc).

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

### Set the Quorum Threshold on the mirror service manager contract

Replace `<ADDR>` with the service manager address.
Replace `<NUMERATOR>` and `<DENOMINATOR>` with the desired quorum values (e.g., 2 and 3 for 2/3).

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager set-quorum-threshold \
    --address <ADDR> \
    --numerator <NUMERATOR> \
    --denominator <DENOMINATOR>
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

### Instantiate an ECDSA service manager

ECDSA and BLS service-managers split the operator-set role (`--owner`) from the operational-config role (`--admin`). Both are required at instantiate. An admin-key compromise cannot reshape the operator set, and vice versa. Optional `--quorum-numerator` / `--quorum-denominator` override the 2/3 default.

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager instantiate-ecdsa \
    --code-id <CODE_ID> \
    --owner <OWNER_ADDR> \
    --admin <ADMIN_ADDR>
```

### Instantiate a BLS service manager

Same shape as ECDSA; the on-chain crypto uses BLS12-381 aggregate signatures via cosmwasm-crypto host calls.

```bash
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager instantiate-bls \
    --code-id <CODE_ID> \
    --owner <OWNER_ADDR> \
    --admin <ADMIN_ADDR>
```

Ownership and admin transfers on ECDSA / BLS are **two-step** (`TransferOwnership` → `AcceptOwnership`, `SetAdmin` → `AcceptAdmin`) — the new key must explicitly accept before the role rotates. Owner-only `Pause` / `Unpause` block weight-mutating writes and validation queries while paused.

### Trigger with pusher allowlist (audit M-1)

The simple trigger defaults to a public message bus (any sender can `Push`). To restrict pushes to a known address set, pass `allowed_pushers` at instantiate. Either drive instantiate through the docker CLI / SDK with an explicit `InstantiateMsg`, or pre-bake the JSON:

```json
{
  "allowed_pushers": ["wavs1...", "wavs1..."]
}
```

`allowed_pushers: null` (the default) preserves legacy public-bus behavior; the field can only be set at instantiate.

## Mirror ownership handoff

The Mirror flavor uses two sync-handlers — `mirror-operator-sync-handler` and `mirror-quorum-sync-handler` — to bridge an EVM-side operator set onto the Cosmos chain. Because the sync-handlers are deployed *after* the stake-registry and service-manager, the deploy flow must end by transferring control to them. Without these two calls, the sync-handlers cannot authorize their downstream `SetOperatorDetails` and `SetQuorumThreshold` messages (audit C-5).

```bash
# Hand the stake-registry OWNER role to the operator-sync-handler
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    registry transfer-ownership \
    --address <STAKE_REGISTRY_ADDR> \
    --new-owner <OPERATOR_SYNC_HANDLER_ADDR>

# Hand the service-manager ADMIN role to the quorum-sync-handler
docker run --rm \
    -v $(pwd)/backend/wavs-home:/wavs-home:ro \
    --env-file .docker.env \
    ghcr.io/lay3rlabs/cw-middleware:{TAG} \
    service-manager set-mirror-admin \
    --address <SERVICE_MANAGER_ADDR> \
    --new-admin <QUORUM_SYNC_HANDLER_ADDR>
```

Mirror handoff is **single-step** (no accept) because the target is a contract address that can't sign an `accept` message.

# Local docker builds

If you want to build and test changes locally, you can build the docker image yourself:

```bash
docker build -t cw-middleware:local .
```
