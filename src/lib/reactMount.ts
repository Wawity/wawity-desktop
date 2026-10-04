import type { ReactNode } from 'react';
import { createRoot } from 'react-dom/client';
import type { Root } from 'react-dom/client';

export interface ReactHandle {
  render(node: ReactNode): void;
  unmount(): void;
}

export function mountReactRoot(host: HTMLElement): ReactHandle {
  const root: Root = createRoot(host);
  let alive = true;
  return {
    render(node: ReactNode) {
      if (!alive) return;
      root.render(node);
    },
    unmount() {
      if (!alive) return;
      alive = false;
      queueMicrotask(() => root.unmount());
    },
  };
}
