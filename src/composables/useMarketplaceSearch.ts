import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { RemoteSkill, MarketStatus } from "./types";
import { CACHE_TTL_MS } from "./constants";

export interface SearchCacheEntry {
  timestamp: number;
  data: {
    skills: RemoteSkill[];
    total: number;
    limit: number;
    offset: number;
    marketStatuses: MarketStatus[];
  };
}

export function useMarketplaceSearch(
  marketConfigs: { value: Record<string, string> },
  enabledMarkets: { value: Record<string, boolean> },
  marketStatuses: { value: MarketStatus[] }
) {
  const query = ref("");
  const results = ref<RemoteSkill[]>([]);
  const total = ref(0);
  const limit = ref(20);
  const offset = ref(0);
  const loading = ref(false);

  const searchCache = new Map<string, SearchCacheEntry>();

  const hasMore = computed(() => results.value.length < total.value);

  function dedupeSkills(skills: RemoteSkill[]): RemoteSkill[] {
    const map = new Map<string, RemoteSkill>();
    for (const skill of skills) {
      const sourceKey = skill.sourceUrl?.trim().toLowerCase();
      const nameKey = `${skill.marketId}:${skill.name.trim().toLowerCase()}`;
      const key = sourceKey || nameKey;
      if (!map.has(key)) {
        map.set(key, skill);
      }
    }
    return Array.from(map.values());
  }

  async function searchMarketplace(reset = true, force = false): Promise<void> {
    if (loading.value) return;
    loading.value = true;

    const nextOffset = reset ? 0 : offset.value + limit.value;
    const cacheKey = `${query.value.trim().toLowerCase()}|${limit.value}`;

    if (reset && !force) {
      const cached = searchCache.get(cacheKey);
      if (cached && Date.now() - cached.timestamp < CACHE_TTL_MS) {
        results.value = cached.data.skills;
        total.value = cached.data.total;
        offset.value = cached.data.offset;
        marketStatuses.value = cached.data.marketStatuses;
        loading.value = false;
        return;
      }
    }

    try {
      const response = await invoke("search_marketplaces", {
        query: query.value,
        limit: limit.value,
        offset: nextOffset,
        apiKeys: marketConfigs.value,
        enabledMarkets: enabledMarkets.value
      });
      const data = response as {
        skills: RemoteSkill[];
        total: number;
        limit: number;
        offset: number;
        marketStatuses: MarketStatus[];
      };

      const deduped = dedupeSkills(reset ? data.skills : [...results.value, ...data.skills]);
      results.value = deduped;
      total.value = data.total;
      offset.value = data.offset;
      if (Array.isArray(data.marketStatuses)) {
        marketStatuses.value = data.marketStatuses;
      }

      if (reset) {
        const cachedStatuses = Array.isArray(data.marketStatuses)
          ? data.marketStatuses
          : marketStatuses.value;
        searchCache.set(cacheKey, {
          timestamp: Date.now(),
          data: { ...data, marketStatuses: cachedStatuses }
        });
      }
    } finally {
      loading.value = false;
    }
  }

  return {
    query,
    results,
    total,
    limit,
    offset,
    loading,
    hasMore,
    searchMarketplace,
    dedupeSkills
  };
}
