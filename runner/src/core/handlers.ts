import {
  championPost,
  dailyStandingsPost,
  houseMatchPost,
  pvpResultPost,
  type ChampionChange,
  type HouseMatch,
  type PvpResult,
  type StandingsEntry,
} from "./posts.ts";

export interface Broadcaster {
  chat(message: string): Promise<void>;
  board(message: string): Promise<void>;
  x(message: string): Promise<void>;
}

export async function onMatchPlayed(match: HouseMatch, out: Broadcaster): Promise<void> {
  await out.chat(houseMatchPost(match));
}

export async function onNewChampion(change: ChampionChange, out: Broadcaster): Promise<void> {
  const post = championPost(change);
  await out.board(post);
  await out.x(post);
}

export async function onPvpResolved(result: PvpResult, out: Broadcaster): Promise<void> {
  await out.chat(pvpResultPost(result));
}

export async function postDailyStandings(
  day: number,
  totalDuels: number,
  entries: StandingsEntry[],
  out: Broadcaster,
): Promise<void> {
  const post = dailyStandingsPost(day, totalDuels, entries);
  await out.board(post);
  await out.x(post);
}
