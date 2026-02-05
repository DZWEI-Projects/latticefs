import { useCallback, useState } from "react";
import { cn } from "@/lib/utils";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ObjectRow } from "./ObjectRow";
import { ArrowUp, ArrowDown } from "lucide-react";
import type { ObjectInfo } from "@/lib/lfs";

interface ListViewProps {
  objects: ObjectInfo[];
  selectedObjects: string[];
  onObjectSelect: (objectId: string, multiSelect?: boolean) => void;
  onObjectOpen: (object: ObjectInfo) => void;
}

type SortField = "name" | "extension" | "sizeBytes" | "modifiedAt";
type SortDirection = "asc" | "desc";

export const ListView = ({
  objects,
  selectedObjects,
  onObjectSelect,
  onObjectOpen,
}: ListViewProps) => {
  const [sortField, setSortField] = useState<SortField>("name");
  const [sortDirection, setSortDirection] = useState<SortDirection>("asc");

  const handleSort = useCallback((field: SortField) => {
    if (sortField === field) {
      setSortDirection((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortField(field);
      setSortDirection("asc");
    }
  }, [sortField]);

  const sortedObjects = [...objects].sort((a, b) => {
    let comparison = 0;
    switch (sortField) {
      case "name":
        comparison = a.name.localeCompare(b.name);
        break;
      case "extension":
        comparison = (a.extension || "").localeCompare(b.extension || "");
        break;
      case "sizeBytes":
        comparison = a.sizeBytes - b.sizeBytes;
        break;
      case "modifiedAt":
        comparison = a.modifiedAt - b.modifiedAt;
        break;
    }
    return sortDirection === "asc" ? comparison : -comparison;
  });

  const handleClick = useCallback(
    (obj: ObjectInfo, e: React.MouseEvent) => {
      const multiSelect = e.metaKey || e.ctrlKey;
      onObjectSelect(obj.id, multiSelect);
    },
    [onObjectSelect]
  );

  const handleDoubleClick = useCallback(
    (obj: ObjectInfo) => {
      onObjectOpen(obj);
    },
    [onObjectOpen]
  );

  const SortIcon = sortDirection === "asc" ? ArrowUp : ArrowDown;

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex-shrink-0 flex items-center h-8 px-4 border-b border-border/50 text-xs text-muted-foreground bg-muted/30">
        <button
          className={cn(
            "flex-1 min-w-0 flex items-center gap-1 text-left hover:text-foreground transition-colors",
            sortField === "name" && "text-foreground"
          )}
          onClick={() => handleSort("name")}
        >
          Name
          {sortField === "name" && <SortIcon className="w-3 h-3" />}
        </button>
        <button
          className={cn(
            "w-20 flex items-center gap-1 hover:text-foreground transition-colors",
            sortField === "extension" && "text-foreground"
          )}
          onClick={() => handleSort("extension")}
        >
          Type
          {sortField === "extension" && <SortIcon className="w-3 h-3" />}
        </button>
        <button
          className={cn(
            "w-24 flex items-center gap-1 hover:text-foreground transition-colors",
            sortField === "sizeBytes" && "text-foreground"
          )}
          onClick={() => handleSort("sizeBytes")}
        >
          Size
          {sortField === "sizeBytes" && <SortIcon className="w-3 h-3" />}
        </button>
        <button
          className={cn(
            "w-32 flex items-center gap-1 hover:text-foreground transition-colors",
            sortField === "modifiedAt" && "text-foreground"
          )}
          onClick={() => handleSort("modifiedAt")}
        >
          Modified
          {sortField === "modifiedAt" && <SortIcon className="w-3 h-3" />}
        </button>
        <div className="w-24">Tags</div>
        <div className="w-16 text-right">Trust</div>
      </div>

      {/* Rows */}
      <ScrollArea className="flex-1">
        <div className="divide-y divide-border/30">
          {sortedObjects.map((obj, index) => (
            <ObjectRow
              key={obj.id}
              object={obj}
              isSelected={selectedObjects.includes(obj.id)}
              isAlternate={index % 2 === 1}
              onClick={(e) => handleClick(obj, e)}
              onDoubleClick={() => handleDoubleClick(obj)}
            />
          ))}
        </div>
      </ScrollArea>
    </div>
  );
};
