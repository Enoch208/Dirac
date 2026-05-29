import { VOUCHER_URL } from "./network.ts";

interface VoucherState {
  voucherId: string | null;
  programs: string[];
}

export async function ensureVoucher(operatorHex: string, coverProgramId: string): Promise<string> {
  const state = (await (await fetch(`${VOUCHER_URL}/${operatorHex}`)).json()) as VoucherState;
  if (state.voucherId && state.programs.includes(coverProgramId)) {
    return state.voucherId;
  }
  const response = await fetch(VOUCHER_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ account: operatorHex, programs: [coverProgramId] }),
  });
  const body = (await response.json()) as { voucherId?: string };
  if (!body.voucherId) {
    throw new Error(`voucher request failed (${response.status}): ${JSON.stringify(body)}`);
  }
  return body.voucherId;
}
