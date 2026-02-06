/**
 * Trust level constants for object security.
 */

/** Default trust level for new objects (matches backend default) */
export const DEFAULT_TRUST_LEVEL = 70;

/** Trust level that marks an object as quarantined */
export const QUARANTINE_TRUST_LEVEL = 25;

/**
 * Predefined trust level presets for the UI.
 */
export const TRUST_PRESETS = [
  { label: "Bestätigt", value: 100 },
  { label: "Hoch", value: 85 },
  { label: "Mittel", value: 65 },
  { label: "Niedrig", value: 40 },
  { label: "Quarantäne", value: QUARANTINE_TRUST_LEVEL },
  { label: "Kritisch", value: 15 },
  { label: "Nicht gesetzt", value: null },
] as const;
