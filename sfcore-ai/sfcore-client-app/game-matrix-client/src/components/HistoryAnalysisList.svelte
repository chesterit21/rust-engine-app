<script>
  import { slide } from 'svelte/transition';
  export let historyItems = []; // Array of history items from API
</script>

<div class="history-list">
  {#if historyItems.length === 0}
      <div class="text-center text-muted py-4">
          <i class="bi bi-clock-history fs-1 d-block mb-3 opacity-50"></i>
          <p>No history analysis available yet.</p>
      </div>
  {:else}
      {#each historyItems as item, i (item.periode)}
        <div class="card mb-3 bg-dark-glass border-white-10 shadow-sm animate-fade-in" 
             style="animation-delay: {i * 100}ms;"
             in:slide|local="{{ duration: 300 }}">
          
          <div class="card-header bg-transparent d-flex justify-content-between align-items-center py-2 px-3 border-white-10">
            <span class="fw-bold text-white small text-uppercase ls-1">
                <i class="bi bi-calendar3 me-2 text-primary"></i>Periode {item.periode}
            </span>
            <span class="badge bg-primary bg-gradient shadow-sm font-monospace fs-6 px-3">{item.result}</span>
          </div>
          
          <div class="card-body p-3">
            <!-- Pair Patterns Only -->
            {#if item.pairs && item.pairs.length > 0}
              <div class="d-flex flex-wrap gap-2">
                {#each item.pairs as pair}
                  <div class="d-flex align-items-center bg-black bg-opacity-50 rounded px-3 py-2 border border-white-10">
                      <span class="text-white-50 me-2 small">{pair.positions}</span>
                      <span class="text-warning fw-bold font-monospace me-2">{pair.digits} %</span>
                      {#if pair.status.includes('SERING')}
                          <span class="badge bg-success bg-opacity-25 text-success border border-success border-opacity-25" style="font-size: 0.65rem;">SERING</span>
                      {:else if pair.status.includes('JARANG')}
                          <span class="badge bg-danger bg-opacity-25 text-danger border border-danger border-opacity-25" style="font-size: 0.65rem;">JARANG</span>
                      {:else}
                          <span class="badge bg-secondary bg-opacity-25 text-white-50" style="font-size: 0.65rem;">{pair.status}</span>
                      {/if}
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-center py-2">
                  <span class="text-muted small fst-italic">Tidak ada pola pasangan dominan</span>
              </div>
            {/if}
          </div>
        </div>
      {/each}
  {/if}
</div>

<style>
  .bg-dark-glass {
    background: rgba(20, 20, 20, 0.6);
    backdrop-filter: blur(10px);
  }
  .border-white-10 {
    border-color: rgba(255, 255, 255, 0.08) !important;
  }
  .ls-1 {
      letter-spacing: 1px;
  }
</style>
