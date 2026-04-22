import { reactive } from 'vue';

export type ToastVariant = 'info' | 'success' | 'warning' | 'error';

export interface Toast {
  id: number;
  message: string;
  variant: ToastVariant;
  timeout: number;
}

export interface ShowToastInput {
  message: string;
  variant?: ToastVariant;
  /** Milliseconds; 0 = sticky. Defaults to 4000. */
  timeout?: number;
}

// Module-level singleton — all calls share one queue rendered by
// `ToastHost`. Matches the Vue pattern for host-mounted portal state.
const state = reactive<{ toasts: Toast[]; next: number }>({
  toasts: [],
  next: 1,
});

function show(input: ShowToastInput): number {
  const id = state.next++;
  const toast: Toast = {
    id,
    message: input.message,
    variant: input.variant ?? 'info',
    timeout: input.timeout ?? 4000,
  };
  state.toasts.push(toast);
  if (toast.timeout > 0) {
    setTimeout(() => dismiss(id), toast.timeout);
  }
  return id;
}

function dismiss(id: number): void {
  const idx = state.toasts.findIndex((t) => t.id === id);
  if (idx >= 0) state.toasts.splice(idx, 1);
}

export function useToast() {
  return {
    toasts: state.toasts,
    show,
    dismiss,
  };
}
