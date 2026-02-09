<script>
  import Navbar from './Navbar.svelte';
  import ThemeToggle from './ThemeToggle.svelte';
  import { createEventDispatcher } from 'svelte';
  import { notifications, removeNotification } from '../stores/notifications';

  const dispatch = createEventDispatcher();

  function handleLogout() {
    dispatch('logout');
  }
</script>

<div class="min-vh-100 d-flex flex-column position-relative overflow-hidden" id="main-layout">
  <div class="matrix-bg"></div>
  <div class="glow-orb top-start"></div>
  <div class="glow-orb bottom-end"></div>

  <Navbar on:logout={handleLogout}>
    <div slot="theme-toggle">
      <ThemeToggle />
    </div>
  </Navbar>

  <!-- Global Notifications -->
  <div class="notification-container">
      {#each $notifications as note (note.id)}
          <div class="notification-toast" class:bg-success={note.type === 'success'} class:bg-danger={note.type === 'error'}>
              <div class="d-flex justify-content-between align-items-center">
                  <span>{note.message}</span>
                  <button class="btn-close btn-close-white ms-2" style="width: 0.5em; height: 0.5em;" on:click={() => removeNotification(note.id)} aria-label="Close notification"></button>
              </div>
          </div>
      {/each}
  </div>

  <main class="container-fluid px-4 py-4 flex-grow-1 z-2 position-relative">
    <slot />
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

  /* Utilities */
  .bg-dark-glass {
    background: rgba(0,0,0,0.3);
  }
  .border-white-10 {
    border-color: rgba(255,255,255,0.1) !important;
  }
  
  /* Global Notifications */
  .notification-container {
    position: fixed;
    top: 20px;
    right: 20px;
    z-index: 10000; /* Above everything */
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 300px;
    pointer-events: none; /* Let clicks pass through if not on toast */
  }
  /* Allow clicks on toast */
  .notification-toast {
    pointer-events: auto; 
    padding: 10px 15px;
    border-radius: 6px;
    color: white;
    box-shadow: 0 4px 6px rgba(0,0,0,0.3);
    font-size: 0.85rem;
    animation: slideIn 0.3s ease-out;
    opacity: 0.95;
    transition: all 0.3s;
  }
  @keyframes slideIn {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 0.95; }
  }
  .bg-success { background-color: #198754; }
  .bg-danger { background-color: #dc3545; }
</style>
