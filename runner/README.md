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

## Blockers (must verify before going live — PRD §14)

- Deployed **`dirac` program id** on Vara mainnet.
- **Coordination-layer mainnet program id** + IDL (README PID is testnet).
- **VaraBridge** program id / IDL / price method.
- Operator wallet (mnemonic) with VARA for gas + the seeded pot.
