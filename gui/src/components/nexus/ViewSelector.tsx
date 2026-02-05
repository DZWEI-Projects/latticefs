import { useState, useEffect } from "react";
import { cn } from "@/lib/utils";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Share2, LayoutGrid, List } from "lucide-react";
import type { ViewMode } from "./NexusLayout";

interface ViewSelectorProps {
  value: ViewMode;
  onChange: (mode: ViewMode) => void;
}

const FIRST_TIME_KEY = "nexus-view-selector-seen";

export const ViewSelector = ({ value, onChange }: ViewSelectorProps) => {
  const [showFirstTimeTooltip, setShowFirstTimeTooltip] = useState(false);
  const [tooltipOpen, setTooltipOpen] = useState(false);

  useEffect(() => {
    // Check if user has seen the tooltip before
    const seen = localStorage.getItem(FIRST_TIME_KEY);
    if (!seen) {
      setShowFirstTimeTooltip(true);
      setTooltipOpen(true);
      // Auto-hide after 8 seconds
      const timer = setTimeout(() => {
        setTooltipOpen(false);
      }, 8000);
      return () => clearTimeout(timer);
    }
  }, []);

  const handleChange = (newValue: string) => {
    if (newValue) {
      onChange(newValue as ViewMode);
      // Mark as seen after first interaction
      if (showFirstTimeTooltip) {
        localStorage.setItem(FIRST_TIME_KEY, "true");
        setShowFirstTimeTooltip(false);
        setTooltipOpen(false);
      }
    }
  };

  const content = (
    <ToggleGroup
      type="single"
      value={value}
      onValueChange={handleChange}
      className="bg-muted/30 rounded-md p-0.5"
    >
      <ToggleGroupItem
        value="graph"
        aria-label="Graph view"
        className={cn(
          "h-7 w-7 p-0 data-[state=on]:bg-background data-[state=on]:shadow-sm"
        )}
      >
        <Share2 className="w-3.5 h-3.5" />
      </ToggleGroupItem>
      <ToggleGroupItem
        value="grid"
        aria-label="Grid view"
        className={cn(
          "h-7 w-7 p-0 data-[state=on]:bg-background data-[state=on]:shadow-sm"
        )}
      >
        <LayoutGrid className="w-3.5 h-3.5" />
      </ToggleGroupItem>
      <ToggleGroupItem
        value="list"
        aria-label="List view"
        className={cn(
          "h-7 w-7 p-0 data-[state=on]:bg-background data-[state=on]:shadow-sm"
        )}
      >
        <List className="w-3.5 h-3.5" />
      </ToggleGroupItem>
    </ToggleGroup>
  );

  if (showFirstTimeTooltip) {
    return (
      <Tooltip open={tooltipOpen} onOpenChange={setTooltipOpen}>
        <TooltipTrigger asChild>{content}</TooltipTrigger>
        <TooltipContent
          side="bottom"
          className="max-w-[280px] p-3"
          onPointerDownOutside={() => {
            localStorage.setItem(FIRST_TIME_KEY, "true");
            setShowFirstTimeTooltip(false);
            setTooltipOpen(false);
          }}
        >
          <p className="text-sm">
            <strong>Graph View</strong> is the default — it shows how your
            objects connect across views. If you prefer a traditional layout,
            switch to Grid or List view anytime.
          </p>
        </TooltipContent>
      </Tooltip>
    );
  }

  return content;
};
