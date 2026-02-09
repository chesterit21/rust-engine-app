<script>
  import { createEventDispatcher } from 'svelte';
  import Loader from '../components/Loader.svelte';
  
  const dispatch = createEventDispatcher();
  
  let username = '';
  let password = '';
  let loading = false;

  async function handleLogin() {
    loading = true;
    // Simulate API call
    setTimeout(() => {
      loading = false;
      dispatch('login', { username, password });
    }, 2000); // Increased time to enjoy the spinner
  }
</script>

<div class="login-container d-flex min-vh-100 align-items-center justify-content-center">
  <!-- Background Effects -->
  <div class="matrix-bg"></div>
  <div class="glow-orb top-start"></div>
  <div class="glow-orb bottom-end"></div>

  <div class="login-card card border-0 p-4 p-md-5">
    <div class="text-center mb-4 z-2 position-relative">
      <div class="logo-container mx-auto mb-3">
        <i class="bi bi-cpu-fill fs-1 text-matrix"></i>
      </div>
      <h2 class="fw-bold text-white mb-1">SFCore TradeX</h2>
      <p class="text-muted small">AI POWERED TRADE MATRIX</p>
    </div>

    <form on:submit|preventDefault={handleLogin} class="z-2 position-relative">
      <div class="form-floating mb-3">
        <input type="text" class="form-control" id="floatingInput" placeholder="Username" bind:value={username} required autocomplete="off">
        <label for="floatingInput">Username</label>
      </div>
      <div class="form-floating mb-4">
        <input type="password" class="form-control" id="floatingPassword" placeholder="Password" bind:value={password} required>
        <label for="floatingPassword">Password</label>
      </div>

      <button class="btn btn-login w-100 py-3 fw-bold text-uppercase tracking-wider d-flex align-items-center justify-content-center" type="submit" disabled={loading}>
        {#if loading}
          <div class="py-1">
             <Loader size="sm" text="ACCESSING..." />
          </div>
        {:else}
          <span class="text-dark">ENTER MATRIX</span>
          <i class="bi bi-arrow-right-short fs-4 ms-2 text-dark"></i>
        {/if}
      </button>
    </form>

    <div class="mt-4 text-center z-2 position-relative">
      <button type="button" class="btn btn-link text-decoration-none text-muted small hover-matrix p-0 border-0 bg-transparent">Forgot Access Code?</button>
    </div>
  </div>
</div>

<style>
  :global(:root) {
    --matrix-green: #00ff41;
    --matrix-glow: rgba(0, 255, 65, 0.5);
    --matrix-bg: #050505;
    --glass-bg: rgba(20, 20, 20, 0.7);
    --glass-border: rgba(255, 255, 255, 0.08);
  }

  /* Container & Backgrounds */
  .login-container {
    background-color: var(--matrix-bg);
    position: relative;
    overflow: hidden;
  }

  .matrix-bg {
    position: absolute;
    inset: 0;
    background-image: 
      linear-gradient(rgba(0, 255, 65, 0.03) 1px, transparent 1px),
      linear-gradient(90deg, rgba(0, 255, 65, 0.03) 1px, transparent 1px);
    background-size: 30px 30px;
    mask-image: radial-gradient(circle at center, black 40%, transparent 100%);
    pointer-events: none;
  }

  .glow-orb {
    position: absolute;
    width: 300px;
    height: 300px;
    border-radius: 50%;
    background: var(--matrix-green);
    filter: blur(150px);
    opacity: 0.15;
    pointer-events: none;
    z-index: 0;
  }
  .top-start { top: -100px; left: -100px; }
  .bottom-end { bottom: -100px; right: -100px; }

  /* Card Styles */
  .login-card {
    width: 100%;
    max-width: 420px;
    background: var(--glass-bg);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid var(--glass-border) !important;
    border-radius: 24px;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
    transform: translateY(0);
    transition: transform 0.3s ease, box-shadow 0.3s ease;
  }

  .login-card:hover {
    box-shadow: 0 0 30px rgba(0, 255, 65, 0.15);
    border-color: rgba(0, 255, 65, 0.3) !important;
  }

  /* Logo */
  .logo-container {
    width: 80px;
    height: 80px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: inset 0 0 20px rgba(0, 0, 0, 0.5);
    border: 1px solid var(--glass-border);
  }
  
  .text-matrix {
    color: var(--matrix-green);
    filter: drop-shadow(0 0 10px var(--matrix-glow));
  }

  /* Form Inputs */
  .form-control {
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid var(--glass-border);
    color: #fff;
    border-radius: 12px;
    height: 56px;
  }

  .form-control:focus {
    background: rgba(0, 0, 0, 0.6);
    border-color: var(--matrix-green);
    box-shadow: 0 0 0 4px rgba(0, 255, 65, 0.1);
    color: #fff;
  }
  
  .form-floating label {
    color: rgba(255, 255, 255, 0.5);
  }
  
  .form-floating > .form-control:focus ~ label,
  .form-floating > .form-control:not(:placeholder-shown) ~ label {
    color: var(--matrix-green);
    opacity: 0.8;
  }

  /* Button */
  .btn-login {
    background: var(--matrix-green);
    border: none;
    border-radius: 12px;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
  }

  .btn-login:hover:not(:disabled) {
    background: #00cc33;
    transform: translateY(-2px);
    box-shadow: 0 10px 20px -5px rgba(0, 255, 65, 0.4);
  }

  .btn-login:disabled {
    background: rgba(255, 255, 255, 0.1);
    cursor: not-allowed;
  }

  /* Links */
  .hover-matrix:hover {
    color: var(--matrix-green) !important;
    text-shadow: 0 0 8px var(--matrix-glow);
  }

  .tracking-wider {
    letter-spacing: 0.1em;
  }
</style>
