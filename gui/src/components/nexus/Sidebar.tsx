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
import { useEffect, useMemo, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { deleteView, type ViewInfo } from "@/lib/lfs";
import { NewViewDialog } from "./NewViewDialog";
import { EditViewDialog } from "./EditViewDialog";
import { ViewTree } from "./ViewTree";
import { useConfirmDialog } from "@/lib/confirm-dialog";
import { toast } from "sonner";

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
    <div
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onClick();
        }
      }}
      className={cn(
        "w-full flex items-center gap-2.5 py-1.5 px-3 rounded-md text-sm",
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
    </div>
  );
};

export const Sidebar = ({ currentViewId, onViewSelect, onOpenSettings }: SidebarProps) => {
  const { data: views, isLoading } = useViews();
  const queryClient = useQueryClient();
  const [builtinOpen, setBuiltinOpen] = useState(true);
  const [dynamicOpen, setDynamicOpen] = useState(true);
  const [viewTreeKey, setViewTreeKey] = useState(0);
  const [newViewDialogOpen, setNewViewDialogOpen] = useState(false);
  const [newSubViewParentId, setNewSubViewParentId] = useState<string | null>(null);
  const [editingView, setEditingView] = useState<ViewInfo | null>(null);
  const [pendingDelete, setPendingDelete] = useState<ViewInfo | null>(null);

  const builtinViews = useMemo(
    () => views?.filter((view) => view.viewType === "builtin") ?? [],
    [views]
  );
  const dynamicViews = useMemo(
    () => views?.filter((view) => view.viewType === "dynamic") ?? [],
    [views]
  );

  const { confirm, DialogComponent } = useConfirmDialog({
    title: "Perspektive löschen",
    message: pendingDelete
      ? `"${pendingDelete.name}" wirklich löschen? Diese Aktion kann nicht rückgängig gemacht werden.`
      : "Diese Perspektive wirklich löschen?",
    confirmLabel: "Löschen",
    cancelLabel: "Abbrechen",
  });

  const handleViewCreated = (viewId: string) => {
    onViewSelect(viewId);
    setDynamicOpen(true);
    // Re-mount tree so nested folders expand immediately for the new selection.
    setViewTreeKey((current) => current + 1);
    setNewSubViewParentId(null);
  };

  const handleCreateSubView = (parentId: string) => {
    setNewSubViewParentId(parentId);
    setNewViewDialogOpen(true);
  };

  useEffect(() => {
    if (!pendingDelete) return;

    let isMounted = true;

    const run = async () => {
      const confirmed = await confirm();
      if (!isMounted || !confirmed) {
        if (isMounted) setPendingDelete(null);
        return;
      }

      try {
        await deleteView(pendingDelete.name);
        if (!isMounted) return;

        await queryClient.invalidateQueries({ queryKey: ["views"] });
        await queryClient.invalidateQueries({
          queryKey: ["view-objects", pendingDelete.id],
        });
        if (currentViewId === pendingDelete.id) {
          onViewSelect("recent");
        }
        toast.success("Perspektive gelöscht");
      } catch (err) {
        if (!isMounted) return;
        toast.error(
          err instanceof Error
            ? err.message
            : "Perspektive konnte nicht gelöscht werden"
        );
      } finally {
        if (isMounted) setPendingDelete(null);
      }
    };

    run();

    return () => {
      isMounted = false;
    };
  }, [pendingDelete, confirm, queryClient, currentViewId, onViewSelect]);

  return (
    <div className="w-60 xl:w-72 2xl:w-[21rem] transition-width duration-200 flex-shrink-0 border-r border-border/50 flex flex-col bg-background/50">
      <ScrollArea className="flex-1 py-2">
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
          <CollapsibleContent className="px-2 space-y-0.5 overflow-hidden data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down">
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
            <CollapsibleContent className="px-2 overflow-hidden data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down">
              <ViewTree
                key={viewTreeKey}
                views={dynamicViews}
                currentViewId={currentViewId}
                onViewSelect={onViewSelect}
                onEditView={setEditingView}
                onDeleteView={setPendingDelete}
                onCreateSubView={handleCreateSubView}
              />
            </CollapsibleContent>
          </Collapsible>
        )}
      </ScrollArea>

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

      <NewViewDialog
        open={newViewDialogOpen}
        onOpenChange={(open) => {
          setNewViewDialogOpen(open);
          if (!open) setNewSubViewParentId(null);
        }}
        onViewCreated={handleViewCreated}
        parentId={newSubViewParentId ?? undefined}
      />

      <EditViewDialog
        open={!!editingView}
        view={editingView}
        onOpenChange={(open) => {
          if (!open) setEditingView(null);
        }}
      />
      <DialogComponent />
    </div>
  );
};
