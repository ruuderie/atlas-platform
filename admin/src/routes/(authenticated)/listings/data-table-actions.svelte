<script lang="ts">
    import { Ellipsis } from "@lucide/svelte";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
    import { Button } from "$lib/components/ui/button";
    import { goto } from '$app/navigation';
    
    export let id: string;
    console.log('DataTableActions instantiated with id:', id);

    function viewUserDetails(): void {
        goto(`/users/${id}`);
    }

    function editUser(): void {
        goto(`/users/${id}?edit=true`);
    }

    function deactivateUser(): void {
        goto(`/users/${id}?deactivate=true`);
    }

    async function copyUserId(): Promise<void> {
        try {
            await navigator.clipboard.writeText(id);
            console.log(`User ID ${id} copied to clipboard`);
        } catch (err) {
            console.error('Failed to copy user ID: ', err);
        }
    }
</script>

<DropdownMenu.Root>
    <DropdownMenu.Trigger asChild let:builder>
        <Button
            variant="ghost"
            builders={[builder]}
            size="icon"
            class="relative h-8 w-8 p-0"
        >
            <span class="sr-only">Open menu</span>
            <Ellipsis class="h-4 w-4" />
        </Button>
    </DropdownMenu.Trigger>
    <DropdownMenu.Content class="w-56 bg-background border border-border rounded-md shadow-md">
        <DropdownMenu.Group>
            <DropdownMenu.Label class="px-2 py-1.5 text-sm font-semibold text-foreground">Actions</DropdownMenu.Label>
            <DropdownMenu.Item on:click={viewUserDetails} class="px-2 py-1.5 text-sm text-foreground hover:bg-accent hover:text-accent-foreground cursor-pointer">
                View user details
            </DropdownMenu.Item>
            <DropdownMenu.Item on:click={editUser} class="px-2 py-1.5 text-sm text-foreground hover:bg-accent hover:text-accent-foreground cursor-pointer">
                Edit user
            </DropdownMenu.Item>
            <DropdownMenu.Item on:click={deactivateUser} class="px-2 py-1.5 text-sm text-foreground hover:bg-accent hover:text-accent-foreground cursor-pointer">
                Deactivate user
            </DropdownMenu.Item>
            <DropdownMenu.Item on:click={copyUserId} class="px-2 py-1.5 text-sm text-foreground hover:bg-accent hover:text-accent-foreground cursor-pointer">
                Copy user ID
            </DropdownMenu.Item>
        </DropdownMenu.Group>
    </DropdownMenu.Content>
</DropdownMenu.Root>