<script>
    import { onMount } from 'svelte';
    import { fade, slide } from 'svelte/transition';
    import { quintInOut } from 'svelte/easing';
    import * as api from '../lib/api';
    import PatternConsistencyWidget from './PatternConsistencyWidget.svelte';
    import EntropyAnalysisWidget from './EntropyAnalysisWidget.svelte';
    import PairPatternWidget from './PairPatternWidget.svelte';
    import SmartSummaryWidget from './SmartSummaryWidget.svelte';

    export let gameCode;

    let frequencyWindow = 300;
    let frequencyData = null;
    let frequencyLoading = false;
    let frequencyTab = 'Kepala'; // Default tab
    let frequencyError = null;

    // Simple debounce for hardware safety (Core i3)
    function debounce(func, wait) {
        let timeout;
        return function executedFunction(...args) {
            const later = () => {
                clearTimeout(timeout);
                func(...args);
            };
            clearTimeout(timeout);
            timeout = setTimeout(later, wait);
        };
    }

    const loadFrequencyAnalysis = debounce(async () => {
        if (!gameCode) return;
        frequencyLoading = true;
        frequencyError = null;
        try {
            const res = await fetch(`${api.API_BASE_URL}/games/${gameCode}/frequency?window_size=${frequencyWindow}`);
            if (!res.ok) throw new Error("Failed to load frequency analysis");
            frequencyData = await res.json();
        } catch (e) {
            console.error(e);
            frequencyError = e.message;
        } finally {
            frequencyLoading = false;
        }
    }, 150);

    let scrollPositions = {
        'As': 0,
        'Kop': 0,
        'Kepala': 0,
        'Ekor': 0
    };
    let scrollContainer;

    function handleTabChange(newTab) {
        // Save current scroll
        if (scrollContainer) {
            scrollPositions[frequencyTab] = scrollContainer.scrollTop;
        }
        
        frequencyTab = newTab;

        // Restore scroll on next tick
        setTimeout(() => {
            if (scrollContainer) {
                scrollContainer.scrollTo({
                    top: scrollPositions[newTab] || 0,
                    behavior: 'instant'
                });
            }
        }, 30); // Slight delay to let DOM settle
    }

    onMount(() => {
        loadFrequencyAnalysis();
    });

</script>

