import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { useQuery } from "@tanstack/react-query";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import {
  checkoutObjectVersion,
  diffObjectVersions,
  exportObjectVersion,
  getObjectVersionText,
  pickExportPath,
  pickFiles,
  reviseObjectFromFile,
  reviseObjectFromText,
  setObjectVersionState,
  type ObjectInfo,
  type ObjectVersion,
  type VersionDiffResult,
  type VersionState,
} from "@/lib/lfs";

interface ObjectVersionsSectionProps {
  object: ObjectInfo;
  versions: ObjectVersion[] | undefined;
  isLoading: boolean;
  onRefresh: () => void;
}

const versionStates: VersionState[] = [
  "draft",
  "review",
  "approved",
  "discarded",
  "sealed",
  "archived",
];

const textExtensions = new Set([
  "txt",
  "md",
  "markdown",
  "json",
  "yaml",
  "yml",
  "toml",
  "csv",
  "log",
  "ini",
  "xml",
  "js",
  "ts",
  "jsx",
  "tsx",
  "py",
  "rs",
  "go",
  "java",
]);

const formatDate = (timestamp: number) =>
  new Date(timestamp).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });

const formatBytes = (bytes: number) => {
  if (!bytes) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`.replace(
    ".",
    ",",
  );
};

export const ObjectVersionsSection = ({
  object,
  versions,
  isLoading,
  onRefresh,
}: ObjectVersionsSectionProps) => {
  const [reviseMessage, setReviseMessage] = useState("");
  const [exportMode, setExportMode] = useState<"tree" | "archive">("tree");
  const [leftVersionId, setLeftVersionId] = useState<string | null>(null);
  const [rightVersionId, setRightVersionId] = useState<string | null>(null);
  const [editorVersionId, setEditorVersionId] = useState<string | null>(null);
  const [editorContent, setEditorContent] = useState("");
  const [editorError, setEditorError] = useState<string | null>(null);
  const [editorLoading, setEditorLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  const sortedVersions = useMemo(
    () => (versions ? [...versions].sort((a, b) => a.index - b.index) : []),
    [versions],
  );

  const currentVersion = useMemo(
    () => sortedVersions.find((version) => version.isCurrent) ?? null,
    [sortedVersions],
  );
  const previousVersionId = currentVersion?.parentVersion ?? null;
  const isSealed = currentVersion?.state === "sealed";

  const isTextEditable = useMemo(() => {
    if (!object.extension) return false;
    return textExtensions.has(object.extension.toLowerCase());
  }, [object.extension]);

  useEffect(() => {
    if (!sortedVersions.length) return;
    setLeftVersionId((prev) => prev ?? previousVersionId ?? sortedVersions[0].id);
    setRightVersionId((prev) => prev ?? currentVersion?.id ?? sortedVersions.at(-1)?.id ?? null);
    setEditorVersionId((prev) => prev ?? currentVersion?.id ?? sortedVersions[0].id);
  }, [sortedVersions, previousVersionId, currentVersion?.id]);

  useEffect(() => {
    const loadText = async () => {
      if (!isTextEditable || !editorVersionId) {
        setEditorContent("");
        setEditorError(null);
        return;
      }
      setEditorLoading(true);
      setEditorError(null);
      try {
        const text = await getObjectVersionText(object.id, editorVersionId);
        setEditorContent(text);
      } catch (error) {
        setEditorError(
          error instanceof Error ? error.message : "Inhalt konnte nicht geladen werden.",
        );
        setEditorContent("");
      } finally {
        setEditorLoading(false);
      }
    };
    void loadText();
  }, [editorVersionId, isTextEditable, object.id]);

  const diffQuery = useQuery<VersionDiffResult, Error>({
    queryKey: ["versionDiff", object.id, leftVersionId, rightVersionId],
    queryFn: () =>
      diffObjectVersions(object.id, leftVersionId!, rightVersionId!),
    enabled: Boolean(
      leftVersionId && rightVersionId && leftVersionId !== rightVersionId,
    ),
  });

  const handleUploadVersion = async () => {
    const paths = await pickFiles();
    if (!paths || paths.length === 0) return;
    setIsSaving(true);
    try {
      await reviseObjectFromFile(object.id, paths[0], reviseMessage || undefined);
      toast.success("Neue Version erstellt");
      setReviseMessage("");
      onRefresh();
    } catch (error) {
      toast.error(
        error instanceof Error
          ? error.message
          : "Neue Version konnte nicht erstellt werden",
      );
    } finally {
      setIsSaving(false);
    }
  };

  const handleSaveEditor = async () => {
    if (!isTextEditable || isSealed) return;
    setIsSaving(true);
    try {
      await reviseObjectFromText(object.id, editorContent, reviseMessage || undefined);
      toast.success("Neue Version gespeichert");
      setReviseMessage("");
      onRefresh();
    } catch (error) {
      toast.error(
        error instanceof Error
          ? error.message
          : "Neue Version konnte nicht gespeichert werden",
      );
    } finally {
      setIsSaving(false);
    }
  };

  const handleSetState = async (versionId: string, state: VersionState) => {
    setIsSaving(true);
    try {
      await setObjectVersionState(object.id, versionId, state);
      toast.success("Status aktualisiert");
      onRefresh();
    } catch (error) {
      toast.error(
        error instanceof Error
          ? error.message
          : "Status konnte nicht geändert werden",
      );
    } finally {
      setIsSaving(false);
    }
  };

  const handleCheckout = async (versionId: string) => {
    setIsSaving(true);
    try {
      await checkoutObjectVersion(object.id, versionId);
      toast.success("Version aktiviert");
      onRefresh();
    } catch (error) {
      toast.error(
        error instanceof Error
          ? error.message
          : "Version konnte nicht aktiviert werden",
      );
    } finally {
      setIsSaving(false);
    }
  };

  const handleExport = async (versionId: string) => {
    const outputPath = await pickExportPath(object.name);
    if (!outputPath) return;
    setIsSaving(true);
    try {
      await exportObjectVersion(object.id, versionId, outputPath, exportMode);
      toast.success("Export abgeschlossen");
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Export fehlgeschlagen",
      );
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-semibold text-foreground/75 uppercase tracking-wider">
          Versionen
        </h3>
        <span className="text-xs text-muted-foreground">
          {sortedVersions.length}
        </span>
      </div>
      <Tabs defaultValue="list" className="space-y-3">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="list">Übersicht</TabsTrigger>
          <TabsTrigger value="diff">Diff</TabsTrigger>
          <TabsTrigger value="editor">Editor</TabsTrigger>
        </TabsList>
        <TabsContent value="list" className="space-y-3">
          <div className="space-y-2 rounded-lg border border-border/60 p-3">
            <p className="text-xs text-muted-foreground">
              Neue Versionen übernehmen automatisch den Statuswechsel von Draft
              → Discarded oder Review → Approved.
            </p>
            {isSealed && (
              <p className="text-xs text-amber-500">
                Dieses Objekt ist versiegelt. Es können keine neuen Versionen
                erstellt werden.
              </p>
            )}
            <Input
              value={reviseMessage}
              onChange={(event) => setReviseMessage(event.target.value)}
              placeholder="Versionsnachricht (optional)"
              className="h-8 text-xs"
            />
            <div className="flex flex-wrap items-center gap-2">
              <Button
                size="sm"
                onClick={handleUploadVersion}
                disabled={isSaving || isSealed}
              >
                Neue Version hochladen
              </Button>
              <Select
                value={exportMode}
                onValueChange={(value) => setExportMode(value as "tree" | "archive")}
              >
                <SelectTrigger className="h-8 w-[160px] text-xs">
                  <SelectValue placeholder="Exportmodus" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="tree">Export (Datei)</SelectItem>
                  <SelectItem value="archive">Export (Archiv)</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          {isLoading ? (
            <p className="text-xs text-muted-foreground">Lädt...</p>
          ) : sortedVersions.length === 0 ? (
            <p className="text-xs text-muted-foreground">Keine Versionen gefunden.</p>
          ) : (
            <div className="space-y-2">
              {sortedVersions.map((version) => {
                const isCurrent = version.isCurrent;
                const isPrevious = version.id === previousVersionId;
                const canChangeState = isCurrent || isPrevious;
                return (
                  <div
                    key={version.id}
                    className="rounded-lg border border-border/60 p-3 space-y-2"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <Badge variant="secondary">v{version.index}</Badge>
                        {isCurrent && <Badge>Aktuell</Badge>}
                        {isPrevious && (
                          <Badge variant="outline">Vorherig</Badge>
                        )}
                        <Badge variant="outline" className="capitalize">
                          {version.state}
                        </Badge>
                      </div>
                      <span className="text-xs text-muted-foreground">
                        {formatDate(version.createdAt)}
                      </span>
                    </div>
                    <div className="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
                      <span>{formatBytes(version.sizeBytes)}</span>
                      {version.message && <span>„{version.message}“</span>}
                    </div>
                    <div className="flex flex-wrap items-center gap-2">
                      {!isCurrent && (
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => handleCheckout(version.id)}
                          disabled={isSaving}
                        >
                          Aktivieren
                        </Button>
                      )}
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleExport(version.id)}
                        disabled={isSaving}
                      >
                        Exportieren
                      </Button>
                      {canChangeState && (
                        <Select
                          value={version.state}
                          onValueChange={(value) =>
                            handleSetState(version.id, value as VersionState)
                          }
                          disabled={isSaving}
                        >
                          <SelectTrigger className="h-8 w-[160px] text-xs">
                            <SelectValue placeholder="Status ändern" />
                          </SelectTrigger>
                          <SelectContent>
                            {versionStates.map((state) => (
                              <SelectItem key={state} value={state}>
                                {state}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      )}
                    </div>
                    {canChangeState && (
                      <p className="text-xs text-muted-foreground">
                        Statusänderungen sind für die aktuelle oder vorherige
                        Version verfügbar.
                      </p>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </TabsContent>
        <TabsContent value="diff" className="space-y-3">
          {sortedVersions.length < 2 ? (
            <p className="text-xs text-muted-foreground">
              Für einen Vergleich werden mindestens zwei Versionen benötigt.
            </p>
          ) : (
            <div className="space-y-3">
              <div className="grid grid-cols-2 gap-2">
                <Select
                  value={leftVersionId ?? undefined}
                  onValueChange={(value) => setLeftVersionId(value)}
                >
                  <SelectTrigger className="h-8 text-xs">
                    <SelectValue placeholder="Linke Version" />
                  </SelectTrigger>
                  <SelectContent>
                    {sortedVersions.map((version) => (
                      <SelectItem key={version.id} value={version.id}>
                        v{version.index} · {version.state}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Select
                  value={rightVersionId ?? undefined}
                  onValueChange={(value) => setRightVersionId(value)}
                >
                  <SelectTrigger className="h-8 text-xs">
                    <SelectValue placeholder="Rechte Version" />
                  </SelectTrigger>
                  <SelectContent>
                    {sortedVersions.map((version) => (
                      <SelectItem key={version.id} value={version.id}>
                        v{version.index} · {version.state}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              {leftVersionId === rightVersionId ? (
                <p className="text-xs text-muted-foreground">
                  Bitte zwei unterschiedliche Versionen auswählen.
                </p>
              ) : diffQuery.isLoading ? (
                <p className="text-xs text-muted-foreground">Vergleich läuft...</p>
              ) : diffQuery.error ? (
                <p className="text-xs text-destructive">
                  {diffQuery.error.message}
                </p>
              ) : diffQuery.data ? (
                <DiffViewer diff={diffQuery.data} />
              ) : null}
            </div>
          )}
        </TabsContent>
        <TabsContent value="editor" className="space-y-3">
          {!isTextEditable ? (
            <p className="text-xs text-muted-foreground">
              Dieses Dateiformat ist nicht als Text editierbar.
            </p>
          ) : (
            <div className="space-y-3">
              {isSealed && (
                <p className="text-xs text-amber-500">
                  Dieses Objekt ist versiegelt. Es können keine neuen Versionen
                  gespeichert werden.
                </p>
              )}
              <Select
                value={editorVersionId ?? undefined}
                onValueChange={(value) => setEditorVersionId(value)}
              >
                <SelectTrigger className="h-8 text-xs">
                  <SelectValue placeholder="Version auswählen" />
                </SelectTrigger>
                <SelectContent>
                  {sortedVersions.map((version) => (
                    <SelectItem key={version.id} value={version.id}>
                      v{version.index} · {version.state}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {editorLoading ? (
                <p className="text-xs text-muted-foreground">Lädt...</p>
              ) : editorError ? (
                <p className="text-xs text-destructive">{editorError}</p>
              ) : (
                <Textarea
                  value={editorContent}
                  onChange={(event) => setEditorContent(event.target.value)}
                  className="min-h-[160px] font-mono text-xs"
                />
              )}
              <Button
                size="sm"
                onClick={handleSaveEditor}
                disabled={isSaving || isSealed || editorLoading || Boolean(editorError)}
              >
                Speichern & neue Version erzeugen
              </Button>
            </div>
          )}
        </TabsContent>
      </Tabs>
    </section>
  );
};

const DiffViewer = ({ diff }: { diff: VersionDiffResult }) => {
  if (diff.kind === "none") {
    return <p className="text-xs text-muted-foreground">Keine Unterschiede.</p>;
  }
  if (diff.kind === "binary") {
    return (
      <div className="rounded-lg border border-border/60 p-3 text-xs">
        <p className="font-medium">Binärer Vergleich</p>
        <p className="text-muted-foreground">{diff.diff}</p>
        <p className="text-muted-foreground">
          Links: {diff.leftSize} Bytes · Rechts: {diff.rightSize} Bytes
        </p>
        {diff.firstDiff !== null && diff.firstDiff !== undefined && (
          <p className="text-muted-foreground">
            Erstes abweichendes Byte: {diff.firstDiff}
          </p>
        )}
      </div>
    );
  }
  return (
    <div className="rounded-lg border border-border/60 bg-muted/20 p-3 text-xs">
      <pre className="whitespace-pre-wrap font-mono">
        {diff.diff.split("\n").map((line, index) => {
          let className = "text-muted-foreground";
          if (line.startsWith("+")) className = "text-green-500";
          if (line.startsWith("-")) className = "text-red-500";
          if (line.startsWith("@")) className = "text-blue-500";
          if (line.startsWith("+++")) className = "text-green-400";
          if (line.startsWith("---")) className = "text-red-400";
          return (
            <div key={`${line}-${index}`} className={className}>
              {line || " "}
            </div>
          );
        })}
      </pre>
    </div>
  );
};
