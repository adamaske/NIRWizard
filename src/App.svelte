<script>
  import MenuBar from "./lib/components/menubar.svelte";
  import Statusbar from "./lib/components/statusbar.svelte";
</script>

<div class="app">
  <MenuBar />
  
  <Statusbar />
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: "Segoe UI", system-ui, sans-serif;
    overflow: hidden;
  }

  .app {
    width: 100%;
    height: 100vh;
    background: #0f0f1a;
    color: #d0d0e0;
  }
</style>
