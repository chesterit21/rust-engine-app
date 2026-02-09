<script>
    export let consistencyData = []; // Array of ConsistencyResult
    export let position = 'Kepala'; // 'As', 'Kop', 'Kepala', 'Ekor'

    // Filter data for the current position
    $:  items = consistencyData.filter(i => i.position === position).sort((a,b) => a.digit - b.digit);

    function getStrengthText(label) {
        if (label === 'STRONG') return 'Sering Teramati';
        if (label === 'MEDIUM') return 'Cukup Teramati';
        return 'Jarang Teramati';
    }

    function getSummary(label) {
        if (label === 'STRONG') return 'Pola ini sering teramati konsisten';
        if (label === 'MEDIUM') return 'Pola mulai terlihat konsisten';
        return 'Pola ini jarang teramati konsisten';
    }
</script>

<div class="card bg-dark-glass border-0 rounded-4 mb-4 overflow-hidden animate-fade-in mt-2">
    <div class="card-header bg-transparent border-white-10 text-white">
        <h6 class="mb-0 fw-bold">Konsistensi Kemunculan (Consistency)</h6>
        <small class="text-muted" style="font-size: 0.65rem;">
            Seberapa sering bias ini teramati dalam 3 periode waktu berbeda
        </small>
    </div>
    <div class="card-body p-3">
        {#if items.length === 0}
            <div class="text-center text-muted small py-3">
                Data belum cukup untuk analisa konsistensi (Min 100 periode)
            </div>
        {:else}
            <!-- Consistency Grid -->
             <div class="d-flex flex-wrap justify-content-between">
                 {#each [0,1,2,3,4,5,6,7,8,9] as digit}
                    {@const item = items.find(r => r.digit === digit)}
                    {#if item}
                    <div class="mb-2" style="width: 19%;">
                        <div class="card h-100 text-center border-0 bg-dark bg-opacity-25 relative">
                             <!-- Strength Dots -->
                             <div class="position-absolute top-0 start-50 translate-middle-x mt-1 d-flex gap-1" style="height: 6px;">
                                 <div class="dot rounded-circle {item.strength >= 1 ? 'bg-warning' : 'bg-secondary bg-opacity-25'}" style="width: 4px; height: 4px;"></div>
                                 <div class="dot rounded-circle {item.strength >= 2 ? 'bg-warning' : 'bg-secondary bg-opacity-25'}" style="width: 4px; height: 4px;"></div>
                                 <div class="dot rounded-circle {item.strength >= 3 ? 'bg-warning' : 'bg-secondary bg-opacity-25'}" style="width: 4px; height: 4px;"></div>
                             </div>

                            <div class="card-body p-2 d-flex flex-column align-items-center justify-content-center pt-3">
                                <h4 class="mb-0 fw-bold text-white-50" style="font-size: 1.1rem;">{digit}</h4>
                                <small style="font-size: 0.55rem;" class="text-uppercase mt-1 w-100 text-truncate text-muted">
                                    {getStrengthText(item.label)}
                                </small>
                            </div>
                        </div>
                    </div>
                    {/if}
                {/each}
             </div>
             
             <div class="mt-2 p-2 bg-dark bg-opacity-50 rounded border border-white-10">
                 <small class="text-muted d-block text-center" style="font-size: 0.65rem;">
                     <span class="text-warning">Sering Teramati</span> berarti pola bias (Jarang/Sering) terlihat konsisten di 3 window waktu berbeda.
                     Ini memperkuat indikasi bahwa itu bukan kebetulan sesaat.
                 </small>
            </div>
        {/if}
    </div>
</div>

<style>
  .bg-dark-glass {
    background: rgba(0,0,0,0.3);
  }
  .border-white-10 {
      border-color: rgba(255, 255, 255, 0.1) !important;
  }
  /* .animate-fade-in ... inherited or define if needed */
</style>
