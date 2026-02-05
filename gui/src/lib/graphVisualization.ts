/**
 * Utilities for graph visualization, including heat-based colors,
 * line thickness, opacity, and glow effects based on shared view counts.
 */

/**
 * Calculate heat-based color based on shared view count.
 * Cool colors (blue/cyan) for few views, warm colors (orange/red) for many views.
 * 
 * @param sharedViewCount - Number of shared views between objects
 * @returns HSL color string
 */
export const getHeatColor = (sharedViewCount: number): string => {
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
};

/**
 * Calculate line thickness based on shared view count.
 * Minimum 1.0px for 1 shared view, up to 3.5px for many shared views.
 * 
 * @param sharedViewCount - Number of shared views between objects
 * @returns Line thickness in pixels
 */
export const getConnectionThickness = (sharedViewCount: number): number => {
  const minThickness = 1.0;
  const maxThickness = 3.5;
  return Math.min(minThickness + (sharedViewCount - 1) * 0.5, maxThickness);
};

/**
 * Calculate opacity for connection lines based on shared view count.
 * Stronger connections have higher opacity.
 * 
 * @param sharedViewCount - Number of shared views between objects
 * @returns Opacity value between 0.6 and 0.95
 */
export const getConnectionOpacity = (sharedViewCount: number): number => {
  return Math.min(0.6 + sharedViewCount * 0.05, 0.95);
};

/**
 * Calculate glow filter ID based on shared view count.
 * More shared views = stronger glow effect.
 * 
 * @param sharedViewCount - Number of shared views between objects
 * @returns Filter ID string (e.g., "glow-5", "glow-10")
 */
export const getGlowFilterId = (sharedViewCount: number): string => {
  // More shared views = stronger glow
  const intensity = Math.min(sharedViewCount * 0.5, 3); // 0.5 to 3
  return `glow-${Math.round(intensity * 10)}`;
};

/**
 * Check if a connection should have a glow effect.
 * Only connections with 3+ shared views get glow.
 * 
 * @param sharedViewCount - Number of shared views between objects
 * @returns True if glow should be applied
 */
export const shouldApplyGlow = (sharedViewCount: number): boolean => {
  return sharedViewCount >= 3;
};
