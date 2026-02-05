import { useQuery } from "@tanstack/react-query";
import { getViewObjects, evaluateQuery, type ObjectInfo } from "@/lib/lfs";
import { toast } from "sonner";

export function useViewObjects(viewName: string | undefined) {
  return useQuery<ObjectInfo[], Error>({
    queryKey: ["view-objects", viewName],
    queryFn: () => getViewObjects(viewName!),
    enabled: !!viewName,
    staleTime: 10_000, // 10 seconds
  });
}

export function useQueryObjects(query: string | undefined) {
  if(!query) return { data: [], isLoading: false, error: new Error("Die Abfrage ist nicht gültig") };
  return useQuery<ObjectInfo[], Error>({
    queryKey: ["query-objects", query],
    queryFn: () => evaluateQuery(query!),
    enabled: !!query,
    staleTime: 10_000,
  });
}