<div class="card bg-dark-glass border-0 rounded-4 mb-4 overflow-hidden animate-fade-in">
    <div class="card-header bg-transparent border-white-10 d-flex justify-content-between align-items-center">
        <div class="d-flex align-items-center">
            <h6 class="text-white mb-0 fw-bold">Analisa TradeX Pattern</h6>
            <span class="ms-2 badge bg-dark border border-secondary text-muted" style="font-size: 0.65rem;">
                 {frequencyData ? frequencyData.total_periods : 0} Periods
            </span>
        </div>
        <!-- Window Size Selector -->
        <div class="d-flex align-items-center">
             <span class="text-muted small me-2" style="font-size: 0.7rem;">Window:</span>
             <select class="form-select form-select-sm py-0 bg-dark text-white border-secondary" style="font-size: 0.75rem; width: auto;" bind:value={frequencyWindow} on:change={loadFrequencyAnalysis}>
                <option value={100}>100</option>
                <option value={200}>200</option>
                <option value={300}>300 (Default)</option>
                <option value={400}>400</option>
                <option value={500}>500</option>
                <option value={720}>720</option>
                <option value={1000}>1000</option>
             </select>
        </div>
    </div>

    <div class="card-body p-3 position-relative">
         <!-- Seamless Background loading indicator -->
         {#if frequencyLoading && frequencyData}
            <div class="position-absolute top-0 start-0 w-100" style="z-index: 10;">
                <div class="progress bg-transparent rounded-0" style="height: 3px;">
                    <div class="progress-bar progress-bar-striped progress-bar-animated bg-primary" 
                         role="progressbar" style="width: 100%;"></div>
                </div>
            </div>
         {/if}

         {#if frequencyLoading && !frequencyData}
             <div class="text-center py-5">
                 <div class="spinner-border spinner-border-sm text-primary" role="status"></div>
                 <div class="text-muted small mt-2">Initializing Analysis...</div>
             </div>
         {:else if frequencyError}
             <div class="text-center py-4 text-danger small">{frequencyError}</div>
         {:else if frequencyData}
             <div class="transition-all {frequencyLoading ? 'opacity-50 grayscale-dim' : ''}">
                 <!-- Cold Start Protection -->
                 {#if frequencyData.total_periods < 100}
                     <div class="alert alert-info d-flex align-items-center mb-0" role="alert">
                         <i class="bi bi-info-circle-fill me-2 fs-5"></i>
                         <div>
                             <strong>Data belum cukup!</strong><br/>
                             Butuh minimal 100 periode data untuk analisa pola yang valid. 
                             Saat ini baru tersedia {frequencyData.total_periods} periode.
                         </div>
                     </div>
                 {:else}
                     <!-- 1. Smart Summary (Full Width Top) -->
                     <div class="mb-4">
                         <SmartSummaryWidget frequencyData={frequencyData} position={frequencyTab} />
                     </div>

                     <!-- Split Layout -->
                     <div bind:this={scrollContainer} class="analysis-scroll-container" style="max-height: 85vh; overflow-y: auto; overflow-x: hidden;">
                         <div class="row g-4" style="min-height: 500px;"> <!-- Prevent layout collapse -->
                             <!-- LEFT COLUMN: Single Position Analysis -->
                             <div class="col-md-6">
                                 <!-- Tabs -->
                                 <div class="d-flex justify-content-center mb-3">
                                    <div class="btn-group shadow-sm bg-dark rounded-pill p-1">
                                         {#each ['As', 'Kop', 'Kepala', 'Ekor'] as tab}
                                            {@const labelMap = {'As': 'TradeX Ax', 'Kop': 'TradeX Kx', 'Kepala': 'TradeX Kpx', 'Ekor': 'TradeX Ex'}}
                                            <button class="btn btn-sm px-4 rounded-pill transition-all {frequencyTab === tab ? 'btn-primary fw-bold shadow' : 'btn-ghost text-white-50 hover-text-white'}" 
                                                    on:click|preventDefault={() => handleTabChange(tab)}>
                                                {labelMap[tab]}
                                            </button>
                                         {/each}
                                    </div>
                                 </div>

                                 {#key frequencyTab}
                                    <div in:fade={{ duration: 250, delay: 100 }} out:fade={{ duration: 150 }}>
                                        <!-- 2. Pola Kemunculan Angka (Grid 0-9) -->
                                        <div class="mb-4">
                                            <h6 class="text-white mb-2 fw-bold small ms-1 border-start border-3 border-primary ps-2">Pattern Vector Visibility</h6>
                                            <div class="d-flex flex-wrap justify-content-between">
                                                {#each [0,1,2,3,4,5,6,7,8,9] as digit}
                                                    {@const item = frequencyData.results.find(r => r.position === frequencyTab && r.digit === digit)}
                                                    {#if item}
                                                    <div class="mb-2" style="width: 19%;" in:slide={{ duration: 300, delay: digit * 20, easing: quintInOut }}>
                                                        <div class="card h-100 text-center border-0 {item.label === 'UNDERREPRESENTED' ? 'bg-info bg-opacity-25' : item.label === 'OVERREPRESENTED' ? 'bg-warning bg-opacity-25' : 'bg-secondary bg-opacity-10'}">
                                                            <div class="card-body p-2 d-flex flex-column align-items-center justify-content-center">
                                                                <div class="d-flex align-items-center gap-1">
                                                                    <h4 class="mb-0 fw-bold {item.label === 'UNDERREPRESENTED' ? 'text-info' : item.label === 'OVERREPRESENTED' ? 'text-warning' : 'text-white-50'}">{digit}</h4>
                                                                    {#if item.label === 'OVERREPRESENTED'}
                                                                        <i class="bi bi-caret-up-fill" style="color: #32CD32; font-size: 1rem;"></i>
                                                                    {:else if item.label === 'UNDERREPRESENTED'}
                                                                        <i class="bi bi-caret-down-fill" style="color: #DC143C; font-size: 1rem;"></i>
                                                                    {:else}
                                                                        <i class="bi bi-question-circle" style="color: #6c757d; font-size: 0.8rem;"></i>
                                                                    {/if}
                                                                </div>
                                                                <small style="font-size: 0.6rem;" class="text-uppercase mt-1 w-100 text-truncate">
                                                                    {item.label === 'UNDERREPRESENTED' ? 'Jarang' : item.label === 'OVERREPRESENTED' ? 'Sering' : 'Normal'}
                                                                </small>
                                                            </div>
                                                        </div>
                                                    </div>
                                                    {/if}
                                                {/each}
                                            </div>
                                        </div>

                                        <!-- 3. Kemerataan Sebaran (Entropy) -->
                                        {#if frequencyData.entropy}
                                            <div class="mb-3">
                                                <EntropyAnalysisWidget entropyData={frequencyData.entropy} position={frequencyTab} />
                                            </div>
                                        {/if}

                                        <!-- 4. Konsistensi Kemunculan (Consistency) -->
                                        {#if frequencyData.consistency}
                                            <div>
                                                <PatternConsistencyWidget consistencyData={frequencyData.consistency} position={frequencyTab} />
                                            </div>
                                        {/if}
                                    </div>
                                 {/key}
                             </div>

                             <!-- RIGHT COLUMN: Pair Patterns -->
                             <div class="col-md-6">
                                 <!-- 5. Pola Gabungan (Pairs) -->
                                 {#if frequencyData.pairs}
                                    <div class="h-100">
                                        <PairPatternWidget pairData={frequencyData.pairs} />
                                    </div>
                                 {/if}
                             </div>
                         </div>
                     </div>
                 {/if}
             </div>
         {/if}
    </div>
</div>

<style>
  .bg-dark-glass {
    background: rgba(0,0,0,0.4);
    backdrop-filter: blur(5px);
  }
  .border-white-10 {
      border-color: rgba(255, 255, 255, 0.1) !important;
  }
  .animate-fade-in {
      animation: fadeIn 0.5s ease-out;
  }
  @keyframes fadeIn {
      from { opacity: 0; transform: translateY(10px); }
      to { opacity: 1; transform: translateY(0); }
  }
  .text-info {
      color: #0dcaf0 !important;
  }
  .text-warning {
      color: #ffc107 !important;
  }
  .grayscale-dim {
      filter: grayscale(0.2) opacity(0.7);
      pointer-events: none;
  }
  .transition-all {
      transition: all 0.3s ease-in-out;
  }
</style>
