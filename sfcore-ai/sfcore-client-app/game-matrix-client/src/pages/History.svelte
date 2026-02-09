<script>
    import { onMount } from 'svelte';
    import { fade, slide } from 'svelte/transition';
    import { quintInOut } from 'svelte/easing';
    import { fetchHistorySummary, deleteHistoryByGameCode, resetAllHistory, fetchMissingNumbers } from '../lib/api';
    import Loader from '../components/Loader.svelte';

    let historyData = [];
    let loading = true;
    let error = null;
    let deletingCode = null;
    let resetting = false;
    let missingNumbersText = '';
    let fetchingMissing = false;

    async function handleGetMissing(transCode) {
        if (fetchingMissing) return;
        fetchingMissing = true;
        missingNumbersText = "Loading missing numbers...";
        try {
            const data = await fetchMissingNumbers(transCode);
            missingNumbersText = data.missing || "No missing numbers found.";
        } catch (e) {
            console.error("Failed missing numbers", e);
            missingNumbersText = "Error loading data.";
        } finally {
            fetchingMissing = false;
        }
    }

    async function loadHistory() {
        loading = true;
        try {
            historyData = await fetchHistorySummary();
        } catch (e) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    async function handleDelete(gameCode) {
        if (!confirm(`Are you sure you want to delete all history for ${gameCode}?\nThis will clear data from Queue, History, and DataPlaying tables.`)) {
            return;
        }

        deletingCode = gameCode;
        try {
            await deleteHistoryByGameCode(gameCode);
            historyData = historyData.filter(h => h.GameCode !== gameCode);
        } catch (e) {
            alert('Failed to delete: ' + e.message);
        } finally {
            deletingCode = null;
        }
    }

    async function handleResetAll() {
        if (resetting || loading) return;
        resetting = true;
        try {
            await resetAllHistory();
            // Reload list immediately
            await loadHistory();
        } catch (e) {
            console.error("Reset failed", e);
            // No alert as requested
        } finally {
             resetting = false;
        }
    }

    function formatCurrency(value) {
        if (!value) return 'IDR 0';
        // Remove "IDR", trim, parse int
        const number = parseInt(value.replace(/[^0-9-]/g, '')); 
        if (isNaN(number)) return value;
        return 'IDR ' + number.toLocaleString('id-ID');
    }

    onMount(loadHistory);
</script>

<div class="container-fluid py-4">
    <div class="d-flex justify-content-between align-items-center mb-4">
        <div>
            <h4 class="mb-0 fw-bold text-white">Transaction History</h4>
            <p class="text-white-50 small mb-0">Summary of all playing sessions and collections</p>
        </div>
        <div class="d-flex gap-2">
            <button class="btn btn-danger btn-sm rounded-pill px-3" on:click={handleResetAll} disabled={loading || resetting}>
                {#if resetting}
                    <span class="spinner-border spinner-border-sm me-1" role="status" aria-hidden="true"></span>
                {:else}
                    <i class="bi bi-trash-fill me-1"></i> 
                {/if}
                Reset All
            </button>
            <button class="btn btn-outline-primary btn-sm rounded-pill px-3" on:click={loadHistory} disabled={loading || resetting}>
                <i class="bi bi-arrow-clockwise me-1"></i> Refresh
            </button>
        </div>
    </div>

    <div class="row overflow-y-auto" style="max-height: 400px;">
    {#if loading && !historyData.length}
        <div class="d-flex justify-content-center py-5">
            <Loader text="FETCHING HISTORY..." />
        </div>
    {:else if error}
        <div class="alert alert-danger border-0 shadow-sm" role="alert">
            <i class="bi bi-exclamation-triangle-fill me-2"></i> {error}
        </div>
    {:else if historyData.length === 0}
        <div class="text-center py-5 text-white-50">
            <i class="bi bi-journal-x display-1 mb-3 d-block opacity-25"></i>
            <h5>No history records found</h5>
            <p class="small">Start playing to see your transaction summary here.</p>
        </div>
    {:else}
        <div class="row g-3" style="max-height: 400px;">
            {#each historyData as item, i}
                <div class="col-12 col-md-3 col-lg-2" in:slide={{ duration: 300, delay: i * 50, easing: quintInOut }}>
                    <div class="card h-100 bg-dark border-secondary bg-opacity-50 hover-shadow transition-all position-relative overflow-hidden">
                        <!-- Glass Effect Background -->
                        <div class="position-absolute top-0 start-0 w-100 h-100 bg-primary opacity-5" style="z-index: 0;"></div>
                        
                        <div class="card-body p-3 position-relative" style="z-index: 1;">
                            <div class="d-flex justify-content-between align-items-start mb-3">
                                <div>
                                    <!-- <h5 class="mb-0 fw-bold text-dark uppercase">{item.GameCode}</h5> -->
                                    <a 
                                    href="#/analysis/{item.GameCode}"
                                    class="fw-bold text-truncate text-dark tracking-wide game-code-link d-block text-decoration-none"
                                    title={item.GameCode}>
                                    {item.GameCode}
                                    </a>
                                </div>
                                <button 
                                    class="btn btn-link text-danger p-0 opacity-50 hover-opacity-100 transition-all" 
                                    on:click={() => handleDelete(item.GameCode)}
                                    disabled={deletingCode === item.GameCode}
                                    title="Delete History"
                                    aria-label="Delete History"
                                >
                                    {#if deletingCode === item.GameCode}
                                        <span class="spinner-border spinner-border-sm" role="status"></span>
                                    {:else}
                                        <i class="bi bi-trash3-fill"></i>
                                    {/if}
                                </button>
                            </div>

                            <div class="bg-black bg-opacity-25 rounded p-2 mb-2">
                                <div class="small text-white-50 mb-1" style="font-size: 0.65rem;">TRANSACTION CODE</div>
                                <div class="font-monospace text-info small text-truncate" title={item.TransCode}>
                                    {item.TransCode}
                                </div>
                            </div>

                            <div class="row g-2">
                                <div class="col-6">
                                    <div class="p-2 rounded bg-dark border border-white-10 text-center h-100 cursor-pointer hover-bright" 
                                         on:click={() => handleGetMissing(item.TransCode)} role="button" tabindex="0" on:keydown={(e) => e.key === 'Enter' && handleGetMissing(item.TransCode)}>
                                        <div class="tiny text-white-50 mb-1 text-uppercase" style="font-size: 0.55rem;">Buys</div>
                                        <div class="fw-bold text-warning" style="font-size: 0.85rem;">{formatCurrency(item.BUYS)}</div>
                                    </div>
                                </div>
                                <div class="col-6">
                                    <div class="p-2 rounded bg-dark border border-white-10 text-center h-100 text-truncate cursor-pointer hover-bright"
                                         on:click={() => handleGetMissing(item.TransCode)} role="button" tabindex="0" on:keydown={(e) => e.key === 'Enter' && handleGetMissing(item.TransCode)}>
                                        <div class="tiny text-white-50 mb-1 text-uppercase " style="font-size: 0.55rem;">Collect</div>
                                        <div class="fw-bold text-success" style="font-size: 0.85rem;">{formatCurrency(item.TOTAL_COLLECT)}</div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
    </div>
    <div class="row">
        <div class="col-md-12" style="max-height: 155px; overflow-y: auto; padding-top:25px">
            <textarea class="form-control bg-transparent text-dark border-white-10" style="font-size: 0.4rem;" rows="4" readonly bind:value={missingNumbersText} placeholder="Click 'Buys' or 'Collect' to see missing numbers..."></textarea>
        </div>
    </div>


</div>

<style>
    .hover-shadow:hover {
        transform: translateY(-3px);
        box-shadow: 0 10px 20px rgba(0,0,0,0.4) !important;
        border-color: var(--bs-primary) !important;
    }
    .transition-all {
        transition: all 0.25s ease-in-out;
    }
    .hover-opacity-100:hover {
        opacity: 1 !important;
    }
    .border-white-10 {
        border-color: rgba(255,255,255,0.1) !important;
    }
    .cursor-pointer {
        cursor: pointer;
    }
    .hover-bright:hover {
        background-color: rgba(255, 255, 255, 0.1) !important;
        border-color: rgba(255, 255, 255, 0.3) !important;
    }
</style>
