<script>
  import { createEventDispatcher } from 'svelte';
  import { onMount } from 'svelte';
  import { notifications, addNotification, removeNotification } from '../stores/notifications';
  import CircularProgress from './CircularProgress.svelte';
  
  export let gameCode; // Prop passed from parent

  const dispatch = createEventDispatcher();

  // State for visibility
  let isVisible = false;
  let isMinimized = false;

  function toggleVisibility() {
      isVisible = !isVisible;
  }
  
  function toggleMinimize() {
      isMinimized = !isMinimized;
  }

  // Composition logic
  let selectedCompLabel = "Select Type";
  let selectedCompValue = "";
  
  let compButtons = [
      { label: 'Buy Comp (A+K)', class: 'btn-outline-info', value: '2DF' },
      { label: 'Buy Comp (K+Kp)', class: 'btn-outline-info', value: '2DM' },
      { label: 'Buy Comp (Kp+E)', class: 'btn-outline-info', value: '2DB' },
      { label: 'Sell Comp (A+Kp)', class: 'btn-outline-warning', value: '2DAKp' },
      { label: 'Sell Comp (K+E)', class: 'btn-outline-warning', value: '2DKE' },
      { label: 'Sell Comp (A+E)', class: 'btn-outline-warning', value: '2DAE' }
  ];

  function selectCompType(btn) {
      selectedCompLabel = btn.label;
      selectedCompValue = btn.value;
      addNotification('success', `Selected: ${btn.label}`);
  }

  // Number Grid Generation
  let numberRows = [];
  onMount(() => {
      let currentInfo = [];
      for (let i = 0; i < 100; i++) {
          let num = i.toString().padStart(2, '0');
          currentInfo.push(num);
          if (currentInfo.length === 10) {
              numberRows.push(currentInfo);
              currentInfo = [];
          }
      }
      numberRows = numberRows; // triggering reactivity
  });

  function handleNumberClick(num) {
       if (!selectedCompValue) {
           addNotification('error', 'Please select a composition type first.');
           return;
       }
       // Dispatch event to parent
       dispatch('savePattern', {
           type: selectedCompValue,
           digit: num
       });
       // Optional: Notify UI immediately or wait for parent success? 
       // Start with immediate feedback or rely on global notification from parent (LogAnalisa will call addNotification on success/fail).
       // However, we can show a "Processing" or just let the parent handle it. 
       // User asked why it doesn't save. 
       
       // I will rely on parent's notification to avoid double notification or "success" when it actually fails.
       // But I'll leave a debug log if needed.
  }
</script>

<!-- Notifications removed, handled by Layout -->

