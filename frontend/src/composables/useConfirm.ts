import { reactive } from 'vue';

export interface ConfirmOptions {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
}

interface ConfirmState extends ConfirmOptions {
  open: boolean;
  resolve: ((ok: boolean) => void) | null;
}

const state = reactive<ConfirmState>({
  open: false,
  title: '',
  message: '',
  confirmText: 'Bestätigen',
  cancelText: 'Abbrechen',
  danger: false,
  resolve: null,
});

/**
 * Show a modal confirm dialog and return a promise that resolves to
 * `true` if the user confirms, `false` if they cancel (including via
 * Escape or backdrop click). Host component `ConfirmHost.vue` reads the
 * shared state.
 */
function confirm(opts: ConfirmOptions): Promise<boolean> {
  // If another dialog is already open, resolve it as cancelled first.
  if (state.open && state.resolve) {
    state.resolve(false);
  }
  state.title = opts.title;
  state.message = opts.message;
  state.confirmText = opts.confirmText ?? 'Bestätigen';
  state.cancelText = opts.cancelText ?? 'Abbrechen';
  state.danger = opts.danger ?? false;
  state.open = true;
  return new Promise<boolean>((resolve) => {
    state.resolve = resolve;
  });
}

function resolve(ok: boolean): void {
  const r = state.resolve;
  state.open = false;
  state.resolve = null;
  if (r) r(ok);
}

export function useConfirm() {
  return {
    state,
    confirm,
    resolve,
  };
}
