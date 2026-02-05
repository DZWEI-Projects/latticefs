import { useState, useCallback, useMemo, useRef } from "react";
import { cn } from "@/lib/utils";
import { ObjectNode } from "./ObjectNode";
import type { ObjectInfo, TagInfo } from "@/lib/lfs";
import { ObjectContextMenu } from "./ObjectContextMenu";

interface GraphViewProps {
  objects: ObjectInfo[];
  selectedObjects: string[];
  onObjectSelect: (objectId: string, multiSelect?: boolean) => void;
  onObjectOpen: (object: ObjectInfo) => void;
  onObjectFocus: (object: ObjectInfo) => void;
  onRequestAddTag: (object: ObjectInfo) => void;
  onRemoveTag: (object: ObjectInfo, tag: TagInfo) => void;
  onSetTrust: (object: ObjectInfo, trust: number | null) => void;
  onShowDetails: (object: ObjectInfo) => void;
}

// Graph container dimensions
const GRAPH_SIZE = 600;
const GRAPH_CENTER = GRAPH_SIZE / 2;
const HUB_RADIUS = 32;
const ORBIT_RADIUS_BASE = 180;
const ORBIT_RADIUS_INCREMENT = 60;

export const GraphView = ({
  objects,
  selectedObjects,
  onObjectSelect,
  onObjectOpen,
  onObjectFocus,
  onRequestAddTag,
  onRemoveTag,
  onSetTrust,
  onShowDetails,
}: GraphViewProps) => {
  const [hoveredObject, setHoveredObject] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Calculate node positions in orbital layout
  const nodePositions = useMemo(() => {
    const positions: Map<string, { x: number; y: number; orbit: number }> = new Map();
    const maxPerOrbit = 12;
    
    objects.forEach((obj, index) => {
      const orbit = Math.floor(index / maxPerOrbit);
      const indexInOrbit = index % maxPerOrbit;
      const itemsInThisOrbit = Math.min(maxPerOrbit, objects.length - orbit * maxPerOrbit);
      
      const angle = (indexInOrbit / itemsInThisOrbit) * Math.PI * 2 - Math.PI / 2;
      const radius = ORBIT_RADIUS_BASE + orbit * ORBIT_RADIUS_INCREMENT;
      
      positions.set(obj.id, {
        x: Math.cos(angle) * radius,
        y: Math.sin(angle) * radius,
        orbit,
      });
    });
    
    return positions;
  }, [objects]);

  const handleNodeClick = useCallback(
    (obj: ObjectInfo, e: React.MouseEvent) => {
      const multiSelect = e.metaKey || e.ctrlKey;
      onObjectSelect(obj.id, multiSelect);
      onObjectFocus(obj);
    },
    [onObjectSelect, onObjectFocus]
  );

  const handleNodeDoubleClick = useCallback(
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

  // Get objects that share views with the hovered object
  const connectedObjects = useMemo(() => {
    if (!hoveredObject) return new Set<string>();
    const hovered = objects.find((o) => o.id === hoveredObject);
    if (!hovered) return new Set<string>();
    
    const connected = new Set<string>();
    objects.forEach((obj) => {
      if (obj.id === hoveredObject) return;
      const sharedViews = obj.views.filter((v) => hovered.views.includes(v));
      if (sharedViews.length > 0) {
        connected.add(obj.id);
      }
    });
    return connected;
  }, [hoveredObject, objects]);

  return (
    <div
      ref={containerRef}
      className="w-full h-full flex items-center justify-center overflow-auto"
    >
      <div
        className="relative"
        style={{ width: GRAPH_SIZE, height: GRAPH_SIZE }}
      >
        {/* SVG for connections */}
        <svg
          className="absolute inset-0 w-full h-full pointer-events-none"
          style={{ zIndex: 0 }}
        >
          <defs>
            <linearGradient id="connection-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stopColor="hsl(var(--primary))" stopOpacity="0.3" />
              <stop offset="100%" stopColor="hsl(var(--secondary))" stopOpacity="0.3" />
            </linearGradient>
          </defs>
          
          <g transform={`translate(${GRAPH_CENTER}, ${GRAPH_CENTER})`}>
            {/* Connections from hub to nodes */}
            {objects.map((obj) => {
              const pos = nodePositions.get(obj.id);
              if (!pos) return null;
              
              const isHighlighted =
                hoveredObject === obj.id ||
                connectedObjects.has(obj.id) ||
                selectedObjects.includes(obj.id);
              
              return (
                <line
                  key={`hub-${obj.id}`}
                  x1="0"
                  y1="0"
                  x2={pos.x}
                  y2={pos.y}
                  stroke="url(#connection-gradient)"
                  strokeWidth={isHighlighted ? 1.5 : 0.5}
                  opacity={isHighlighted ? 0.8 : 0.2}
                  className="transition-all duration-300"
                />
              );
            })}
            
            {/* Connections between related objects */}
            {hoveredObject && (
              <>
                {Array.from(connectedObjects).map((connectedId) => {
                  const hoveredPos = nodePositions.get(hoveredObject);
                  const connectedPos = nodePositions.get(connectedId);
                  if (!hoveredPos || !connectedPos) return null;
                  
                  return (
                    <line
                      key={`conn-${hoveredObject}-${connectedId}`}
                      x1={hoveredPos.x}
                      y1={hoveredPos.y}
                      x2={connectedPos.x}
                      y2={connectedPos.y}
                      stroke="hsl(var(--primary))"
                      strokeWidth="1.5"
                      strokeOpacity="0.6"
                      strokeDasharray="4 2"
                    />
                  );
                })}
              </>
            )}
          </g>
        </svg>

        {/* Central hub */}
        <div
          className={cn(
            "absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2",
            "w-16 h-16 rounded-full bg-primary/20 border-2 border-primary/40",
            "flex items-center justify-center z-10"
          )}
        >
          <div className="w-10 h-10 rounded-full bg-primary/30 flex items-center justify-center">
            <div className="w-5 h-5 rounded-full bg-primary" />
          </div>
        </div>

        {/* Object nodes */}
        {objects.map((obj) => {
          const pos = nodePositions.get(obj.id);
          if (!pos) return null;
          
          const isSelected = selectedObjects.includes(obj.id);
          const isHovered = hoveredObject === obj.id;
          const isConnected = connectedObjects.has(obj.id);
          
          return (
            <ObjectContextMenu
              key={obj.id}
              object={obj}
              onOpen={onObjectOpen}
              onShowDetails={onShowDetails}
              onRequestAddTag={onRequestAddTag}
              onRemoveTag={onRemoveTag}
              onSetTrust={onSetTrust}
            >
              <ObjectNode
                object={obj}
                position={pos}
                isSelected={isSelected}
                isHovered={isHovered}
                isConnected={isConnected}
                onHover={() => setHoveredObject(obj.id)}
                onLeave={() => setHoveredObject(null)}
                onClick={(e) => handleNodeClick(obj, e)}
                onDoubleClick={() => handleNodeDoubleClick(obj)}
                onContextMenu={(e) => handleContextMenu(obj, e)}
              />
            </ObjectContextMenu>
          );
        })}
      </div>
    </div>
  );
};
