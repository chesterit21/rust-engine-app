export const API_BASE_URL = '/api';

export async function login(email, password) {
  const res = await fetch(`${API_BASE_URL}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  });
  
  if (!res.ok) {
    const err = await res.json();
    throw new Error(err.error || 'Login failed');
  }
  return res.json();
}

export async function fetchGames() {
    const res = await fetch('/api/games');
    if (!res.ok) throw new Error('Failed to fetch games');
    return res.json();
}

export async function fetchDashboardGames() {
    const res = await fetch('/api/games/dashboard');
    if (!res.ok) throw new Error('Failed to fetch dashboard games');
    return res.json();
}

export async function fetchGame(code, page = 1) {
    const res = await fetch(`/api/games/${code}?page=${page}`);
    if (!res.ok) throw new Error('Failed to fetch game');
    return res.json();
}

export async function placeBet(gameCode, betData) {
    const res = await fetch(`/api/games/${gameCode}/play`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(betData),
    });
    if (!res.ok) {
        const err = await res.json();
        throw new Error(err.error?.message || 'Failed to place bet');
    }
    return res.json();
}

export async function fetchHistorySummary() {
    const res = await fetch(`${API_BASE_URL}/history/summary`);
    if (!res.ok) throw new Error('Failed to fetch history summary');
    return res.json();
}

export async function deleteHistoryByGameCode(gameCode) {
    const res = await fetch(`${API_BASE_URL}/history/${gameCode}`, {
        method: 'DELETE',
    });
    if (!res.ok) {
        const err = await res.json();
        throw new Error(err.error?.message || 'Failed to delete history');
    }
    return res.json();
}

export async function resetAllHistory() {
    const res = await fetch(`${API_BASE_URL}/history/reset-all`, {
        method: 'DELETE',
    });
    if (!res.ok) {
        const err = await res.json();
        throw new Error(err.error?.message || 'Failed to reset all history');
    }
    return res.json();
}

export async function fetchFrequencyAnalysis(gameCode, windowSize) {
    try {
        const response = await fetch(`${API_BASE_URL}/games/${gameCode}/frequency?window_size=${windowSize}`);
        if (!response.ok) throw new Error('Failed to fetch frequency analysis');
        return await response.json();
    } catch (error) {
        console.error('Error fetching frequency analysis:', error);
        throw error;
    }
}

export async function fetchHistoryAnalysis(gameCode, windowSize = 100, depth = 7) {
    try {
        const response = await fetch(`${API_BASE_URL}/games/${gameCode}/history-analysis?window_size=${windowSize}&depth=${depth}`);
        if (!response.ok) throw new Error('Failed to fetch history analysis');
        return await response.json();
    } catch (error) {
        console.error('Error fetching history analysis:', error);
        throw error;
    }
}

export async function fetchMissingNumbers(transCode) {
    try {
        const response = await fetch(`${API_BASE_URL}/history/${transCode}/missing`);
        if (!response.ok) throw new Error('Failed to fetch missing numbers');
        return await response.json();
    } catch (error) {
        console.error('Error fetching missing numbers:', error);
        throw error;
    }
}

// --- Setup API ---

// Generic Helper
async function setupFetch(url, method = 'GET', body = null) {
    const options = {
        method,
        headers: { 'Content-Type': 'application/json' },
    };
    if (body) options.body = JSON.stringify(body);
    
    const res = await fetch(url, options);
    if (!res.ok) {
        const err = await res.json();
        throw new Error(err.error?.message || `Failed to ${method} ${url}`);
    }
    return res.json();
}

// Master Game
export const fetchMasterGames = () => setupFetch(`${API_BASE_URL}/setup/master-game`);
export const createMasterGame = (data) => setupFetch(`${API_BASE_URL}/setup/master-game`, 'POST', data);
export const updateMasterGame = (data) => setupFetch(`${API_BASE_URL}/setup/master-game`, 'PUT', data);
export const deleteMasterGame = (id) => setupFetch(`${API_BASE_URL}/setup/master-game/${id}`, 'DELETE');

// Member Game
export const fetchMemberGames = () => setupFetch(`${API_BASE_URL}/setup/member-game`);
export const createMemberGame = (data) => setupFetch(`${API_BASE_URL}/setup/member-game`, 'POST', data);
export const updateMemberGame = (data) => setupFetch(`${API_BASE_URL}/setup/member-game`, 'PUT', data);
export const deleteMemberGame = (id) => setupFetch(`${API_BASE_URL}/setup/member-game/${id}`, 'DELETE');

// Site Master
export const fetchSiteMasters = () => setupFetch(`${API_BASE_URL}/setup/site-master`);
export const createSiteMaster = (data) => setupFetch(`${API_BASE_URL}/setup/site-master`, 'POST', data);
export const updateSiteMaster = (data) => setupFetch(`${API_BASE_URL}/setup/site-master`, 'PUT', data);
export const deleteSiteMaster = (id) => setupFetch(`${API_BASE_URL}/setup/site-master/${id}`, 'DELETE');

// Link Game
export const fetchLinkGames = () => setupFetch(`${API_BASE_URL}/setup/link-game`);
export const createLinkGame = (data) => setupFetch(`${API_BASE_URL}/setup/link-game`, 'POST', data);
export const updateLinkGame = (data) => setupFetch(`${API_BASE_URL}/setup/link-game`, 'PUT', data);
export const deleteLinkGame = (id) => setupFetch(`${API_BASE_URL}/setup/link-game/${id}`, 'DELETE');
