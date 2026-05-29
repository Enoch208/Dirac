# Dirac runner

The off-chain autonomous agent for the Dirac Colosseum. It subscribes to on-chain
`dirac` events, broadcasts results to the network's Chat/Board and to X, writes the
VARA/USD rate on-chain (feeding `Admin.SetVaraUsdRate`), and prompts winners to claim
payouts from their mailbox.

## Status

| Piece | State |
|---|---|
| Typed client generated from `dirac.idl` (`sails-js`) | ✅ `src/generated/dirac-client.ts` |
| Post formatting — shoutouts, champion, PvP result, daily standings | ✅ `src/core/posts.ts`, unit-tested |
| Event → channel routing (chat / board / x) | ✅ `src/core/handlers.ts`, unit-tested |
| Live event subscription + broadcasting + VaraBridge write | ⛔ blocked on deploy + §14 values |

`12 tests` cover the formatting and routing core (`npm test`). These are the
correctness-sensitive, deploy-independent pieces; the live chain I/O is wired through the
`Broadcaster` interface and the generated client once the blockers below are resolved.

## Commands

```bash
npm install      # test toolchain (chain deps added when wiring live I/O)
npm test         # vitest — core formatting + routing
npm run typecheck
```

## Wiring plan (once unblocked)

1. Connect `GearApi` to the mainnet RPC; instantiate `SailsProgram` with the deployed
   `dirac` program id.
2. `subscribeToMatchPlayedEvent` / `subscribeToNewChampionEvent` / `subscribeToPvpResolvedEvent`
   → normalize payloads to the `core` types → `onMatchPlayed` / `onNewChampion` / `onPvpResolved`.
3. Implement `Broadcaster` against the coordination layer (Chat + Board) and the X API.
4. On an interval, read VaraBridge price (an outgoing on-chain write = integration credit)
   and call `Admin.SetVaraUsdRate`; post daily standings via `postDailyStandings`.
5. Watch `PvpResolved` / `MatchForfeited` and prompt/automate winners to claim mailbox value
   before the mailbox timeout returns it to the program.

## Network constants (verified — see `src/network.ts`)

- Coordination layer: `0x19f27f4c906a5ac230be82d907850d44c7a7fff1b4c6903f62e78e09e0b353f3`
- VaraBridge: `0xfb7ed5a79dc2ff15283a524a4489321b5e1f6341db2b9892be83b9568cc1fcb4`, price via `GetPrice("VARA")` (a write → integration credit)
- RPC `wss://rpc.vara.network`, indexer `https://agents-api.vara.network/graphql`, voucher `https://voucher-backend-agents.vara.network/voucher`
- Registration track: `Open`

## Remaining blockers

- Deployed **`dirac` program id** on Vara mainnet (set after deploy).
- Operator wallet (mnemonic) with VARA for gas + the seeded pot.
