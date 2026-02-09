<script>
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { fetchLinkGames, createLinkGame, updateLinkGame, deleteLinkGame } from '../lib/api';
    import Loader from '../components/Loader.svelte';

    let links = [];
    let loading = true;
    let error = null;
    let showModal = false;
    let editingLink = null;

    // Form State
    let formData = {
        id: 0,
        link_game: '',
        link_type: '',
        game_code: ''
    };

    async function loadLinks() {
        loading = true;
        try {
            links = await fetchLinkGames();
        } catch (e) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    function openModal(link = null) {
        if (link) {
            editingLink = link;
            formData = { ...link };
        } else {
            editingLink = null;
            formData = {
                id: 0,
                link_game: '',
                link_type: '',
                game_code: ''
            };
        }
        showModal = true;
    }

    async function handleSubmit() {
        try {
            if (editingLink) {
                await updateLinkGame(formData);
            } else {
                await createLinkGame(formData);
            }
            showModal = false;
            loadLinks();
        } catch (e) {
            alert('Error: ' + e.message);
        }
    }

    async function handleDelete(id) {
        if (!confirm('Are you sure you want to delete this link game?')) return;
        try {
            await deleteLinkGame(id);
            loadLinks();
        } catch (e) {
            alert('Error: ' + e.message);
        }
    }

    onMount(loadLinks);
</script>

<div class="container-fluid py-4">
    <div class="d-flex justify-content-between align-items-center mb-4">
        <div>
            <h4 class="mb-0 fw-bold text-white">Setup Link Game</h4>
            <p class="text-white-50 small mb-0">Map specific game codes to play links and types</p>
        </div>
        <button class="btn btn-primary rounded-pill px-4 shadow-sm" on:click={() => openModal()}>
            <i class="bi bi-link me-1"></i> Add New Link
        </button>
    </div>

    {#if loading}
        <div class="d-flex justify-content-center py-5">
            <Loader text="LOADING LINKS..." />
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
                            <th>Link Type</th>
                            <th>Target URL</th>
                            <th class="text-end pe-4">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each links as link}
                            <tr in:fade>
                                <td class="ps-4 fw-bold text-primary">{link.game_code}</td>
                                <td><span class="badge bg-info bg-opacity-10 text-info border border-info border-opacity-25 px-2">{link.link_type}</span></td>
                                <td>
                                    <div class="text-truncate text-white-50 small" style="max-width: 400px;">
                                        {link.link_game}
                                    </div>
                                </td>
                                <td class="text-end pe-4">
                                    <button class="btn btn-sm btn-link text-info me-2" on:click={() => openModal(link)} title="Edit Link">
                                        <i class="bi bi-pencil-square"></i>
                                    </button>
                                    <button class="btn btn-sm btn-link text-danger" on:click={() => handleDelete(link.id)} title="Delete Link">
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
    <div class="modal d-block" tabindex="-1" in:fade={{duration: 200}}>
        <div class="modal-backdrop bg-black bg-opacity-75 position-fixed top-0 start-0 w-100 h-100" on:click={() => showModal = false} on:keydown={(e) => e.key === 'Escape' && (showModal = false)} role="button" aria-label="Close Modal" tabindex="-1"></div>
        <div class="modal-dialog modal-dialog-centered position-relative z-3">
            <div class="modal-content bg-dark border-secondary shadow-lg">
                <div class="modal-header border-white-10 pb-2">
                    <h5 class="modal-title text-white fw-bold">
                        {editingLink ? 'Edit Link Game' : 'Add New Link Game'}
                    </h5>
                    <button type="button" class="btn-close btn-close-white" on:click={() => showModal = false} aria-label="Close"></button>
                </div>
                <div class="modal-body py-3">
                    <form on:submit|preventDefault={handleSubmit}>
                        <div class="row g-3">
                            <div class="col-12">
                                <label for="game_code" class="form-label small text-white-50">Game Code</label>
                                <input id="game_code" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.game_code} placeholder="Example: HK, SYD, SGP" required>
                            </div>
                            <div class="col-12">
                                <label for="link_type" class="form-label small text-white-50">Link Type</label>
                                <input id="link_type" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.link_type} placeholder="Example: 4D, 3D, Colok" required>
                            </div>
                            <div class="col-12">
                                <label for="link_game" class="form-label small text-white-50">Target URL / Link Path</label>
                                <input id="link_game" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.link_game} required>
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
</style>
