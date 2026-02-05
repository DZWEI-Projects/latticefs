import { useCallback } from "react";
import { cn } from "@/lib/utils";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ObjectRow } from "./ObjectRow";
import { ArrowUp, ArrowDown } from "lucide-react";
import type { ObjectInfo, TagInfo } from "@/lib/lfs";
import { ObjectContextMenu } from "./ObjectContextMenu";
import type { SortState, SortField } from "./NexusLayout";

interface ListViewProps {
  objects: ObjectInfo[];
  selectedObjects: string[];
  onObjectSelect: (objectId: string, multiSelect?: boolean) => void;
  onObjectOpen: (object: ObjectInfo) => void;
  onObjectFocus: (object: ObjectInfo) => void;
  onRequestAddTag: (object: ObjectInfo) => void;
  onRemoveTag: (object: ObjectInfo, tag: TagInfo) => void;
  onSetTrust: (object: ObjectInfo, trust: number | null) => void;
  onShowDetails: (object: ObjectInfo) => void;
  sort: SortState;
  onSortChange: (next: SortState) => void;
}

export const ListView = ({
  objects,
  selectedObjects,
  onObjectSelect,
  onObjectOpen,
  onObjectFocus,
  onRequestAddTag,
  onRemoveTag,
  onSetTrust,
  onShowDetails,
  sort,
  onSortChange,
}: ListViewProps) => {
  const handleSort = useCallback(
    (field: SortField) => {
      if (sort.field === field) {
        onSortChange({
          field,
          direction: sort.direction === "asc" ? "desc" : "asc",
        });
      } else {
        onSortChange({ field, direction: "asc" });
      }
    },
    [onSortChange, sort.direction, sort.field]
  );

  const handleClick = useCallback(
    (obj: ObjectInfo, e: React.MouseEvent) => {
      const multiSelect = e.metaKey || e.ctrlKey;
      onObjectSelect(obj.id, multiSelect);
      onObjectFocus(obj);
    },
    [onObjectSelect, onObjectFocus]
  );

  const handleDoubleClick = useCallback(
    (obj: ObjectInfo) => {
      onObjectOpen(obj);
    },
    [onObjectOpen]
  );

  const handleContextMenu = useCallback(
    (obj: ObjectInfo, e: React.MouseEvent) => {
      e.preventDefault();
      onObjectSelect(obj.id, false);
      onObjectFocus(obj);
    },
    [onObjectSelect, onObjectFocus]
  );

  const SortIcon = sort.direction === "asc" ? ArrowUp : ArrowDown;

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex-shrink-0 flex items-center h-8 px-4 border-b border-border/50 text-xs text-muted-foreground bg-muted/30">
        <button
          className={cn(
            "flex-1 min-w-0 flex items-center gap-1 text-left hover:text-foreground transition-colors",
            sort.field === "name" && "text-foreground"
          )}
          onClick={() => handleSort("name")}
        >
          Name
          {sort.field === "name" && <SortIcon className="w-3 h-3" />}
        </button>
        <button
          className={cn(
            "w-20 flex items-center gap-1 hover:text-foreground transition-colors",
            sort.field === "extension" && "text-foreground"
          )}
          onClick={() => handleSort("extension")}
        >
          Typ
          {sort.field === "extension" && <SortIcon className="w-3 h-3" />}
        </button>
        <button
          className={cn(
            "w-24 flex items-center gap-1 hover:text-foreground transition-colors",
            sort.field === "sizeBytes" && "text-foreground"
          )}
          onClick={() => handleSort("sizeBytes")}
        >
          Größe
          {sort.field === "sizeBytes" && <SortIcon className="w-3 h-3" />}
        </button>
        <button
          className={cn(
            "w-32 flex items-center gap-1 hover:text-foreground transition-colors",
            sort.field === "modifiedAt" && "text-foreground"
          )}
          onClick={() => handleSort("modifiedAt")}
        >
          Geändert
          {sort.field === "modifiedAt" && <SortIcon className="w-3 h-3" />}
        </button>
        <div className="w-24">Eigenschaften</div>
        <div className="w-24 text-right">Sicherheitsgrad</div>
      </div>

      {/* Rows */}
      <ScrollArea className="flex-1">
        <div className="divide-y divide-border/30">
          {objects.map((obj, index) => (
            <ObjectContextMenu
              key={obj.id}
              object={obj}
              onOpen={onObjectOpen}
              onShowDetails={onShowDetails}
              onRequestAddTag={onRequestAddTag}
              onRemoveTag={onRemoveTag}
              onSetTrust={onSetTrust}
            >
              <ObjectRow
                object={obj}
                isSelected={selectedObjects.includes(obj.id)}
                isAlternate={index % 2 === 1}
                onClick={(e) => handleClick(obj, e)}
                onDoubleClick={() => handleDoubleClick(obj)}
                onContextMenu={(e) => handleContextMenu(obj, e)}
              />
            </ObjectContextMenu>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
};
