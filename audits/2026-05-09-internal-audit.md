# cw-middleware — Security Audit (Heartbeat 2026-05-09)

> Internal audit by Arc (heartbeat agent). Not a substitute for an external audit. EF-credibility framing — residual risks called out alongside findings.
>
> Scope: `contracts/cw-middleware/packages/contracts/{ecdsa, bls, mirror, mock, trigger}` (~4012 LOC of Rust across 5 contract families). Reviewed against repo HEAD `69b9ab5` ("update lock"). No tests run; no fork against any deployed instance.

---

## TL;DR

**This is a partially-implemented codebase. Two of the five contract families ("ECDSA" and "BLS") are byte-for-byte copies of each other and contain no actual signature verification — they are stubs. The "mock" family is functional but unauthenticated by design. The "trigger" family is a public message bus. Only the "mirror" family contains real cryptography (secp256k1 ECDSA recovery against EIP-191) and quorum logic.**

The real implementation lives entirely in `mirror/stake-registry`. Even there, two structural issues stand out:

1. **`reference_block` snapshot semantics are silently broken** by a "fall back to latest" branch in both `SIGNING_KEY_TO_OPERATOR` and `OPERATOR_WEIGHTS` lookups. An operator registered *after* `reference_block` can contribute weight to a signature that was supposed to be evaluated against the historical operator set.
2. **The mirror sync handlers cannot actually call into their downstream contracts** under the CLI's documented deploy flow. The stake-registry `OWNER` is the CLI signer, not the `mirror-operator-sync-handler` contract; the service-manager `admin` is also the CLI signer, not the `mirror-quorum-sync-handler`. So both sync handlers' main code paths revert with `Unauthorized` when invoked from a signed envelope.

Three of the four ECDSA/BLS stub-contracts are deployable but **leave their `WavsSetServiceUri` and `WavsSetQuorumThreshold` open to any caller** — the source code admits this in a TODO comment. The `mock` service-manager has the same shape plus an unauthenticated `SetSigningKey`.

If anyone is planning to use cw-middleware to operate a Cosmos-side WAVS AVS today, they should know that **only the mirror stack is real**, and even the mirror stack's quorum-update path does not match the wiring its CLI deploys.

---

## Architecture Overview

Five contract families:

```
  ┌────────────┐ ┌────────────┐ ┌────────────┐
  │   ecdsa    │ │    bls     │ │    mock    │   ← stub managers, no real validation
  ├────────────┤ ├────────────┤ ├────────────┤
  │ svc-mgr    │ │ svc-mgr    │ │ svc-mgr    │
  │ svc-handler│ │ svc-handler│ │ svc-handler│
  │   api      │ │   api      │ │   api      │
  └────────────┘ └────────────┘ └────────────┘

  ┌──────────────────────────────────────────┐
  │              mirror                       │   ← real implementation
  ├──────────────────────────────────────────┤
  │ stake-registry  (435 LOC, secp256k1)     │
  │ service-manager (234 LOC, quorum check)  │
  │ service-handler (98 LOC, base)           │
  │ mirror-operator-sync-handler             │
  │ mirror-quorum-sync-handler               │
  │ api / state                              │
  └──────────────────────────────────────────┘

  ┌────────────┐
  │  trigger   │   ← public unauthenticated message bus
  └────────────┘
```

**Signature flow (mirror only):**

```
Cosmos-side caller → handler.WavsHandleSignedEnvelope(envelope, sigs)
                       │
                       ▼ delegates to service-manager.WavsValidate (query)
                       │   └─ service-manager → stake-registry.ValidateSignature
                       │       └─ for each (signer, sig): secp256k1_recover_pubkey
                       │           └─ accumulate weight (snapshotted at reference_block)
                       │       └─ check signed_weight ≥ (total × numerator/denominator)
                       │
                       └─ on success: handler-specific side effect
                          (save envelope, sync operator set, sync quorum)
```

---

## Findings

### Severity scale

