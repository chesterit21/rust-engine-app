<script>
    export let entropyData = []; // Array of EntropyResult
    export let position = 'Kepala'; 

    $: item = entropyData.find(i => i.position === position);

    function getStatusColor(label) {
        if (label === 'SANGAT MERATA') return 'text-info'; // Blue
        if (label === 'CUKUP MERATA') return 'text-light'; // White/Neutral
        return 'text-warning'; // Yellow (Kurang Merata)
    }

    function getBarHeight(count, allCounts) {
        const max = Math.max(...allCounts);
        if (max === 0) return 0;
        return (count / max) * 100; 
    }
</script>

<div class="card bg-dark-glass border-0 rounded-4 mb-4 overflow-hidden animate-fade-in mt-3">
    <div class="card-header bg-transparent border-white-10 text-white">
        <h6 class="mb-0 fw-bold">Kemerataan Sebaran Angka</h6>
        <small class="text-muted" style="font-size: 0.65rem;">
            Melihat apakah angka tersebar merata atau cenderung berulang
        </small>
    </div>
    <div class="card-body p-3">
        {#if !item}
            <div class="text-center text-muted small py-3">
                Data belum cukup untuk analisa sebaran (Min 150 periode)
            </div>
        {:else}
            <!-- Large Status Display -->
            <div class="text-center mb-4">
                <h5 class="fw-bold mb-1 {getStatusColor(item.label)}" style="letter-spacing: 1px;">
                    {#if item.label === 'KURANG MERATA'}
                        🟡 KURANG MERATA
                    {:else if item.label === 'CUKUP MERATA'}
                        ⚪ CUKUP MERATA
                    {:else}
                        🔵 SANGAT MERATA
                    {/if}
                </h5>
                <small class="text-white-50" style="font-size: 0.75rem;">
                    {#if item.label === 'KURANG MERATA'}
                        Angka di posisi ini <strong>cenderung berulang di beberapa digit</strong>.
                    {:else}
                        Angka di posisi ini <strong>muncul relatif merata</strong>.
                    {/if}
                </small>
            </div>

            <!-- Simple Bar Chart -->
            <div class="d-flex align-items-end justify-content-between px-2" style="height: 60px;">
                {#each item.distribution as count, i}
                    <div class="d-flex flex-column align-items-center" style="width: 8%;">
                        <div class="w-100 bg-secondary bg-opacity-50 rounded-top" 
                             style="height: {getBarHeight(count, item.distribution)}%; min-height: 2px;">
                        </div>
                        <small class="text-muted mt-1" style="font-size: 0.6rem;">{i}</small>
                    </div>
                {/each}
            </div>

            <!-- Disclaimer Box -->
            <div class="mt-4 p-3 bg-warning bg-opacity-10 rounded border border-warning border-opacity-25">
                 <div class="d-flex">
                     <i class="bi bi-exclamation-triangle-fill text-warning me-2 fs-6"></i>
                     <div>
                         <strong class="text-warning d-block" style="font-size: 0.7rem;">Catatan Penting</strong>
                         <small class="text-muted d-block" style="font-size: 0.65rem; line-height: 1.3;">
                             Angka yang terlihat "kurang merata" <strong>tidak berarti</strong> akan berubah di periode berikutnya.
                             Ini hanya deskripsi pola dari data historis — bukan prediksi.
                             Setiap periode tetap memiliki peluang yang sama.
                         </small>
                     </div>
                 </div>
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
  .animate-fade-in {
      animation: fadeIn 0.5s ease-out;
  }
</style>
