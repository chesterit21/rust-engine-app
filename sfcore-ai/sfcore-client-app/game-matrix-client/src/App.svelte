<script>
  import Login from './pages/Login.svelte';
  import Dashboard from './pages/Dashboard.svelte';
  import LogAnalisa from './pages/LogAnalisa.svelte';
  import History from './pages/History.svelte';
  import MasterGame from './pages/MasterGame.svelte';
  import MemberGame from './pages/MemberGame.svelte';
  import SiteMaster from './pages/SiteMaster.svelte';
  import SetupLinkGame from './pages/SetupLinkGame.svelte';
  import Layout from './components/Layout.svelte';
  import Loader from './components/Loader.svelte';
  import ThemeToggle from './components/ThemeToggle.svelte';
  
  import * as api from './lib/api';
  import { onMount } from 'svelte';

  let isLoggedIn = false;
  let username = '';
  let games = [];
  let loading = false;
  let error = null;
  let toastMessage = '';
  let toastType = 'error'; // 'error' | 'success'
  let showToast = false;

  function showToastMessage(message, type = 'error') {
    toastMessage = message;
    toastType = type;
    showToast = true;
    setTimeout(() => {
      showToast = false;
    }, 4000);
  }

  // Router State
  let currentRoute = 'dashboard';
  let routeParams = {};

  // Session Storage Key
  const SESSION_KEY = 'game_matrix_session';

  function handleHashChange() {
    const hash = window.location.hash;
    if (hash.startsWith('#/analysis/')) {
        const code = hash.replace('#/analysis/', '');
        if (code) {
            currentRoute = 'analysis';
            routeParams = { code };
        } else {
             window.location.hash = ''; // Invalid
        }
    } else if (hash === '#/history') {
        currentRoute = 'history';
        routeParams = {};
    } else if (hash === '#/setup/master-game') {
        currentRoute = 'master-game';
        routeParams = {};
    } else if (hash === '#/setup/member-game') {
        currentRoute = 'member-game';
        routeParams = {};
    } else if (hash === '#/setup/site-master') {
        currentRoute = 'site-master';
        routeParams = {};
    } else if (hash === '#/setup/link-game') {
        currentRoute = 'link-game';
        routeParams = {};
    } else {
        currentRoute = 'dashboard';
        routeParams = {};
    }
  }

  function saveSession(user) {
    try {
      localStorage.setItem(SESSION_KEY, JSON.stringify({ username: user, timestamp: Date.now() }));
    } catch (e) {
      console.warn('Failed to save session:', e);
    }
  }

  function loadSession() {
    try {
      const data = localStorage.getItem(SESSION_KEY);
      if (data) {
        const session = JSON.parse(data);
        // Optional: Check session expiry (e.g., 24 hours)
        const maxAge = 24 * 60 * 60 * 1000; // 24 hours
        if (Date.now() - session.timestamp < maxAge) {
          return session.username;
        } else {
          localStorage.removeItem(SESSION_KEY);
        }
      }
    } catch (e) {
      console.warn('Failed to load session:', e);
    }
    return null;
  }

  function clearSession() {
    try {
      localStorage.removeItem(SESSION_KEY);
    } catch (e) {
      console.warn('Failed to clear session:', e);
    }
  }

  onMount(async () => {
    window.addEventListener('hashchange', handleHashChange);
    handleHashChange(); // Initial check

    // Try to restore session
    const savedUser = loadSession();
    if (savedUser) {
      isLoggedIn = true;
      username = savedUser;
      await loadGames();
    }
  });

  async function handleLogin(event) {
    loading = true;
    error = null;
    try {
      const { username: user, password } = event.detail;
      const response = await api.login(user, password);
      
      isLoggedIn = true;
      username = response.username;
      
      // Save session to localStorage
      saveSession(response.username);
      
      await loadGames();
    } catch (e) {
      error = e.message;
      showToastMessage(e.message || 'Login failed. Please check username and password.', 'error');
      loading = false;
    }
  }

  function handleLogout() {
    isLoggedIn = false;
    games = [];
    username = '';
    error = null;
    clearSession();
    window.location.hash = '';
  }

  async function loadGames() {
    loading = true;
    try {
      games = await api.fetchDashboardGames();
    } catch (e) {
      error = e.message;
    } finally {
       loading = false;
    }
  }
</script>

{#if showToast}
  <div class="toast-container position-fixed top-0 end-0 p-3" style="z-index: 9999;">
    <div class="toast show shadow-lg border-0" class:bg-danger={toastType === 'error'} class:bg-success={toastType === 'success'}>
      <div class="toast-header border-0" class:bg-danger={toastType === 'error'} class:bg-success={toastType === 'success'} class:text-white={true}>
        <i class="bi me-2" class:bi-exclamation-triangle-fill={toastType === 'error'} class:bi-check-circle-fill={toastType === 'success'}></i>
        <strong class="me-auto">{toastType === 'error' ? 'Error' : 'Success'}</strong>
        <button type="button" class="btn-close btn-close-white" on:click={() => showToast = false}></button>
      </div>
      <div class="toast-body text-white">
        {toastMessage}
      </div>
    </div>
  </div>
{/if}

{#if !isLoggedIn}
  <main class="page-container">
    <div class="position-absolute top-0 end-0 p-3 z-3">
        <div class="bg-dark rounded-circle p-1 shadow">
             <ThemeToggle />
        </div>
    </div>
    <Login on:login={handleLogin} />
  </main>
{:else}
  {#if loading}
    <div class="min-vh-100 d-flex align-items-center justify-content-center bg-black">
      <Loader size="lg" text="INITIALIZING TRADE DATA..." />
    </div>
  {:else}
    <Layout on:logout={handleLogout}>
        {#if error}
            <div class="alert alert-danger shadow-sm border-0 rounded-3 mb-4" role="alert">
              <i class="bi bi-exclamation-triangle-fill me-2"></i> {error}
            </div>
        {/if}

        {#if currentRoute === 'dashboard'}
            <Dashboard {games} />
        {:else if currentRoute === 'analysis'}
            <LogAnalisa gameCode={routeParams.code} />
        {:else if currentRoute === 'history'}
            <History />
        {:else if currentRoute === 'master-game'}
            <MasterGame />
        {:else if currentRoute === 'member-game'}
            <MemberGame />
        {:else if currentRoute === 'site-master'}
            <SiteMaster />
        {:else if currentRoute === 'link-game'}
            <SetupLinkGame />
        {/if}
    </Layout>
  {/if}
{/if}

<style>
  :global(#app) {
    min-height: 100vh;
  }
  .page-container {
    min-height: 100vh;
    width: 100%;
  }
</style>

