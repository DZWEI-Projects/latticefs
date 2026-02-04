# Guided Workflows

These walkthroughs show end‑to‑end flows with LatticeFS. Each one is written as a small story that explains *why* you are running a command and what it changes in the system. The central idea is simple: **you retrieve and organize data by meaning**, not by file paths or filenames.

## Music library: tag by artist and export a view
Goal: ingest two MP3s, let the system read their ID3 metadata, and then ask for “everything by this artist” without remembering filenames or folder locations.

1. Add files with a user tag to group them at a high level. You are telling LatticeFS that both files belong to a personal music collection:
```bash
lsf add ./LeadingTorture.mp3 --tag "mukke:meins"
lsf add ./Legendary_Torture.mp3 --tag "mukke:meins"
```
At this point, the files are stored as objects. LatticeFS also extracts ID3 tags automatically, so it already knows artist, album, track number, and other context.

2. Inspect extracted metadata. This lets you confirm what the system learned from the file:
```bash
lsf meta <object-id>
```
You should see `auto:id3:*` tags (artist/title/album/track/year) and `auto:mimetype:audio/mpeg`. These tags are the raw material for semantic queries.

3. Create a view scoped by the ID3 artist tag. This is where the workflow departs from the traditional filesystem: you stop thinking about where a file lives, and you ask for what it *is*:
```bash
lsf view create "Benn Musik" --query "tag:auto:id3:artist:BENN"
```
This is the key shift: you are **asking for content by meaning** (“artist is BENN”) rather than by filename. The view is a living query, not a static folder.

4. Export the view to a directory. Exporting materializes the query result so other tools can consume it:
```bash
lsf export "Benn Musik" --output ./bennmusik --mode tree
```
Export materializes the selection so other tools can consume it, but the selection itself is **semantic**: it came from metadata, not filenames. If you re‑import more music later, the same view will pick it up automatically.

## Document revisions: versioning + review state
Goal: revise a document, mark it for review, and export the approved set so you only share vetted versions.

1. Add the document. This creates a stable object ID that represents the document across all revisions:
```bash
lsf add ./report.md --tag doc:report
```
You now have a stable object ID representing this document, so you no longer need “final‑final.pdf” naming games.

2. Revise content. This creates a new version under the same object ID, preserving history:
```bash
lsf revise <object-id> ./report.md -m "add summary"
```
This keeps the **same object**, but adds a new version, so history remains intact and diffable.

3. Mark the latest version as review. State is a workflow signal, not a content change:
```bash
lsf state set <object-id> review
```
State is a workflow marker; it doesn’t change content, it changes how you **query** and **organize**. It also makes it easy to build “approved‑only” views later.

4. Approve after review. This is the moment where you decide the version is safe to share:
```bash
lsf state set <object-id> approved
```

5. Create a view of approved documents. The view captures intent in a single query:
```bash
lsf view create "Approved Reports" --query "tag:doc:report AND state:approved"
```
Again, the view is driven by meaning (“doc:report” and “approved”), not by directory structure. You can export or share this view with confidence.

6. Export the view. This produces a shareable directory without changing the underlying data model:
```bash
lsf export "Approved Reports" --output ./approved-reports --mode tree
```

## Safety flow: quarantine then seal
Goal: import an executable, quarantine it, then seal once vetted to prevent further updates and accidental changes.

1. Add a script. LatticeFS auto‑detects executables on Unix and tags them accordingly:
```bash
lsf add ./scripts/scan.sh --tag project:security
```
LatticeFS tags executables automatically, which makes policy and review flows easier. You do not have to remember which files were “dangerous.”

2. Quarantine it. This records explicit caution in metadata without moving the file anywhere:
```bash
lsf trust set <object-id> quarantined
```
Now it’s explicitly flagged for caution without moving files around or duplicating data.

3. If it passes review, update trust and seal the version. Sealing is a hard lock:
```bash
lsf trust set <object-id> approved
lsf state set <object-id> sealed
```
Sealing is a hard lock: the object can no longer be updated. It is a deliberate “finalize” step.

4. Any attempt to create a new version now fails, which protects the object from accidental changes:
```bash
lsf revise <object-id> ./scripts/scan.sh -m "attempted update"
```
Expected: a clear error about the object being sealed. This prevents accidental or unauthorized changes once a version is finalized.
