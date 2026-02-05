import { useCallback } from "react";
import { cn } from "@/lib/utils";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ObjectCard } from "./ObjectCard";
import type { ObjectInfo } from "@/lib/lfs";

interface GridViewProps {
  objects: ObjectInfo[];
  selectedObjects: string[];
  onObjectSelect: (objectId: string, multiSelect?: boolean) => void;
  onObjectOpen: (object: ObjectInfo) => void;
}

export const GridView = ({
  objects,
  selectedObjects,
  onObjectSelect,
  onObjectOpen,
}: GridViewProps) => {
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

  return (
    <ScrollArea className="h-full">
      <div className="p-4">
        <div className="grid grid-cols-[repeat(auto-fill,minmax(100px,1fr))] gap-3">
          {objects.map((obj) => (
            <ObjectCard
              key={obj.id}
              object={obj}
              isSelected={selectedObjects.includes(obj.id)}
              onClick={(e) => handleClick(obj, e)}
              onDoubleClick={() => handleDoubleClick(obj)}
            />
          ))}
        </div>
      </div>
    </ScrollArea>
  );
};
