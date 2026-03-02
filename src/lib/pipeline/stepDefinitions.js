export const STEP_DEFINITIONS = {
   bandpass: {
     id: "bandpass",
     label: "Bandpass Filter",
     description: "Removes slow drift and high-frequency noise using a zero-phase IIR filter.",
     category: "filter",
     params: {
       low_cutoff: {
         type: "number",
         label: "Low cutoff (Hz)",
         default: 0.01,
         min: 0.001,
         max: 1.0,
         step: 0.001,
       },
       high_cutoff: {
         type: "number",
         label: "High cutoff (Hz)",
         default: 0.5,
         min: 0.01,
         max: 5.0,
         step: 0.01,
       },
       order: {
         type: "integer",
         label: "Filter order",
         default: 3,
         min: 1,
         max: 10,
         step: 1,
       },
     },
   },

   pruning: {
     id: "pruning",
     label: "Channel Pruning",
     description: "Removes low-quality channels based on signal quality metrics.",
     category: "quality",
     params: {
       pruning_method: {
         type: "select",
         label: "Method",
         default: "Sci",
         options: [
           { value: "Sci", label: "SCI (Scalp Coupling Index)" },
           { value: "Psp", label: "PSP (Peak Spectral Power)" },
           { value: "Snr", label: "SNR (Signal-to-Noise Ratio)" },
         ],
       },
       threshold: {
         type: "number",
         label: "Threshold",
         default: 0.8,
         min: 0.0,
         max: 1.0,
         step: 0.01,
       },
     },
   },
 };