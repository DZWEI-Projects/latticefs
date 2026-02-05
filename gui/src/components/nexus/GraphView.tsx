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

  // Get objects that share views with the hovered object, along with shared view count
  const connectedObjects = useMemo(() => {
    if (!hoveredObject) return new Map<string, number>();
    const hovered = objects.find((o) => o.id === hoveredObject);
    if (!hovered) return new Map<string, number>();
    
    const connected = new Map<string, number>();
    objects.forEach((obj) => {
      if (obj.id === hoveredObject) return;
      const sharedViews = obj.views.filter((v) => hovered.views.includes(v));
      if (sharedViews.length > 0) {
        connected.set(obj.id, sharedViews.length);
      }
    });
    return connected;
  }, [hoveredObject, objects]);

  // Calculate heat-based color based on shared view count
  // Cool colors (blue/cyan) for few views, warm colors (orange/red) for many views
  const getHeatColor = useCallback((sharedViewCount: number): string => {
    // Normalize to 0-1 range (assuming max ~10 shared views for good gradient)
    const maxViews = 10;
    const heat = Math.min(sharedViewCount / maxViews, 1);
    
    // Color gradient: blue (200°) -> purple (280°) -> pink (320°) -> orange (30°)
    // Hue ranges from 200 (cool blue) to 30 (warm orange), wrapping through purple/pink
    // We go from 200° to 30° which is 200° + 190° = 390°, wrapping to 30°
    let hue: number;
    if (heat < 0.33) {
      // Cool: blue to purple (200° to 280°)
      hue = 200 + heat * 240; // 200 to ~280
    } else if (heat < 0.66) {
      // Warm: purple to pink (280° to 320°)
      hue = 280 + (heat - 0.33) * 120; // 280 to ~320
    } else {
      // Hot: pink to orange (320° to 30°, wrapping around)
      // 320° + (heat - 0.66) * 190 wraps: 320° -> 360° -> 30°
      const progress = (heat - 0.66) / 0.34; // 0 to 1
      hue = 320 + progress * 70; // 320 to 390
      if (hue >= 360) hue = hue - 360; // Wrap: 390 -> 30
    }
    
    // Saturation and lightness increase with heat for more vibrant "hot" colors
    const saturation = 60 + heat * 30; // 60% to 90%
    const lightness = 50 + heat * 15; // 50% to 65%
    
    return `hsl(${Math.round(hue)}, ${Math.round(saturation)}%, ${Math.round(lightness)}%)`;
  }, []);

  // Calculate glow intensity based on shared view count
  const getGlowFilterId = useCallback((sharedViewCount: number): string => {
    // More shared views = stronger glow
    const intensity = Math.min(sharedViewCount * 0.5, 3); // 0.5 to 3
    return `glow-${Math.round(intensity * 10)}`;
  }, []);

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
            
            {/* Glow filters for heat effect - different intensities based on shared view count */}
            {/* Generate filters for intensities from 0.5 to 3.0 in 0.5 increments */}
            {Array.from({ length: 6 }, (_, i) => {
              const intensity = 0.5 + i * 0.5; // 0.5, 1.0, 1.5, 2.0, 2.5, 3.0
              const filterId = `glow-${Math.round(intensity * 10)}`; // glow-5, glow-10, etc.
              return (
                <filter key={filterId} id={filterId} x="-50%" y="-50%" width="200%" height="200%">
                  <feGaussianBlur stdDeviation={intensity} result="coloredBlur"/>
                  <feMerge>
                    <feMergeNode in="coloredBlur"/>
                    <feMergeNode in="SourceGraphic"/>
                  </feMerge>
                </filter>
              );
            })}
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
                  strokeWidth={isHighlighted ? 1.75 : 0.75}
                  opacity={isHighlighted ? 0.99 : 0.8}
                  className="transition-all duration-300"
                />
              );
            })}
            
            {/* Connections between related objects */}
            {hoveredObject && (
              <>
                {Array.from(connectedObjects.entries()).map(([connectedId, sharedViewCount]) => {
                  const hoveredPos = nodePositions.get(hoveredObject);
                  const connectedPos = nodePositions.get(connectedId);
                  if (!hoveredPos || !connectedPos) return null;
                  
                  // Calculate line thickness based on shared view count
                  // Minimum 1.0px for 1 shared view, up to 3.5px for many shared views
                  const minThickness = 1.0;
                  const maxThickness = 3.5;
                  const thickness = Math.min(minThickness + (sharedViewCount - 1) * 0.5, maxThickness);
                  
                  // Increase opacity for stronger connections (heat effect)
                  const opacity = Math.min(0.6 + sharedViewCount * 0.05, 0.95);
                  
                  // Get heat-based color (cool to warm gradient)
                  const heatColor = getHeatColor(sharedViewCount);
                  
                  // Get glow filter for stronger connections
                  const glowFilterId = getGlowFilterId(sharedViewCount);
                  const hasGlow = sharedViewCount >= 3; // Only glow for 3+ shared views
                  
                  return (
                    <line
                      key={`conn-${hoveredObject}-${connectedId}`}
                      x1={hoveredPos.x}
                      y1={hoveredPos.y}
                      x2={connectedPos.x}
                      y2={connectedPos.y}
                      stroke={heatColor}
                      strokeWidth={thickness}
                      strokeOpacity={opacity}
                      strokeDasharray="4 2"
                      filter={hasGlow ? `url(#${glowFilterId})` : undefined}
                      className="transition-all duration-300"
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
