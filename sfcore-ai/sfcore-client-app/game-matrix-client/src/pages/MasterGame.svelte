<script>
    import { onMount } from 'svelte';
    import { fade, slide } from 'svelte/transition';
    import { fetchMasterGames, createMasterGame, updateMasterGame, deleteMasterGame } from '../lib/api';
    import Loader from '../components/Loader.svelte';

    let games = [];
    let loading = true;
    let error = null;
    let showModal = false;
    let editingGame = null;

    // Form State
    let formData = {
        id: 0,
        game_code: '',
        game_name: '',
        game_hour: 0,
        game_minute: 0,
        start_bet_hour: 0,
        start_bet_minute: 0,
        last_result: '',
        last_periode_in_real_game: 0,
        date_result: '',
        input_result_date: '',
        holiday: ''
    };

    async function loadGames() {
        loading = true;
        try {
            games = await fetchMasterGames();
        } catch (e) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    function openModal(game = null) {
        if (game) {
            editingGame = game;
            formData = { ...game };
        } else {
            editingGame = null;
            formData = {
                id: 0,
                game_code: '',
                game_name: '',
                game_hour: 0,
                game_minute: 0,
                start_bet_hour: 0,
                start_bet_minute: 0,
                last_result: '',
                last_periode_in_real_game: 0,
                date_result: '',
                input_result_date: '',
                holiday: ''
            };
        }
        showModal = true;
    }

    async function handleSubmit() {
        try {
            if (editingGame) {
                await updateMasterGame(formData);
            } else {
                await createMasterGame(formData);
            }
            showModal = false;
            loadGames();
        } catch (e) {
            alert('Error: ' + e.message);
        }
    }

    async function handleDelete(id) {
        if (!confirm('Are you sure you want to delete this game?')) return;
        try {
            await deleteMasterGame(id);
            loadGames();
        } catch (e) {
            alert('Error: ' + e.message);
        }
    }

    onMount(loadGames);
</script>

<div class="container-fluid py-4">
    <div class="d-flex justify-content-between align-items-center mb-4">
        <div>
            <h4 class="mb-0 fw-bold text-white">Master Game Management</h4>
            <p class="text-white-50 small mb-0">Configure game schedules and basic settings</p>
        </div>
        <button class="btn btn-primary rounded-pill px-4 shadow-sm" on:click={() => openModal()}>
            <i class="bi bi-plus-lg me-1"></i> Add New Game
        </button>
    </div>

    {#if loading}
        <div class="d-flex justify-content-center py-5">
            <Loader text="LOADING GAMES..." />
        </div>
    {:else if error}
        <div class="alert alert-danger border-0 shadow-sm" role="alert">
            <i class="bi bi-exclamation-triangle-fill me-2"></i> {error}
        </div>
    {:else}
        <div class="card bg-dark border-white-10 shadow-sm overflow-hidden">
            <div class="table-responsive">
                <table class="table table-dark table-hover mb-0 align-middle">
                    <thead class="bg-black bg-opacity-50">
                        <tr>
                            <th class="ps-4">Game Code</th>
                            <th>Game Name</th>
                            <th>Draw Time</th>
                            <th>Bet Starts</th>
                            <th>Last Result</th>
                            <th>Holiday</th>
                            <th class="text-end pe-4">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each games as game}
                            <tr in:fade>
                                <td class="ps-4 fw-bold text-primary">{game.game_code}</td>
                                <td>{game.game_name}</td>
                                <td>{String(game.game_hour).padStart(2, '0')}:{String(game.game_minute).padStart(2, '0')}</td>
                                <td>{String(game.start_bet_hour).padStart(2, '0')}:{String(game.start_bet_minute).padStart(2, '0')}</td>
                                <td>
                                    {#if game.last_result}
                                        <span class="badge bg-secondary bg-opacity-25 text-info">{game.last_result}</span>
                                    {:else}
                                        <span class="text-white-50 small">-</span>
                                    {/if}
                                </td>
                                <td>
                                    {#if game.holiday && game.holiday !== ''}
                                        <span class="text-warning small">{game.holiday}</span>
                                    {:else}
                                        <span class="text-success small">None</span>
                                    {/if}
                                </td>
                                <td class="text-end pe-4">
                                    <button class="btn btn-sm btn-link text-info me-2" on:click={() => openModal(game)} title="Edit Game">
                                        <i class="bi bi-pencil-square"></i>
                                    </button>
                                    <button class="btn btn-sm btn-link text-danger" on:click={() => handleDelete(game.id)} title="Delete Game">
                                        <i class="bi bi-trash3"></i>
                                    </button>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </div>
    {/if}
</div>

<!-- Modal -->
{#if showModal}
    <div class="modal d-block" tabindex="-1" in:fade={{duration: 200}} out:fade={{duration: 150}}>
        <div class="modal-backdrop bg-black bg-opacity-75 position-fixed top-0 start-0 w-100 h-100" on:click={() => showModal = false} on:keydown={(e) => e.key === 'Escape' && (showModal = false)} role="button" aria-label="Close Modal" tabindex="-1"></div>
        <div class="modal-dialog modal-lg modal-dialog-centered position-relative z-3">
            <div class="modal-content bg-dark border-secondary shadow-lg">
                <div class="modal-header border-white-10 pb-2">
                    <h5 class="modal-title text-white fw-bold">
                        {editingGame ? 'Edit Master Game' : 'Add New Master Game'}
                    </h5>
                    <button type="button" class="btn-close btn-close-white" on:click={() => showModal = false} aria-label="Close"></button>
                </div>
                <div class="modal-body py-3">
                    <form on:submit|preventDefault={handleSubmit}>
                        <div class="row g-3">
                            <div class="col-md-6">
                                <label for="game_code" class="form-label small text-white-50">Game Code</label>
                                <input id="game_code" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.game_code} required>
                            </div>
                            <div class="col-md-6">
                                <label for="game_name" class="form-label small text-white-50">Game Name</label>
                                <input id="game_name" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.game_name} required>
                            </div>
                            
                            <div class="col-md-3">
                                <label for="game_hour" class="form-label small text-white-50">Draw Hour (0-23)</label>
                                <input id="game_hour" type="number" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.game_hour} min="0" max="23">
                            </div>
                            <div class="col-md-3">
                                <label for="game_minute" class="form-label small text-white-50">Draw Minute (0-59)</label>
                                <input id="game_minute" type="number" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.game_minute} min="0" max="59">
                            </div>
                            <div class="col-md-3">
                                <label for="start_bet_hour" class="form-label small text-white-50">Start Bet Hour</label>
                                <input id="start_bet_hour" type="number" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.start_bet_hour} min="0" max="23">
                            </div>
                            <div class="col-md-3">
                                <label for="start_bet_minute" class="form-label small text-white-50">Start Bet Minute</label>
                                <input id="start_bet_minute" type="number" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.start_bet_minute} min="0" max="59">
                            </div>

                            <div class="col-md-6">
                                <label for="last_result" class="form-label small text-white-50">Last Result (Manual Sync)</label>
                                <input id="last_result" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.last_result}>
                            </div>
                            <div class="col-md-6">
                                <label for="last_periode" class="form-label small text-white-50">Last Periode</label>
                                <input id="last_periode" type="number" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.last_periode_in_real_game}>
                            </div>

                            <div class="col-md-12">
                                <label for="holiday" class="form-label small text-white-50">Holidays (JSON/Comma string)</label>
                                <input id="holiday" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.holiday} placeholder="Example: Sunday, Monday">
                            </div>
                        </div>

                        <div class="mt-4 text-end">
                            <button type="button" class="btn btn-ghost btn-sm text-white-50 me-2" on:click={() => showModal = false}>Cancel</button>
                            <button type="submit" class="btn btn-primary btn-sm px-4 rounded-pill shadow">Save Changes</button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    .modal-backdrop {
        z-index: 1050;
    }
    .modal-dialog {
        z-index: 1060;
    }
    .border-white-10 {
        border-color: rgba(255, 255, 255, 0.1) !important;
    }
    .table th {
        font-size: 0.7rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        font-weight: 600;
        border-top: none;
    }
    .table td {
        font-size: 0.85rem;
        border-color: rgba(255, 255, 255, 0.05);
    }
    .form-control:focus {
        background-color: black;
        border-color: var(--bs-primary);
        color: white;
        box-shadow: 0 0 0 0.25rem rgba(var(--bs-primary-rgb), 0.1);
    }
    .btn-ghost:hover {
        background: rgba(255,255,255,0.05);
    }
</style>
