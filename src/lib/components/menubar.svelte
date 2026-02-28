<script>
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";

    let activeMenu = null;

    const menus = [
        { name: "File", items: ["Open", "Save", "Exit"] },
        { name: "Edit", items: ["Undo", "Redo", "Cut", "Copy", "Paste"] },
        { name: "Preprocessing", items: ["Filter", "Baseline Correction", "Motion Correction"], },
        { name: "Analysis", items: ["Run Analysis", "View Results"] },
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
                await invoke("load_snirf", { path });
            }
            return;
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
        background: #1a1a2e;
        border-bottom: 1px solid #2a2a3e;
        padding: 0 8px;
        gap: 2px;
        user-select: none;
        flex-shrink: 0;
    }
    .app-title {
        font-weight: 600;
        font-size: 13px;
        color: #8888cc;
        margin-right: 12px;
        letter-spacing: 0.5px;
    }

    .menu-container {
        position: relative;
    }

    .menu-trigger {
        background: none;
        border: none;
        color: #b0b0c8;
        font-size: 13px;
        padding: 4px 10px;
        border-radius: 4px;
        cursor: pointer;
    }

    .menu-trigger:hover,
    .menu-trigger.active {
        background: #2a2a4a;
        color: #e0e0f0;
    }

    .menu-button {
        background: none;
        border: none;
        color: #b0b0c8;
        font-size: 13px;
        padding: 4px 10px;
        border-radius: 4px;
        cursor: pointer;
    }

    .menu-button:hover,
    .menu-button.active {
        background: #2a2a4a;
        color: #e0e0f0;
    }

    .dropdown {
        position: absolute;
        top: 100%;
        left: 0;
        background: #1e1e34;
        border: 1px solid #3a3a5e;
        border-radius: 6px;
        padding: 4px 0;
        min-width: 200px;
        z-index: 100;
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    }

    .dropdown-item {
        display: block;
        width: 100%;
        background: none;
        border: none;
        color: #c0c0d8;
        font-size: 13px;
        padding: 6px 16px;
        text-align: left;
        cursor: pointer;
    }

    .dropdown-item:hover {
        background: #2a2a4e;
        color: #ffffff;
    }
</style>
