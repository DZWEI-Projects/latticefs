import { useQuery } from "@tanstack/react-query";
import { getViewObjects, evaluateQuery, type ObjectInfo } from "@/lib/lfs";

export function useViewObjects(viewId: string | undefined) {
  return useQuery<ObjectInfo[], Error>({
    queryKey: ["view-objects", viewId],
    queryFn: () => getViewObjects(viewId!),
    enabled: !!viewId,
    staleTime: 10_000, // 10 seconds
  });
}

export function useQueryObjects(query: string | undefined) {
  return useQuery<ObjectInfo[], Error>({
    queryKey: ["query-objects", query],
    queryFn: () => evaluateQuery(query!),
    enabled: !!query,
    staleTime: 10_000,
  });
}
