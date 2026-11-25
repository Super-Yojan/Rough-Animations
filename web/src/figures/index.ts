/**
 * The first four figures, as plain specs. Kept for convenience: the full catalogue, with
 * metadata and typed React components, is in `rough-figure/library`.
 */
import { emptyInbox, rocket, servers, settings } from "../library/index.js";

export const inbox = emptyInbox.spec;
export const figures = { inbox, settings: settings.spec, rocket: rocket.spec, servers: servers.spec } as const;
export type FigureName = keyof typeof figures;
