#!/usr/bin/env sh
# Build the web engine and library and copy them into the docs site, so the live figures and the
# catalog work on GitHub Pages (which only serves static files from docs/site/).
#
#   scripts/build-site.sh        # rebuild, then commit docs/site/engine and docs/site/catalog.html
#
# Needs what scripts/build-web.sh needs (wasm32 target, wasm-bindgen-cli) plus pnpm.
set -eu
cd "$(dirname "$0")/.."

./scripts/build-web.sh >/dev/null
(cd web && pnpm install --silent && pnpm build >/dev/null)

rm -rf docs/site/engine
mkdir -p docs/site/engine
rsync -a --exclude='*.d.ts' web/dist/ docs/site/engine/

# The catalog page, pointed at the copied engine and given a link back to the docs.
python3 - <<'PY'
src = open("web/catalog/index.html").read()
src = src.replace("../dist/", "./engine/")
nav = (
    '<div style="background:#23232a;color:#fff;font:14px ui-sans-serif,system-ui,sans-serif;padding:8px 24px;'
    'display:flex;gap:18px;align-items:center">'
    '<a href="index.html" style="color:#fff;text-decoration:none;font-weight:700">Rough-Animations</a>'
    '<a href="docs.html" style="color:#cfcbe6;text-decoration:none">Docs</a>'
    '<a href="docs.html#web-figures" style="color:#cfcbe6;text-decoration:none">About the library</a>'
    '<span style="flex:1"></span>'
    '<a href="https://github.com/Super-Yojan/Rough-Animations" style="color:#cfcbe6;text-decoration:none">GitHub</a></div>'
)
assert "<body>" in src
src = src.replace("<body>", "<body>\n" + nav, 1)
src = src.replace("<title>rough-figure · figure library</title>", "<title>Figure library · Rough-Animations</title>")
open("docs/site/catalog.html", "w").write(src)
PY
du -sh docs/site/engine | cut -f1
