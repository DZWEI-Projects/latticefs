import { cn } from "@/lib/utils";
import { useViews } from "@/hooks/useViews";
import {
  Clock,
  Folder,
  FolderOpen,
  FileEdit,
  Eye,
  CheckCircle,
  Grid,
  Plus,
  Settings,
  ChevronDown,
  ChevronRight,
  MoreVertical,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useEffect, useState, useMemo } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { deleteView, type ViewInfo } from "@/lib/lfs";
import { NewViewDialog } from "./NewViewDialog";
import { EditViewDialog } from "./EditViewDialog";
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
  actions?: React.ReactNode;
  leading?: React.ReactNode;
  iconOverride?: React.ElementType;
}

const ViewItem = ({
  view,
  isActive,
  onClick,
  actions,
  leading,
  iconOverride,
  indent = 0,
}: ViewItemProps & { indent?: number }) => {
  const Icon = iconOverride || (view.icon ? iconMap[view.icon] || Folder : Folder);

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
        "group w-full flex items-center gap-1.5 py-1.5 rounded-md text-sm",
        "transition-colors duration-150",
        "hover:bg-muted/60",
        isActive && "bg-primary/10 text-primary hover:bg-primary/15"
      )}
      style={{ paddingLeft: `${0.75 + indent * 0.75}rem` }}
    >
      {leading ? (
        <span className="w-4 h-4 flex items-center justify-center flex-shrink-0">
          {leading}
        </span>
      ) : (
        <span className="w-4 h-4 flex-shrink-0" />
      )}
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
      {actions && (
        <div
          className={cn(
            "flex items-center overflow-hidden w-0 opacity-0 ml-0 pointer-events-none",
            "transition-[width,opacity,margin] duration-150 ease-out",
            "group-hover:w-6 group-hover:opacity-100 group-hover:ml-1 group-hover:pointer-events-auto",
            "group-focus-within:w-6 group-focus-within:opacity-100 group-focus-within:ml-1 group-focus-within:pointer-events-auto"
          )}
          onClick={(event) => event.stopPropagation()}
          onPointerDown={(event) => event.stopPropagation()}
        >
          {actions}
        </div>
      )}
    </div>
  );
};

interface ViewTreeItemProps {
  view: ViewInfo;
  children: ViewInfo[];
  childrenMap: Map<string, ViewInfo[]>;
  currentViewId?: string;
  onViewSelect: (viewId: string) => void;
  onEditView: (view: ViewInfo) => void;
  onDeleteView: (view: ViewInfo) => void;
  onCreateSubView: (parentId: string) => void;
  indent?: number;
}

const ViewTreeItem = ({
  view,
  children,
  childrenMap,
  currentViewId,
  onViewSelect,
  onEditView,
  onDeleteView,
  onCreateSubView,
  indent = 0,
}: ViewTreeItemProps) => {
  const [isOpen, setIsOpen] = useState(true);
  const hasChildren = children.length > 0;
  const isActive = currentViewId === view.id;
  const FolderIcon = hasChildren && isOpen ? FolderOpen : Folder;

  return (
    <div>
      <ViewItem
        view={view}
        isActive={isActive}
        onClick={() => onViewSelect(view.id)}
        indent={indent}
        iconOverride={FolderIcon}
        leading={
          hasChildren ? (
            <button
              type="button"
              aria-label={isOpen ? "Unterelemente einklappen" : "Unterelemente ausklappen"}
              className="h-4 w-4 flex items-center justify-center rounded-sm text-muted-foreground hover:text-foreground hover:bg-muted/60 transition-colors"
              onClick={(event) => {
                event.stopPropagation();
                setIsOpen((prev) => !prev);
              }}
            >
              {isOpen ? (
                <ChevronDown className="h-3.5 w-3.5" />
              ) : (
                <ChevronRight className="h-3.5 w-3.5" />
              )}
            </button>
          ) : null
        }
        actions={
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6 text-muted-foreground hover:text-foreground"
                aria-label={`Optionen für ${view.name}`}
              >
                <MoreVertical className="w-3.5 h-3.5" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-48">
              <DropdownMenuItem onSelect={() => onCreateSubView(view.id)}>
                Neue Teilperspektive
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onSelect={() => onEditView(view)}>
                Bearbeiten
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="text-destructive focus:text-destructive"
                onSelect={() => onDeleteView(view)}
              >
                Löschen
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        }
      />
      {hasChildren && isOpen && (
        <div className="space-y-0.5 ml-5 pl-1.5 border-l border-border/50">
          {children.map((child) => {
            const childChildren = childrenMap.get(child.id) || [];
            return (
              <ViewTreeItem
                key={child.id}
                view={child}
                children={childChildren}
                childrenMap={childrenMap}
                currentViewId={currentViewId}
                onViewSelect={onViewSelect}
                onEditView={onEditView}
                onDeleteView={onDeleteView}
                onCreateSubView={onCreateSubView}
                indent={indent + 1}
              />
            );
          })}
        </div>
      )}
    </div>
  );
};

