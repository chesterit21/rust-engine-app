<script>
  export let text = "LOADING SYSTEM...";
  export let size = "md"; // sm, md, lg
</script>

<div class="loader-container {size}">
  <div class="ring"></div>
  <div class="ring"></div>
  <div class="ring"></div>
  <span class="loading-text">{text}</span>
</div>

<style>
  .loader-container {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  /* Sizes */
  .loader-container.sm { width: 50px; height: 50px; }
  .loader-container.sm .loading-text { font-size: 0.5rem; }
  
  .loader-container.md { width: 150px; height: 150px; }
  .loader-container.md .loading-text { font-size: 0.8rem; letter-spacing: 2px; }

  .loader-container.lg { width: 300px; height: 300px; }
  .loader-container.lg .loading-text { font-size: 1.2rem; letter-spacing: 4px; }

  .ring {
    position: absolute;
    border-radius: 50%;
    border: 2px solid transparent;
    border-top: 2px solid #00ff41;
    border-left: 2px solid transparent;
    border-bottom: 2px solid #00ff41; 
    border-right: 2px solid transparent;
    box-shadow: 0 0 10px rgba(0, 255, 65, 0.5);
    animation: rotate 2s linear infinite;
  }

  .ring:nth-child(1) {
    width: 100%;
    height: 100%;
    animation-direction: normal;
  }

  .ring:nth-child(2) {
    width: 70%;
    height: 70%;
    border-top: 2px solid #00DD33;
    border-bottom: 2px solid #00DD33;
    animation-duration: 3s;
    animation-direction: reverse;
    opacity: 0.8;
  }

  .ring:nth-child(3) {
    width: 40%;
    height: 40%;
    border-top: 2px solid #00AA22;
    border-bottom: 2px solid #00AA22;
    animation-duration: 1.5s;
    animation-direction: normal;
    opacity: 0.6;
  }

  .loading-text {
    color: #00ff41;
    font-family: 'Courier New', monospace;
    font-weight: bold;
    text-align: center;
    animation: blink 1s infinite alternate;
    text-shadow: 0 0 5px rgba(0, 255, 65, 0.7);
    z-index: 10;
  }

  @keyframes rotate {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  @keyframes blink {
    0% { opacity: 0.5; }
    100% { opacity: 1; }
  }
</style>
