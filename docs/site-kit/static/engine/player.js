import { Figure, loadEngine } from "./engine.js";
import { toJSON } from "./spec.js";
const DEG_PER_PIXEL = 0.45;
const CLICK_SLOP = 4; // pixels the pointer may move and still count as a click
/**
 * Draws one figure on a canvas and handles time, turning and clicking. The engine does all the
 * drawing; this class decides *when* to draw, and only draws when something changed.
 */
export class FigurePlayer {
    canvas;
    ctx;
    fig;
    opts;
    raf = 0;
    dirty = true;
    last = 0;
    t = 0;
    visible = true;
    holdUntil = 0;
    atEnd = false;
    _playing = false;
    spin = 0; // degrees per second left over from a fling
    hover = null;
    drag = null;
    cleanup = [];
    destroyed = false;
    ratio = 1;
    /** How long the last frame took to draw, in milliseconds. */
    lastFrameMs = 0;
    constructor(canvas, opts) {
        this.canvas = canvas;
        this.opts = opts;
        const ctx = canvas.getContext("2d");
        if (!ctx)
            throw new Error("rough-figure: this canvas has no 2d context");
        this.ctx = ctx;
    }
    /** Load the engine, build the figure and start it. */
    static async create(canvas, spec, opts = {}) {
        const p = new FigurePlayer(canvas, opts);
        try {
            await loadEngine(opts.wasmUrl);
            if (p.destroyed)
                return p;
            const ratio = opts.pixelRatio ?? Math.min(Math.max(globalThis.devicePixelRatio ?? 1, 1), 3);
            p.fig = new Figure(typeof spec === "string" ? spec : toJSON(spec), ratio);
            p.ratio = ratio;
        }
        catch (e) {
            const err = e instanceof Error ? e : new Error(String(e));
            opts.onError?.(err);
            throw err;
        }
        p.start();
        return p;
    }
    get duration() { return this.fig?.duration() ?? 0; }
    get time() { return this.t; }
    get playing() { return this._playing; }
    /** Current turn in degrees. */
    get yaw() { return this.fig?.yaw() ?? 0; }
    play() {
        if (!this.fig)
            return;
        if (this.t >= this.duration)
            this.seek(0);
        this._playing = true;
        this.last = 0;
        this.kick();
    }
    pause() { this._playing = false; }
    /** Start the timeline again from the top. */
    replay() { this.seek(0); this.play(); }
    /** Jump to time `t` seconds. */
    seek(t) {
        if (!this.fig)
            return;
        this.atEnd = false;
        this.t = Math.min(Math.max(t, 0), this.duration);
        this.fig.seek(this.t);
        this.invalidate();
    }
    /** Jump to a fraction `u` (0 to 1) of the timeline and pause: for scroll-driven stories. */
    scrub(u) {
        this.pause();
        this.seek(Math.min(Math.max(u, 0), 1) * this.duration);
    }
    setYaw(degrees) {
        if (!this.fig)
            return;
        this.fig.set_yaw(degrees);
        this.invalidate();
    }
    setTheme(name) { this.fig?.set_theme(name); this.invalidate(); }
    setLook(look) { this.fig?.set_look(look); this.invalidate(); }
    setSpeed(speed) { this.opts.speed = speed; }
    /** The current state as an SVG document: sharp at any size, for posters and static fallbacks. */
    toSVG() {
        return this.fig?.to_svg() ?? "";
    }
    /** Draw the current state as a PNG blob (for "download" buttons). */
    toBlob(type = "image/png") {
        this.drawNow();
        return new Promise((resolve) => this.canvas.toBlob(resolve, type));
    }
    destroy() {
        this.destroyed = true;
        cancelAnimationFrame(this.raf);
        this.cleanup.forEach((f) => f());
        this.cleanup = [];
        this.fig?.free();
        this.fig = undefined;
    }
    // ---- internals ---------------------------------------------------------------------------
    start() {
        const fig = this.fig;
        this.canvas.width = fig.width();
        this.canvas.height = fig.height();
        // The canvas has device pixels; show it at the figure's own size, or smaller to fit.
        if (!this.canvas.style.maxWidth)
            this.canvas.style.maxWidth = `${fig.width() / this.ratio}px`;
        this.canvas.setAttribute("role", "img");
        if (this.opts.interactive !== false)
            this.attachInput();
        this.observeVisibility();
        const reduced = matchMedia?.("(prefers-reduced-motion: reduce)").matches;
        if (this.opts.autoplay === false || reduced)
            this.seek(this.duration);
        else
            this.play();
        this.invalidate();
        this.opts.onReady?.(this);
    }
    invalidate() { this.dirty = true; this.kick(); }
    kick() {
        if (!this.raf && !this.destroyed && this.visible)
            this.raf = requestAnimationFrame((n) => this.frame(n));
    }
    frame(now) {
        this.raf = 0;
        if (!this.fig || this.destroyed)
            return;
        const dt = this.last ? Math.min((now - this.last) / 1000, 0.1) : 0;
        this.last = now;
        if (this._playing) {
            if (this.atEnd) {
                // Showing the finished figure before a loop restarts.
                if (now >= this.holdUntil) {
                    this.atEnd = false;
                    this.t = 0;
                    this.fig.seek(0);
                    this.dirty = true;
                }
            }
            else {
                this.t += dt * (this.opts.speed ?? 1);
                if (this.t >= this.duration) {
                    this.t = this.duration;
                    if (this.opts.loop) {
                        this.atEnd = true;
                        this.holdUntil = now + (this.opts.loopDelay ?? 1.2) * 1000;
                    }
                    else {
                        this._playing = false;
                    }
                }
                this.fig.seek(this.t);
                this.dirty = true;
            }
        }
        if (this.spin !== 0 && !this.drag) {
            this.fig.set_yaw(this.fig.yaw() + this.spin * dt);
            this.spin *= Math.pow(0.04, dt); // friction: most of the fling is gone in about a second
            if (Math.abs(this.spin) < 1)
                this.spin = 0;
            this.dirty = true;
        }
        if (this.dirty)
            this.drawNow();
        if (this._playing || this.spin !== 0 || this.dirty)
            this.kick();
    }
    drawNow() {
        if (!this.fig)
            return;
        this.dirty = false;
        const t0 = performance.now();
        const bytes = this.fig.render();
        const w = this.fig.width();
        const h = this.fig.height();
        // `render` hands back a copy, so the ImageData can own the buffer.
        this.ctx.putImageData(new ImageData(new Uint8ClampedArray(bytes.buffer, bytes.byteOffset, bytes.length), w, h), 0, 0);
        this.lastFrameMs = performance.now() - t0;
    }
    toCanvas(e) {
        const r = this.canvas.getBoundingClientRect();
        return [((e.clientX - r.left) * this.canvas.width) / r.width, ((e.clientY - r.top) * this.canvas.height) / r.height];
    }
    pick(x, y) {
        const hit = this.fig?.pick(x, y);
        return hit ? JSON.parse(hit) : null;
    }
    attachInput() {
        const c = this.canvas;
        c.style.touchAction = "pan-y"; // vertical scroll still works; horizontal drags turn the figure
        c.tabIndex = 0;
        const on = (type, fn) => {
            c.addEventListener(type, fn);
            this.cleanup.push(() => c.removeEventListener(type, fn));
        };
        on("pointerdown", (e) => {
            c.setPointerCapture(e.pointerId);
            this.spin = 0;
            this.drag = { x: e.clientX, y: e.clientY, moved: false, lastX: e.clientX, lastT: e.timeStamp, v: 0 };
            c.style.cursor = "grabbing";
        });
        on("pointermove", (e) => {
            const d = this.drag;
            if (d) {
                if (!d.moved && Math.hypot(e.clientX - d.x, e.clientY - d.y) > CLICK_SLOP)
                    d.moved = true;
                if (d.moved && this.fig) {
                    const dx = e.clientX - d.lastX;
                    this.fig.set_yaw(this.fig.yaw() + dx * DEG_PER_PIXEL);
                    const dt = Math.max(e.timeStamp - d.lastT, 1);
                    d.v = (dx * DEG_PER_PIXEL * 1000) / dt;
                    d.lastX = e.clientX;
                    d.lastT = e.timeStamp;
                    this.invalidate();
                }
                return;
            }
            const [x, y] = this.toCanvas(e);
            const hit = this.pick(x, y);
            const id = hit?.action ? hit.id ?? "" : null;
            c.style.cursor = hit?.action ? "pointer" : "grab";
            if (id !== this.hover) {
                this.hover = id;
                this.fig?.set_hover(id === "" ? undefined : id);
                this.invalidate();
            }
        });
        const release = (e) => {
            const d = this.drag;
            this.drag = null;
            c.style.cursor = "grab";
            if (!d)
                return;
            if (!d.moved) {
                const [x, y] = this.toCanvas(e);
                const hit = this.pick(x, y);
                if (hit?.action)
                    this.opts.onAction?.({ action: hit.action, id: hit.id });
            }
            else if (Math.abs(d.v) > 40 && e.timeStamp - d.lastT < 80) {
                this.spin = Math.max(-720, Math.min(720, d.v));
                this.kick();
            }
        };
        on("pointerup", release);
        on("pointercancel", release);
        on("pointerleave", () => {
            if (this.hover !== null) {
                this.hover = null;
                this.fig?.set_hover(undefined);
                this.invalidate();
            }
        });
        on("keydown", (e) => {
            if (!this.fig)
                return;
            const step = e.shiftKey ? 45 : 15;
            if (e.key === "ArrowLeft")
                this.setYaw(this.fig.yaw() - step);
            else if (e.key === "ArrowRight")
                this.setYaw(this.fig.yaw() + step);
            else
                return;
            e.preventDefault();
        });
        c.style.cursor = "grab";
    }
    /** Stop drawing while the figure is scrolled out of view. */
    observeVisibility() {
        if (typeof IntersectionObserver === "undefined")
            return;
        const io = new IntersectionObserver((entries) => {
            this.visible = entries.some((e) => e.isIntersecting);
            if (this.visible) {
                this.last = 0;
                this.kick();
            }
        });
        io.observe(this.canvas);
        this.cleanup.push(() => io.disconnect());
    }
}