| | Description |
|---|---|
| Critical | Loss-of-soundness path or invariant break under the assumed model (signatures bypassed, replay possible, quorum wrong) |
| High | Material correctness or operational failure (wiring broken under documented deploy, snapshot semantics violated) |
| Medium | Footguns, unbounded behavior, missing validation |
| Low | Hygiene |
| Info | Notes |

### Critical

**C-1 — `ecdsa::service_manager::WavsValidate` does not verify signatures** (`contracts/cw-middleware/packages/contracts/ecdsa/service-manager/src/entry.rs:93-110`)

```rust
ServiceManagerQueryMessages::WavsValidate { envelope: _, signature_data } => {
    // TODO: real validation logic
    for signer in &signature_data.signers {
        let _operator_addr =
            match state::OPERATOR_SIGNING_KEY_ADDRS.load(deps.storage, signer) {
                Ok(addr) => addr,
                Err(e) => return to_json_binary(&WavsValidateResult::Err(...)),
            };
    }
    to_json_binary(&WavsValidateResult::Ok)
}
```

The query iterates `signers`, looks each up in a map, and returns `Ok` if all are registered. **No call to `secp256k1_verify` or `secp256k1_recover_pubkey`. No quorum check. `envelope` is bound to `_`.** Anyone holding a signed envelope blob from any past signing event can re-submit it indefinitely; anyone able to spell the address of a registered operator can submit a forged envelope with no signature.

Compounding: **the contract has no `SetSigningKey` execute message**, so `OPERATOR_SIGNING_KEY_ADDRS` is permanently empty. In practice every `WavsValidate` call returns `InvalidSignature` because the lookup fails, not because of a real check. The contract is effectively bricked-by-accident.

`api/src/service_manager.rs:12` carries the comment `// TODO - uhh... ecdsa stuff` — the maintainers know.

**Fix:** either delete the ecdsa family and use `mirror` (which already does what ecdsa pretends to), or implement real validation by porting the stake-registry's `is_valid_signature`.

---

**C-2 — `bls::service_manager` is byte-for-byte the same as `ecdsa::service_manager`** (`contracts/cw-middleware/packages/contracts/bls/service-manager/src/entry.rs`)

`diff` produces only the api crate import and the comment string. There is no BLS pairing, no operator key registration, no quorum verification. Same stub status as C-1.

**Fix:** same as C-1 — either delete or implement. If keeping, route through `bls12_381` host calls if Cosmos chain supports them, or import a verifier from the EVM `poa-middleware:bls` work.

---

**C-3 — Unauthenticated `WavsSetServiceUri` and `WavsSetQuorumThreshold`** (`ecdsa/service-manager/src/entry.rs:47-71`, identical in `bls/service-manager` and `mock/service-manager`)

```rust
ServiceManagerExecuteMessages::WavsSetQuorumThreshold { numerator, denominator } => {
    // For ECDSA service manager, we'll allow any sender to set quorum threshold for simplicity
    // In a real implementation, this should be restricted to admin/owner
    ...
}
ServiceManagerExecuteMessages::WavsSetServiceUri { service_uri } => {
    state::SERVICE_URI.save(deps.storage, &service_uri)?;
    ...
}
```

