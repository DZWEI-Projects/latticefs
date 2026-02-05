import { useQuery } from "@tanstack/react-query";
import { listViews, type ViewInfo } from "@/lib/lfs";

export function useViews() {
  return useQuery<ViewInfo[], Error>({
    queryKey: ["views"],
    queryFn: listViews,
    staleTime: 30_000, // 30 seconds
  });
}

export function useViewById(viewId: string | undefined) {
  const { data: views, ...rest } = useViews();
  const view = views?.find((v) => v.id === viewId);
  return { data: view, views, ...rest };
}
