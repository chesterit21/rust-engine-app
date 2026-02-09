<script>
  import { onMount, onDestroy } from 'svelte';

  // Props
  export let height = "400px";
  
  let canvas;
  let ctx;
  let animationId;
  let candles = [];

  // Generate Random OHLC Data
  function generateCandles(count) {
      const data = [];
      let open = 150 + Math.random() * 50;
      
      for (let i = 0; i < count; i++) {
          const changePercent = (Math.random() - 0.48) * 5;
          const change = open * (changePercent / 100);
          const close = open + change;
          const high = Math.max(open, close) + Math.abs(change) * Math.random() * 0.5;
          const low = Math.min(open, close) - Math.abs(change) * Math.random() * 0.5;
          const volume = Math.random() * 100 + 20;
          
          data.push({ open, high, low, close, volume, isUp: close >= open });
          open = close;
      }
      return data;
  }

  function drawChart() {
      if (!canvas || !ctx) return;
      
      const width = canvas.width;
      const height = canvas.height;
      
      // Clear
      ctx.fillStyle = '#0d1117';
      ctx.fillRect(0, 0, width, height);
      
      if (candles.length === 0) return;
      
      // Calculate price range
      let minPrice = Infinity;
      let maxPrice = -Infinity;
      candles.forEach(c => {
          minPrice = Math.min(minPrice, c.low);
          maxPrice = Math.max(maxPrice, c.high);
      });
      
      const padding = 40;
      const chartWidth = width - padding * 2;
      const chartHeight = height - padding * 2 - 50; // Leave room for volume
      const volumeHeight = 40;
      
      const priceRange = maxPrice - minPrice;
      const candleWidth = chartWidth / candles.length;
      const bodyWidth = candleWidth * 0.7;
      
      // Draw Grid Lines
      ctx.strokeStyle = 'rgba(42, 46, 57, 0.6)';
      ctx.lineWidth = 1;
      for (let i = 0; i <= 5; i++) {
          const y = padding + (chartHeight / 5) * i;
          ctx.beginPath();
          ctx.moveTo(padding, y);
          ctx.lineTo(width - padding, y);
          ctx.stroke();
          
          // Price Label
          const price = maxPrice - (priceRange / 5) * i;
          ctx.fillStyle = 'rgba(255, 255, 255, 0.5)';
          ctx.font = '10px monospace';
          ctx.fillText(price.toFixed(2), width - padding + 5, y + 4);
      }
      
      // Draw Candles
      candles.forEach((c, i) => {
          const x = padding + i * candleWidth + candleWidth / 2;
          
          // Price to Y coordinate
          const yHigh = padding + ((maxPrice - c.high) / priceRange) * chartHeight;
          const yLow = padding + ((maxPrice - c.low) / priceRange) * chartHeight;
          const yOpen = padding + ((maxPrice - c.open) / priceRange) * chartHeight;
          const yClose = padding + ((maxPrice - c.close) / priceRange) * chartHeight;
          
          const color = c.isUp ? '#32CD32' : '#DC143C'; // Lime / Crimson
          
          // Wick
          ctx.strokeStyle = color;
          ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.moveTo(x, yHigh);
          ctx.lineTo(x, yLow);
          ctx.stroke();
          
          // Body
          ctx.fillStyle = color;
          const bodyTop = Math.min(yOpen, yClose);
          const bodyHeight = Math.max(Math.abs(yClose - yOpen), 1);
          ctx.fillRect(x - bodyWidth / 2, bodyTop, bodyWidth, bodyHeight);
          
          // Volume Bar
          const maxVolume = Math.max(...candles.map(c => c.volume));
          const volHeight = (c.volume / maxVolume) * volumeHeight;
          const volY = height - padding - volHeight;
          ctx.fillStyle = c.isUp ? 'rgba(50, 205, 50, 0.5)' : 'rgba(220, 20, 60, 0.5)'; // Lime / Crimson
          ctx.fillRect(x - bodyWidth / 2, volY, bodyWidth, volHeight);
      });
      
      // Draw Current Price Line
      if (candles.length > 0) {
          const lastCandle = candles[candles.length - 1];
          const yPrice = padding + ((maxPrice - lastCandle.close) / priceRange) * chartHeight;
          
          ctx.strokeStyle = lastCandle.isUp ? '#32CD32' : '#DC143C'; // Lime / Crimson
          ctx.lineWidth = 1;
          ctx.setLineDash([4, 4]);
          ctx.beginPath();
          ctx.moveTo(padding, yPrice);
          ctx.lineTo(width - padding, yPrice);
          ctx.stroke();
          ctx.setLineDash([]);
          
          // Price Tag
          ctx.fillStyle = lastCandle.isUp ? '#32CD32' : '#DC143C'; // Lime / Crimson
          ctx.fillRect(width - padding, yPrice - 10, 55, 20);
          ctx.fillStyle = '#fff';
          ctx.font = 'bold 10px monospace';
          ctx.fillText(lastCandle.close.toFixed(2), width - padding + 5, yPrice + 4);
      }
  }

  function resizeCanvas() {
      if (!canvas) return;
      const rect = canvas.parentElement.getBoundingClientRect();
      canvas.width = rect.width;
      canvas.height = rect.height;
      drawChart();
  }

  onMount(() => {
      ctx = canvas.getContext('2d');
      candles = generateCandles(80);
      
      // Initial resize
      setTimeout(() => {
          resizeCanvas();
      }, 100);
      
      // Handle window resize
      window.addEventListener('resize', resizeCanvas);
  });

  onDestroy(() => {
      window.removeEventListener('resize', resizeCanvas);
      if (animationId) cancelAnimationFrame(animationId);
  });
</script>

<div class="chart-container" style="height: {height}; width: 100%;">
    <canvas bind:this={canvas}></canvas>
</div>

<style>
    .chart-container {
        border-radius: 8px;
        overflow: hidden;
        background: #0d1117;
        position: relative;
    }
    canvas {
        display: block;
        width: 100%;
        height: 100%;
    }
</style>
