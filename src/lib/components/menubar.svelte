<script>
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";

  let activeMenu = null;

  const menus = [
    { name: "File", items: ["Open", "Save", "Exit"] },
    { name: "Export", items: ["Export as .sNIRF"] },
    { name: "Edit", items: ["Undo", "Redo", "Cut", "Copy", "Paste"] },
    {
      name: "Preprocessing",
      items: ["Filter", "Baseline Correction", "Motion Correction"],
    },
    { name: "Analysis", items: ["Run Analysis", "View Results"] },
    {
      name: "Anatomy",
      items: [
        "Open MRI (.nii.gz)",
        "Open Cortex Anatomy (.obj)",
        "Open Scalp Anatomy (.obj)",
      ],
    },
    { name: "View", items: ["Zoom In", "Zoom Out", "Reset Zoom"] },
    { name: "Help", items: ["Documentation", "About"] },
  ];

  function toggleMenu(menu) {
    activeMenu = activeMenu === menu ? null : menu;
  }

  async function handleItemClick(menuLabel, item) {
    activeMenu = null;

    if (menuLabel === "File" && item === "Open") {
      const path = await open({
        multiple: false,
        filters: [{ name: "SNIRF", extensions: ["snirf"] }],
      });
      if (path) {
        try {
          await invoke("load_snirf", { path });
        } catch (err) {
          console.error("Failed to load SNIRF:", err);
          alert(`Failed to load file:\n\n${err}`);
        }
      }
      return;
    }

    if (
      menuLabel === "Anatomy" &&
      (item === "Open Cortex Anatomy (.obj)" ||
        item === "Open Scalp Anatomy (.obj)")
    ) {
      const cmd = item.includes("Cortex") ? "load_cortex_obj" : "load_scalp_obj";
      const path = await open({
        multiple: false,
        filters: [{ name: "OBJ Mesh", extensions: ["obj"] }],
      });
      if (path) {
        try {
          await invoke(cmd, { path });
        } catch (err) {
          console.error("Failed to load OBJ:", err);
          alert(`Failed to load mesh:\n\n${err}`);
        }
      }
      return;
    }

    if (menuLabel == "Export" && item == "Export as .sNIRF") {
      const path = await save({
        filters: [{ name: "SNIRF", extensions: ["snirf"] }],
        defaultPath: "export.snirf",
      });
      if (path) {
        try {
          await invoke("export_snirf", { path });
        } catch (err) {
          console.error("Failed to write SNIRF:", err);
          alert(`Failed to write file:\n\n${err}`);
        }
      }
    }

    console.log(`Clicked: ${menuLabel} -> ${item}`);
  }

  function closeMenus() {
    activeMenu = null;
  }
</script>

<!--    
    Svelte:Window for window-level event,
    if we we press outside of the menu, we want to close any open menu
-->
<svelte:window on:click={closeMenus} />

<nav class="menubar">
  <span class="app-title">NIRWizard</span>

  {#each menus as menu}
    <div class="menu-container">
      <!-- 
                on:click|stopPropagation to capture the clicks
                and prevent it from immediatly closing the menu
            -->

      <button
        class="menu-button"
        class:active={activeMenu === menu.name}
        on:click|stopPropagation={() => toggleMenu(menu.name)}
      >
        {menu.name}
      </button>

      {#if activeMenu === menu.name}
        <div class="dropdown" on:click|stopPropagation>
          {#each menu.items as item}
            <button
              class="dropdown-item"
              on:click={() => handleItemClick(menu.name, item)}
            >
              {item}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</nav>

<style>
  .menubar {
    display: flex;
    align-items: center;
    height: 32px;
    background: var(--bg-raised);
    border-bottom: 1px solid var(--border-default);
    padding: 0 8px;
    gap: 2px;
    user-select: none;
    flex-shrink: 0;
  }
  .app-title {
    font-weight: 600;
    font-size: 13px;
    color: var(--accent-green);
    margin-right: 12px;
    letter-spacing: 0.5px;
  }

  .menu-container {
    position: relative;
  }

  .menu-trigger {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 13px;
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
  }

  .menu-trigger:hover,
  .menu-trigger.active {
    background: var(--bg-overlay);
    color: var(--text-primary);
  }

  .menu-button {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 13px;
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
  }

  .menu-button:hover,
  .menu-button.active {
    background: var(--bg-overlay);
    color: var(--text-primary);
  }

  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    background: var(--bg-raised);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    padding: 4px 0;
    min-width: 200px;
    z-index: 100;
    box-shadow: 0 8px 24px var(--shadow-dropdown);
  }

  .dropdown-item {
    display: block;
    width: 100%;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 13px;
    padding: 6px 16px;
    text-align: left;
    cursor: pointer;
  }

  .dropdown-item:hover {
    background: var(--bg-overlay);
    color: var(--text-primary);
  }
</style>
