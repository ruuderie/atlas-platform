<script>
    import { onMount } from 'svelte';
    import { writable } from 'svelte/store';
    import { api } from '$lib/api';
    import DataTableActions from "../users/data-table-actions.svelte";
    import DataTableCheckbox from "../users/data-table-checkbox.svelte";
    import * as Table from "$lib/components/ui/table";
    import { Button } from "$lib/components/ui/button";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
    import { Input } from "$lib/components/ui/input";
    import { 
        createSvelteTable, 
        FlexRender
    } from "$lib/components/ui/data-table";
    import { 
        getCoreRowModel,
        getSortedRowModel,
        getPaginationRowModel,
        getFilteredRowModel,
    } from   '@tanstack/table-core';
    import { cn } from "$lib/utils.js";
    import { ChevronDown, ChevronUp } from  '@lucide/svelte';
    import { Plus } from  '@lucide/svelte';
    import { goto } from '$app/navigation';

    let users = [];
    let loading = true;
    let error = null;

    // State for table
    let pagination = $state({ pageIndex: 0, pageSize: 10 });
    let sorting = $state([]);
    let columnFilters = $state([]);
    let columnVisibility = $state({});
    let rowSelection = $state({});
    let globalFilter = $state('');

    let table;

    onMount(async () => {
        try {
            users = await api.admin.fetchUsers();
            console.log('users', users);
            loading = false;
            initializeTable();
        } catch (err) {
            error = err.message;
            loading = false;
        }
    });

    function initializeTable() {
        if (users.length === 0) return;

        const columns = [
            {
                id: "select",
                header: ({ table }) => ({
                    component: DataTableCheckbox,
                    props: {
                        checked: table.getIsAllPageRowsSelected(),
                        indeterminate: table.getIsSomePageRowsSelected() && !table.getIsAllPageRowsSelected(),
                        onCheckedChange: (value) => table.toggleAllPageRowsSelected(!!value)
                    }
                }),
                cell: ({ row }) => ({
                    component: DataTableCheckbox,
                    props: {
                        checked: row.getIsSelected(),
                        onCheckedChange: (value) => row.toggleSelected(!!value)
                    }
                }),
                enableSorting: false,
                enableHiding: false
            },
            {
                accessorKey: 'username',
                header: 'Username',
                cell: ({ row }) => row.getValue('username')
            },
            {
                accessorKey: 'email',
                header: 'Email',
                cell: ({ row }) => row.getValue('email')
            },
            {
                accessorKey: 'is_admin',
                header: 'Is Admin',
                cell: ({ row }) => row.getValue('is_admin') ? 'Yes' : 'No'
            },
            {
                accessorKey: 'is_active',
                header: 'Is Active',
                cell: ({ row }) => row.getValue('is_active') ? 'Yes' : 'No'
            },
            {
                accessorKey: 'created_at',
                header: 'Created At',
                cell: ({ row }) => new Date(row.getValue('created_at')).toLocaleString()
            },
            {
                id: 'actions',
                header: 'Actions',
                cell: ({ row }) => ({
                    component: DataTableActions,
                    props: { id: row.original.id }
                }),
                enableSorting: false,
                enableHiding: false
            }
        ];

        table = createSvelteTable({
            data: users,
            columns,
            state: {
                get pagination() { return pagination; },
                get sorting() { return sorting; },
                get columnFilters() { return columnFilters; },
                get columnVisibility() { return columnVisibility; },
                get rowSelection() { return rowSelection; },
                get globalFilter() { return globalFilter; }
            },
            onPaginationChange: (updater) => {
                if (typeof updater === 'function') {
                    pagination = updater(pagination);
                } else {
                    pagination = updater;
                }
            },
            onSortingChange: (updater) => {
                if (typeof updater === 'function') {
                    sorting = updater(sorting);
                } else {
                    sorting = updater;
                }
            },
            onColumnFiltersChange: (updater) => {
                if (typeof updater === 'function') {
                    columnFilters = updater(columnFilters);
                } else {
                    columnFilters = updater;
                }
            },
            onColumnVisibilityChange: (updater) => {
                if (typeof updater === 'function') {
                    columnVisibility = updater(columnVisibility);
                } else {
                    columnVisibility = updater;
                }
            },
            onRowSelectionChange: (updater) => {
                if (typeof updater === 'function') {
                    rowSelection = updater(rowSelection);
                } else {
                    rowSelection = updater;
                }
            },
            onGlobalFilterChange: (updater) => {
                if (typeof updater === 'function') {
                    globalFilter = updater(globalFilter);
                } else {
                    globalFilter = updater;
                }
            },
            getCoreRowModel: getCoreRowModel(),
            getSortedRowModel: getSortedRowModel(),
            getPaginationRowModel: getPaginationRowModel(),
            getFilteredRowModel: getFilteredRowModel(),
        });
    }

    function handleCreateUser() {
        goto('/users/create');
    }
