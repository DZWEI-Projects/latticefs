const MIME_TYPE_LABELS: Record<string, string> = {
  "text/plain": "Textdatei",
  "text/markdown": "Markdown-Datei",
  "text/html": "HTML-Dokument",
  "text/css": "CSS-Datei",
  "text/csv": "CSV-Datei",
  "text/xml": "XML-Datei",
  "text/javascript": "JavaScript-Datei",
  "application/json": "JSON-Datei",
  "application/xml": "XML-Datei",
  "application/pdf": "PDF-Dokument",
  "application/rtf": "Rich-Text-Dokument",
  "application/zip": "ZIP-Archiv",
  "application/x-7z-compressed": "7z-Archiv",
  "application/x-rar-compressed": "RAR-Archiv",
  "application/x-tar": "TAR-Archiv",
  "application/gzip": "GZIP-Archiv",
  "application/x-gzip": "GZIP-Archiv",
  "application/x-bzip2": "BZIP2-Archiv",
  "application/x-xz": "XZ-Archiv",
  "application/x-sqlite3": "SQLite-Datenbank",
  "application/sql": "SQL-Datei",
  "application/msword": "Word-Dokument",
  "application/vnd.openxmlformats-officedocument.wordprocessingml.document":
    "Word-Dokument (DOCX)",
  "application/vnd.ms-excel": "Excel-Tabelle",
  "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet":
    "Excel-Tabelle (XLSX)",
  "application/vnd.ms-powerpoint": "PowerPoint-Präsentation",
  "application/vnd.openxmlformats-officedocument.presentationml.presentation":
    "PowerPoint-Präsentation (PPTX)",
  "application/vnd.oasis.opendocument.text": "OpenDocument-Text",
  "application/vnd.oasis.opendocument.spreadsheet": "OpenDocument-Tabelle",
  "application/vnd.oasis.opendocument.presentation":
    "OpenDocument-Präsentation",
  "application/epub+zip": "EPUB-E-Book",
  "application/x-sh": "Shell-Skript",
  "application/octet-stream": "Binärdatei",
  "image/jpeg": "JPEG",
  "image/png": "PNG",
  "image/gif": "GIF",
  "image/webp": "WEBP",
  "image/svg+xml": "SVG",
  "image/bmp": "BMP",
  "image/tiff": "TIFF",
  "image/heic": "HEIC",
  "image/heif": "HEIF",
  "audio/mpeg": "MP3",
  "audio/wav": "WAV",
  "audio/flac": "FLAC",
  "audio/aac": "AAC",
  "audio/ogg": "OGG",
  "audio/mp4": "M4A",
  "audio/webm": "WebM Audio",
  "video/mp4": "MP4",
  "video/webm": "WebM Video",
  "video/quicktime": "QuickTime (MOV)",
  "video/x-msvideo": "AVI",
  "video/x-matroska": "MKV",
  "video/mpeg": "MPEG",
};

const EXIF_FIELD_LABELS: Record<string, string> = {
  datetimeoriginal: "Aufnahmezeit",
  createdate: "Erstellungsdatum",
  modifydate: "Änderungsdatum",
  make: "Kamera-Hersteller",
  model: "Kameramodell",
  lensmodel: "Objektiv",
  lensserialnumber: "Objektiv-Seriennummer",
  serialnumber: "Kamera-Seriennummer",
  software: "Software",
  artist: "Urheber",
  copyright: "Copyright",
  orientation: "Ausrichtung",
  imagewidth: "Breite",
  imagelength: "Höhe",
  xresolution: "Auflösung (X)",
  yresolution: "Auflösung (Y)",
  resolutionunit: "Auflösungseinheit",
  exposuretime: "Belichtungszeit",
  exposureprogram: "Belichtungsprogramm",
  exposuremode: "Belichtungsmodus",
  fnumber: "Blende",
  isospeedratings: "ISO",
  photometricinterpretation: "Farbmodell",
  whitebalance: "Weißabgleich",
  meteringmode: "Messmethode",
  flash: "Blitz",
  focal_length: "Brennweite",
  focallength: "Brennweite",
  focallengthin35mmfilm: "Brennweite (35mm)",
  digitalzoomratio: "Digitalzoom",
  gpslatitude: "GPS-Breite",
  gpslatituderef: "GPS-Breite (Richtung)",
  gpslongitude: "GPS-Länge",
  gpslongituderef: "GPS-Länge (Richtung)",
  gpsaltitude: "GPS-Höhe",
  gpsimgdirection: "GPS-Richtung",
  gpsdatestamp: "GPS-Datum",
  gpstimestamp: "GPS-Zeit",
};

const KNOWN_MIME_SUFFIXES: Record<string, string> = {
  jpeg: "JPEG",
  jpg: "JPEG",
  png: "PNG",
  gif: "GIF",
  webp: "WEBP",
  svg: "SVG",
  tif: "TIFF",
  tiff: "TIFF",
  mp3: "MP3",
  mp4: "MP4",
  m4a: "M4A",
  mov: "MOV",
  avi: "AVI",
  mkv: "MKV",
  wav: "WAV",
  flac: "FLAC",
  aac: "AAC",
  ogg: "OGG",
};

export function formatMimeType(mime?: string | null): string {
  if (!mime) return "—";
  const normalized = mime.toLowerCase();
  if (MIME_TYPE_LABELS[normalized]) {
    return MIME_TYPE_LABELS[normalized];
  }
  const subtype = normalized.split("/")[1];
  if (subtype && KNOWN_MIME_SUFFIXES[subtype]) {
    return KNOWN_MIME_SUFFIXES[subtype];
  }
  return mime;
}

export function formatMetadataKeyLabel(raw: string): string {
  if (!raw) return "—";
  const withSpaces = raw.replace(/_/g, " ");
  return withSpaces.charAt(0).toUpperCase() + withSpaces.slice(1);
}

export function formatExifFieldLabel(raw: string): string {
  const key = raw.toLowerCase();
  return EXIF_FIELD_LABELS[key] ?? formatMetadataKeyLabel(raw);
}

export { MIME_TYPE_LABELS, EXIF_FIELD_LABELS };
