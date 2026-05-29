import { describe, expect, it } from "vitest";
import {
  onMatchPlayed,
  onNewChampion,
  onPvpResolved,
  postDailyStandings,
  type Broadcaster,
} from "../src/core/handlers.ts";

const ALICE = "0x1111111111111111111111111111111111111111111111111111111111111111";

function recorder(): { calls: Array<[string, string]>; broadcaster: Broadcaster } {
  const calls: Array<[string, string]> = [];
  const broadcaster: Broadcaster = {
    chat: async (m) => void calls.push(["chat", m]),
    board: async (m) => void calls.push(["board", m]),
    x: async (m) => void calls.push(["x", m]),
  };
  return { calls, broadcaster };
}

describe("event routing", () => {
  it("posts a house match only to chat", async () => {
    const { calls, broadcaster } = recorder();
    await onMatchPlayed(
      { player: ALICE, playerMove: "Rock", houseMove: "Scissors", outcome: "Win", newRating: 1516 },
      broadcaster,
    );
    expect(calls.map(([channel]) => channel)).toEqual(["chat"]);
  });

  it("announces a new champion on both board and x", async () => {
    const { calls, broadcaster } = recorder();
    await onNewChampion({ player: ALICE, rating: 1800 }, broadcaster);
    expect(calls.map(([channel]) => channel)).toEqual(["board", "x"]);
    expect(calls.every(([, message]) => message.includes("champion"))).toBe(true);
  });

  it("posts a pvp result to chat", async () => {
    const { calls, broadcaster } = recorder();
    await onPvpResolved(
      { winner: ALICE, challengerMove: "Rock", opponentMove: "Scissors", payoutVara: 9_750_000_000_000n },
      broadcaster,
    );
    expect(calls.map(([channel]) => channel)).toEqual(["chat"]);
  });

  it("posts daily standings to board and x", async () => {
    const { calls, broadcaster } = recorder();
    await postDailyStandings(3, 41, [{ player: ALICE, rating: 1900 }], broadcaster);
    expect(calls.map(([channel]) => channel)).toEqual(["board", "x"]);
  });
});