</script>

<div class="w-full">
    <div class="mb-4 flex items-center gap-4">
        <Input
            class="max-w-sm"
            placeholder="Filter users..."
            type="text"
            bind:value={globalFilter}
        />
        <Button variant="outline" on:click={handleCreateUser} class="ml-auto">
            <Plus class="mr-2 h-4 w-4" />
            Create User
        </Button>
        <DropdownMenu.Root>
            <DropdownMenu.Trigger asChild let:builder>
                <Button variant="outline" class="ml-auto" builders={[builder]}>
                    Columns <ChevronDown class="ml-2 h-4 w-4" />
                </Button>
            </DropdownMenu.Trigger>
            <DropdownMenu.Content class="bg-white shadow-md rounded-md">
                {#if table}
                    {#each table.getAllColumns().filter(col => col.getCanHide()) as column}
                        <DropdownMenu.CheckboxItem
                            checked={column.getIsVisible()}
                            onCheckedChange={(value) => column.toggleVisibility(!!value)}
                        >
                            {column.id}
                        </DropdownMenu.CheckboxItem>
                    {/each}
                {/if}
            </DropdownMenu.Content>
        </DropdownMenu.Root>
    </div>
    {#if loading}
        <p>Loading users...</p>
    {:else if error}
        <p class="text-red-500">Error: {error}</p>
    {:else if table}
        <div class="rounded-md border">
            <Table.Root>
                <Table.Header>
                    {#each table.getHeaderGroups() as headerGroup}
                        <Table.Row>
                            {#each headerGroup.headers as header}
                                <Table.Head class={cn("[&:has([role=checkbox])]:pl-3")}>
                                    {#if !header.isPlaceholder}
                                        <div class="flex items-center">
                                            {#if header.column.getCanSort()}
                                                <Button 
                                                    variant="ghost" 
                                                    on:click={() => header.column.toggleSorting(header.column.getIsSorted() === "asc")}
                                                    class="flex items-center gap-1"
                                                >
                                                    <FlexRender 
                                                        content={header.column.columnDef.header}
                                                        context={header.getContext()}
                                                    />
                                                    {#if header.column.getIsSorted() === "asc"}
                                                        <ChevronUp class="h-4 w-4" />
                                                    {:else if header.column.getIsSorted() === "desc"}
                                                        <ChevronDown class="h-4 w-4" />
                                                    {:else}
                                                        <ChevronUp class="h-4 w-4 opacity-0 group-hover:opacity-100" />
                                                    {/if}
                                                </Button>
                                            {:else}
                                                <FlexRender 
                                                    content={header.column.columnDef.header}
                                                    context={header.getContext()}
                                                />
                                            {/if}
                                        </div>
                                    {/if}
                                </Table.Head>
                            {/each}
                        </Table.Row>
                    {/each}
                </Table.Header>
                <Table.Body>
                    {#if table.getRowModel().rows.length}
                        {#each table.getRowModel().rows as row}
                            <Table.Row data-state={row.getIsSelected() && "selected"}>
                                {#each row.getVisibleCells() as cell}
                                    <Table.Cell class="[&:has([role=checkbox])]:pl-3">
                                        <FlexRender 
                                            content={cell.column.columnDef.cell}
                                            context={cell.getContext()}
                                        />
                                    </Table.Cell>
                                {/each}
                            </Table.Row>
                        {/each}
                    {:else}
                        <Table.Row>
                            <Table.Cell colspan={table.getAllColumns().length} class="h-24 text-center">
                                No results.
                            </Table.Cell>
                        </Table.Row>
                    {/if}
                </Table.Body>
            </Table.Root>
        </div>
        <div class="flex items-center justify-end space-x-2 py-4">
            <div class="text-muted-foreground flex-1 text-sm">
                {table.getFilteredSelectedRowModel().rows.length} of {table.getFilteredRowModel().rows.length} row(s) selected.
            </div>
            <Button
                variant="outline"
                size="sm"
                on:click={() => table.previousPage()}
                disabled={!table.getCanPreviousPage()}>Previous</Button
            >
            <Button
                variant="outline"
                size="sm"
                disabled={!table.getCanNextPage()}
                on:click={() => table.nextPage()}>Next</Button
            >
        </div>
    {:else}
        <p>No data available or table not initialized</p>
    {/if}
</div>