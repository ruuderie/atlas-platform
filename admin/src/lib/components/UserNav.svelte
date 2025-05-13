<script>
  import { createEventDispatcher } from 'svelte';
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { User, Settings, LogOut } from '@lucide/svelte';

  export const user = { name: 'Admin User', email: 'admin@oply.com' };

  let { onLogout } = $props();

  function handleLogout() {
    onLogout?.();
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger asChild let:builder>
    <Button
      variant="ghost"
      class="relative h-8 w-8 rounded-full"
      builders={[builder]}
    >
      <User class="h-5 w-5" />
      <span class="sr-only">Open user menu</span>
    </Button>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-56" align="end">
    <DropdownMenu.Label class="font-normal">
      <div class="flex flex-col space-y-1">
        <p class="text-sm font-medium leading-none">{user.name}</p>
        <p class="text-xs leading-none text-muted-foreground">
          {user.email}
        </p>
      </div>
    </DropdownMenu.Label>
    <DropdownMenu.Separator />
    <DropdownMenu.Group>
      <DropdownMenu.Item>
        <User class="mr-2 h-4 w-4" />
        <span>Profile</span>
      </DropdownMenu.Item>
      <DropdownMenu.Item>
        <Settings class="mr-2 h-4 w-4" />
        <span>Settings</span>
      </DropdownMenu.Item>
    </DropdownMenu.Group>
    <DropdownMenu.Separator />
    <DropdownMenu.Item on:click={handleLogout}>
      <LogOut class="mr-2 h-4 w-4" />
      <span>Log out</span>
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>