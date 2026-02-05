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
import { createView } from "@/lib/lfs";
import { Loader2 } from "lucide-react";

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
  const [name, setName] = useState("");
  const [query, setQuery] = useState("");
  const [description, setDescription] = useState("");
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreate = async () => {
    if (!name.trim() || !query.trim()) {
      setError("Name and query are required");
      return;
    }

    setIsCreating(true);
    setError(null);

    try {
      const view = await createView({
        name: name.trim(),
        query: query.trim(),
        description: description.trim() || undefined,
      });

      // Invalidate the views query to refresh the sidebar
      await queryClient.invalidateQueries({ queryKey: ["views"] });

      // Reset form
      setName("");
      setQuery("");
      setDescription("");
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
      setError(null);
      onOpenChange(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Create New View</DialogTitle>
          <DialogDescription>
            Views are saved queries that organize your objects. Use LQL syntax
            to define what objects appear in this view.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              placeholder="My Custom View"
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={isCreating}
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="query">Query (LQL)</Label>
            <Textarea
              id="query"
              placeholder="tag:project AND updated within 30d"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              disabled={isCreating}
              className="font-mono text-sm"
              rows={3}
            />
            <p className="text-xs text-muted-foreground">
              Examples: <code>tag:work</code>, <code>type:pdf</code>,{" "}
              <code>updated within 7d</code>
            </p>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="description">Description (optional)</Label>
            <Input
              id="description"
              placeholder="Files related to..."
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
            Cancel
          </Button>
          <Button onClick={handleCreate} disabled={isCreating || !name || !query}>
            {isCreating && <Loader2 className="w-4 h-4 mr-2 animate-spin" />}
            Create View
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
