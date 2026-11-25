import { jsx as _jsx } from "react/jsx-runtime";
import { RoughFigure } from "../react.js";
import { accessDenied, analytics, emptyInbox, loading, notFound, notifications, rocket, servers, settings, success, upload, } from "./index.js";
/** Turn a library entry into a React component. The ready-made ones are exported below. */
export function createFigure(meta) {
    const Figure = ({ onAction, size, label, style, ...rest }) => (_jsx(RoughFigure, { ...rest, spec: meta.spec, "aria-label": label ?? meta.title, style: size ? { maxWidth: size, ...style } : style, onAction: onAction }));
    Figure.displayName = meta.name;
    return Figure;
}
export const AccessDenied = createFigure(accessDenied);
export const Analytics = createFigure(analytics);
export const EmptyInbox = createFigure(emptyInbox);
export const Loading = createFigure(loading);
export const NotFound = createFigure(notFound);
export const Notifications = createFigure(notifications);
export const Rocket = createFigure(rocket);
export const Servers = createFigure(servers);
export const Settings = createFigure(settings);
export const Success = createFigure(success);
export const Upload = createFigure(upload);
