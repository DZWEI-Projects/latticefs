import { useQuery } from "@tanstack/react-query";
import { listObjectVersions, type ObjectVersion } from "@/lib/lfs";

export function useObjectVersions(objectId: string | undefined) {
  return useQuery<ObjectVersion[], Error>({
    queryKey: ["objectVersions", objectId],
    queryFn: () => listObjectVersions(objectId!),
    enabled: Boolean(objectId),
  });
}
