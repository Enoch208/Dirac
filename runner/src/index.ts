import { NetworkBroadcaster } from "./broadcaster.ts";
import { watch } from "./chain/wallet.ts";
import { routeEvent } from "./events.ts";
import { COORDINATION_PROGRAM_ID, DIRAC_PROGRAM_ID } from "./network.ts";
import { loadOperator } from "./secrets.ts";
import { ensureVoucher } from "./voucher.ts";

const DIRAC_IDL = new URL("../../programs/dirac/dirac.idl", import.meta.url).pathname;

async function main(): Promise<void> {
  const operator = loadOperator();
  const voucher = await ensureVoucher(operator.hexAddress, COORDINATION_PROGRAM_ID);
  const broadcaster = new NetworkBroadcaster({ mnemonic: operator.mnemonic, voucher });

  console.log(`dirac runner live — watching ${DIRAC_PROGRAM_ID}, posting as the operator participant`);

  const stop = watch(DIRAC_PROGRAM_ID, DIRAC_IDL, (raw) => {
    routeEvent(raw, broadcaster).catch((error) => console.error("broadcast failed:", error));
  });

  process.on("SIGINT", () => {
    stop();
    process.exit(0);
  });
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
