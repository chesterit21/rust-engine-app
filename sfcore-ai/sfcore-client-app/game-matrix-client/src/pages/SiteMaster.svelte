<script>
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { fetchSiteMasters, createSiteMaster, updateSiteMaster, deleteSiteMaster } from '../lib/api';
    import Loader from '../components/Loader.svelte';

    let sites = [];
    let loading = true;
    let error = null;
    let showModal = false;
    let editingSite = null;

    // Form State
    let formData = {
        id: 0,
        group_name: '',
        provider_name: '',
        link_site: ''
    };

    async function loadSites() {
        loading = true;
        try {
            sites = await fetchSiteMasters();
        } catch (e) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    function openModal(site = null) {
        if (site) {
            editingSite = site;
            formData = { ...site };
        } else {
            editingSite = null;
            formData = {
                id: 0,
                group_name: '',
                provider_name: '',
                link_site: ''
            };
        }
        showModal = true;
    }

    async function handleSubmit() {
        try {
            if (editingSite) {
                await updateSiteMaster(formData);
            } else {
                await createSiteMaster(formData);
            }
            showModal = false;
            loadSites();
        } catch (e) {
            alert('Error: ' + e.message);
        }
    }

    async function handleDelete(id) {
        if (!confirm('Are you sure you want to delete this site master?')) return;
        try {
            await deleteSiteMaster(id);
            loadSites();
        } catch (e) {
            alert('Error: ' + e.message);
        }
    }

    onMount(loadSites);
</script>

<div class="container-fluid py-4">
    <div class="d-flex justify-content-between align-items-center mb-4">
        <div>
            <h4 class="mb-0 fw-bold text-white">Site Master Management</h4>
            <p class="text-white-50 small mb-0">Manage provider connections and site links</p>
        </div>
        <button class="btn btn-primary rounded-pill px-4 shadow-sm" on:click={() => openModal()}>
            <i class="bi bi-plus-square-fill me-1"></i> Add New Site
        </button>
    </div>

    {#if loading}
        <div class="d-flex justify-content-center py-5">
            <Loader text="LOADING SITES..." />
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
                            <th class="ps-4">Group Name</th>
                            <th>Provider</th>
                            <th>Link Site</th>
                            <th class="text-end pe-4">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each sites as site}
                            <tr in:fade>
                                <td class="ps-4 fw-bold text-primary">{site.group_name}</td>
                                <td><span class="badge bg-secondary bg-opacity-25 text-white fw-normal px-2">{site.provider_name}</span></td>
                                <td>
                                    <a href={site.link_site} target="_blank" class="text-white-50 small text-decoration-none hover-text-primary">
                                        {site.link_site} <i class="bi bi-box-arrow-up-right ms-1 tiny"></i>
                                    </a>
                                </td>
                                <td class="text-end pe-4">
                                    <button class="btn btn-sm btn-link text-info me-2" on:click={() => openModal(site)} title="Edit Site">
                                        <i class="bi bi-pencil-square"></i>
                                    </button>
                                    <button class="btn btn-sm btn-link text-danger" on:click={() => handleDelete(site.id)} title="Delete Site">
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
                        {editingSite ? 'Edit Site Master' : 'Add New Site Master'}
                    </h5>
                    <button type="button" class="btn-close btn-close-white" on:click={() => showModal = false} aria-label="Close"></button>
                </div>
                <div class="modal-body py-3">
                    <form on:submit|preventDefault={handleSubmit}>
                        <div class="row g-3">
                            <div class="col-12">
                                <label for="group_name" class="form-label small text-white-50">Group Name</label>
                                <input id="group_name" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.group_name} required>
                            </div>
                            <div class="col-12">
                                <label for="provider_name" class="form-label small text-white-50">Provider Name</label>
                                <input id="provider_name" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.provider_name} required>
                            </div>
                            <div class="col-12">
                                <label for="link_site" class="form-label small text-white-50">Link Site</label>
                                <input id="link_site" type="url" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.link_site} placeholder="https://..." required>
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
    .hover-text-primary:hover {
        color: var(--bs-primary) !important;
    }
    .tiny {
        font-size: 0.7rem;
    }
</style>
