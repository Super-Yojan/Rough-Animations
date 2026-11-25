/**
 * The figure library: every ready-made figure with its metadata. Import a single figure to keep
 * your bundle small (`import { emptyInbox } from "rough-figure/library"` still pulls in the whole
 * list, so for one figure prefer `rough-figure/library/react` and its component).
 */
import { accessDenied } from "./figures/access-denied.js";
import { analytics } from "./figures/analytics.js";
import { emptyInbox } from "./figures/empty-inbox.js";
import { loading } from "./figures/loading.js";
import { notFound } from "./figures/not-found.js";
import { notifications } from "./figures/notifications.js";
import { rocket } from "./figures/rocket.js";
import { servers } from "./figures/servers.js";
import { settings } from "./figures/settings.js";
import { success } from "./figures/success.js";
import { upload } from "./figures/upload.js";
export * from "./types.js";
export { accessDenied, analytics, emptyInbox, loading, notFound, notifications, rocket, servers, settings, success, upload };
/** Every figure in the library. */
export const library = [
    emptyInbox, upload, notFound, accessDenied, loading, success, notifications, servers, analytics, settings, rocket,
];
/** Look a figure up by its id (`"empty-inbox"`). */
export const figureById = (id) => library.find((f) => f.id === id);
