<script>
    import { onMount } from 'svelte';
    import { api } from '$lib/api';
    import DataTableActions from "./data-table-actions.svelte";
    import DataTable from "$lib/components/DataTable.svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Plus } from 'lucide-svelte';
    import { goto } from '$app/navigation';
    import { createColumnHelper } from '@tanstack/svelte-table';

    let users = [];
    let loading = true;
    let error = null;
    let searchValue = '';

    // Create column helper
    const columnHelper = createColumnHelper();

    // Define columns
    const columns = [
        columnHelper.accessor('username', {
            header: 'Username',
            cell: info => info.getValue()
        }),
        columnHelper.accessor('email', {
            header: 'Email',
            cell: info => info.getValue()
        }),
        columnHelper.accessor('is_admin', {
            header: 'Is Admin',
            cell: info => info.getValue() ? 'Yes' : 'No'
        }),
        columnHelper.accessor('is_active', {
            header: 'Is Active',
            cell: info => info.getValue() ? 'Yes' : 'No'
        }),
        columnHelper.accessor('created_at', {
            header: 'Created At',
            cell: info => new Date(info.getValue()).toLocaleString()
        }),
        columnHelper.accessor('id', {
            header: 'Actions',
            cell: info => ({
                component: DataTableActions,
                props: { id: info.getValue() }
            })
        })
    ];

    onMount(async () => {
        try {
            users = await api.admin.fetchUsers();
            loading = false;
        } catch (err) {
            error = err.message;
            loading = false;
        }
    });

    function handleCreateUser() {
        goto('/users/create');
    }

    // Filter data based on search input
    $: filteredData = searchValue 
        ? users.filter(user => 
            user.username.toLowerCase().includes(searchValue.toLowerCase()) ||
            user.email.toLowerCase().includes(searchValue.toLowerCase()))
        : users;
</script>

<div class="w-full">
    <div class="mb-4 flex items-center gap-4">
        <Input
            class="max-w-sm"
            placeholder="Filter users..."
            type="text"
            bind:value={searchValue}
        />
        <Button variant="outline" on:click={handleCreateUser} class="ml-auto">
            <Plus class="mr-2 h-4 w-4" />
            Create User
        </Button>
    </div>
    
    {#if loading}
        <p>Loading users...</p>
    {:else if error}
        <p class="text-red-500">Error: {error}</p>
    {:else}
        <DataTable data={filteredData} columns={columns} />
    {/if}
</div>
