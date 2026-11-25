import init, { Figure } from "./wasm/rough_engine.js";

export { Figure };

let loading: Promise<void> | undefined;

/**
 * Load the WebAssembly engine once. Pass `wasmUrl` to serve the `.wasm` file from somewhere
 * else (a CDN, a bundler asset URL). By default it is found next to this module.
 */
export function loadEngine(wasmUrl?: string | URL): Promise<void> {
  loading ??= init({ module_or_path: wasmUrl ?? new URL("./wasm/rough_engine_bg.wasm", import.meta.url) }).then(
    () => undefined,
  );
  return loading;
}
