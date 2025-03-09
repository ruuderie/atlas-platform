<script>
  import UserAuthForm from "$lib/components/UserAuthForm.svelte";
  import { Button } from "$lib/components/ui/button";
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { checkAuth } from '$lib/auth';
  import { isAuthenticated } from '$lib/stores/authStore';

  onMount(() => {
    console.log("onMount called on login page");
    checkAuth();
    if ($isAuthenticated) {
      goto('/(authenticated)/home');
    }
  });

  async function handleLogin(loginSuccessful) {
    if (loginSuccessful) {
      goto('/(authenticated)/home');
    }
  }
</script>


<div class="container relative hidden h-[800px] flex-col items-center justify-center md:grid lg:max-w-none lg:grid-cols-2 lg:px-0">
  <div class="relative hidden h-full flex-col bg-muted p-10 text-white dark:border-r lg:flex">
    <div class="absolute inset-0 bg-zinc-900"></div>
    <div class="relative z-20 flex items-center text-lg font-medium">
      <span class="text-xl font-bold">Oply Command Center</span>
    </div>
    <div class="relative z-20 mt-auto">
      <blockquote class="space-y-2">
        <p class="text-lg">
          &ldquo;The Oply Command Center gives us unprecedented control and insight into our directory operations. It's an indispensable tool for our admin team.&rdquo;
        </p>
        <footer class="text-sm">Sophia Chen, Directory Operations Manager</footer>
      </blockquote>
    </div>
  </div>
  <div class="lg:p-8">
    <div class="mx-auto flex w-full flex-col justify-center space-y-6 sm:w-[350px]">
      <div class="flex flex-col space-y-2 text-center">
        <h1 class="text-2xl font-semibold tracking-tight">Admin Access</h1>
        <p class="text-sm text-muted-foreground">
          Enter your credentials to access the Oply Command Center
        </p>
      </div>
      <UserAuthForm mode="login" />
      <p class="px-8 text-center text-sm text-muted-foreground">
        By logging in, you agree to our
        <a href="/terms" class="underline underline-offset-4 hover:text-primary">Terms of Service</a>
        and
        <a href="/privacy" class="underline underline-offset-4 hover:text-primary">Privacy Policy</a>.
      </p>
    </div>
  </div>
</div>
