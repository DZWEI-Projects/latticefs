import { useCallback } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ObjectCard } from "./ObjectCard";
import { ObjectContextMenu } from "./ObjectContextMenu";
import type { ObjectInfo, TagInfo, ViewInfo } from "@/lib/lfs";

interface GridViewProps {
  objects: ObjectInfo[];
  selectedObjects: string[];
  onObjectSelect: (objectId: string, multiSelect?: boolean) => void;
  onObjectOpen: (object: ObjectInfo) => void;
  onObjectFocus: (object: ObjectInfo) => void;
  onRequestAddTag: (object: ObjectInfo) => void;
  onRemoveTag: (object: ObjectInfo, tag: TagInfo) => void;
  onSetTrust: (object: ObjectInfo, trust: number | null) => void;
  onShowDetails: (object: ObjectInfo) => void;
  onOpenVersions: (object: ObjectInfo) => void;
  onOpenEditor: (object: ObjectInfo) => void;
  onRenameObject: (object: ObjectInfo) => void;
  views: ViewInfo[];
  onViewSelect: (viewId: string) => void;
}

export const GridView = ({
  objects,
  selectedObjects,
  onObjectSelect,
  onObjectOpen,
  onObjectFocus,
  onRequestAddTag,
  onRemoveTag,
  onSetTrust,
  onShowDetails,
  onOpenVersions,
  onOpenEditor,
  onRenameObject,
  views,
  onViewSelect,
}: GridViewProps) => {
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

  return (
    <ScrollArea className="h-full">
      <div className="p-4">
        <div className="grid grid-cols-[repeat(auto-fill,minmax(100px,1fr))] gap-3">
          {objects.map((obj) => (
            <ObjectContextMenu
              key={obj.id}
              object={obj}
              views={views}
              onViewSelect={onViewSelect}
              onOpen={onObjectOpen}
              onShowDetails={onShowDetails}
              onOpenVersions={onOpenVersions}
              onOpenEditor={onOpenEditor}
              onRename={onRenameObject}
              onRequestAddTag={onRequestAddTag}
              onRemoveTag={onRemoveTag}
              onSetTrust={onSetTrust}
            >
              <ObjectCard
                object={obj}
                isSelected={selectedObjects.includes(obj.id)}
                onClick={(e) => handleClick(obj, e)}
                onDoubleClick={() => handleDoubleClick(obj)}
                onContextMenu={(e) => handleContextMenu(obj, e)}
              />
            </ObjectContextMenu>
          ))}
        </div>
      </div>
    </ScrollArea>
  );
};
