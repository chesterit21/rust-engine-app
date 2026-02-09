<script>
  import { onMount, onDestroy } from 'svelte';
  import * as api from '../lib/api';
  import Chart from 'chart.js/auto';
  import { getDigits, findFront3Matches, findBack3Matches, buildPCNList, checkPatternMatch } from '../lib/analysis';
  import FloatingControlPanel from '../components/FloatingControlPanel.svelte';
  import FrequencyAnalysisWidget from '../components/FrequencyAnalysisWidget.svelte';

  import HistoryAnalysisList from '../components/HistoryAnalysisList.svelte';

  export let gameCode;

  let logs = [];
  let loading = false;
  let error = null;
  let gridRows = []; 
  let currentLog = null; 
  let chartCanvas;
  let chartInstance;

  // --- Analysis State ---
  let selectedLog = null;
  let selectedPeriode = null;
  let frontMatches = [];
  let midMatches = [];
  let backMatches = [];
  let selectedCellPeriode = null; // Single cell selection
  let selectedCellColor = 1; // Default to first non-transparent color

  // 3-Digit Front & Back matches
  let front3Matches = [];
  let back3Matches = [];
  let matchGrids = []; // Dynamic log tables for each 3-digit match

  // History Analysis State
  let historyItems = [];
  let historyLoading = false;

  // Hit Textbox values (same naming as legacy)
  let Tb3DA = '';
  let Tb3DB = '';
  let Tb3DC = '';
  let Tb3DD = '';
  let matchGridCellSelected = null; // Track selected cell in match grids
  let activeAnalysisTab = 'match-grids'; // 'match-grids' | 'tradex-analysis' | 'history-analysis'



  const COLOR_STATES = [
      'rgba(0,0,0,0)',           // transparent
      //'rgb(232, 157, 192)',      // pink
      'rgb(136, 205, 208)',      // cyan
      'rgb(221, 222, 222)',      // gray
      'rgb(198, 246, 210)',      // light green
      'rgb(198, 220, 246)',      // light blue
  ];

  // --- Chart Data & Options ---
  function generateRandomData(count) {
      return Array.from({ length: count }, () => Math.floor(Math.random() * 100));
  }

  function initChart() {
      if (!chartCanvas) return;
      
      const ctx = chartCanvas.getContext('2d');
      if (chartInstance) chartInstance.destroy();

      const chartLabels = Array.from({ length: 30 }, (_, i) => i + 1);
      
      chartInstance = new Chart(ctx, {
          type: 'bar',
          data: {
              labels: chartLabels,
              datasets: [
                  {
                      label: 'Set A',
                      data: generateRandomData(30),
                      backgroundColor: 'rgba(0, 255, 65, 0.8)',
                      borderRadius: 4,
                      barPercentage: 0.6,
                      categoryPercentage: 0.8
                  },
                  {
                      label: 'Set B',
                      data: generateRandomData(30),
                      backgroundColor: 'rgba(255, 255, 255, 0.3)',
                      borderRadius: 4,
                      barPercentage: 0.6,
                      categoryPercentage: 0.8
                  }
              ]
          },
          options: {
              responsive: true,
              maintainAspectRatio: false,
              plugins: {
                  legend: { display: false },
                  tooltip: { 
                      enabled: true,
                      backgroundColor: 'rgba(0,0,0,0.8)',
                      titleColor: '#0f0',
                      bodyFont: { family: 'monospace' }
                  }
              },
              scales: {
                  x: { display: false },
                  y: { display: false, min: 0 }
              },
              animation: { duration: 1500, easing: 'easeOutQuart' }
          }
      });
  }

  // --- Digit Extraction Helper (using imported getDigits from analysis.js) ---

  // --- Log Cell Click Handler ---
  function handleLogClick(periode) {
      // Get log by periode
      const log = logs.find(l => (l.periode || l.Periode) == periode);
      if (!log) return;

      // Single select: clicking same cell clears it, clicking another cell selects it
      if (selectedCellPeriode === periode) {
          // Clicking same cell - cycle color or clear
          selectedCellColor = (selectedCellColor % (COLOR_STATES.length - 1)) + 1;
      } else {
          // Clicking different cell - select new one with first color
          selectedCellPeriode = periode;
          selectedCellColor = 1;
      }

      // Set selected log and run analysis
      selectedLog = log;
      selectedPeriode = periode;

      // Copy to hit textboxes
      copyDataToHitTextBox(log.formatted_result || log.logResult || '');

      runPatternAnalysis();
  }

  // --- Copy Data to Hit TextBoxes (reusable) ---
  function copyDataToHitTextBox(logValue) {
      const log = logValue.replace('.', '').trim();
      if (log.length < 4) return;

      const asx = log.substring(0, 1);
      const kop = log.substring(1, 2);
      const kepala = log.substring(2, 3);
      const ekor = log.substring(3, 4);

      Tb3DA = `${asx}-${kop}-${kepala}`;
      Tb3DB = `${asx}-${kop}-${ekor}`;
      Tb3DC = `${asx}-${kepala}-${ekor}`;
      Tb3DD = `${kop}-${kepala}-${ekor}`;
  }

  function format3Digit(input) {
      // Remove all non-digits
      const digits = input.replace(/\D/g, '').slice(0, 3);
      let res = '';
      for (let i = 0; i < digits.length; i++) {
          if (i > 0) res += '-';
          res += digits[i];
      }
      return res;
  }

  import { addNotification } from '../stores/notifications';

  // --- Save Pattern Match ---
  async function handleSavePattern(tipe, digitValue) {
    // Validation: 
    // 3-digit patterns (F Match, etc) -> "1-2-3" (len 5)
    // 2-digit compositions (2DF, etc) -> "12" (len 2) or "1-2" (len 3)
    const isComposition = tipe.startsWith('2D');
    const minLen = isComposition ? 2 : 5;

    if (!digitValue || digitValue.length < minLen) {
        addNotification('error', `Data input "${tipe}" tidak boleh kosong atau tidak lengkap.`);
        return;
    }

      if (!gameCode) {
         addNotification('error', "GameCode tidak ditemukan (Hidden Input Error).");
         return;
      }

      const payload = {
          game_code: gameCode,
          digit: digitValue,
          tipe: tipe
      };

      try {
          const response = await fetch(`${api.API_BASE_URL}/games/save-pattern`, {
              method: 'POST',
              headers: {
                  'Content-Type': 'application/json'
              },
              body: JSON.stringify(payload)
          });

          const result = await response.json();

          if (response.ok) {
              addNotification('success', "Data berhasil disimpan!");
          } else {
              addNotification('error', `Gagal menyimpan: ${result.message || 'Unknown error'}`);
          }

      } catch (err) {
          console.error("Save pattern error:", err);
          if (err instanceof TypeError || err.message.includes('Failed to fetch')) {
             addNotification('error', "Server Is Down Not Connected, please check the server application");
          } else {
             addNotification('error', "Terjadi kesalahan koneksi saat menyimpan data.");
          }
      }
  }

  // --- Handle Floating Panel Save Event ---
  function handleFloatingPanelSave(event) {
      const { type, digit } = event.detail;
      // We can reuse format3Digit if needed, but the floating panel sends raw digit.
      // Assuming floating panel sends "00", "01" etc. which is correct.
      // We pass it to handleSavePattern. 
      // handleSavePattern expects (tipe, digitValue).
      
      // Add notification handled by handleSavePattern logic.
      handleSavePattern(type, digit);
  }

  // --- Match Grid Cell Click Handler ---
  function handleMatchCellClick(cell, gridId) {
      // Set selected cell (for background highlight)
      matchGridCellSelected = `${gridId}-${cell.periode}`;

      // Copy data to hit textboxes
      copyDataToHitTextBox(cell.log);
  }

  let historyAttempted = false;

  // --- Reactive Fetch for History Analysis ---
  $: if (activeAnalysisTab === 'history-analysis' && gameCode) {
      if (!historyLoading && !historyAttempted && historyItems.length === 0) {
          loadHistoryAnalysis();
      }
  }

  // Reset attempted when gameCode changes
  $: if (gameCode) {
      historyAttempted = false;
      historyItems = [];
  }

  async function loadHistoryAnalysis() {
      historyLoading = true;
      historyAttempted = true;
      try {
          // Default window=100, depth=7 (handled by backend default if omitted, but let's be explicit)
          const response = await api.fetchHistoryAnalysis(gameCode, 100, 7);
          historyItems = response.history || [];
      } catch (err) {
          console.error("Failed to load history analysis:", err);
          addNotification('error', 'Failed to load history analysis');
      } finally {
          historyLoading = false;
      }
  }

  // --- Pattern Analysis ---
  function runPatternAnalysis() {
      if (!selectedLog || !logs.length) return;

      const sel = getDigits(selectedLog);
      if (!sel) return;

      const selPeriode = selectedLog.periode || selectedLog.Periode;

      // Find matches (only past periodes)
      let fronts = [];
      let mids = [];
      let backs = [];

      for (const log of logs) {
          const logPeriode = log.periode || log.Periode;
          if (logPeriode >= selPeriode) continue; // Only past

          const d = getDigits(log);
          if (!d) continue;

          // Front Match: AS + KOP same
          if (sel.as === d.as && sel.kop === d.kop) {
              fronts.push(log);
          }
          // Mid Match: KOP + KEPALA same
          if (sel.kop === d.kop && sel.kepala === d.kepala) {
              mids.push(log);
          }
          // Back Match: KEPALA + EKOR same
          if (sel.kepala === d.kepala && sel.ekor === d.ekor) {
              backs.push(log);
          }
      }

      // For each match, get P(rev), C(urrent), N(ext)
      frontMatches = buildPCNListLocal(fronts);
      midMatches = buildPCNListLocal(mids);
      backMatches = buildPCNListLocal(backs);

      // 3-Digit Front & Back matching
      front3Matches = findFront3Matches(logs, selectedLog);
      back3Matches = findBack3Matches(logs, selectedLog);

      // Tag, Combine, and Dedup matches by Periode
      const taggedFronts = front3Matches.map(m => ({ log: m, type: 'front' }));
      const taggedBacks = back3Matches.map(m => ({ log: m, type: 'back' }));
      
      const allTagged = [...taggedFronts, ...taggedBacks];
      const uniqueMap = new Map();

      allTagged.forEach(item => {
          const p = item.log.periode || item.log.Periode;
          if (uniqueMap.has(p)) {
              const existing = uniqueMap.get(p);
              if (existing.type !== item.type) {
                   existing.type = 'mix'; // Mark as both front and back
              }
          } else {
              uniqueMap.set(p, item);
          }
      });

      // Convert back to array and sort
      let allMatches = Array.from(uniqueMap.values());
      allMatches.sort((a, b) => (Number(b.log.periode || b.log.Periode) - Number(a.log.periode || a.log.Periode)));

      // --- Pre-calculate Grid Data (Augmentation) inside runPatternAnalysis (User Request) ---
      // 1. Create O(1) Map
      const logMap = new Map();
      logs.forEach(l => {
          const p = Number(l.periode || l.Periode);
          if (!isNaN(p)) logMap.set(p, l);
      });

      // 2. Augment matches
      allMatches = allMatches.map(matchWrapper => {
          const matchLog = matchWrapper.log;
          const matchPeriode = Number(matchLog.periode || matchLog.Periode);
          const limitMin = matchPeriode - 133;
          
          // Build Top 7 Rows (Future/Placeholder)
          const top7 = [];
          for (let offset = 7; offset >= 1; offset--) {
               const targetP = matchPeriode + offset;
               const foundLog = logMap.get(targetP);
               if (foundLog) {
                   top7.push(foundLog);
               } else {
                   top7.push({ 
                       isPlaceholder: true,
                       formatted_result: '#X',
                       logResult: '#X',
                       periode: targetP,
                       trend: '',
                       style: 'color: rgba(255, 255, 255, 0.3);'
                   });
               }
          }

          // Build Bottom Rows (History)
          const bottomLogs = logs.filter(log => {
              const p = Number(log.periode || log.Periode);
              return p <= matchPeriode && p >= limitMin;
          }).sort((a, b) => (Number(b.periode || b.Periode) - Number(a.periode || a.Periode)));

          const combinedData = [...top7, ...bottomLogs];
          
          return {
              ...matchWrapper,
              gridData: combinedData, // Pass full data to generator
              totalLogs: bottomLogs.length
          };
      });

      // Generate log grids (Simple formatting now)
      matchGrids = generateMatchGrids(allMatches);
  }

  // Generate log grids for each matched periode
  function generateMatchGrids(matches) {
      return matches.map((matchWrapper, index) => {
          const matchLog = matchWrapper.log;
          const matchType = matchWrapper.type;
          
          const matchPeriode = Number(matchLog.periode || matchLog.Periode);

          // Use pre-calculated data from runPatternAnalysis
          const gridData = matchWrapper.gridData || [];

          // Process into grid format (7 columns per row) using simple slicer
          const gridRows = processGridDataForMatch(gridData, matchPeriode);

          return {
              id: index,
              periode: matchPeriode,
              value: matchLog.formatted_result || matchLog.logResult,
              trend: matchLog.trend,
              type: matchType,
              rows: gridRows,
              totalLogs: matchWrapper.totalLogs
          };
      });
  }

  // Process grid data for a single match (similar to main grid but independent)
  function processGridDataForMatch(data, highlightPeriode, referenceLog = null) {
      if (!data || data.length === 0) return [];
      
      let rows = [];
      let tempBuffer = [];
      const COLS = 7;
      
      // Use referenceLog if provided, otherwise fallback to first log
      const firstLog = referenceLog || data[0];

      for (let i = 0; i < data.length; i++) {
          if (i > 140) break; // Limit for performance
          
          tempBuffer.push(data[i]);
          
          if (tempBuffer.length === COLS || i === data.length - 1) {
              const rowCells = tempBuffer.map(log => {
                  if (log.isPlaceholder) {
                      return {
                          periode: log.periode || '',
                          log: '#X',
                          trend: '',
                          style: log.style || '',
                          isHighlight: false,
                          isPlaceholder: true
                      };
                  }
                  
                  return {
                      periode: log.periode || log.Periode,
                      log: log.formatted_result || log.logResult || '----',
                      trend: log.trend || '',
                      style: checkLogIfSameData(log, firstLog),
                      isHighlight: (log.periode || log.Periode) === highlightPeriode,
                      isPlaceholder: false
                  };
              });
              rows.push(rowCells);
              tempBuffer = [];
          }
      }
      return rows;
  }

  function buildPCNListLocal(matchedLogs) {
      return matchedLogs.map(log => {
          const logPeriode = log.periode || log.Periode;
          const prevPeriode = logPeriode - 1;
          const nextPeriode = logPeriode + 1;

          const prevLog = logs.find(l => (l.periode || l.Periode) == prevPeriode);
          const nextLog = logs.find(l => (l.periode || l.Periode) == nextPeriode);

          return {
              prev: prevLog ? (prevLog.formatted_result || '----') : '----',
              prevTrend: prevLog ? (prevLog.trend || '') : '',
              current: log.formatted_result || '----',
              currentTrend: log.trend || '',
              next: nextLog ? (nextLog.formatted_result || '----') : '----',
              nextTrend: nextLog ? (nextLog.trend || '') : '',
              periode: logPeriode
          };
      }).slice(0, 15); // Limit to 15 rows
  }

  // --- Grid Logic Helpers ---

  function checkLogIfSameData(curr, compare) {
      if (!curr || !compare) return "";
      
      const c = getDigits(curr);
      const o = getDigits(compare);
      if (!c || !o) return "";

      if (c.as == o.as && c.kop == o.kop) return "border:2px solid orangered";
      if (c.as == o.kop && c.kop == o.kepala) return "border-bottom:2px solid orangered";
      if (c.as == o.kepala && c.kop == o.ekor) return "border-bottom:2px solid orangered";
      
      if (c.kop == o.as && c.kepala == o.kop) return "border-bottom:2px solid yellow";
      if (c.kop == o.kop && c.kepala == o.kepala) return "border:2px solid yellow";
      if (c.kop == o.kepala && c.kepala == o.ekor) return "border-bottom:2px solid yellow";

      if (c.kepala == o.as && c.ekor == o.kop) return "border-bottom:2px solid green";
      if (c.kepala == o.kop && c.ekor == o.kepala) return "border-bottom:2px solid green";
      if (c.kepala == o.kepala && c.ekor == o.ekor) return "border:2px solid green";

      return "";
  }

  function processGridData(data) {
      if (!data || data.length === 0) return [];
      
      currentLog = data[0]; 
      let rows = [];
      let tempBuffer = [];
      let baris = 0;

      for (let i = 0; i < data.length; i++) {
          if (i > 210) break;
          
          tempBuffer.push(data[i]);
          
          if (tempBuffer.length === 7) {
              let rowCells = [];
              for (let k = 0; k < tempBuffer.length; k++) {
                  let log = tempBuffer[k];
                  let style = checkLogIfSameData(currentLog, log);
                  
                  let isHighlight = false;
                  if ((baris == 3 && k == 0) || (baris == 8 && k == 0)) isHighlight = true;
                  else if ((baris == 4 && k == 1) || (baris == 9 && k == 1)) isHighlight = true;
                  else if (baris == 5 && k == 2) isHighlight = true;
                  else if (baris == 6 && k == 3) isHighlight = true;
                  else if (baris == 7 && k == 4) isHighlight = true;
                  else if (baris == 8 && k == 5) isHighlight = true;
                  else if (baris == 9 && k == 6) {
                      isHighlight = true;
                  }

                  let cellStyle = "cursor:pointer;";
                  if (style) cellStyle += style + ";";
                  if (isHighlight) cellStyle += "background-color:rgb(236, 230, 255) !important;color:black !important;font-weight:bold;";
                  
                  const logText = log.formatted_result || '????';
                  const logPeriode = log.periode || log.Periode || '---';

                  rowCells.push({ 
                      log: logText, 
                      trend: log.trend || '', 
                      periode: logPeriode, 
                      style: cellStyle,
                      isHighlight: isHighlight
                  });
              }
              
              rows.push(rowCells);

              if (baris == 9) baris = baris - 5;
              baris++;
              
              tempBuffer = [];
          }
      }
      return rows;
  }

  async function loadAnalysis() {
    if (!gameCode) return;
    loading = true;
    try {
        const res = await fetch(`${api.API_BASE_URL}/games/${gameCode}/analysis`);
        if (!res.ok) throw new Error("Failed to load analysis");
        logs = await res.json();
        gridRows = processGridData(logs);
        
        setTimeout(initChart, 0); 
    } catch (e) {
        console.error(e);
        if (e instanceof TypeError || e.message.includes('Failed to fetch')) {
             error = "Server Is Down Not Connected, please check the server application";
             addNotification('error', error);
        } else {
             error = e.message;
             // addNotification('error', error); // Optional, maybe too noisy if just initial load
        }
    } finally {
        loading = false;
    }
  }

  onMount(() => {
    if (!gameCode) {
         window.location.hash = ''; 
         return;
    }
    loadAnalysis();
  });

  onDestroy(() => {
      if (chartInstance) chartInstance.destroy();
  });