<!-- Trigger Button -->
{#if !isVisible}
    <button 
      class="btn btn-dark position-fixed end-0 top-50 translate-middle-y rounded-start-4 p-3 shadow-lg border border-end-0 border-white-10 z-3 floating-trigger"
      on:click={toggleVisibility}
      style="z-index: 9999;"
      title="Open Control Panel"
    >
      <i class="bi bi-gear-fill fs-4 text-white"></i>
    </button>
{/if}

{#if isVisible}
    <div class="floating-panel card bg-dark border-start border-white-10 shadow-lg" 
       class:minimized={isMinimized}>
       
      <div class="card-header bg-transparent border-white-10 py-2 px-3 d-flex justify-content-between align-items-center">
          <small class="text-white fw-bold"><i class="bi bi-grid-3x3 me-1"></i>Control Panel</small>
          <div class="d-flex gap-1">
              <button class="btn btn-sm btn-link text-white-50 p-0" on:click={toggleMinimize}>
                  <i class="bi {isMinimized ? 'bi-plus-lg' : 'bi-dash-lg'}"></i>
              </button>
              <button class="btn btn-sm btn-link text-white-50 p-0" on:click={toggleVisibility}>
                  <i class="bi bi-x-lg"></i>
              </button>
          </div>
      </div>
    
      {#if !isMinimized}
          <div class="card-body p-2 h-100 overflow-hidden">
              <div class="row g-2 h-100">
                  <!-- Helper Buttons (Left Side: col-md-4) -->
                  <div class="col-md-3 border-end border-white-10 h-100 overflow-y-auto custom-scrollbar">
                    <div class="d-flex flex-column gap-2 mb-2">
                       <small class="text-muted text-uppercase font-monospace" style="font-size: 0.65rem;">Composition</small>
                    </div>
                    <div class="d-flex flex-column gap-2">
                        {#each compButtons as btn}
                            <button class="btn btn-sm fw-bold font-monospace {btn.class} text-start ps-3 py-2 text-truncate" 
                                    style="font-size: 0.65rem;" 
                                    title={btn.label}
                                    on:click={() => selectCompType(btn)}>
                               <i class:bi-graph-up-arrow={btn.label.includes('Buy')} class:bi-graph-down-arrow={btn.label.includes('Sell')} class="me-2"></i>
                               {btn.label}
                            </button>
                        {/each}
                    </div>
                    <div class="d-flex flex-column gap-2" style="padding-top: 10px;">
                      <label class="text-white" style="font-size: 0.85rem;">{selectedCompLabel}</label>
                      <input type="hidden" id="HidTypeComposition" value={selectedCompValue}>
                    </div>
                  </div>
    
                  <!-- Number Grid (Right Side: col-md-8)-->
                  <div class="col-md-9 h-100 overflow-y-auto custom-scrollbar">
                      <div class="mb-3">
                         <small class="text-muted text-uppercase d-block mb-2 font-monospace" style="font-size: 0.65rem;">Matrix Input (00-99)</small>
                         <div class="number-grid">
                            {#each numberRows as row}
                              <div class="d-flex w-100 gap-2 justify-content-between mb-2">
                                 {#each row as num}
                                    <button class="circular-btn p-0 border-0 bg-transparent position-relative" 
                                            title="Select {num} %"
                                            on:click={() => handleNumberClick(num)}>
                                        <CircularProgress 
                                          value={parseInt(num)} 
                                          displayText={num}
                                          maxValue={99}
                                          size={35}
                                          strokeWidth={3}
                                          compact={true}
                                        />
                                    </button>
                                 {/each}
                              </div>
                            {/each}
                         </div>
                      </div>
                  </div>
              </div>
          </div>
      {/if}
    </div>
{/if}

<style>
.floating-panel {
    position: fixed;
    top: 55px; /* Below navbar */
    right: 0;
    width: 52%;
    height: 100vh;
    z-index: 9990;
    user-select: none;
    backdrop-filter: blur(10px);
    background: rgba(0, 0, 0, 0.95) !important;
    border-radius: 0;
    transition: width 0.3s ease;
}
.floating-panel.minimized {
    width: 380px;
    height: auto;
    border-bottom-left-radius: 8px;
}

.cursor-move {
    cursor: move;
}
.number-grid {
    max-height: none;
    overflow-y: visible;
}
.circular-btn {
    cursor: pointer;
    transition: transform 0.2s ease, filter 0.2s ease;
}
.circular-btn:hover {
    transform: scale(1.1);
    filter: brightness(1.2);
}

  .border-white-10 {
    border-color: rgba(255,255,255,0.1) !important;
  }
  .bg-dark-glass {
      background: rgba(0,0,0,0.5);
  }
  .ls-1 {
      letter-spacing: 1px;
  }
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: rgba(0,0,0,0.2);
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255,255,255,0.1);
    border-radius: 3px;
  }
  .d-grid {
      display: grid;
      grid-template-columns: 1fr 1fr; /* 2 columns for buttons looks better */
  }
  .fa-spin {
      animation: spin 4s linear infinite;
  }
  @keyframes spin { 
      100% { transform: rotate(360deg); } 
  }
</style>
