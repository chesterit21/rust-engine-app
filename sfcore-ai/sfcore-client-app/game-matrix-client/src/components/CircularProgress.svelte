<script>
  export let value = 0;           // Value in format #.### (e.g., 6.432) - max 10
  export let displayText = '';    // Original text to display (e.g., "6.432")
  export let maxValue = 10;       // Max value (10.000)
  export let size = 100;          // Size in pixels
  export let strokeWidth = 8;     // Stroke width
  export let trend = '';          // 'UP' or 'DOWN' for trend icon
  export let compact = false;     // Compact mode: smaller, lighter text
  
  // Calculate percentage (0-100) - ensure proper calculation
  $: numericValue = typeof value === 'number' ? value : parseFloat(value) || 0;
  $: percentage = Math.min(Math.max((numericValue / maxValue) * 100, 0), 100);
  
  // SVG calculations
  $: radius = (size - strokeWidth) / 2;
  $: circumference = 2 * Math.PI * radius;
  $: dashOffset = circumference - (percentage / 100) * circumference;
  
  // Random gradient colors (3-color gradients for extra beauty)
  const gradients = [
    { id: 'grad-pink-purple', colors: ['#ff0080', '#a855f7', '#7c3aed'] },
    { id: 'grad-orange-yellow', colors: ['#ef4444', '#f97316', '#fbbf24'] },
    { id: 'grad-cyan-blue', colors: ['#22d3ee', '#3b82f6', '#6366f1'] },
    { id: 'grad-green-teal', colors: ['#10b981', '#14b8a6', '#06b6d4'] },
    { id: 'grad-red-pink', colors: ['#f43f5e', '#ec4899', '#d946ef'] },
    { id: 'grad-sunset', colors: ['#fbbf24', '#f97316', '#ef4444'] },
  ];
  
  // Pick random gradient once on component creation
  const selectedGradient = gradients[Math.floor(Math.random() * gradients.length)];
  
  // Use displayText if provided, otherwise format the value
  $: shownValue = displayText || (numericValue >= 10 ? numericValue.toFixed(0) : numericValue.toFixed(3));
</script>

<div class="circular-progress" style="width: {size}px; height: {size}px;">
  <svg width={size} height={size} viewBox="0 0 {size} {size}">
    <!-- Gradient definitions with 3 colors -->
    <defs>
      {#each gradients as grad}
        <linearGradient id={grad.id} x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color={grad.colors[0]} />
          <stop offset="50%" stop-color={grad.colors[1]} />
          <stop offset="100%" stop-color={grad.colors[2]} />
        </linearGradient>
      {/each}
    </defs>
    
    <!-- Background circle (track) -->
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke="rgba(255, 255, 255, 0.08)"
      stroke-width={strokeWidth}
    />
    
    <!-- Progress circle with gradient -->
    <circle
      class="progress-ring"
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke="url(#{selectedGradient.id})"
      stroke-width={strokeWidth}
      stroke-linecap="round"
      stroke-dasharray={circumference}
      stroke-dashoffset={dashOffset}
      transform="rotate(-90 {size / 2} {size / 2})"
    />
  </svg>
  
  <!-- Center content -->
  <div class="content" class:compact={compact}>
    <span class="value" class:compact={compact}>{shownValue} %</span>
    {#if trend}
      <span class="trend-icon" class:up={trend === 'UP'} class:down={trend === 'DOWN'}>
        {#if trend === 'UP'}
          <i class="bi bi-caret-up-fill"></i>
        {:else}
          <i class="bi bi-caret-down-fill"></i>
        {/if}
      </span>
    {/if}
  </div>
</div>

<style>
  .circular-progress {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  
  .progress-ring {
    transition: stroke-dashoffset 1s cubic-bezier(0.4, 0, 0.2, 1);
  }
  
  .content {
    position: absolute;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding-top: 16px;
  }
  
  .value {
    font-size: 1.0rem;
    font-weight: 800;
    color: #fff;
    font-family: 'Inter', system-ui, sans-serif;
    line-height: 1;
    text-shadow: 0 2px 8px rgba(0,0,0,0.3);
  }
  
  .value.compact {
    font-size: 0.7rem;
    font-weight: 500;
    text-shadow: none;
  }
  
  .content.compact {
    padding-top: 0;
  }
  
  .trend-icon {
    font-size: 0.9rem;
    margin-top: 2px;
  }
  
  .trend-icon.up {
    color: #22c55e;
    text-shadow: 0 0 8px rgba(34, 197, 94, 0.5);
  }
  
  .trend-icon.down {
    color: #ef4444;
    text-shadow: 0 0 8px rgba(239, 68, 68, 0.5);
  }
</style>
