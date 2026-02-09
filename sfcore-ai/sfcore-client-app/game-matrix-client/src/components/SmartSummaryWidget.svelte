<script>
    import PaddleChart from './PaddleChart.svelte';
    /** @type {any} */
    export let frequencyData = null;
    export let position = 'Kepala';

    // Helper to get Entropy Label for current position
    $: entropyItem = frequencyData && frequencyData.entropy 
        ? frequencyData.entropy.find(e => e.position === position) 
        : null;

    // Helper to check consistency for current position
    $: consistencyItems = frequencyData && frequencyData.consistency
        ? frequencyData.consistency.filter(c => c.position === position)
        : [];
    
    $: hasStrongConsistency = consistencyItems.some(c => c.strength >= 2); // Medium or Strong

    // Helper to check pairs for current position (as pair A or B)
    $: pairItems = frequencyData && frequencyData.pairs
         ? getRelevantPairs(frequencyData.pairs, position)
         : [];

    $: hasSignificantPairs = pairItems.some(p => p.deviation >= 0.8 || p.deviation <= -0.8);

    function getRelevantPairs(pairsMap, pos) {
        // Find which pair keys involve this position
        // Keys are "As-Kop", "Kop-Kepala", "Kepala-Ekor"
        let relevant = [];
        if (pairsMap["As-Kop"] && (pos === 'As' || pos === 'Kop')) relevant = relevant.concat(pairsMap["As-Kop"]);
        if (pairsMap["Kop-Kepala"] && (pos === 'Kop' || pos === 'Kepala')) relevant = relevant.concat(pairsMap["Kop-Kepala"]);
        if (pairsMap["Kepala-Ekor"] && (pos === 'Kepala' || pos === 'Ekor')) relevant = relevant.concat(pairsMap["Kepala-Ekor"]);
        return relevant;
    }

    const labelMap = {
        'As': 'TradeX Ax',
        'Kop': 'TradeX Kx',
        'Kepala': 'TradeX Kpx',
        'Ekor': 'TradeX Ex'
    };

</script>

<div class="card bg-dark-glass border-0 rounded-4 mb-4 overflow-hidden animate-fade-in shadow-sm">
    <div class="card-header bg-transparent border-white-10 text-white">
        <h6 class="mb-0 fw-bold">Ringkasan Analisa Posisi {labelMap[position] || position}</h6>
        <small class="text-muted" style="font-size: 0.65rem;">
            Kesimpulan berdasarkan data historis — bukan prediksi
        </small>
    </div>
    <div class="card-body p-3 text-white-90" style="font-size: 0.85rem; line-height: 1.6;">
        <div class="row g-3">
            <!-- Textual Summary -->
            <div class="col-md-7">
                {#if !frequencyData || frequencyData.total_periods < 100}
                    <div class="text-center text-muted small py-4">
                        <div class="spinner-border spinner-border-sm text-primary mb-2" role="status"></div>
                        <div>Menunggu data minimum (100 periode)...</div>
                    </div>
                {:else}
                    <p class="mb-2">
                        Dari data historis, beberapa angka terlihat <strong>lebih sering muncul</strong>,<br> 
                        sementara sisanya <strong>lebih jarang muncul</strong>.
                    </p>

                    <p class="mb-2">
                        {#if !entropyItem}
                            Sebaran data belum lengkap.
                        {:else if entropyItem.normalized_entropy >= 0.92}
                            Sebaran angka di posisi ini <strong>sangat merata</strong>.
                        {:else if entropyItem.normalized_entropy >= 0.80}
                            Sebaran angka <strong>cukup merata</strong> dengan variasi normal.
                        {:else}
                            Sebaran angka <strong>kurang merata</strong> (terlihat dominasi digit tertentu).
                        {/if}
                    </p>

                    <p class="mb-0">
                        {#if hasStrongConsistency}
                            Pola ini <strong>sering teramati</strong> secara berulang (konsisten).
                        {:else}
                            Pola belum menunjukkan konsistensi kuat (fluktuatif).
                        {/if}
                    </p>
                {/if}

        <!-- Disclaimer Footer (Full Width below local row) -->
        {#if frequencyData && frequencyData.total_periods >= 100}
            <div class="mt-3 p-2 bg-warning bg-opacity-10 rounded border border-warning border-opacity-25">
                 <div class="d-flex align-items-center">
                     <i class="bi bi-shield-exclamation text-warning me-2"></i>
                     <small class="text-muted" style="font-size: 0.65rem; line-height: 1.2;">
                         Analisa historis — bukan prediksi. Setiap periode tetap memiliki peluang independen 0-9.
                     </small>
                 </div>
            </div>
        {/if}

    </div>
            
            <!-- Chart Section -->
            <div class="col-md-5">
                <div class="chart-wrapper rounded-3 overflow-hidden bg-black bg-opacity-20 shadow-inner" style="height: 200px;">
                    <PaddleChart height="200px" />
                </div>
            </div>
        </div>

    </div>
</div>

<style>
  .bg-dark-glass {
    background: rgba(0,0,0,0.3);
  }
  .border-white-10 {
      border-color: rgba(255, 255, 255, 0.1) !important;
  }
  .animate-fade-in {
      animation: fadeIn 0.5s ease-out;
  }
  .text-white-90 {
      color: rgba(255, 255, 255, 0.9);
  }
</style>
