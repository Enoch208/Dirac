export type MoveName = "Rock" | "Paper" | "Scissors";
export type OutcomeName = "Win" | "Loss" | "Draw";

export interface HouseMatch {
  player: string;
  playerMove: MoveName;
  houseMove: MoveName;
  outcome: OutcomeName;
  newRating: number;
}

export interface ChampionChange {
  player: string;
  rating: number;
}

export interface PvpResult {
  winner: string | null;
  challengerMove: MoveName;
  opponentMove: MoveName;
  payoutVara: bigint;
}

export interface StandingsEntry {
  player: string;
  rating: number;
}

const ARENA_HANDLE = "@dirac";
const ARENA_NAME = "the Dirac Colosseum";
const ADDRESS_PREFIX_LEN = 6;
const ADDRESS_SUFFIX_LEN = 4;
const VARA_DECIMALS = 12n;
const VARA_FRACTION_DIGITS = 2;
const DEFAULT_STANDINGS_LIMIT = 10;

const MOVE_GLYPH: Record<MoveName, string> = {
  Rock: "🪨",
  Paper: "📄",
  Scissors: "✂️",
};

export function shortActor(id: string): string {
  const body = id.startsWith("0x") ? id.slice(2) : id;
  if (body.length <= ADDRESS_PREFIX_LEN + ADDRESS_SUFFIX_LEN) {
    return `0x${body}`;
  }
  return `0x${body.slice(0, ADDRESS_PREFIX_LEN)}…${body.slice(-ADDRESS_SUFFIX_LEN)}`;
}

export function formatVara(raw: bigint): string {
  const unit = 10n ** VARA_DECIMALS;
  const whole = raw / unit;
  const scale = 10n ** BigInt(VARA_FRACTION_DIGITS);
  const fraction = ((raw % unit) * scale) / unit;
  return `${whole}.${fraction.toString().padStart(VARA_FRACTION_DIGITS, "0")}`;
}

export function houseMatchPost(match: HouseMatch): string {
  const who = shortActor(match.player);
  const duel = `${MOVE_GLYPH[match.playerMove]} vs ${MOVE_GLYPH[match.houseMove]}`;
  const verdict =
    match.outcome === "Win"
      ? `${who} beat the house`
      : match.outcome === "Loss"
        ? `${who} fell to the house`
        : `${who} drew the house`;
  return `⚔️ ${verdict} ${duel} — now rated ${match.newRating} in ${ARENA_NAME}. Challenge it: ${ARENA_HANDLE}`;
}

export function championPost(change: ChampionChange): string {
  return `👑 New champion — ${shortActor(change.player)} tops ${ARENA_NAME} at ${change.rating}. Come dethrone them. ${ARENA_HANDLE}`;
}

export function pvpResultPost(result: PvpResult): string {
  const duel = `${MOVE_GLYPH[result.challengerMove]} vs ${MOVE_GLYPH[result.opponentMove]}`;
  if (result.winner === null) {
    return `🤝 A staked duel in ${ARENA_NAME} ended in a draw ${duel} — stakes returned. ${ARENA_HANDLE}`;
  }
  return `⚔️ ${shortActor(result.winner)} won a staked duel ${duel}, taking ${formatVara(result.payoutVara)} VARA in ${ARENA_NAME}. ${ARENA_HANDLE}`;
}

export function dailyStandingsPost(
  day: number,
  totalDuels: number,
  entries: StandingsEntry[],
  limit: number = DEFAULT_STANDINGS_LIMIT,
): string {
  const board = entries
    .slice(0, limit)
    .map((entry, index) => `${index + 1}. ${shortActor(entry.player)} — ${entry.rating}`)
    .join("\n");
  return `Day ${day} in ${ARENA_NAME} — ${totalDuels} duels today.\n${board}\nOne message to play: ${ARENA_HANDLE}`;
}
