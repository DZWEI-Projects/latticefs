import { useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
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
import { Textarea } from "@/components/ui/textarea";
import { updateView, type ViewInfo } from "@/lib/lfs";
import { Loader2 } from "lucide-react";

interface EditViewDialogProps {
  view: ViewInfo | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export const EditViewDialog = ({ view, open, onOpenChange }: EditViewDialogProps) => {
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const [query, setQuery] = useState("");
  const [description, setDescription] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!view || !open) return;
    setName(view.name);
    setQuery(view.query);
    setDescription(view.description || "");
    setError(null);
  }, [view, open]);

  if (!view) return null;

  const handleSave = async () => {
    if (!name.trim() || !query.trim()) {
      setError("Name und Filtersyntax sind erforderlich");
      return;
    }

    setIsSaving(true);
    setError(null);

    try {
      await updateView({
        id: view.id,
        name: name.trim(),
        query: query.trim(),
        description: description.trim() || undefined,
      });

      await queryClient.invalidateQueries({ queryKey: ["views"] });
      await queryClient.invalidateQueries({ queryKey: ["view-objects", view.id] });

      onOpenChange(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsSaving(false);
    }
  };

  const handleClose = () => {
    if (!isSaving) {
      setError(null);
      onOpenChange(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Perspektive bearbeiten</DialogTitle>
          <DialogDescription>
            Passe Name, Filtersyntax und Beschreibung deiner Perspektive an.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="edit-view-name">Name</Label>
            <Input
              id="edit-view-name"
              placeholder="Meine eigene Perspektive"
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={isSaving}
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="edit-view-query">Filtersyntax (LQL)</Label>
            <Textarea
              id="edit-view-query"
              placeholder="tag:projekt AND updated within 30d"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              disabled={isSaving}
              className="font-mono text-sm"
              rows={3}
            />
            <p className="text-xs text-muted-foreground">
              Beispiele: <code>tag:arbeit</code>, <code>type:pdf</code>,{" "}
              <code>updated within 7d</code>
            </p>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="edit-view-description">Beschreibung (optional)</Label>
            <Input
              id="edit-view-description"
              placeholder="Dateien zu..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              disabled={isSaving}
            />
          </div>

          {error && (
            <div className="text-sm text-destructive bg-destructive/10 px-3 py-2 rounded-md">
              {error}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={handleClose} disabled={isSaving}>
            Abbrechen
          </Button>
          <Button onClick={handleSave} disabled={isSaving || !name || !query}>
            {isSaving && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
            Änderungen speichern
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
