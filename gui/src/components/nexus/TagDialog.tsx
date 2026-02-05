import { useEffect, useState } from "react";
import type { TagInfo } from "@/lib/lfs";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";

interface TagDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (tag: TagInfo) => void;
  initialTag?: TagInfo | null;
}

export const TagDialog = ({
  open,
  onOpenChange,
  onSubmit,
  initialTag,
}: TagDialogProps) => {
  const [key, setKey] = useState("");
  const [value, setValue] = useState("");

  useEffect(() => {
    setKey(initialTag?.key ?? "");
    setValue(initialTag?.value ?? "");
  }, [initialTag, open]);

  const canSubmit = key.trim().length > 0 && value.trim().length > 0;

  const handleSubmit = () => {
    if (!canSubmit) return;
    onSubmit({ key: key.trim(), value: value.trim() });
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[360px]">
        <DialogHeader>
          <DialogTitle>Add tag</DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          <Input
            placeholder="Key (e.g. project)"
            value={key}
            onChange={(e) => setKey(e.target.value)}
          />
          <Input
            placeholder="Value (e.g. phoenix)"
            value={value}
            onChange={(e) => setValue(e.target.value)}
          />
        </div>
        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!canSubmit}>
            Add tag
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
