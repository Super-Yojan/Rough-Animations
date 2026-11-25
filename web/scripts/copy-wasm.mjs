// tsc does not copy the generated wasm glue, so put it next to the compiled output.
import { cpSync, existsSync } from "node:fs";
if (!existsSync("src/wasm/rough_engine_bg.wasm")) {
  console.error("src/wasm is missing: run `pnpm build:wasm` first (see scripts/build-web.sh).");
  process.exit(1);
}
cpSync("src/wasm", "dist/wasm", { recursive: true });
