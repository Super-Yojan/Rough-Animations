// A tiny static server for the demo (the page imports ../dist, so serve the package root).
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join, normalize } from "node:path";
const types = { ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript", ".wasm": "application/wasm", ".json": "application/json", ".css": "text/css" };
const port = Number(process.env.PORT ?? 5173);
createServer(async (req, res) => {
  const path = normalize(decodeURIComponent(new URL(req.url, "http://x").pathname)).replace(/^(\.\.[/\\])+/, "");
  try {
    const file = join(process.cwd(), path.endsWith("/") ? path + "index.html" : path);
    const body = await readFile(file);
    res.writeHead(200, { "content-type": types[extname(file)] ?? "application/octet-stream" });
    res.end(body);
  } catch { res.writeHead(404).end("not found"); }
}).listen(port, () => console.log(`demo: http://localhost:${port}/demo/`));
