<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { isAuthenticated } from '$lib/stores/authStore';
  import { checkAuth } from '$lib/auth';
  import Ripple from '$lib/components/Ripple.svelte';
  import { Button } from '$lib/components/ui/button';
  import PulsatingButton from '$lib/components/PulsatingButton.svelte';

  onMount(async () => {
    await checkAuth();
    if ($isAuthenticated) {
      goto('/home');
    }
  });

  function handleLogin() {
    goto('/login');
  }
</script>

<div class="absolute inset-0 flex items-center justify-center">
  <div class="container mx-auto flex flex-col items-center justify-center p-4 z-10">
    <div class="max-w-2xl w-full text-center">
      <span class="pointer-events-none z-10 whitespace-pre-wrap bg-gradient-to-b from-[#00d3ff] via-[#ffffff] to-[#8c1eff] bg-clip-text text-center text-7xl font-bold leading-none tracking-tighter text-transparent">
        Welcome to Oply Command Center
      </span>
      <p class="text-lg text-primary/80 mb-8">
        Please log in to access the dashboard
      </p>
      <div class="flex flex-wrap justify-center items-center gap-10">
        <PulsatingButton text="Log In" onclick={handleLogin} />
      </div>
    </div>
  </div>
</div>
