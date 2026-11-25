import type { FigureSpec } from "../spec.js";

export type Category = "empty-states" | "errors" | "loaders" | "status" | "product" | "onboarding";

export const categories: { id: Category; title: string }[] = [
  { id: "empty-states", title: "Empty states" },
  { id: "errors", title: "Errors" },
  { id: "loaders", title: "Loaders" },
  { id: "status", title: "Status" },
  { id: "product", title: "Product" },
  { id: "onboarding", title: "Onboarding" },
];

/** One interactive part of a figure: what `onAction` reports when it is clicked. */
export interface ActionInfo<A extends string = string> {
  name: A;
  label: string;
  description: string;
}

/** A figure in the library: its description plus what a page needs to know to use it. */
export interface FigureMeta<A extends string = string> {
  /** Kebab-case id, used by `<rough-figure figure="...">` and the catalog. */
  id: string;
  /** The React component name. */
  name: string;
  title: string;
  description: string;
  category: Category;
  tags: string[];
  /** The clickable parts. Empty for figures that are only to look at. */
  actions: readonly ActionInfo<A>[];
  spec: FigureSpec;
}

/** Declare a figure. The `A` type parameter becomes the union of its action names. */
export function defineFigure<const A extends string>(meta: FigureMeta<A>): FigureMeta<A> {
  return meta;
}
