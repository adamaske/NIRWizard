// Mirror summary.rs
// An interface is a struct without methods,
// export is public
export interface EventSummary {
    name: string;
    marker_count: number;
}

export interface SnirfSummary {
    filename: string;
    format_version: string;
    data_kind: "raw_cw" | "processed_hemoglobin" | "empty";
    channels: number;
    sources: number;
    detectors: number;
    timepoints: number;
    sampling_rate: number;
    duration: number;
    wavelengths: number[];
    events: EventSummary[];
    aux_count: number;
}

export interface ChannelPayload {
    id: number;
    name: string;
    series_a: number[];
    series_b: number[];
    series_a_label: string;
    series_b_label: string;
}

export interface EventMakrerPayload {
    onset: number;
    duration: number;
    value: number;
}

export interface EventPayload {
    name: string;
    markers: EventMakrerPayload[];
}
export interface TimeseriesPayload {
    time: number[];
    data_kind: string;
    channels: ChannelPayload[];
    events: EventPayload[];
}
