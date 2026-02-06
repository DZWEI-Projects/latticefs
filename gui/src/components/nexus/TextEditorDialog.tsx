import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import type { ObjectInfo } from "@/lib/lfs";
import { getVersionContent, getObjectVersions, reviseObject } from "@/lib/lfs";
import { Lock, Save } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "sonner";

interface TextEditorDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  object: ObjectInfo;
  onObjectUpdated: (updated: ObjectInfo) => void;
}

export const TextEditorDialog = ({
  open,
  onOpenChange,
  object,
  onObjectUpdated,
}: TextEditorDialogProps) => {
  const [content, setContent] = useState("");
  const [originalContent, setOriginalContent] = useState("");
  const [commitMessage, setCommitMessage] = useState("");
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const loadContent = useCallback(async () => {
    setLoading(true);
    try {
      const versions = await getObjectVersions(object.id);
      const current = versions.find((v) => v.isCurrent);
      if (!current) {
        toast.error("Aktuelle Version nicht gefunden");
        return;
      }
      const text = await getVersionContent(object.id, current.id);
      if (text === null) {
        toast.error("Datei enthält keinen Text und kann nicht bearbeitet werden");
        onOpenChange(false);
        return;
      }
      setContent(text);
      setOriginalContent(text);
      setCommitMessage("");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Inhalt konnte nicht geladen werden",
      );
    } finally {
      setLoading(false);
    }
  }, [object.id, onOpenChange]);

  useEffect(() => {
    if (open) {
      loadContent();
    }
  }, [open, loadContent]);

  const hasChanges = content !== originalContent;

  const handleSave = async () => {
    if (!hasChanges) return;
    setSaving(true);
    try {
      const updated = await reviseObject(
        object.id,
        content,
        commitMessage || undefined,
      );
      onObjectUpdated(updated);
      setOriginalContent(content);
      setCommitMessage("");
      toast.success("Neue Version erstellt");
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Speichern fehlgeschlagen",
      );
    } finally {
      setSaving(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      if (hasChanges && !saving) {
        handleSave();
      }
    }
    if (e.key === "Tab") {
      e.preventDefault();
      const textarea = textareaRef.current;
      if (!textarea) return;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const newContent =
        content.substring(0, start) + "  " + content.substring(end);
      setContent(newContent);
      requestAnimationFrame(() => {
        textarea.selectionStart = start + 2;
        textarea.selectionEnd = start + 2;
      });
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            {object.isSealed && <Lock className="w-4 h-4 text-destructive" />}
            {object.name}
            {hasChanges && (
              <span className="text-xs font-normal text-muted-foreground">
                (geändert)
              </span>
            )}
          </DialogTitle>
        </DialogHeader>

        {object.isSealed ? (
          <div className="flex-1 flex flex-col items-center justify-center py-12 gap-2">
            <Lock className="w-8 h-8 text-destructive/50" />
            <p className="text-sm text-muted-foreground">
              Dieses Objekt ist versiegelt und kann nicht mehr bearbeitet werden.
            </p>
          </div>
        ) : loading ? (
          <div className="flex-1 flex items-center justify-center py-12 text-muted-foreground text-sm">
            Inhalt wird geladen...
          </div>
        ) : (
          <>
            <div className="flex-1 min-h-0">
              <textarea
                ref={textareaRef}
                value={content}
                onChange={(e) => setContent(e.target.value)}
                onKeyDown={handleKeyDown}
                className="w-full h-full min-h-[400px] resize-none rounded-md border border-border/50 bg-muted/20 px-3 py-2 font-mono text-sm leading-relaxed focus:outline-none focus:ring-1 focus:ring-ring"
                spellCheck={false}
                disabled={saving}
              />
            </div>
            <div className="flex items-center gap-2 pt-2 border-t border-border/50">
              <Input
                placeholder="Nachricht (optional)"
                value={commitMessage}
                onChange={(e) => setCommitMessage(e.target.value)}
                className="flex-1 h-8 text-xs"
                disabled={saving}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && hasChanges && !saving) {
                    e.preventDefault();
                    handleSave();
                  }
                }}
              />
              <Button
                size="sm"
                onClick={handleSave}
                disabled={!hasChanges || saving}
              >
                <Save className="w-3.5 h-3.5 mr-1.5" />
                {saving ? "Speichert..." : "Speichern"}
              </Button>
            </div>
            <p className="text-[10px] text-muted-foreground">
              Beim Speichern wird eine neue Version erstellt. Strg+S zum
              Schnellspeichern.
            </p>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
};
