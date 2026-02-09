<script>
    import { onMount } from 'svelte';
    import { fade } from 'svelte/transition';
    import { fetchMemberGames, createMemberGame, updateMemberGame, deleteMemberGame } from '../lib/api';
    import Loader from '../components/Loader.svelte';

    let members = [];
    let loading = true;
    let error = null;
    let showModal = false;
    let editingMember = null;

    // Form State
    let formData = {
        id: 0,
        game_group: '',
        game_uxs: '',
        game_pxw: '',
        is_active: 1,
        bank_account_name: '',
        bank_account_number: '',
        bank_name: '',
        is_flag: 0,
        bet: ''
    };

    async function loadMembers() {
        loading = true;
        try {
            members = await fetchMemberGames();
        } catch (e) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    function openModal(member = null) {
        if (member) {
            editingMember = member;
            formData = { ...member };
        } else {
            editingMember = null;
            formData = {
                id: 0,
                game_group: '',
                game_uxs: '',
                game_pxw: '',
                is_active: 1,
                bank_account_name: '',
                bank_account_number: '',
                bank_name: '',
                is_flag: 0,
                bet: ''
            };
        }
        showModal = true;
    }

    async function handleSubmit() {
        try {
            if (editingMember) {
                await updateMemberGame(formData);
            } else {
                await createMemberGame(formData);
            }
            showModal = false;
            loadMembers();
        } catch (e) {
            alert('Error: ' + e.message);
        }
    }

    async function handleDelete(id) {
        if (!confirm('Are you sure you want to delete this member game config?')) return;
        try {
            await deleteMemberGame(id);
            loadMembers();
        } catch (e) {
            alert('Error: ' + e.message);
        }
    }

    onMount(loadMembers);
</script>

<div class="container-fluid py-4">
    <div class="d-flex justify-content-between align-items-center mb-4">
        <div>
            <h4 class="mb-0 fw-bold text-white">Member Game Management</h4>
            <p class="text-white-50 small mb-0">Manage member accounts and site assignments</p>
        </div>
        <button class="btn btn-primary rounded-pill px-4 shadow-sm" on:click={() => openModal()}>
            <i class="bi bi-person-plus-fill me-1"></i> Add New Member
        </button>
    </div>

    {#if loading}
        <div class="d-flex justify-content-center py-5">
            <Loader text="LOADING MEMBERS..." />
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
                            <th class="ps-4">Group</th>
                            <th>Username (UXS/PXW)</th>
                            <th>Status</th>
                            <th>Bank Account</th>
                            <th>Bank</th>
                            <th>Bet</th>
                            <th class="text-end pe-4">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each members as member}
                            <tr in:fade>
                                <td class="ps-4 fw-bold">{member.game_group}</td>
                                <td>
                                    <div class="small fw-semibold text-primary">{member.game_uxs}</div>
                                    <div class="tiny text-white-50">{member.game_pxw}</div>
                                </td>
                                <td>
                                    {#if member.is_active}
                                        <span class="badge bg-success bg-opacity-10 text-success border border-success border-opacity-25 rounded-pill px-2">ACTIVE</span>
                                    {:else}
                                        <span class="badge bg-danger bg-opacity-10 text-danger border border-danger border-opacity-25 rounded-pill px-2">INACTIVE</span>
                                    {/if}
                                </td>
                                <td>
                                    <div class="small fw-semibold">{member.bank_account_number}</div>
                                    <div class="tiny text-white-50">{member.bank_account_name}</div>
                                </td>
                                <td><span class="small">{member.bank_name}</span></td>
                                <td><span class="font-monospace small">{member.bet}</span></td>
                                <td class="text-end pe-4">
                                    <button class="btn btn-sm btn-link text-info me-2" on:click={() => openModal(member)} title="Edit Member">
                                        <i class="bi bi-pencil-square"></i>
                                    </button>
                                    <button class="btn btn-sm btn-link text-danger" on:click={() => handleDelete(member.id)} title="Delete Member">
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
                        {editingMember ? 'Edit Member Config' : 'Add New Member Config'}
                    </h5>
                    <button type="button" class="btn-close btn-close-white" on:click={() => showModal = false} aria-label="Close"></button>
                </div>
                <div class="modal-body py-3">
                    <form on:submit|preventDefault={handleSubmit}>
                        <div class="row g-3">
                            <div class="col-md-4">
                                <label for="game_group" class="form-label small text-white-50">Game Group</label>
                                <input id="game_group" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.game_group} required>
                            </div>
                            <div class="col-md-4">
                                <label for="game_uxs" class="form-label small text-white-50">Username (UXS)</label>
                                <input id="game_uxs" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.game_uxs} required>
                            </div>
                            <div class="col-md-4">
                                <label for="game_pxw" class="form-label small text-white-50">Password (PXW)</label>
                                <input id="game_pxw" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.game_pxw} required>
                            </div>
                            
                            <div class="col-md-4">
                                <label for="bank_name" class="form-label small text-white-50">Bank Name</label>
                                <input id="bank_name" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.bank_name}>
                            </div>
                            <div class="col-md-4">
                                <label for="bank_account_name" class="form-label small text-white-50">Bank Account Name</label>
                                <input id="bank_account_name" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.bank_account_name}>
                            </div>
                            <div class="col-md-4">
                                <label for="bank_account_number" class="form-label small text-white-50">Bank Account Number</label>
                                <input id="bank_account_number" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.bank_account_number}>
                            </div>

                            <div class="col-md-4">
                                <label for="bet" class="form-label small text-white-50">Bet Config (e.g. 1000)</label>
                                <input id="bet" type="text" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.bet}>
                            </div>
                            <div class="col-md-4">
                                <label for="status" class="form-label small text-white-50">Status</label>
                                <select id="status" class="form-select form-select-sm bg-black border-secondary text-white" bind:value={formData.is_active}>
                                    <option value={1}>Active</option>
                                    <option value={0}>Inactive</option>
                                </select>
                            </div>
                            <div class="col-md-4">
                                <label for="flag" class="form-label small text-white-50">Flag</label>
                                <input id="flag" type="number" class="form-control form-control-sm bg-black border-secondary text-white" bind:value={formData.is_flag}>
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
    .form-control:focus, .form-select:focus {
        background-color: black;
        border-color: var(--bs-primary);
        color: white;
        box-shadow: 0 0 0 0.25rem rgba(var(--bs-primary-rgb), 0.1);
    }
    .tiny {
        font-size: 0.7rem;
    }
</style>
