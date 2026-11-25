import { jsx as _jsx } from "react/jsx-runtime";
import { useEffect, useRef } from "react";
import { FigurePlayer } from "./player.js";
import { toJSON } from "./spec.js";
/** A figure as a React component. The figure restarts when `spec` changes (compared by value). */
export function RoughFigure({ spec, theme, look, yaw, loop, autoplay, interactive, speed, wasmUrl, onAction, onReady, className, style, ...rest }) {
    const canvas = useRef(null);
    const player = useRef();
    const handlers = useRef({ onAction, onReady });
    handlers.current = { onAction, onReady };
    const json = toJSON(theme ? { ...spec, theme } : spec);
    useEffect(() => {
        const el = canvas.current;
        if (!el)
            return;
        let dead = false;
        FigurePlayer.create(el, json, {
            autoplay, loop, interactive, speed, wasmUrl,
            onAction: (e) => handlers.current.onAction?.(e),
            onReady: (p) => handlers.current.onReady?.(p),
        })
            .then((p) => {
            if (dead)
                return p.destroy();
            player.current = p;
            if (look)
                p.setLook(look);
            if (yaw !== undefined)
                p.setYaw(yaw);
        })
            .catch(() => { });
        return () => {
            dead = true;
            player.current?.destroy();
            player.current = undefined;
        };
        // Look and yaw are applied live below, so they are not part of the figure's identity.
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [json, autoplay, loop, interactive, wasmUrl]);
    useEffect(() => { if (look)
        player.current?.setLook(look); }, [look]);
    useEffect(() => { if (yaw !== undefined)
        player.current?.setYaw(yaw); }, [yaw]);
    useEffect(() => { if (speed !== undefined)
        player.current?.setSpeed(speed); }, [speed]);
    return _jsx("canvas", { ref: canvas, className: className, style: { width: "100%", height: "auto", display: "block", ...style }, "aria-label": rest["aria-label"] ?? "Interactive illustration" });
}
