import { cn } from "@/lib/utils";
import { useViews } from "@/hooks/useViews";
import {
  Clock,
  Folder,
  FileEdit,
  Eye,
  CheckCircle,
  Grid,
  Plus,
  Settings,
  ChevronDown,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { useState } from "react";
import type { ViewInfo } from "@/lib/lfs";
import { NewViewDialog } from "./NewViewDialog";

const iconMap: Record<string, React.ElementType> = {
  Clock,
  Folder,
  FileEdit,
  Eye,
  CheckCircle,
  Grid,
};

interface SidebarProps {
  currentViewId?: string;
  onViewSelect: (viewId: string) => void;
  onOpenSettings: () => void;
}

interface ViewItemProps {
  view: ViewInfo;
  isActive: boolean;
  onClick: () => void;
}

const ViewItem = ({ view, isActive, onClick }: ViewItemProps) => {
  const Icon = view.icon ? iconMap[view.icon] || Folder : Folder;

  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full flex items-center gap-2.5 px-3 py-1.5 rounded-md text-sm",
        "transition-colors duration-150",
        "hover:bg-muted/60",
        isActive && "bg-primary/10 text-primary hover:bg-primary/15"
      )}
    >
      <Icon className="w-4 h-4 flex-shrink-0" />
      <span className="flex-1 text-left truncate">{view.name}</span>
      <span
        className={cn(
          "text-xs tabular-nums",
          isActive ? "text-primary/70" : "text-muted-foreground"
        )}
      >
        {view.objectCount}
      </span>
    </button>
  );
};

export const Sidebar = ({ currentViewId, onViewSelect, onOpenSettings }: SidebarProps) => {
  const { data: views, isLoading } = useViews();
  const [builtinOpen, setBuiltinOpen] = useState(true);
  const [dynamicOpen, setDynamicOpen] = useState(true);
  const [newViewDialogOpen, setNewViewDialogOpen] = useState(false);

  const builtinViews = views?.filter((v) => v.viewType === "builtin") || [];
  const dynamicViews = views?.filter((v) => v.viewType === "dynamic") || [];

  const handleViewCreated = (viewId: string) => {
    onViewSelect(viewId);
    setDynamicOpen(true);
  };

  return (
    <div className="w-60 flex-shrink-0 border-r border-border/50 flex flex-col bg-background/50">
      <ScrollArea className="flex-1 py-2">
        {/* Built-in Views */}
        <Collapsible open={builtinOpen} onOpenChange={setBuiltinOpen}>
          <CollapsibleTrigger className="flex items-center gap-1.5 w-full px-3 py-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider hover:text-foreground transition-colors">
            <ChevronDown
              className={cn(
                "w-3 h-3 transition-transform",
                !builtinOpen && "-rotate-90"
              )}
            />
            Perspektiven
          </CollapsibleTrigger>
          <CollapsibleContent className="px-2 space-y-0.5">
            {isLoading ? (
              <div className="px-3 py-2 text-sm text-muted-foreground">
                Lädt...
              </div>
            ) : (
              builtinViews.map((view) => (
                <ViewItem
                  key={view.id}
                  view={view}
                  isActive={currentViewId === view.id}
                  onClick={() => onViewSelect(view.id)}
                />
              ))
            )}
          </CollapsibleContent>
        </Collapsible>

        {/* Dynamic Views */}
        {dynamicViews.length > 0 && (
          <Collapsible open={dynamicOpen} onOpenChange={setDynamicOpen} className="mt-4">
            <CollapsibleTrigger className="flex items-center gap-1.5 w-full px-3 py-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider hover:text-foreground transition-colors">
              <ChevronDown
                className={cn(
                  "w-3 h-3 transition-transform",
                  !dynamicOpen && "-rotate-90"
                )}
              />
              Eigene Perspektiven
            </CollapsibleTrigger>
            <CollapsibleContent className="px-2 space-y-0.5">
              {dynamicViews.map((view) => (
                <ViewItem
                  key={view.id}
                  view={view}
                  isActive={currentViewId === view.id}
                  onClick={() => onViewSelect(view.id)}
                />
              ))}
            </CollapsibleContent>
          </Collapsible>
        )}
      </ScrollArea>

      {/* Sidebar footer */}
      <div className="p-2 border-t border-border/50 flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          className="flex-1 justify-start gap-2 h-8 text-muted-foreground hover:text-foreground"
          onClick={() => setNewViewDialogOpen(true)}
        >
          <Plus className="w-4 h-4" />
          <span className="text-sm">Neue Perspektive</span>
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-muted-foreground hover:text-foreground"
          onClick={onOpenSettings}
        >
          <Settings className="w-4 h-4" />
        </Button>
      </div>

      {/* New View Dialog */}
      <NewViewDialog
        open={newViewDialogOpen}
        onOpenChange={setNewViewDialogOpen}
        onViewCreated={handleViewCreated}
      />
    </div>
  );
};