The off-chain WAVS node fetches its workflow config from `WavsSetServiceUri`. **Any caller can hijack the service URI to point at attacker-controlled IPFS** (or any URL). Any caller can also unilaterally tighten or loosen the quorum to bypass downstream checks (such checks aren't actually applied in ecdsa/bls — but third parties writing handlers against this surface would be misled).

Severity: in the ecdsa/bls families this is critical because no other check exists. In `mock` it's "by design" but the contract doesn't enforce that it's only deployed in test environments — a producer who builds against the api unit could ship the mock implementation thinking it was a real one (the contract names are nearly identical from the outside).

**Fix:** add an `OWNER` item, set in `instantiate`, gate both messages on `info.sender == owner`.

---

**C-4 — `mock::service_manager::SetSigningKey` is unauthenticated** (`mock/service-manager/src/entry.rs:73-82`)

```rust
ExecuteMsg::SetSigningKey { operator, signing_key, weight } => {
    state::OPERATOR_SIGNING_KEY_ADDRS.save(deps.storage, &operator, &signing_key)?;
    state::SIGNING_KEY_OPERATOR_ADDRS.save(deps.storage, &signing_key, &operator)?;
    state::OPERATOR_WEIGHTS.save(deps.storage, &operator, &weight)?;
    Ok(Response::default())
}
```

Anyone can register any address as an operator with any weight. Acceptable in test scaffolding, **dangerous if shipped under a name that looks like a service-manager**. The `mock` prefix is in the contract name but a downstream user wiring `cw_wavs_mock_api::service_manager::ExecuteMsg` doesn't get a compile-time signal that this surface is unsafe.

**Fix:** if mock is dev-only, don't expose its `ExecuteMsg::SetSigningKey` in published `api` crates, or hard-fail on instantiate unless a `dev_only: bool` flag is passed.

---

**C-5 — Mirror sync handlers can't authorize their downstream calls under the CLI deploy flow**

Two related instances:

**(a) `mirror-operator-sync-handler` → `stake-registry.BatchSetOperatorDetails`** (`mirror/mirror-operator-sync-handler/src/entry.rs:77-86`):

```rust
msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
    contract_addr: stake_registry.clone(),
    msg: to_json_binary(&cw_wavs_mirror_api::stake_registry::ExecuteMsg::BatchSetOperatorDetails { ... })?,
    funds: vec![],
}));
```

When this CosmosMsg fires, the stake registry sees `info.sender == mirror_operator_sync_handler_contract_address`. But the stake-registry checks `info.sender == OWNER` (`mirror/stake-registry/src/entry.rs:142-144`), and `OWNER` was set in `instantiate` to whoever called the stake-registry's `instantiate` — namely the CLI signer (`packages/cli/src/main.rs:454`).

Result: every `mirror-operator-sync-handler::WavsHandleSignedEnvelope` returns `Unauthorized` from the inner BatchSetOperatorDetails. **The operator-sync pipeline is non-functional under the documented deploy flow.**

**(b) `mirror-quorum-sync-handler` → `service-manager.WavsSetQuorumThreshold`** (`mirror/mirror-quorum-sync-handler/src/entry.rs:77-89`):

The service-manager admin check (`mirror/service-manager/src/entry.rs:63-67`) requires `info.sender == admin`, where `admin` is set in `instantiate` to whatever address was passed (CLI uses `client.addr`, line 440 of `cli/src/main.rs`). The quorum-sync-handler's contract address is not the admin. Same `Unauthorized` failure.

**Severity rationale:** the mirror sync handlers are the entire reason the mirror family exists — to mirror EVM operator-set state into Cosmos via signed updates. If they can't update their downstream contracts, the entire architecture is decorative. This is critical even though no funds are at risk *because* the system can't function.

**Fix:** either
- transfer stake-registry ownership to the operator-sync-handler after deploy (`AddOwnerTransfer { new_owner }` execute message + corresponding handler instantiation order), or
- whitelist the sync-handler addresses on the stake-registry/service-manager during their respective instantiations.

The cleaner shape: have the sync handlers be the deployer-of-record for their downstream contracts. The CLI then deploys the handlers (passing the admin/EVM-source addresses), and the handlers instantiate the stake registry / service manager with themselves as owner.

---

### High

**H-1 — `ecdsa::service_handler` and `bls::service_handler` are no-ops** (`ecdsa/service-handler/src/entry.rs:46-67`, BLS identical at `bls/service-handler/src/entry.rs:45-67`)

After querying the (broken) service-manager for validation, the handler **does literally nothing else** — no envelope save, no message dispatch, no business logic. Even if validation worked, calling these handlers would be a gas-wasting no-op.

**Fix:** define what these handlers are *supposed* to do (the mirror handler saves envelopes; do these need similar persistence?) or delete them.

---

**H-2 — Mirror stake-registry "snapshot or fall back to latest" breaks reference_block semantics** (`mirror/stake-registry/src/entry.rs:288-300, 311-324`)

```rust
let operator = match SIGNING_KEY_TO_OPERATOR.may_load_at_height(
    deps.storage, signing_key_str.clone(), signature_data.reference_block as u64,
)? {
    Some(op) => op,
    None => match SIGNING_KEY_TO_OPERATOR.may_load(deps.storage, signing_key_str)? {
        Some(op) => op,
        None => { return Err(StdError::msg("Signer not registered")); }
    },
};
```

When the snapshot at `reference_block` returns `None`, the lookup falls back to the **current** mapping. Same fallback for `OPERATOR_WEIGHTS` (lines 312-324). Combined effect: an operator registered with weight *after* `reference_block` can contribute their weight to a signature that the protocol intends to evaluate against the historical operator set.

The whole point of `reference_block` is "we agree on a frozen view of the operator set." Silently widening that view defeats the abstraction. Concrete attack:
- At block N, operator set is {A=10, B=10, C=10}, total=30, threshold 2/3 = 20.
- Attacker controls A and C; signs message claiming `reference_block = N`.
- After block N, attacker registers D=10 as an additional operator.
- Attacker now signs message with {A, C, D}. Snapshot lookup of D at block N returns None → falls back to latest → D contributes weight 10. Signed weight = 30. Threshold check passes against historical total 30.

**Fix:** remove the fallback. If `may_load_at_height` returns `None` for the signing key, reject the signer.

---

**H-3 — Mirror stake-registry `set_operator_details_at` arithmetic underflows on Uint256** (`mirror/stake-registry/src/entry.rs:197`)

```rust
let new_total_weight = total_weight - current_weight + weight;
```

`Uint256` panics on underflow in cosmwasm (the operator-overload path). If `current_weight > total_weight` (storage corruption, double-decrement, or a reorder bug), this panics → contract becomes unable to update operators.

**Fix:** use `total_weight.checked_sub(current_weight).unwrap_or_default().checked_add(weight).map_err(...)`. Or save the new total only if both ops succeed.

---

**H-4 — Mirror service-handler base `save_envelope` overwrites silently** (`mirror/service-handler/src/state.rs:14-30`)

```rust
TRIGGER_MESSAGE.save(storage, message_with_id.trigger_id, &message_with_id.message)?;
SIGNATURE_DATA.save(storage, message_with_id.trigger_id, &signature_data)?;
```

`Map::save` overwrites without complaint. Two different signed envelopes that happen to share the same `trigger_id` (operator-quorum signs both — accidentally or maliciously) **silently overwrite each other**. Downstream consumers that read `TRIGGER_MESSAGE[trigger_id]` get the second message, not the first.

The `mirror-operator-sync-handler` and `mirror-quorum-sync-handler` (which embed this base for queries but have their own `execute`) do enforce monotonic `LAST_TRIGGER_ID` checks. The bare `mirror/service-handler` does not. So a deployment that uses the base handler directly is exposed.

**Fix:** add `if TRIGGER_MESSAGE.has(storage, trigger_id) { return Err(...) }` to `save_envelope`, or store a monotonic last-id and reject regression.

---

**H-5 — `LAST_TRIGGER_ID` strictly-greater check enables grief-DoS** (`mirror/mirror-operator-sync-handler/src/entry.rs:48-57`, same in quorum-sync-handler)

```rust
if let Some(last_trigger_id) = LAST_TRIGGER_ID.may_load(deps.storage)? {
    ensure!(last_trigger_id < triggerId, StdError::msg("Invalid trigger id"));
    LAST_TRIGGER_ID.save(deps.storage, &triggerId)?;
}
```

A malicious operator quorum (or an honest one signing a buggy event) that submits `triggerId == u64::MAX` permanently bricks future updates — every subsequent legitimate `triggerId < MAX` is rejected.

**Fix:** require monotonicity within a reasonable bound (`triggerId <= last + MAX_GAP`), or use a window-based replay cache instead of a hard sentinel.

---

**H-6 — Owner transfer absent on stake-registry** (`mirror/stake-registry/src/entry.rs:52`)

`OWNER` is set once in `instantiate`. No `TransferOwnership` execute message. If the deploying CLI key is compromised or rotated, the only recovery is full redeploy + state migration. Combined with C-5(a), this means there is no in-protocol path to give the operator-sync-handler control of the stake registry.

**Fix:** add `ExecuteMsg::TransferOwnership { new_owner }` gated on current owner.

---

### Medium

**M-1 — Trigger contract has no auth on `Push`** (`trigger/simple/src/entry.rs:33-47`)

Anyone can push trigger messages. Likely intentional (it's a public bus), but worth documenting — relayers and indexers reading `TriggerMessages` should treat them as user-input, not operator-authoritative.

---

**M-2 — `eip191_hash_message(keccak256(envelope.as_slice()))` matches Solidity convention but is undocumented** (`mirror/stake-registry/src/entry.rs:370`)

Double-hash: keccak first, then EIP-191 personal-sign prefix. This matches `signer.isValidSignatureNow(digest, sig)` on the EVM side where `digest = keccak256(envelope)` and the signer treats it as `personal_sign`. Verified against the comment on line 340 ("Mimics Solidity's signer.isValidSignatureNow"), but the cross-chain digest contract is not documented anywhere outside the comment. If the EVM side ever deviates (e.g., uses raw `keccak256(envelope)` without EIP-191), all signatures break silently.

**Fix:** add a unit test that pins the digest format with a known-vector envelope/signature pair.

---

**M-3 — Mock api duplicates state outside Mock service-manager** (`mock/api/src/service_manager.rs`, `mock/service-manager/src/entry.rs:73-82`)

The `SetSigningKey` execute message is defined in api and consumed by service-manager. Both are needed for the mock workflow, but the api crate is publishable as a dependency — anyone using the api thinking they want a service-manager interface would import the unauthenticated `SetSigningKey` definition.

**Fix:** put mock-only types behind a `mock` feature flag in the api crate, or rename to make the test-only intent obvious (`MockServiceManagerExecuteMsg`).

---

**M-4 — Quorum threshold change has no validation against existing signed work** (`mirror/service-manager/src/entry.rs:59-83`)

When the admin (or in the broken pipe-dream of C-5(b), the quorum-sync-handler) updates `WavsSetQuorumThreshold`, in-flight envelopes that were signed under the old threshold may now be re-validated against the new one. Solution: require quorum thresholds to be evaluated as-of `signature_data.reference_block`, parallel to how operator weights are. Currently the threshold is read from storage at validation time, not snapshotted.

**Fix:** snapshot quorum threshold like `OPERATOR_WEIGHTS` (use `SnapshotItem` from cw-storage-plus).

---

**M-5 — `ecdsa::api::ExecuteMsg::Wavs` is `serde(untagged)` over a `ServiceManagerExecuteMessages` enum** (`ecdsa/api/src/service_manager.rs:11-14`)

Untagged enums in serde can swallow malformed input and dispatch to the wrong variant. Combined with the open `WavsSetServiceUri`/`WavsSetQuorumThreshold` (C-3), an attacker crafting odd JSON could exercise unintended paths. Low-likelihood given the small variant set, but worth tightening — explicit tag preferred.

---

### Low

- **L-1** Many `// TODO` comments in production code paths (`ecdsa/service-manager/src/entry.rs:51, 90, 97`, BLS same, `mock` same). These are signal — review them before any deploy.
- **L-2** `OPERATOR_WEIGHTS` map in ecdsa/bls service-managers is never written to; still queried. Returns `Err` (load failure) if queried for any operator. Pure dead code.
- **L-3** Mirror stake-registry returns `[u8; 20]` from `ethereum_address_raw` with `.unwrap()` on the `try_into`; can panic if logic above breaks (currently safe, but defensive `?` is preferred).
- **L-4** No `#[entry_point]` for `migrate` on any contract — chain-level migrations would have no in-contract handler. Acceptable for v0; worth thinking about before mainnet.
- **L-5** `cw_serde` derives include `Debug` on `WavsSignatureData`; signature bytes appear in any error messages or events. Probably fine for ECDSA sigs but worth knowing.

### Info

- **I-1** Total Rust LOC in `packages/contracts`: ~4012. Mirror (~1397) is the only family with substantive logic.
- **I-2** Memory note `memory/cw-middleware.md` says "Mock, ECDSA, BLS variants of ServiceHandler and ServiceManager contracts (CosmWasm)" — needs updating to reflect that only mirror is real.
- **I-3** Quorum check uses `Uint256.full_mul + Uint512` to avoid overflow when computing `total * numerator`. Good defensive practice.
- **I-4** `EvmAddr` from `layer_climb_address` is the cross-chain address representation. Worth a separate review of *that* crate's encoding/decoding correctness — out of scope here.

---

## Recommendations (priority-ordered)

1. **Decide the future of the ecdsa/bls families.** Either delete them or port real verification logic from `mirror/stake-registry::is_valid_signature`. Right now they are misleading dead code (C-1, C-2, H-1).
2. **Fix the deploy-flow ownership gap** (C-5). Either give sync-handlers ownership of their downstream contracts during deploy, or add transfer-ownership messages and update the CLI.
3. **Remove the snapshot-fallback in mirror/stake-registry** (H-2). This is the highest-impact correctness fix in the audit.
4. **Add owner gates to the ecdsa/bls/mock service-managers** (C-3, C-4) before they ship under any name a third party might mistake for production.
5. **Add replay protection to mirror/service-handler base** (H-4) and bound the trigger-id monotonicity check (H-5).
6. **Pin the cross-chain digest format with a vector test** (M-2).

## Residual risks acknowledged

- **No external audit, no fork tests, no test runs.** This is a code-read audit. The mirror stack has tests in `packages/tests/` that I did not exercise.
- **`layer_climb_address::EvmAddr`** correctness assumed; out-of-scope here.
- **Cosmos chain native module assumptions.** The audit assumes `secp256k1_recover_pubkey` and `secp256k1_verify` host calls behave per cosmwasm spec. Some chains have surprising semantics (e.g., custom signing curves); a per-chain integration check is needed before deploy.
- **CLI deploy flow assumed canonical.** If a different deploy script is used in production, C-5 may not apply (handler could be made the owner). That deploy script should be audited too.

## Coverage limits of this audit

- Did not run `cargo test` or `cargo build` against the workspace.
- Did not analyze `packages/cli/`, `packages/sdk/`, or `packages/utils/` beyond the `instantiate` flow needed to evaluate C-5.
- Did not analyze the `wavs_types::contracts::cosmwasm` upstream types crate (signature/envelope schema lives there).
- Did not audit the BLS curve operations (because there are none — see C-2).

## Suggested next steps (if Jake wants to push this further)

1. **Write a unit test for C-5**: instantiate the stake registry from the CLI signer, then attempt `BatchSetOperatorDetails` from a contract address — assert it fails. Then prove the fix by re-instantiating with the handler as owner and re-running.
2. **Write a snapshot-correctness test for H-2**: register operator A at block N with weight 10, register B at block N+5, sign envelope with `reference_block=N` listing both A and B; assert that B's weight does NOT contribute.
3. **Diff the `bls/` and `ecdsa/` directories** to confirm they are truly identical except for crate-name imports, then make a decision: delete or implement.

---

_Audit produced 2026-05-09 by Arc heartbeat agent (iteration 2)._