export const Sidebar = ({ currentViewId, onViewSelect, onOpenSettings }: SidebarProps) => {
  const { data: views, isLoading } = useViews();
  const queryClient = useQueryClient();
  const [builtinOpen, setBuiltinOpen] = useState(true);
  const [dynamicOpen, setDynamicOpen] = useState(true);
  const [newViewDialogOpen, setNewViewDialogOpen] = useState(false);
  const [newSubViewParentId, setNewSubViewParentId] = useState<string | null>(null);
  const [editingView, setEditingView] = useState<ViewInfo | null>(null);
  const [pendingDelete, setPendingDelete] = useState<ViewInfo | null>(null);

  const builtinViews = useMemo(
    () => views?.filter((v) => v.viewType === "builtin") ?? [],
    [views]
  );
  const dynamicViews = useMemo(
    () => views?.filter((v) => v.viewType === "dynamic") ?? [],
    [views]
  );

  // Build tree structure for dynamic views
  const dynamicViewTree = useMemo(() => {
    const viewMap = new Map(dynamicViews.map((view) => [view.id, view]));
    const childrenMap = new Map<string, ViewInfo[]>();

    // Build parent-child relationships
    const rootViews: ViewInfo[] = [];
    for (const view of dynamicViews) {
      if (!view.parentId || !viewMap.has(view.parentId)) {
        rootViews.push(view);
      } else {
        const children = childrenMap.get(view.parentId) || [];
        children.push(view);
        childrenMap.set(view.parentId, children);
      }
    }

    const sortByName = (items: ViewInfo[]) => {
      items.sort((a, b) => a.name.localeCompare(b.name, "de", { sensitivity: "base" }));
      for (const item of items) {
        const nested = childrenMap.get(item.id);
        if (nested) sortByName(nested);
      }
    };

    sortByName(rootViews);

    return { rootViews, childrenMap };
  }, [dynamicViews]);

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
        await queryClient.invalidateQueries({ queryKey: ["view-objects", pendingDelete.id] });
        if (currentViewId === pendingDelete.id) {
          onViewSelect("recent");
        }
        toast.success("Perspektive gelöscht");
      } catch (err) {
        if (!isMounted) return;
        toast.error(err instanceof Error ? err.message : "Perspektive konnte nicht gelöscht werden");
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
    <div className="w-60 xl:w-72 2xl:w-[21rem] transition-all duration-200 ease-in-out flex-shrink-0 border-r border-border/50 flex flex-col bg-background/50">
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
            <CollapsibleContent className="px-2 space-y-0.5 overflow-hidden data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down">
              {dynamicViewTree.rootViews.map((view) => {
                const children = dynamicViewTree.childrenMap.get(view.id) || [];
                return (
                  <ViewTreeItem
                    key={view.id}
                    view={view}
                    children={children}
                    childrenMap={dynamicViewTree.childrenMap}
                    currentViewId={currentViewId}
                    onViewSelect={onViewSelect}
                    onEditView={setEditingView}
                    onDeleteView={setPendingDelete}
                    onCreateSubView={handleCreateSubView}
                  />
                );
              })}
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
