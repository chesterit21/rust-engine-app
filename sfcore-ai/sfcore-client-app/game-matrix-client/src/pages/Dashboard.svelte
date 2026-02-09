<script>
  import Navbar from '../components/Navbar.svelte';
  import ThemeToggle from '../components/ThemeToggle.svelte';
  import PaddleChart from '../components/PaddleChart.svelte';
  import CircularProgress from '../components/CircularProgress.svelte';
  import { onMount, createEventDispatcher } from 'svelte';
  
  export let games = [];
  
  const dispatch = createEventDispatcher();

  function handleLogout() {
    dispatch('logout');
  }

  // Parse last_result format "#.###" to number (e.g., "6.432" -> 6.432)
  function parseResultValue(lastResult) {
    if (!lastResult) return 0;
    // Remove any non-numeric characters except dots
    const cleaned = lastResult.replace(/[^0-9.]/g, '');
    const value = parseFloat(cleaned);
    return isNaN(value) ? 0 : value;
  }

</script>

<div class="min-vh-100 d-flex flex-column position-relative overflow-hidden">
  <div class="container-fluid px-4 pt-4 pb-0 z-2 position-relative">
       <div class="card bg-dark-glass border-0 rounded-4 p-3 mb-2 shadow-lg">
            <h6 class="text-white-50 text-uppercase small ls-1 mb-2 text-center">Live Matrix Trade</h6>
            <PaddleChart height="400px" />
       </div>
  </div>

  <main class="container-fluid px-4 py-4 flex-grow-1 z-2 position-relative">
    <div class="d-flex align-items-center justify-content-between mb-4">
      <h4 class="mb-0 fw-bold text-white"><i class="bi bi-grid-fill me-2 text-primary"></i>Live TradeX</h4>
      <div class="input-group w-auto">
        <!-- <span class="input-group-text bg-dark-glass border-0 ps-3 text-muted"><i class="bi bi-search"></i></span>
        <input type="text" class="form-control border-0 focus-ring-none bg-dark-glass text-white" placeholder="Search games..."> -->
      </div>
    </div>

    <div class="row g-3 g-xl-4">
      {#each games as game}
        <!-- Grid col-md-3 as requested by user -->
        <div class="col-12 col-sm-3 col-md-2">
          <div class="card h-100 shadow-hover border-0 rounded-4 overflow-hidden position-relative glass-card">
            
            <div class="card-body p-4 d-flex flex-column">
              <div class="d-flex justify-content-between align-items-start mb-3">
                <div class="d-flex flex-column">
                  <!-- Display GameCode only as requested -->
                  <h5 class="mb-1">
                    <a 
                      href="#/analysis/{game.game_code}"
                      class="fw-bold text-truncate text-white tracking-wide game-code-link d-block text-decoration-none"
                      title={game.game_code}>
                      {game.game_code}
                    </a>
                  </h5>
                </div>
                <div class="text-end">
                  <!-- Updated text to show "Periode" -->
                  <small class="d-block text-muted small fw-semibold text-uppercase" style="font-size: 0.7rem;">Periode</small>
                  <small class="fw-bold font-monospace text-white">{game.periode}</small>
                </div>
              </div>

              <!-- Circular Progress Section -->
              <div class="bg-dark-glass rounded-3 p-3 text-center mb-3 mt-auto border border-white-10">
                  <small class="text-muted d-block mb-2 text-uppercase" style="font-size: 0.55rem; letter-spacing: 1px;">{game.date_result}</small>
                  <div class="d-flex justify-content-center">
                    <CircularProgress 
                      value={parseResultValue(game.last_result)} 
                      displayText={game.last_result || '----'}
                      maxValue={10}
                      size={90}
                      strokeWidth={7}
                      trend={game.trend}
                    />
                  </div>
              </div>
              <small class="text-muted d-block mb-1 text-uppercase " style="font-size: 0.55rem; letter-spacing: 1px;">time {game.game_hour}:{game.game_minute}</small>
              <small class="text-muted d-block mb-1 text-uppercase " style="font-size: 0.55rem; letter-spacing: 1px;">{game.input_result_date}</small>
              
            </div>
          </div>
        </div>
      {/each}
    </div>
  </main>
  
  <footer class="bg-dark-glass py-4 mt-auto border-top border-white-10 z-2 position-relative">
      <div class="container text-center text-muted small">
          &copy; 2026 TradeX Matrix. All rights reserved.
      </div>
  </footer>
</div>

<style>
  /* Reuse Matrix Backgrounds */
  .matrix-bg {
    position: fixed;
    inset: 0;
    background-color: var(--matrix-bg);
    background-image: 
      linear-gradient(rgba(0, 255, 65, 0.03) 1px, transparent 1px),
      linear-gradient(90deg, rgba(0, 255, 65, 0.03) 1px, transparent 1px);
    background-size: 30px 30px;
    mask-image: radial-gradient(circle at center, black 40%, transparent 100%);
    pointer-events: none;
    z-index: 0;
  }
  .glow-orb {
    position: fixed;
    width: 400px;
    height: 400px;
    border-radius: 50%;
    background: var(--matrix-green);
    filter: blur(180px);
    opacity: 0.1;
    pointer-events: none;
    z-index: 0;
  }
  .top-start { top: -150px; left: -150px; }
  .bottom-end { bottom: -150px; right: -150px; }

  /* Glass Card Styles */
  .glass-card {
    background: var(--glass-bg);
    backdrop-filter: blur(10px);
    border: 1px solid var(--glass-border) !important;
    transition: all 0.3s ease;
  }
  .glass-card:hover {
    transform: translateY(-5px);
    box-shadow: 0 10px 30px rgba(0, 255, 65, 0.1) !important;
    border-color: rgba(0, 255, 65, 0.4) !important;
  }
  
  /* Utilities */
  .bg-dark-glass {
    background: rgba(0,0,0,0.3);
  }
  .border-white-10 {
    border-color: rgba(255,255,255,0.1) !important;
  }
  .text-glow {
    text-shadow: 0 0 10px rgba(0, 255, 65, 0.5);
  }
  .text-glow-danger {
    text-shadow: 0 0 10px rgba(255, 0, 51, 0.5);
  }
  
  .btn-glow:not(:disabled) {
    box-shadow: 0 0 15px rgba(0, 255, 65, 0.3);
  }
  .btn-glow:hover:not(:disabled) {
     box-shadow: 0 0 25px rgba(0, 255, 65, 0.5);
  }

  .fw-black {
      font-weight: 900;
  }
  .tracking-widest {
      letter-spacing: 0.1em;
  }
  .focus-ring-none:focus {
      box-shadow: none;
  }
</style>
