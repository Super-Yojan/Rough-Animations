import { FigurePlayer, type ActionEvent } from "./player.js";
import type { FigureSpec, ThemeName } from "./spec.js";

/**
 * `<rough-figure>`: drop a figure into any page.
 *
 *   <rough-figure src="/figures/inbox.json" theme="dark" loop></rough-figure>
 *
 * or put the description inline:
 *
 *   <rough-figure><script type="application/json">{ "width": 900, ... }</script></rough-figure>
 *
 * Attributes: `src`, `theme`, `look`, `yaw`, `speed`, `loop`, `static` (show the end, no
 * autoplay), `fixed` (no dragging or clicking). Events: `ready`, `error`, and `action`
 * (`event.detail` is `{ action, id }`).
 */
export class RoughFigureElement extends HTMLElement {
  static observedAttributes = ["src", "theme", "look", "yaw", "speed"];
  #player: FigurePlayer | undefined;
  #canvas = document.createElement("canvas");
  #gen = 0;

  /** The running player, once the figure is ready. */
  get player(): FigurePlayer | undefined { return this.#player; }

  connectedCallback(): void {
    const root = this.shadowRoot ?? this.attachShadow({ mode: "open" });
    root.innerHTML = `<style>
      :host { display: block; max-width: 100%; }
      canvas { width: 100%; height: auto; display: block; outline-offset: 4px; }
    </style>`;
    root.append(this.#canvas);
    this.#canvas.setAttribute("aria-label", this.getAttribute("aria-label") ?? "Interactive illustration");
    void this.#load();
  }

  disconnectedCallback(): void {
    this.#gen++;
    this.#player?.destroy();
    this.#player = undefined;
  }

  attributeChangedCallback(name: string, _old: string | null, value: string | null): void {
    const p = this.#player;
    if (!p || value === null) { if (name === "src" && this.isConnected && this.#player) void this.#load(); return; }
    if (name === "theme") p.setTheme(value as ThemeName);
    else if (name === "look") p.setLook(value as "sketch" | "clean");
    else if (name === "yaw") p.setYaw(Number(value));
    else if (name === "speed") p.setSpeed(Number(value));
    else if (name === "src") void this.#load();
  }

  async #spec(): Promise<FigureSpec | string> {
    const src = this.getAttribute("src");
    if (src) {
      const res = await fetch(src);
      if (!res.ok) throw new Error(`rough-figure: could not load ${src} (${res.status})`);
      return res.text();
    }
    const inline = this.querySelector('script[type="application/json"]');
    if (!inline?.textContent) throw new Error("rough-figure: give it a src or an inline JSON <script>");
    return inline.textContent;
  }

  async #load(): Promise<void> {
    const gen = ++this.#gen;
    this.#player?.destroy();
    this.#player = undefined;
    try {
      let spec = await this.#spec();
      if (gen !== this.#gen) return;
      const theme = this.getAttribute("theme");
      const yaw = this.getAttribute("yaw");
      if (theme || yaw) {
        const obj = typeof spec === "string" ? (JSON.parse(spec) as FigureSpec) : spec;
        if (theme) obj.theme = theme as ThemeName;
        if (yaw) obj.yaw = Number(yaw);
        spec = JSON.stringify(obj);
      }
      const player = await FigurePlayer.create(this.#canvas, spec, {
        autoplay: !this.hasAttribute("static"),
        loop: this.hasAttribute("loop"),
        interactive: !this.hasAttribute("fixed"),
        speed: Number(this.getAttribute("speed") ?? 1),
        onAction: (detail: ActionEvent) => this.dispatchEvent(new CustomEvent("action", { detail, bubbles: true, composed: true })),
      });
      if (gen !== this.#gen) { player.destroy(); return; }
      this.#player = player;
      const look = this.getAttribute("look");
      if (look) player.setLook(look as "sketch" | "clean");
      this.dispatchEvent(new CustomEvent("ready"));
    } catch (e) {
      this.dispatchEvent(new CustomEvent("error", { detail: e }));
      console.error(e);
    }
  }
}

if (!customElements.get("rough-figure")) customElements.define("rough-figure", RoughFigureElement);