</script>

<div class="row">
    <div class="col-12 mb-3">
        <div class="d-flex align-items-center mb-0">
             <a href="#/" class="btn btn-outline-light btn-sm me-3 border-white-10 text-muted hover-white">
                 <i class="bi bi-arrow-left"></i>
             </a>
             <h4 class="mb-0 fw-bold text-white">Analysis: <span class="text-primary font-monospace">{gameCode}</span></h4>
             {#if selectedLog}
                 <span class="ms-3 badge bg-primary font-monospace fs-6 text-dark">{selectedLog.formatted_result}</span>
             {/if}
        </div>
    </div>

    <!-- LEFT: TradeX History Grid (col-md-5) -->
    <div class="col-md-5">
        <div class="card bg-dark-glass border-0 rounded-4 overflow-hidden mb-4">
             <div class="card-header bg-transparent border-white-10 text-white fw-bold small text-uppercase">
                 TradeX History
             </div>
             <div class="table-responsive p-0" style="max-height: 500px; overflow-y: auto;">
                 {#if loading}
                    <div class="text-center py-4 text-muted">Loading Grid...</div>
                 {:else if error}
                    <div class="text-center py-4 text-danger">{error}</div>
                 {:else}
                    <div class="log-grid">
                        {#each gridRows as row}
                            <div class="log-row">
                                {#each row as cell}
                                    <div 
                                        class="log-cell" 
                                        class:highlight={cell.isHighlight} 
style="{cell.style} {selectedCellPeriode === cell.periode ? `background-color: ${COLOR_STATES[selectedCellColor]} !important;` : ''}"
                                        on:click={() => handleLogClick(cell.periode)}
                                        role="button"
                                        tabindex="0"
                                        on:keypress={(e) => e.key === 'Enter' && handleLogClick(cell.periode)}
                                    >
                                        <div class="cell-content">
                                            {#if cell.trend === 'UP'}
                                                <i class="bi bi-caret-up-fill" style="color: #32CD32;"></i>
                                            {:else if cell.trend === 'DOWN'}
                                                <i class="bi bi-caret-down-fill" style="color: #DC143C;"></i>
                                            {/if}
                                            <span class="log-value" class:text-dark={cell.isHighlight || selectedCellPeriode === cell.periode}>{cell.log}</span>
                                        </div>
                                        <div class="log-periode" class:text-dark={cell.isHighlight || selectedCellPeriode === cell.periode}>{cell.periode}</div>
                                    </div>
                                {/each}
                            </div>
                        {/each}
                    </div>
                 {/if}
             </div>
        </div>
        <div class="card bg-dark-glass border-0 rounded-4 overflow-hidden mb-4">
             <div class="card-header bg-transparent border-white-8 text-white fw-bold text-uppercase">
                 <h3>🧠 Intrument Fix TradeX 💡</h3>
             </div>
        </div>

    </div>

    <!-- RIGHT: Analysis Tables (col-md-7) -->
    <div class="col-md-7">

        <!-- Chart Section -->
        <div class="card bg-dark-glass border-0 rounded-4 mb-4 p-3 overflow-hidden position-relative animate-fade-in" style="min-height: 200px;">
             <h6 class="text-white-50 mb-3 text-uppercase small ls-1">Signal Strength</h6>
             <div style="height: 150px; width: 100%;">
                 <canvas bind:this={chartCanvas}></canvas>
             </div>
        </div>

        <!-- Analysis Tables Row -->
        {#if selectedLog}
            <div class="row g-2 mb-4">
                <!-- Front Match Table -->
                <div class="col-4">
                    <div class="card bg-dark-glass border-0 rounded-3 overflow-hidden" style="border-left: 3px solid orangered !important;">
                        <div class="card-header bg-transparent py-2 px-2">
                            <small class="text-muted text-uppercase" style="font-size: 0.65rem;">Component F</small>
                        </div>
                        <div class="analysis-table-container">
                            <table class="table table-sm mb-0">
                                <thead>
                                    <tr class="text-muted" style="font-size: 0.6rem;">
                                        <th class="text-center">P</th>
                                        <th class="text-center">C</th>
                                        <th class="text-center">N</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each frontMatches as match}
                                        <tr>
                                            <td class="text-center font-monospace analysis-cell clickable-cell" on:click={() => copyDataToHitTextBox(match.prev)} role="button" tabindex="0" on:keypress={(e) => e.key === 'Enter' && copyDataToHitTextBox(match.prev)}>
                                                {#if match.prevTrend === 'UP'}<i class="bi bi-caret-up-fill trend-up"></i>{:else if match.prevTrend === 'DOWN'}<i class="bi bi-caret-down-fill trend-down"></i>{/if}
                                                <span class="cell-text">{match.prev}</span>
                                            </td>
                                            <td class="text-center font-monospace fw-bold analysis-cell clickable-cell" style="border: 1px solid orangered;" on:click={() => copyDataToHitTextBox(match.current)} role="button" tabindex="0" on:keypress={(e) => e.key === 'Enter' && copyDataToHitTextBox(match.current)}>
                                                {#if match.currentTrend === 'UP'}<i class="bi bi-caret-up-fill trend-up"></i>{:else if match.currentTrend === 'DOWN'}<i class="bi bi-caret-down-fill trend-down"></i>{/if}
                                                <span class="cell-text">{match.current}</span>
                                            </td>
                                            <td class="text-center font-monospace analysis-cell clickable-cell" on:click={() => copyDataToHitTextBox(match.next)} role="button" tabindex="0" on:keypress={(e) => e.key === 'Enter' && copyDataToHitTextBox(match.next)}>
                                                {#if match.nextTrend === 'UP'}<i class="bi bi-caret-up-fill trend-up"></i>{:else if match.nextTrend === 'DOWN'}<i class="bi bi-caret-down-fill trend-down"></i>{/if}
                                                <span class="cell-text">{match.next}</span>
                                            </td>
                                        </tr>
                                    {/each}
                                    {#if frontMatches.length === 0}
                                        <tr><td colspan="3" class="text-center text-muted" style="font-size: 0.65rem;">No matches</td></tr>
                                    {/if}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>

                <!-- Mid Match Table -->
                <div class="col-4">
                    <div class="card bg-dark-glass border-0 rounded-3 overflow-hidden" style="border-left: 3px solid yellow !important;">
                        <div class="card-header bg-transparent py-2 px-2">
                            <small class="text-muted text-uppercase" style="font-size: 0.65rem;">Component M</small>
                        </div>
                        <div class="analysis-table-container">
                            <table class="table table-sm mb-0">
                                <thead>
                                    <tr class="text-muted" style="font-size: 0.6rem;">
                                        <th class="text-center">P</th>
                                        <th class="text-center">C</th>
                                        <th class="text-center">N</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each midMatches as match}
                                        <tr>
                                            <td class="text-center font-monospace analysis-cell clickable-cell" on:click={() => copyDataToHitTextBox(match.prev)} role="button" tabindex="0" on:keypress={(e) => e.key === 'Enter' && copyDataToHitTextBox(match.prev)}>
                                                {#if match.prevTrend === 'UP'}<i class="bi bi-caret-up-fill trend-up"></i>{:else if match.prevTrend === 'DOWN'}<i class="bi bi-caret-down-fill trend-down"></i>{/if}
                                                <span class="cell-text">{match.prev}</span>
                                            </td>
                                            <td class="text-center font-monospace fw-bold analysis-cell clickable-cell" style="border: 1px solid yellow;" on:click={() => copyDataToHitTextBox(match.current)} role="button" tabindex="0" on:keypress={(e) => e.key === 'Enter' && copyDataToHitTextBox(match.current)}>
                                                {#if match.currentTrend === 'UP'}<i class="bi bi-caret-up-fill trend-up"></i>{:else if match.currentTrend === 'DOWN'}<i class="bi bi-caret-down-fill trend-down"></i>{/if}
                                                <span class="cell-text">{match.current}</span>
                                            </td>
                                            <td class="text-center font-monospace analysis-cell clickable-cell" on:click={() => copyDataToHitTextBox(match.next)} role="button" tabindex="0" on:keypress={(e) => e.key === 'Enter' && copyDataToHitTextBox(match.next)}>
                                                {#if match.nextTrend === 'UP'}<i class="bi bi-caret-up-fill trend-up"></i>{:else if match.nextTrend === 'DOWN'}<i class="bi bi-caret-down-fill trend-down"></i>{/if}
                                                <span class="cell-text">{match.next}</span>
                                            </td>
                                        </tr>
                                    {/each}
                                    {#if midMatches.length === 0}
                                        <tr><td colspan="3" class="text-center text-muted" style="font-size: 0.65rem;">No matches</td></tr>
                                    {/if}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>

                <!-- Back Match Table -->
                <div class="col-4">
                    <div class="card bg-dark-glass border-0 rounded-3 overflow-hidden" style="border-left: 3px solid lime !important;">
                        <div class="card-header bg-transparent py-2 px-2">
                            <small class="text-muted text-uppercase" style="font-size: 0.65rem;">Component B</small>
                        </div>
                        <div class="analysis-table-container">
                            <table class="table table-sm mb-0">
                                <thead>
                                    <tr class="text-muted" style="font-size: 0.6rem;">
                                        <th class="text-center">P</th>
                                        <th class="text-center">C</th>
                                        <th class="text-center">N</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each backMatches as match}
                                        <tr>
                                            <td class="text-center font-monospace analysis-cell clickable-cell" on:click={() => copyDataToHitTextBox(match.prev)} role="button" tabindex="0" on:keypress={(e) => e.key === 'Enter' && copyDataToHitTextBox(match.prev)}>
                                                {#if match.prevTrend === 'UP'}<i class="bi bi-caret-up-fill trend-up"></i>{:else if match.prevTrend === 'DOWN'}<i class="bi bi-caret-down-fill trend-down"></i>{/if}
                                                <span class="cell-text">{match.prev}</span>
                                            </td>
                                            <td class="text-center font-monospace fw-bold analysis-cell clickable-cell" style="border: 1px solid lime;" on:click={() => copyDataToHitTextBox(match.current)} role="button" tabindex="0" on:keypress={(e) => e.key === 'Enter' && copyDataToHitTextBox(match.current)}>
                                                {#if match.currentTrend === 'UP'}<i class="bi bi-caret-up-fill trend-up"></i>{:else if match.currentTrend === 'DOWN'}<i class="bi bi-caret-down-fill trend-down"></i>{/if}
                                                <span class="cell-text">{match.current}</span>
                                            </td>
                                            <td class="text-center font-monospace analysis-cell clickable-cell" on:click={() => copyDataToHitTextBox(match.next)} role="button" tabindex="0" on:keypress={(e) => e.key === 'Enter' && copyDataToHitTextBox(match.next)}>
                                                {#if match.nextTrend === 'UP'}<i class="bi bi-caret-up-fill trend-up"></i>{:else if match.nextTrend === 'DOWN'}<i class="bi bi-caret-down-fill trend-down"></i>{/if}
                                                <span class="cell-text">{match.next}</span>
                                            </td>
                                        </tr>
                                    {/each}
                                    {#if backMatches.length === 0}
                                        <tr><td colspan="3" class="text-center text-muted" style="font-size: 0.65rem;">No matches</td></tr>
                                    {/if}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        {:else}
            <div class="card bg-dark-glass border-0 rounded-4 p-4 mb-4 text-center">
                <i class="bi bi-hand-index-thumb text-muted fs-1 mb-2"></i>
                <p class="text-muted mb-0">Click a log cell to view pattern analysis</p>
            </div>
        {/if}

        <!-- Additional Info / Legend -->
        <div class="row g-3">
             <div class="col-4">
                 <div class="p-3 rounded-3 bg-dark-glass border border-white-10 h-100">
                     
                     {#if logs.length > 0}
                        <div class="d-flex align-items-end justify-content-between">
                            <small class="text-muted d-block mb-2">✨ TradeX Fix </small>
                            <h2 class="fw-black font-monospace mb-0 text-glow" style="font-size: 0.85rem;color:gray">{logs[0].formatted_result} %</h2>
                            <div class="d-flex align-items-center gap-2">
                                 <span class="badge bg-success bg-opacity-10 text-success border border-success">{logs[0].trend}</span>
                                 {#if logs[0].trend === 'UP'}
                                     <i class="bi bi-caret-up-fill" style="color: #32CD32; font-size: 1.5rem;"></i>
                                 {:else if logs[0].trend === 'DOWN'}
                                     <i class="bi bi-caret-down-fill" style="color: #DC143C; font-size: 1.5rem;"></i>
                                 {/if}
                            </div>
                        </div>
                     {:else}
                        <h2 class="text-muted">----</h2>
                     {/if}
                 </div>
             </div>
             <div class="col-8">
                 <div class="p-3 rounded-3 bg-dark-glass border border-white-10 h-100">
                    <input type="hidden" id="gameCode" bind:value={gameCode} />
                    <div class="d-flex gap-2 flex-wrap">
                        <small class="text-muted d-block mb-2">PATTERN MATCH TradeX</small>
                         <button class="badge" style="border: 1px solid orangered; background: transparent;" on:click={() => handleSavePattern('F Match', Tb3DA)}><i class="bi bi-caret-up-fill" style="color: #32CD32; font-size: 0.85rem;"></i> F Match</button>
                         <button class="badge" style="border: 1px solid green; background: transparent; color: lime;" on:click={() => handleSavePattern('AKE Match', Tb3DB)}><i class="bi bi-caret-down-fill" style="color: #DC143C; font-size: 0.85rem;"></i> AKE Match</button>
                         <button class="badge" style="border: 1px solid darkturquoise; background: transparent; color: dodgerblue;" on:click={() => handleSavePattern('AKpE Match', Tb3DC)}><i class="bi bi-caret-down-fill" style="color: #DC143C; font-size: 0.85rem;"></i> AKpE Match</button>
                         <button class="badge" style="border: 1px solid yellow; background: transparent; color: yellow;" on:click={() => handleSavePattern('B Match', Tb3DD)}><i class="bi bi-caret-up-fill" style="color: #32CD32; font-size: 0.85rem;"></i> B Match</button>
                     </div>
                 </div>
             </div>
        </div>

        <div class="row g-3" style="padding-top: 10px;">
            <!-- Hit Textboxes (same IDs as legacy) -->
            <div class="col-md-3">
                <div class="input-group input-group-sm">
                    <span class="input-group-text bg-dark-glass border-secondary text-muted" style="font-size: 0.7rem;">F</span>
                    <input type="text" id="Tb3DA" class="form-control bg-dark border-secondary text-white font-monospace text-center" style="font-size: 0.8rem;" bind:value={Tb3DA} on:input={() => Tb3DA = format3Digit(Tb3DA)} />
                    <i class="bi bi-caret-up-fill" style="color: #32CD32; font-size: 1.5rem;"></i>
                </div>
            </div>
            <div class="col-md-3">
                <div class="input-group input-group-sm">
                    <span class="input-group-text bg-dark-glass border-secondary text-muted" style="font-size: 0.7rem;">AKE</span>
                    <input type="text" id="Tb3DB" class="form-control bg-dark border-secondary text-white font-monospace text-center" style="font-size: 0.8rem;" bind:value={Tb3DB} on:input={() => Tb3DB = format3Digit(Tb3DB)} />
                    <i class="bi bi-caret-down-fill" style="color: #DC143C; font-size: 1.5rem;"></i>
                </div>
            </div>
            <div class="col-md-3">
                <div class="input-group input-group-sm">
                    <span class="input-group-text bg-dark-glass border-secondary text-muted" style="font-size: 0.7rem;">AKpE</span>
                    <input type="text" id="Tb3DC" class="form-control bg-dark border-secondary text-white font-monospace text-center" style="font-size: 0.8rem;" bind:value={Tb3DC} on:input={() => Tb3DC = format3Digit(Tb3DC)} />
                    <i class="bi bi-caret-down-fill" style="color: #DC143C; font-size: 1.5rem;"></i>
                </div>
            </div>
            <div class="col-md-3">
                <div class="input-group input-group-sm">
                    <span class="input-group-text bg-dark-glass border-secondary text-muted" style="font-size: 0.7rem;">B</span>
                    <input type="text" id="Tb3DD" class="form-control bg-dark border-secondary text-white font-monospace text-center" style="font-size: 0.8rem;" bind:value={Tb3DD} on:input={() => Tb3DD = format3Digit(Tb3DD)} />
                    <i class="bi bi-caret-up-fill" style="color: #32CD32; font-size: 1.5rem;"></i>
                </div>
            </div>
        </div>
    </div>


    <!-- Tab Navigation -->
    <div class="col-12 mb-4 animate-fade-in">
        <ul class="nav nav-pills nav-fill bg-dark bg-opacity-50 rounded-pill p-1 shadow-sm border border-secondary border-opacity-25" style="max-width: 600px; margin: 0 auto;">
            <li class="nav-item">
                <a class="nav-link rounded-pill py-2 text-uppercase small fw-bold ls-1 transition-all {activeAnalysisTab === 'match-grids' ? 'active shadow bg-primary' : 'text-dark-50 hover-text-dark'}" 
                   href={"#"} 
                   on:click|preventDefault={() => activeAnalysisTab = 'match-grids'}>
                    <i class="bi bi-grid-3x3 me-2"></i>TradeX Matches
                </a>
            </li>
            <li class="nav-item">
                <a class="nav-link rounded-pill py-2 text-uppercase small fw-bold ls-1 transition-all {activeAnalysisTab === 'tradex-analysis' ? 'active shadow bg-primary' : 'text-dark-50 hover-text-dark'}" 
                   href={"#"} 
                   on:click|preventDefault={() => activeAnalysisTab = 'tradex-analysis'}>
                    <i class="bi bi-graph-up me-2"></i>Analisis TradeX
                </a>
            </li>
            <li class="nav-item">
                <a class="nav-link rounded-pill py-2 text-uppercase small fw-bold ls-1 transition-all {activeAnalysisTab === 'history-analysis' ? 'active shadow bg-primary' : 'text-dark-50 hover-text-dark'}" 
                   href={"#"} 
                   on:click|preventDefault={() => activeAnalysisTab = 'history-analysis'}>
                    <i class="bi bi-clock-history me-2"></i>Analisis Historis
                </a>
            </li>
        </ul>
    </div>

    <!-- TAB 1: Dynamic Log Grids for 3-Digit Matches -->
    {#if activeAnalysisTab === 'match-grids'}
        <div class="col-12 mb-3 animate-fade-in">
            {#if selectedLog && matchGrids.length > 0}
                <div class="mb-3">
                    <h6 class="text-white-50 text-uppercase small ls-1 mb-2">
                        <i class="bi bi-grid-3x3 me-2"></i>
                        TradeX Match History ({matchGrids.length} matches)
                    </h6>
                </div>
                <div class="row g-3">
                    {#each matchGrids as grid}
                        <div class="col-md-6 col-lg-4">
                            <div class="card bg-dark-glass border-0 rounded-3 overflow-hidden" 
                                 style="border-top: 3px solid {grid.type === 'front' ? '#FF6B35' : grid.type === 'back' ? '#00D4AA' : '#A855F7'} !important;">
                                <div class="card-header bg-transparent py-2 px-3 d-flex justify-content-between align-items-center">
                                    <small class="text-muted text-uppercase" style="font-size: 0.65rem;">
                                        {#if grid.type === 'front'}
                                            <i class="bi bi-arrow-right me-1" style="color: #FF6B35;"></i>Component Fx
                                        {:else if grid.type === 'back'}
                                            <i class="bi bi-arrow-left me-1" style="color: #00D4AA;"></i>Component Bx
                                        {:else}
                                            <i class="bi bi-arrow-left-right me-1" style="color: #A855F7;"></i>Mix
                                        {/if}
                                        <span class="font-monospace ms-1" style="color: #9ca3af;">{grid.value}</span>
                                    </small>
                                    <div class="d-flex align-items-center gap-2">
                                        {#if grid.trend === 'UP'}
                                            <i class="bi bi-caret-up-fill" style="color: #32CD32; font-size: 0.65rem;"></i>
                                        {:else if grid.trend === 'DOWN'}
                                            <i class="bi bi-caret-down-fill" style="color: #DC143C; font-size: 0.65rem;"></i>
                                        {/if}
                                        <span class="badge bg-dark text-muted" style="font-size: 0.6rem;">P:{grid.periode}</span>
                                    </div>
                                </div>
                                <div class="card-body p-0" style="max-height: 250px; overflow-y: auto;">
                                    <div class="match-log-grid">
                                        {#each grid.rows as row}
                                            <div class="match-log-row">
                                                {#each row as cell}
                                                    <div class="match-log-cell" 
                                                         class:highlight={cell.isHighlight}
                                                         class:selected={matchGridCellSelected === `${grid.id}-${cell.periode}`}
                                                         style="{cell.style}"
                                                         title="Periode: {cell.periode}"
                                                         on:click={() => handleMatchCellClick(cell, grid.id)}
                                                         role="button"
                                                         tabindex="0"
                                                         on:keypress={(e) => e.key === 'Enter' && handleMatchCellClick(cell, grid.id)}>
                                                        <div class="cell-content">
                                                            {#if cell.trend === 'UP'}
                                                                <i class="bi bi-caret-up-fill" style="color: #32CD32; font-size: 0.65rem;"></i>
                                                            {:else if cell.trend === 'DOWN'}
                                                                <i class="bi bi-caret-down-fill" style="color: #DC143C; font-size: 0.65rem;"></i>
                                                            {/if}
                                                            <span class="log-value" class:text-dark={cell.isHighlight || matchGridCellSelected === `${grid.id}-${cell.periode}`}>{cell.log || '#X'}</span>
                                                        </div>
                                                    </div>
                                                {/each}
                                            </div>
                                        {/each}
                                    </div>
                                </div>
                            </div>
                        </div>
                    {/each}
                </div>
            {:else if selectedLog}
                <div class="text-center py-4 text-muted" style="font-size: 0.8rem;">
                    <i class="bi bi-search me-2"></i>No 3-digit matches found
                </div>
            {/if}
        </div>
    {/if}

    <!-- TAB 2: Analisis TradeX (Frequency Analysis) -->
    {#if activeAnalysisTab === 'tradex-analysis'}
        <div class="col-12 mb-5 animate-fade-in">
            <FrequencyAnalysisWidget {gameCode} />
        </div>
    {/if}

    <!-- TAB 3: Analisis Historis -->
    {#if activeAnalysisTab === 'history-analysis'}
        <div class="col-12 mb-5 animate-fade-in">
            {#if historyLoading}
                <div class="text-center py-5">
                    <div class="spinner-border text-primary" role="status">
                        <span class="visually-hidden">Loading...</span>
                    </div>
                    <p class="mt-2 text-muted small">Menganalisa data historis...</p>
                </div>
            {:else}
                <HistoryAnalysisList {historyItems} />
            {/if}
        </div>
    {/if}

</div>


<FloatingControlPanel {gameCode} on:savePattern={handleFloatingPanelSave} />

<style>
  .hover-white:hover {
      color: #fff !important;
      border-color: rgba(255,255,255,0.3) !important;
  }
  .bg-dark-glass {
    background: rgba(0,0,0,0.4);
    backdrop-filter: blur(5px);
  }
  .ls-1 { letter-spacing: 1px; }
  .text-glow { text-shadow: 0 0 10px rgba(0,255,65,0.5); }
  .fw-black { font-weight: 900; }

  /* Grid Layout Styles */
  .log-grid {
      display: flex;
      flex-direction: column;
      padding: 0.5rem;
      gap: 2px;
  }
  .log-row {
      display: flex;
      gap: 2px;
  }
  .log-cell {
      flex: 1;
      background: rgba(255,255,255,0.03);
      padding: 6px 4px;
      text-align: center;
      cursor: pointer;
      transition: background 0.2s;
      border-radius: 2px;
  }
  .log-cell:hover {
      background: rgba(255,255,255,0.12);
  }
  .cell-content {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 2px;
  }
  .log-value {
      color: #9ca3af;
      font-family: 'Courier New', monospace;
      font-size: 0.8rem;
      font-weight: 600;
  }
  .log-periode {
      color: #9ca3af;
      font-size: 0.65rem;
      margin-top: 2px;
      font-family: sans-serif;
  }
  
  /* Highlight overrides */
  .text-dark {
      color: #1f2937 !important;
  }


  /* Analysis Tables */
  .analysis-table-container {
      max-height: 250px;
      overflow-y: auto;
  }
  .analysis-table-container table {
      background: transparent;
  }
  .analysis-table-container th,
  .analysis-table-container td {
      background: transparent;
      border: none;
      padding: 4px 2px;
  }
  .analysis-cell {
      font-size: 0.7rem;
  }
  .analysis-cell .cell-text {
      color: #9ca3af;
  }
  .analysis-cell .trend-up {
      color: #32CD32;
      font-size: 0.6rem;
      margin-right: 2px;
  }
  .analysis-cell .trend-down {
      color: #DC143C;
      font-size: 0.6rem;
      margin-right: 2px;
  }

  /* Match Log Grids (Dynamic 3-Digit Tables) */
  .match-log-grid {
      display: flex;
      flex-direction: column;
      padding: 0.25rem;
      gap: 1px;
  }
  .match-log-row {
      display: flex;
      gap: 1px;
  }
  .match-log-cell {
      flex: 1;
      padding: 3px 2px;
      text-align: center;
      background: rgba(255,255,255,0.03);
      border-radius: 2px;
      cursor: pointer;
      transition: background 0.15s ease;
  }
  .clickable-cell {
      cursor: pointer !important;
  }
  .clickable-cell:hover {
      background: rgba(255,255,255,0.1) !important;
  }
  .match-log-cell:hover {
      background: rgba(255,255,255,0.08);
  }
  .match-log-cell.highlight {
      background-color: rgb(236, 230, 255) !important;
  }
  .match-log-cell.selected {
      background-color: rgb(136, 205, 208) !important;
  }
  .match-log-cell .cell-content {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 1px;
  }
  .match-log-cell .log-value {
      font-size: 0.65rem;
      color: #9ca3af;
  }
</style>
