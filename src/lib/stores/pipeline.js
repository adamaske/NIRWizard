import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { STEP_DEFINITIONS } from "../pipeline/stepDefinitions.js";

function createPipelineStore() {
  const { subscribe, update, set } = writable({ steps: [], selectedId: null });

  function buildStepKind(definitionId, params) {
    const tag = definitionId.charAt(0).toUpperCase() + definitionId.slice(1);
    return { [tag]: params };
  }

  return {
    subscribe,
    set,

    addStep(definitionId) {
      const def = STEP_DEFINITIONS[definitionId];
      if (!def) return;
      const params = Object.fromEntries(
        Object.entries(def.params).map(([k, v]) => [k, v.default]),
      );
      const instanceId = crypto.randomUUID();
      update((state) => ({
        steps: [
          ...state.steps,
          { instanceId, definitionId, label: def.label, enabled: true, params },
        ],
        selectedId: instanceId,
      }));

      invoke("add_pipeline_step", { kind: buildStepKind(definitionId, params) })
        .catch((e) => console.error("add_pipeline_step failed:", e));
      
    },

    removeStep(instanceId) {
      let index;
      update((state) => {
        index = state.steps.findIndex((s) => s.instanceId === instanceId);
        return {
          steps: state.steps.filter((s) => s.instanceId !== instanceId),
          selectedId:
            state.selectedId === instanceId ? null : state.selectedId,
        };
      });
      if (index >= 0) {
        invoke("remove_pipeline_step", { index })
          .catch((e) => console.error("remove_pipeline_step failed:", e));
      }
    },

    moveStep(instanceId, dir) {
      let index;
      update((state) => {
        const idx = state.steps.findIndex((s) => s.instanceId === instanceId);
        const newIdx = idx + dir;
        if (idx < 0 || newIdx < 0 || newIdx >= state.steps.length) return state;
        index = idx;
        const steps = [...state.steps];
        [steps[idx], steps[newIdx]] = [steps[newIdx], steps[idx]];
        return { ...state, steps };
      });
      if (index !== undefined) {
        invoke("move_pipeline_step", { index, direction: dir })
          .catch((e) => console.error("move_pipeline_step failed:", e));
      }
    },

    updateParam(instanceId, key, value) {
      update((state) => ({
        ...state,
        steps: state.steps.map((s) =>
          s.instanceId === instanceId
            ? { ...s, params: { ...s.params, [key]: value } }
            : s,
        ),
      }));
    },

    selectStep(instanceId) {
      update((state) => ({ ...state, selectedId: instanceId }));
    },

    toggleEnabled(instanceId) {
      update((state) => ({
        ...state,
        steps: state.steps.map((s) =>
          s.instanceId === instanceId ? { ...s, enabled: !s.enabled } : s,
        ),
      }));
    },

    serialize(state) {
      return JSON.stringify({
        version: "1.0",
        created: new Date().toISOString(),
        steps: state.steps,
      });
    },

    deserialize(json) {
      const data = JSON.parse(json);
      if (data.version !== "1.0") throw new Error("Unsupported pipeline version");
      return {
        steps: data.steps.map((s) => ({ ...s, instanceId: crypto.randomUUID() })),
        selectedId: null,
      };
    },
  };
}

export const pipeline = createPipelineStore();

export const selectedStep = derived(pipeline, ($p) => {
  if (!$p.selectedId) return null;
  const step = $p.steps.find((s) => s.instanceId === $p.selectedId);
  if (!step) return null;
  return { step, def: STEP_DEFINITIONS[step.definitionId] };
});
