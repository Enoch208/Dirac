import { call } from "./chain/wallet.ts";
import type { Broadcaster, ChatPost } from "./core/handlers.ts";
import { COORDINATION_PROGRAM_ID, DIRAC_PROGRAM_ID } from "./network.ts";

const COORD_IDL = new URL("../refs/agents_network_client.idl", import.meta.url).pathname;
const BOARD_TAGS = ["dirac", "game", "rps", "arena"];

export interface BroadcasterConfig {
  mnemonic: string;
  voucher: string;
}

export class NetworkBroadcaster implements Broadcaster {
  constructor(private readonly config: BroadcasterConfig) {}

  async chat(post: ChatPost): Promise<void> {
    const author = { Application: DIRAC_PROGRAM_ID };
    const mentions = post.mentions.map((address) => ({ Participant: address }));
    await call(
      COORDINATION_PROGRAM_ID,
      "Chat/Post",
      [post.body, author, mentions, null],
      { idl: COORD_IDL, mnemonic: this.config.mnemonic, voucher: this.config.voucher },
    );
  }

  async board(title: string, body: string): Promise<void> {
    await call(
      COORDINATION_PROGRAM_ID,
      "Board/PostAnnouncement",
      [DIRAC_PROGRAM_ID, { title, body, tags: BOARD_TAGS }],
      { idl: COORD_IDL, mnemonic: this.config.mnemonic, voucher: this.config.voucher },
    );
  }

  async x(message: string): Promise<void> {
    console.log(`[x] ${message}`);
  }
}
