<script>
  import { createSvelteTable, FlexRender } from '$lib/components/ui/data-table';
  import { 
    getCoreRowModel,
    getSortedRowModel,
    getPaginationRowModel
  } from '@tanstack/table-core';
  
  let { 
    data = $bindable([]),
    columns = $bindable([]),
    pageSize = 10
  } = $props();
  
  // Create table instance
  const table = createSvelteTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    initialState: {
      pagination: {
        pageSize
      }
    }
  });
</script>

<div class="relative w-full overflow-auto">
  <table class="w-full caption-bottom text-sm">
    <thead>
      {#each table.getHeaderGroups() as headerGroup}
        <tr class="border-b transition-colors hover:bg-muted/50">
          {#each headerGroup.headers as header}
            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">
              {#if !header.isPlaceholder}
                <div 
                  class="flex items-center space-x-2"
                  class:cursor-pointer={header.column.getCanSort()}
                  onclick={() => header.column.getToggleSortingHandler()?.()}
                >
                  <FlexRender 
                    content={header.column.columnDef.header}
                    context={header.getContext()}
                  />
                </div>
              {/if}
            </th>
          {/each}
        </tr>
      {/each}
    </thead>
    <tbody>
      {#each table.getRowModel().rows as row}
        <tr class="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
          {#each row.getVisibleCells() as cell}
            <td class="p-4 align-middle">
              <FlexRender 
                content={cell.column.columnDef.cell}
                context={cell.getContext()}
              />
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<!-- Pagination -->
<div class="flex items-center justify-between px-2 py-4">
  <div class="flex-1 text-sm text-muted-foreground">
    {table.getFilteredRowModel().rows.length} row(s).
  </div>
  <div class="flex items-center space-x-6 lg:space-x-8">
    <div class="flex items-center space-x-2">
      <button
        class="rounded border p-1"
        onclick={() => table.previousPage()}
        disabled={!table.getCanPreviousPage()}
      >
        Previous
      </button>
      <button
        class="rounded border p-1"
        onclick={() => table.nextPage()}
        disabled={!table.getCanNextPage()}
      >
        Next
      </button>
    </div>
  </div>
</div> 