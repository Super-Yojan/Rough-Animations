// Tiny syntax highlighter, copy buttons, tabs, docs search and scroll-spy. No dependencies.
const esc = s => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const RUST_KW = /^(use|fn|let|mut|as|pub|struct|enum|impl|for|in|if|else|match|return|const|move|crate|mod|where|dyn|self|Self|true|false)$/;

function highlightRust(src) {
  const re = /(\/\/[^\n]*)|("(?:[^"\\]|\\.)*"|r#?"[^"]*"#?)|(\b\d[\d_.]*\b)|(\b[A-Za-z_][A-Za-z0-9_]*\b)(\s*[(!])?|(\?)/g;
  let out = "", last = 0, m;
  while ((m = re.exec(src))) {
    out += esc(src.slice(last, m.index));
    const [all, com, str, num, id, call] = m;
    if (com) out += `<span class="c">${esc(com)}</span>`;
    else if (str) out += `<span class="s">${esc(str)}</span>`;
    else if (num) out += `<span class="n">${num}</span>`;
    else if (id && RUST_KW.test(id)) out += `<span class="k">${id}</span>${call || ""}`;
    else if (id && call) out += `<span class="f">${id}</span>${call}`;
    else if (id && /^[A-Z]/.test(id)) out += `<span class="t">${id}</span>`;
    else out += esc(all);
    last = m.index + all.length;
  }
  return out + esc(src.slice(last));
}
function highlightShell(src) {
  return src.split("\n").map(l =>
    l.startsWith("#") ? `<span class="c">${esc(l)}</span>` :
    esc(l).replace(/^(\$?\s*)(cargo|ffmpeg|cp|git|brew|apt|apt-get|pdflatex)\b/, '$1<span class="f">$2</span>')
          .replace(/(\s)(--?[a-z][\w-]*)/g, '$1<span class="t">$2</span>')
  ).join("\n");
}
document.querySelectorAll("pre code").forEach(c => {
  const lang = c.dataset.lang || "rust";
  const src = c.textContent.replace(/^\n/, "").replace(/\s+$/, "");
  c.innerHTML = lang === "sh" ? highlightShell(src) : lang === "toml" ? esc(src).replace(/^(\[.*\])$|(&quot;|")([^"\n]*)"/gm, (_m, tbl, _q, body) => tbl ? `<span class="t">${tbl}</span>` : `<span class="s">"${body}"</span>`) : lang === "text" ? esc(src) : highlightRust(src);
  c.dataset.raw = src;
});

// Copy buttons for any .win with a button, and for the install pill.
document.querySelectorAll(".win-bar button").forEach(b => b.addEventListener("click", () => {
  const code = b.closest(".win").querySelector("code");
  navigator.clipboard.writeText(code.dataset.raw);
  const t = b.textContent; b.textContent = "copied"; setTimeout(() => (b.textContent = t), 1200);
}));
document.querySelectorAll(".install").forEach(el => el.addEventListener("click", () => navigator.clipboard.writeText(el.dataset.copy)));

// Tabs
document.querySelectorAll(".tabs").forEach(box => {
  const btns = box.querySelectorAll(".tab-row button"), panes = box.querySelectorAll(".tab-pane");
  btns.forEach((b, i) => b.addEventListener("click", () => {
    btns.forEach((x, j) => { x.setAttribute("aria-selected", i === j); panes[j].classList.toggle("on", i === j); });
  }));
});

// Docs: heading anchors, sidebar search, scroll-spy, right-hand table of contents.
const main = document.querySelector(".main");
if (main) {
  main.querySelectorAll("h2[id], h3[id]").forEach(h => h.insertAdjacentHTML("beforeend", `<a class="anc" href="#${h.id}">#</a>`));
  const toc = document.querySelector(".toc div");
  if (toc) main.querySelectorAll("h2[id]").forEach(h => toc.insertAdjacentHTML("beforeend", `<a href="#${h.id}">${h.firstChild.textContent}</a>`));
  const search = document.querySelector(".search");
  search?.addEventListener("input", () => {
    const q = search.value.toLowerCase();
    document.querySelectorAll(".side a").forEach(a => a.classList.toggle("hide", q && !(a.textContent.toLowerCase().includes(q) || (a.dataset.k || "").includes(q))));
  });
  const links = [...document.querySelectorAll(".side a, .toc a")];
  const spy = new IntersectionObserver(es => es.forEach(e => {
    if (e.isIntersecting) links.forEach(a => a.classList.toggle("on", a.getAttribute("href") === "#" + e.target.id));
  }), { rootMargin: "-80px 0px -75% 0px" });
  main.querySelectorAll("h2[id]").forEach(h => spy.observe(h));
}
