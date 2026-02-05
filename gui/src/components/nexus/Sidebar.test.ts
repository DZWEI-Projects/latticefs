import { describe, expect, it } from "vitest";
import type { ViewInfo } from "@/lib/lfs";
import { buildDynamicRows } from "./Sidebar";

function dynamicView(
  id: string,
  name: string,
  parentId: string | null = null,
): ViewInfo {
  return {
    id,
    name,
    description: "",
    query: "tag:test",
    viewType: "dynamic",
    parentId,
    icon: null,
    objectCount: 0,
  };
}

describe("buildDynamicRows", () => {
  it("creates a depth-first tree with sorted siblings", () => {
    const views: ViewInfo[] = [
      dynamicView("root-b", "B"),
      dynamicView("root-a", "A"),
      dynamicView("child-2", "Y", "root-a"),
      dynamicView("child-1", "X", "root-a"),
    ];

    const rows = buildDynamicRows(views);
    expect(rows.map((row) => [row.view.id, row.depth])).toEqual([
      ["root-a", 0],
      ["child-1", 1],
      ["child-2", 1],
      ["root-b", 0],
    ]);
  });

  it("keeps orphaned views visible at root depth", () => {
    const rows = buildDynamicRows([
      dynamicView("orphan", "Orphan", "missing-parent"),
    ]);

    expect(rows).toHaveLength(1);
    expect(rows[0].view.id).toBe("orphan");
    expect(rows[0].depth).toBe(0);
  });
});
