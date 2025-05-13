<script>
  import { isAuthenticated } from '$lib/stores/authStore';
  import { user } from '$lib/stores/userStore';
  import { Button } from "$lib/components/ui/button";
  import { Avatar, AvatarFallback, AvatarImage } from "$lib/components/ui/avatar";
  import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from '$lib/components/ui/dropdown-menu';
  import { LogOut, UserPlus, Settings, HelpCircle, Command, Sun, Moon } from '@lucide/svelte';
  import { logout } from '$lib/auth';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  let darkMode = $state(false);

  let userName = $derived($user ? `${$user.first_name} ${$user.last_name}` : 'User');
  let userInitials = $derived($user ? `${$user.first_name[0]}${$user.last_name[0]}` : 'U');
  
  onMount(() => {
    darkMode = localStorage.getItem('darkMode') === 'true';
    document.documentElement.classList.toggle('dark', darkMode);
  });

  function toggleDarkMode() {
    darkMode = !darkMode;
    document.documentElement.classList.toggle('dark', darkMode);
    localStorage.setItem('darkMode', darkMode);
  }

  function handleLogin() {
    goto('/login');
  }

  function handleRegister() {
    goto('/register');
  }

  // Use $derived for the home route
  let homeRoute = $derived($isAuthenticated ? '/home' : '/');
</script>

<header class="bg-background border-b border-border">
  <nav class="container mx-auto px-4 py-3 flex justify-between items-center">
    <a href={homeRoute} class="text-xl font-semibold text-foreground flex items-center">
      <Command class="w-8 h-8 mr-2 text-primary" />
      <span>Oply Command Center</span>
    </a>
    <div class="flex items-center space-x-4">
      <Button onclick={toggleDarkMode} variant="outline" size="icon">
        <Sun class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
        <Moon class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
        <span class="sr-only">Toggle theme</span>
      </Button>
      {#if $isAuthenticated}
        <Button variant="ghost" class="text-muted-foreground hover:text-foreground">
          <HelpCircle class="w-5 h-5 mr-2" />
          Help
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger>
            <Avatar class="w-8 h-8 transition duration-300 ease-in-out transform hover:scale-105">
              <AvatarFallback class="bg-muted text-muted-foreground">
                {userInitials}
              </AvatarFallback>
            </Avatar>
          </DropdownMenuTrigger>
          <DropdownMenuContent class="w-56">
            <DropdownMenuLabel class="font-normal">
              <div class="flex flex-col space-y-1">
                <p class="text-sm font-medium leading-none">{userName}</p>
                <p class="text-xs leading-none text-muted-foreground">{$user ? $user.email : 'unknown@error'}</p>
              </div>
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem>
              <Settings class="mr-2 h-4 w-4" />
              <span>Settings</span>
            </DropdownMenuItem>
            <DropdownMenuItem onclick={logout}>
              <LogOut class="mr-2 h-4 w-4" />
              <span>Log out</span>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      {:else}
        <Button variant="ghost" onclick={handleLogin} class="text-muted-foreground hover:text-foreground">
          Login
        </Button>
        <Button variant="default" onclick={handleRegister} class="bg-primary text-primary-foreground hover:bg-primary/90">
          <UserPlus class="mr-2 h-4 w-4" />
          Register
        </Button>
      {/if}
    </div>
  </nav>
</header>
