import { useQuery } from "@tanstack/react-query";
import { getMountStatus, type MountStatus } from "@/lib/lfs";

export function useMountStatus() {
  return useQuery<MountStatus, Error>({
    queryKey: ["mount-status"],
    queryFn: getMountStatus,
    staleTime: 5_000,
    refetchInterval: 10_000,
  });
}
