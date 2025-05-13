<script>
  import '../app.css';
  import { theme } from '$lib/stores/appStore';
  import { browser } from '$app/environment';
  import { isAuthenticated } from '$lib/stores/authStore';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { checkAuth } from '$lib/auth';
  import { onMount } from 'svelte';
  import Header from '$lib/components/Header.svelte';
  import Ripple from '$lib/components/Ripple.svelte';
  import Globe from '$lib/components/Globe.svelte';
  let isLoading = $state(true);

  onMount(async () => {
    if (browser) {
      console.log('Initializing app');
      try {
        await initializeApp();
      } catch (error) {
        console.error('Error initializing app:', error);
        isLoading = false; // Ensure loading state is cleared even on error
      }
    }
  });

  async function initializeApp() {
    try {
      const isAuth = await checkAuth();
      isAuthenticated.set(isAuth);
      console.log('App initialized, isAuthenticated:', isAuth);
      console.log('Current path:', $page.url.pathname);
    } catch (error) {
      console.error('Error checking auth:', error);
      isAuthenticated.set(false);
    } finally {
      console.log('Loading state set to false');
      isLoading = false; // Always set loading to false when done
    }
  }

  // Subscribe to theme changes
  $effect(() => {
    if (browser) {
      theme.subscribe(currentTheme => {
        document.documentElement.classList.toggle('dark', currentTheme === 'dark');
      });
    }
  });

  // Check if we're on the root or login route
  let showGlobe = $derived($page.url.pathname === '/' || $page.url.pathname === '/login');
</script>

{#if isLoading}
  <div class="flex items-center justify-center min-h-screen">
    Loading...
  </div>
{:else}
  <div class="min-h-screen flex flex-col bg-background text-foreground">
    <Header />

    <main class="flex-grow container mx-auto px-4 py-8">
      {#if showGlobe}
        <Globe class="top-28" />
      {/if}

      <slot />
    </main>

    <footer class="bg-background border-t">
      <div class="container mx-auto px-4 py-4 text-center text-sm text-muted-foreground">
        © 2025 Oply Command Center. All rights reserved.
      </div>
    </footer>
  </div>
{/if}
