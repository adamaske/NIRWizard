// A store is an object that holds reactive state
// components can subscribe to changes
// Writeable lets us read and Writeable
// Derived is computed from other sources

import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import type { SnirfSummary, TimeseriesPayload } from "../types/nirs";

export const summary = writable<SnirfSummary | null>(null);
export const timeseries = writable<TimeseriesPayload | null>(null);
export const selectedChannelIds = writable<number[]>([]);

// Derived state
// Derived is a read-only store which is automatically
// recomputed when depnedices change
// Filter timeseries payload to only selected ids
export const selectedChannelData = derived(
    [timeseries, selectedChannelIds],
    ([$ts, $ids]) => {
        if (!$ts) return [];
        if ($ids.length == 0) return [];
        return $ts.channels.filter((ch) => $ids.includes(ch.id));
    },
);

export const samplingRate = derived(summary, ($s) => $s?.sampling_rate ?? 0);

export const maxTime = derived(timeseries, ($ts) => {
    if (!$ts || $ts.time.length === 0) return 0;
    return $ts.time[$ts.time.length - 1];
});

// Actions
// these are plain exported functions
// invoke<SnirfSummary> lets us know what return type
//

export async function LoadSnirf(path: string): Promise<void> {
    const s = await invoke<SnirfSummary>("load_snirf", { path });
    summary.set(s);

    const ts = await invoke<TimeseriesPayload>("load_timeseries", { path });
    if (ts) {
        timeseries.set(ts);
        // Default select all channels
        selectedChannelIds.set(ts.channels.map((ch) => ch.id));
    }
}

export function SelectChannels(ids: number[]): void {
    selectedChannelIds.set(ids);
    invoke("set_selected_channels", { channelIds: ids });
}

export function SelectAllChannels(): void {
    let ts: TimeseriesPayload | null;

    // Subscribe to timeseries store to get the current value
    // akward syntax but standard practice
    const unsub = timeseries.subscribe((val) => (ts = val));
    unsub();

    if (ts) {
        SelectChannels(ts.channels.map((ch) => ch.id));
    }
}

export function clearChannels(): void {
    SelectChannels([]);
}

// Event listener setup
// Call from App.svelte on mount
export async function InitNirsListeners(): Promise<() => void> {
    const unlistenSnirf = await listen<SnirfSummary>(
        "snirf-loaded",
        (event) => {
            summary.set(event.payload);
        },
    );

    const unlistenChannels = await listen<{ channel_ids: number[] }>(
        "channels-selected",
        (event) => {
            selectedChannelIds.set(event.payload.channel_ids);
        },
    );
    // Centralized listeners across the svelte App
    //
    return () => {
        unlistenSnirf();
        unlistenChannels();
    };
}

//<script>
//  import { selectedChannelData, timeseries, maxTime } from "../stores/nirsStore";
//
//  // $selectedChannelData is reactive — when channels change, this updates
//  // $timeseries.time gives you the time array
//  // $maxTime gives you the last timepoint
//
//  // No more fetchAndCacheData(), no more listen("snirf-loaded"),
//  // no more local allData/selectedIds variables
//</script>
