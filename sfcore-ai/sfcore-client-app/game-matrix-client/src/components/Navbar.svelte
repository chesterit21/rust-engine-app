<nav class="navbar navbar-expand-lg navbar-dark bg-dark-glass sticky-top border-bottom border-white-10 z-3">
  <div class="container-fluid px-4">
    <a class="navbar-brand fw-bold d-flex align-items-center tracking-wide" href="#">
      <i class="bi bi-grid-3x3-gap-fill me-2 fs-4 text-primary text-glow"></i>
      <span class="text-white">TradeX Matrix</span>
    </a>
    
    <button class="navbar-toggler border-0" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav">
      <span class="navbar-toggler-icon"></span>
    </button>
    
    <div class="collapse navbar-collapse" id="navbarNav">
      <ul class="navbar-nav me-auto mb-2 mb-lg-0">
        <li class="nav-item">
          <a class="nav-link active fw-semibold text-primary" href="#/"><i class="bi bi-speedometer2 me-1"></i> Dashboard</a>
        </li>
        <li class="nav-item">
          <a class="nav-link fw-semibold hover-glow" href="#/history"><i class="bi bi-clock-history me-1"></i> History</a>
        </li>
        <li class="nav-item dropdown">
          <button class="nav-link dropdown-toggle fw-semibold hover-glow bg-transparent border-0 text-white" 
            id="setupDropdown" 
            data-bs-toggle="dropdown" 
            aria-expanded="false"
            aria-label="Toggle Setup Menu">
            <i class="bi bi-gear-fill me-1"></i> Setup
          </button>
          <ul class="dropdown-menu shadow border-0 rounded-3 mt-2 bg-dark-glass border-white-10" aria-labelledby="setupDropdown">
            <li><a class="dropdown-item text-white hover-bg-primary py-2" href="#/setup/master-game">
              <i class="bi bi-controller me-2 text-primary"></i> Master Game
            </a></li>
            <li><a class="dropdown-item text-white hover-bg-primary py-2" href="#/setup/member-game">
              <i class="bi bi-person-badge-fill me-2 text-info"></i> Member Game
            </a></li>
            <li><a class="dropdown-item text-white hover-bg-primary py-2" href="#/setup/site-master">
              <i class="bi bi-browser-safari me-2 text-success"></i> Site Master
            </a></li>
            <li><a class="dropdown-item text-white hover-bg-primary py-2" href="#/setup/link-game">
              <i class="bi bi-link-45deg me-2 text-warning"></i> Setup Link Game
            </a></li>
          </ul>
        </li>
      </ul>
      
      <div class="d-flex align-items-center">
        <!-- Theme Toggle Slot -->
        <slot name="theme-toggle"></slot>
        
        <div class="vr mx-3 text-white opacity-25 d-none d-lg-block"></div>
        
        <div class="dropdown">
          <a href="#" class="d-flex align-items-center text-white text-decoration-none dropdown-toggle" data-bs-toggle="dropdown" aria-expanded="false">
            <div class="bg-dark border border-secondary text-primary rounded-circle d-flex align-items-center justify-content-center fw-bold me-2" style="width: 32px; height: 32px;">
              U
            </div>
            <span class="d-none d-sm-inline fw-semibold small text-uppercase tracking-wider">SFCore Admin</span>
          </a>
          <ul class="dropdown-menu dropdown-menu-end shadow border-0 rounded-3 mt-2 bg-dark-glass border-white-10">
            <li><a class="dropdown-item text-white hover-bg-primary" href="#"><i class="bi bi-person me-2"></i> Profile</a></li>
            <li><a class="dropdown-item text-white hover-bg-primary" href="#"><i class="bi bi-wallet2 me-2"></i> Wallet</a></li>
            <li><hr class="dropdown-divider border-white-10"></li>
            <li><a class="dropdown-item text-danger hover-bg-danger" href="#" on:click={() => dispatch('logout')}><i class="bi bi-box-arrow-right me-2"></i> Sign out</a></li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</nav>

<script>
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher();
</script>

<style>
  .bg-dark-glass {
    background: rgba(5, 5, 5, 0.85); /* Slightly darker for navbar */
    backdrop-filter: blur(12px);
  }
  .border-white-10 {
    border-color: rgba(255, 255, 255, 0.1) !important;
  }
  .text-glow {
    filter: drop-shadow(0 0 5px rgba(0, 255, 65, 0.6));
  }
  .tracking-wide {
    letter-spacing: 0.05em;
  }
  .tracking-wider {
    letter-spacing: 0.1em;
  }
  
  /* Dropdown customizations */
  .dropdown-menu {
    background: rgba(20, 20, 20, 0.95);
    backdrop-filter: blur(10px);
  }
  .dropdown-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  .hover-bg-primary:hover {
    color: var(--matrix-green) !important;
  }
  .hover-glow:hover {
    color: var(--matrix-green) !important;
    text-shadow: 0 0 8px rgba(0, 255, 65, 0.4);
  }
</style>
