<script>
    import { fade, slide } from 'svelte/transition';
    import { quintInOut } from 'svelte/easing';
    export let pairData = {}; // Object: {"As-Kop": [pairs], "Kop-Kepala": [pairs], ...}
    
    let activeTab = 'As-Kop';
    const tabs = ['As-Kop', 'Kop-Kepala', 'Kepala-Ekor'];
    
    $: currentPairs = pairData ? (pairData[activeTab] || []) : [];
    $: hasData = Object.keys(pairData || {}).length > 0;

    function getLabelColor(label) {
        if (label === 'SANGAT SERING' || label === 'CUKUP SERING') return 'text-warning'; // Orange-ish
        if (label === 'NORMAL') return 'text-white-50'; // Grey
        return 'text-info'; // Blue (Jarang)
    }

    function getCardBg(label) {
        if (label === 'SANGAT SERING' || label === 'CUKUP SERING') return 'bg-warning bg-opacity-10 border-warning border-opacity-25';
        if (label === 'NORMAL') return 'bg-secondary bg-opacity-10 border-secondary border-opacity-25';
        return 'bg-info bg-opacity-10 border-info border-opacity-25';
    }
</script>

<div class="card bg-dark-glass border-0 rounded-4 mb-4 overflow-hidden animate-fade-in mt-3">
    <div class="card-header bg-transparent border-white-10 text-white">
        <h6 class="mb-0 fw-bold">Pola Gabungan Angka (Pair Pattern)</h6>
        <small class="text-muted" style="font-size: 0.65rem;">
            Melihat pasangan angka yang lebih sering atau lebih jarang terlihat bersama
        </small>
    </div>

    <div class="card-body p-3">
        {#if !hasData}
            <div class="text-center text-muted small py-3">
                Data belum cukup untuk analisa pola gabungan (Min 400 periode)
            </div>
        {:else}
            <!-- Tabs -->
            <ul class="nav nav-pills nav-fill mb-3" style="font-size: 0.8rem;">
                {#each tabs as tab}
                    {@const labelMap = {'As-Kop': 'TradeX AKx', 'Kop-Kepala': 'TradeX KKpx', 'Kepala-Ekor': 'TradeX KpEx'}}
                    <li class="nav-item">
                        <a class="nav-link py-1 {activeTab === tab ? 'active bg-primary text-dark fw-bold' : 'text-muted'}" 
                           href={"#"} 
                           on:click|preventDefault={() => activeTab = tab}>
                            {labelMap[tab] || tab}
                        </a>
                    </li>
                {/each}
            </ul>

            <!-- Context Header -->
             <div class="alert alert-info d-flex align-items-center py-2 px-3 mb-3 border-0 bg-opacity-10" style="font-size: 0.75rem;">
                 <i class="bi bi-info-circle me-2"></i>
                 <div>
                     Menampilkan <strong>20 pasangan terpilih</strong> dari total 100 kemungkinan.
                     Analisa berdasarkan conditional probability dari marginal distribution.
                 </div>
             </div>

            <!-- Grid -->
            {#key activeTab}
                <div class="row g-2" in:fade={{ duration: 200, delay: 50 }}>
                    {#each currentPairs as pair, i}
                        <div class="col-6 col-md-3" in:slide={{ duration: 250, delay: i * 15, easing: quintInOut }}>
                            <div class="card h-100 {getCardBg(pair.label)}">
                                <div class="card-body p-2 text-center">
                                    <h4 class="mb-0 fw-bold text-white">{pair.digit_a} – {pair.digit_b}</h4>
                                    <div class="tiny fw-bold mt-1 {getLabelColor(pair.label)}" style="font-size: 0.6rem;">
                                        {pair.label}
                                    </div>
                                    <div class="text-white-50 mt-1" style="font-size: 0.65rem;">
                                        Muncul: <strong>{pair.count}x</strong>
                                    </div>
                                    <div class="text-muted" style="font-size: 0.55rem; opacity: 0.7;">
                                        (Wajar: {pair.expected.toFixed(1)})
                                    </div>
                                </div>
                            </div>
                        </div>
                    {/each}
                </div>
            {/key}

            <!-- Disclaimer -->
            <div class="mt-4 p-3 bg-warning bg-opacity-10 rounded border border-warning border-opacity-25">
                 <div class="d-flex">
                     <i class="bi bi-exclamation-triangle-fill text-warning me-2 fs-6"></i>
                     <div>
                         <strong class="text-warning d-block" style="font-size: 0.7rem;">Observasi Historis</strong>
                         <small class="text-muted d-block" style="font-size: 0.65rem; line-height: 1.3;">
                             "Sering Muncul Bersama" hanya berarti pola tersebut pernah terjadi lebih sering di masa lalu.
                             <strong>Bukan jaminan</strong> akan terus terjadi. Setiap periode tetap independen.
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
  .tiny {
      font-size: 0.6rem;
  }
</style>
