import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";

interface NexusSettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  showDetailsOnSelect: boolean;
  onShowDetailsOnSelectChange: (value: boolean) => void;
}

export const NexusSettingsDialog = ({
  open,
  onOpenChange,
  showDetailsOnSelect,
  onShowDetailsOnSelectChange,
}: NexusSettingsDialogProps) => {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[360px]">
        <DialogHeader>
          <DialogTitle>Nexus settings</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div className="flex items-center justify-between gap-3">
            <Label htmlFor="details-on-select" className="text-sm">
              Open detail panel on select
            </Label>
            <Switch
              id="details-on-select"
              checked={showDetailsOnSelect}
              onCheckedChange={onShowDetailsOnSelectChange}
            />
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};
