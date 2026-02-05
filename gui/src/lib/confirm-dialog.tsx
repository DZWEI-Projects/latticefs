import { useState, useCallback, ReactNode } from "react";
import { isTauriApp } from "./lfs";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

export interface ConfirmDialogOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  /** Optional hint shown on web only (e.g., settings location) */
  hint?: string;
}

/**
 * Shows a platform-aware confirmation dialog.
 * - On Tauri: Uses native OS dialog via @tauri-apps/plugin-dialog
 * - On Web: Uses shadcn AlertDialog
 */
export async function showConfirmDialog(
  options: ConfirmDialogOptions
): Promise<boolean> {
  if (isTauriApp()) {
    const { ask } = await import("@tauri-apps/plugin-dialog");
    return ask(options.message, {
      title: options.title,
      okLabel: options.confirmLabel ?? "OK",
      cancelLabel: options.cancelLabel ?? "Abbrechen",
      kind: "info",
    });
  }

  // For web, we need to use a React-based approach
  // This function will be called from useConfirmDialog hook
  throw new Error(
    "showConfirmDialog cannot be used directly on web. Use useConfirmDialog hook instead."
  );
}

/**
 * Hook for showing platform-aware confirmation dialogs.
 * Returns a trigger function and dialog component to render.
 */
export function useConfirmDialog(options: ConfirmDialogOptions) {
  const [open, setOpen] = useState(false);
  const [resolvePromise, setResolvePromise] =
    useState<((value: boolean) => void) | null>(null);

  const confirm = useCallback(async (): Promise<boolean> => {
    if (isTauriApp()) {
      const { ask } = await import("@tauri-apps/plugin-dialog");
      return ask(options.message, {
        title: options.title,
        okLabel: options.confirmLabel ?? "OK",
        cancelLabel: options.cancelLabel ?? "Abbrechen",
        kind: "info",
      });
    }

    // Web: Show shadcn dialog
    return new Promise<boolean>((resolve) => {
      setResolvePromise(() => resolve);
      setOpen(true);
    });
  }, [options]);

  const handleConfirm = useCallback(() => {
    setOpen(false);
    resolvePromise?.(true);
    setResolvePromise(null);
  }, [resolvePromise]);

  const handleCancel = useCallback(() => {
    setOpen(false);
    resolvePromise?.(false);
    setResolvePromise(null);
  }, [resolvePromise]);

  const DialogComponent = useCallback(
    () => (
      <AlertDialog open={open} onOpenChange={setOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{options.title}</AlertDialogTitle>
            <AlertDialogDescription>
              {options.message}
              {options.hint && (
                <span className="block mt-2 text-xs text-muted-foreground/80">
                  {options.hint}
                </span>
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={handleCancel}>
              {options.cancelLabel ?? "Abbrechen"}
            </AlertDialogCancel>
            <AlertDialogAction onClick={handleConfirm}>
              {options.confirmLabel ?? "OK"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    ),
    [open, options, handleConfirm, handleCancel]
  );

  return { confirm, DialogComponent };
}

/**
 * Wrapper component for declarative usage with children as trigger.
 */
export interface ConfirmDialogProps extends ConfirmDialogOptions {
  onConfirm: () => void;
  onCancel?: () => void;
  children: ReactNode;
}

export function ConfirmDialog({
  onConfirm,
  onCancel,
  children,
  ...options
}: ConfirmDialogProps) {
  const { confirm, DialogComponent } = useConfirmDialog(options);

  const handleClick = async () => {
    const confirmed = await confirm();
    if (confirmed) {
      onConfirm();
    } else {
      onCancel?.();
    }
  };

  return (
    <>
      <span onClick={handleClick} role="button" tabIndex={0} onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          handleClick();
        }
      }}>
        {children}
      </span>
      <DialogComponent />
    </>
  );
}
