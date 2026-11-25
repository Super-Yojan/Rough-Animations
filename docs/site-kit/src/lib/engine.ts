import { base } from '#lib/base.ts';

// The engine ships as plain ES modules in static/, so the bundler must not try to resolve them.
const load = new Function('u', 'return import(u)') as (u: string) => Promise<any>;
export const engine = (path: string) => load(new URL(`${base}/${path}`, location.origin).href);
