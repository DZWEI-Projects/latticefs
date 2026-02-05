import { useState } from "react";
import { useViewByName } from "@/hooks/useViews";
import { ViewSelector } from "./ViewSelector";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Search,
  SlidersHorizontal,
  ArrowUpDown,
  Import,
  MoreHorizontal,
} from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { ViewMode } from "./NexusLayout";
import { ImportDialog } from "./ImportDialog";

interface ToolbarProps {
  currentViewName?: string;
  viewMode: ViewMode;
  onViewModeChange: (mode: ViewMode) => void;
  searchQuery: string;
  onSearchChange: (query: string) => void;
}

export const Toolbar = ({
  currentViewName,
  viewMode,
  onViewModeChange,
  searchQuery,
  onSearchChange,
}: ToolbarProps) => {
  const { data: currentView } = useViewByName(currentViewName);
  const [importDialogOpen, setImportDialogOpen] = useState(false);

  return (
    <>
    
    <div className="h-12 flex-shrink-0 border-b border-border/50 flex items-center gap-3 px-4">
      {/* Current view name */}
      <div className="flex items-center gap-2 min-w-0">
        <h1 className="text-sm font-semibold truncate">
          {currentView?.name || "All Objects"}
        </h1>
        {currentView && (
          <span className="text-xs text-muted-foreground">
            {currentView.objectCount} objects
          </span>
        )}
      </div>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Search */}
      <div className="relative w-64">
        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
        <Input
          type="text"
          placeholder="Search objects..."
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          className="h-8 pl-8 text-sm bg-muted/30 border-transparent focus:border-primary/50"
        />
      </div>

      {/* View mode selector */}
      <ViewSelector value={viewMode} onChange={onViewModeChange} />

      {/* Sort button */}
      <Button variant="ghost" size="icon" className="h-8 w-8">
        <ArrowUpDown className="w-4 h-4" />
      </Button>

      {/* Filter button */}
      <Button variant="ghost" size="icon" className="h-8 w-8">
        <SlidersHorizontal className="w-4 h-4" />
      </Button>

      {/* Actions menu */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon" className="h-8 w-8">
            <MoreHorizontal className="w-4 h-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-48">
          <DropdownMenuItem onClick={() => setImportDialogOpen(true)}>
            <Import className="w-4 h-4 mr-2" />
            Import Files
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem>Export View</DropdownMenuItem>
          <DropdownMenuItem>Share View</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>

    {/* Import Dialog */}
    <ImportDialog
      open={importDialogOpen}
      onOpenChange={setImportDialogOpen}
    />
    </>
  );
};
