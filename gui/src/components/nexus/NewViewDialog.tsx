import { useState } from "react";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { createView } from "@/lib/lfs";
import { Loader2 } from "lucide-react";
import { useViews } from "@/hooks/useViews";

const ROOT_PARENT = "__root__";

interface NewViewDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onViewCreated?: (viewId: string) => void;
}

export const NewViewDialog = ({
  open,
  onOpenChange,
  onViewCreated,
}: NewViewDialogProps) => {
  const queryClient = useQueryClient();
  const { data: views } = useViews();
  const [name, setName] = useState("");
  const [query, setQuery] = useState("");
  const [description, setDescription] = useState("");
  const [parentId, setParentId] = useState(ROOT_PARENT);
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const dynamicViews = views?.filter((view) => view.viewType === "dynamic") || [];

  const handleCreate = async () => {
    if (!name.trim() || !query.trim()) {
      setError("Name und Filtersyntax sind erforderlich");
      return;
    }

    setIsCreating(true);
    setError(null);

    try {
      const view = await createView({
        name: name.trim(),
        query: query.trim(),
        description: description.trim() || undefined,
        parentId: parentId === ROOT_PARENT ? null : parentId,
      });

      // Invalidate the views query to refresh the sidebar
      await queryClient.invalidateQueries({ queryKey: ["views"] });

      // Reset form
      setName("");
      setQuery("");
      setDescription("");
      setParentId(ROOT_PARENT);
      onOpenChange(false);
      onViewCreated?.(view.id);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsCreating(false);
    }
  };

  const handleClose = () => {
    if (!isCreating) {
      setName("");
      setQuery("");
      setDescription("");
      setParentId(ROOT_PARENT);
      setError(null);
      onOpenChange(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Neue Perspektive erstellen</DialogTitle>
          <DialogDescription>
            Perspektiven sind gespeicherte Filterausdrücke, die deine Objekte
            organisieren. Nutze die LQL-Filtersyntax, um festzulegen, welche
            Objekte in dieser Perspektive erscheinen.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              placeholder="Meine eigene Perspektive"
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={isCreating}
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="query">Filtersyntax (LQL)</Label>
            <Textarea
              id="query"
              placeholder="tag:projekt AND updated within 30d"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              disabled={isCreating}
              className="font-mono text-sm"
              rows={3}
            />
            <p className="text-xs text-muted-foreground">
              Beispiele: <code>tag:arbeit</code>, <code>type:pdf</code>,{" "}
              <code>updated within 7d</code>
            </p>
          </div>

          <div className="grid gap-2">
            <Label>Übergeordnete Perspektive</Label>
            <Select
              value={parentId}
              onValueChange={setParentId}
              disabled={isCreating}
            >
              <SelectTrigger>
                <SelectValue placeholder="Keine (Root)" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={ROOT_PARENT}>Keine (Root)</SelectItem>
                {dynamicViews.map((view) => (
                  <SelectItem key={view.id} value={view.id}>
                    {view.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="description">Beschreibung (optional)</Label>
            <Input
              id="description"
              placeholder="Dateien zu..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              disabled={isCreating}
            />
          </div>

          {error && (
            <div className="text-sm text-destructive bg-destructive/10 px-3 py-2 rounded-md">
              {error}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={handleClose} disabled={isCreating}>
            Abbrechen
          </Button>
          <Button onClick={handleCreate} disabled={isCreating || !name || !query}>
            {isCreating && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
            Perspektive erstellen
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
