import { RoughFigure, type RoughFigureProps } from "../react.js";
import type { ActionEvent } from "../player.js";
import {
  accessDenied, analytics, emptyInbox, loading, notFound, notifications, rocket, servers, settings, success, upload,
  type FigureMeta,
} from "./index.js";

export interface FigureProps<A extends string = string>
  extends Omit<RoughFigureProps, "spec" | "onAction" | "aria-label"> {
  /** Called when a clickable part is clicked. `action` is the union of this figure's action names. */
  onAction?: (e: Omit<ActionEvent, "action"> & { action: A }) => void;
  /** Largest width in CSS pixels. By default the figure's own size, shrinking to fit. */
  size?: number;
  /** Accessible description. Defaults to the figure's title. */
  label?: string;
}

/** Turn a library entry into a React component. The ready-made ones are exported below. */
export function createFigure<A extends string>(meta: FigureMeta<A>) {
  const Figure = ({ onAction, size, label, style, ...rest }: FigureProps<A>) => (
    <RoughFigure
      {...rest}
      spec={meta.spec}
      aria-label={label ?? meta.title}
      style={size ? { maxWidth: size, ...style } : style}
      onAction={onAction as ((e: ActionEvent) => void) | undefined}
    />
  );
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
