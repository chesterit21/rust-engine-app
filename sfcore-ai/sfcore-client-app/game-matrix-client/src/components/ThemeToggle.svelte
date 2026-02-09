<script>
  import { onMount } from 'svelte';

  let theme = 'light';

  onMount(() => {
    // Check local storage or system preference
    const storedTheme = localStorage.getItem('theme');
    if (storedTheme) {
      theme = storedTheme;
    } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
      theme = 'dark';
    }
    applyTheme();
  });

  function toggleTheme() {
    theme = theme === 'light' ? 'dark' : 'light';
    localStorage.setItem('theme', theme);
    applyTheme();
  }

  function applyTheme() {
    document.documentElement.setAttribute('data-bs-theme', theme);
  }
</script>

<button class="btn btn-link nav-link px-2 text-white" on:click={toggleTheme} title="Toggle theme">
  {#if theme === 'dark'}
    <i class="bi bi-sun-fill fs-5"></i>
  {:else}
    <i class="bi bi-moon-stars-fill fs-5"></i>
  {/if}
</button>
