import { useEffect, useState } from "react";
import type { ObjectInfo } from "@/lib/lfs";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Loader2 } from "lucide-react";

interface ObjectRenameDialogProps {
  object: ObjectInfo | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onRename: (object: ObjectInfo, name: string) => Promise<void>;
}

export const ObjectRenameDialog = ({
  object,
  open,
  onOpenChange,
  onRename,
}: ObjectRenameDialogProps) => {
  const [name, setName] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!object || !open) return;
    setName(object.name);
    setError(null);
  }, [object, open]);

  if (!object) return null;

  const handleSave = async () => {
    const trimmed = name.trim();
    if (!trimmed) {
      setError("Ein Name ist erforderlich.");
      return;
    }

    setIsSaving(true);
    setError(null);
    try {
      await onRename(object, trimmed);
      onOpenChange(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Umbenennen fehlgeschlagen.");
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(next) => !isSaving && onOpenChange(next)}>
      <DialogContent className="sm:max-w-[420px]">
        <DialogHeader>
          <DialogTitle>Objekt umbenennen</DialogTitle>
          <DialogDescription>
            Der Anzeigename wird als auto:filename_b64-Tag gespeichert.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-3 py-3">
          <Label htmlFor="rename-object-name">Neuer Name</Label>
          <Input
            id="rename-object-name"
            value={name}
            onChange={(event) => setName(event.target.value)}
            disabled={isSaving}
          />
          {error && (
            <div className="text-sm text-destructive bg-destructive/10 px-3 py-2 rounded-md">
              {error}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isSaving}>
            Abbrechen
          </Button>
          <Button onClick={handleSave} disabled={isSaving || !name.trim()}>
            {isSaving && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
            Umbenennen
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
