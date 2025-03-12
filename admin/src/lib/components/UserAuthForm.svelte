<script>
    import { createEventDispatcher } from 'svelte';
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { cn } from "$lib/utils";
    import { Lock, Loader2, UserPlus } from 'lucide-svelte';
    import { api } from '$lib/api'; // Import the api module
    import { goto } from '$app/navigation';
    import { login } from '$lib/auth';

    const dispatch = createEventDispatcher();

    let className = undefined;
    export { className as class };
    export const mode = 'login'; // 'login' or 'register'

    let isLoading = false;
    let username = '';
    let email = '';
    let password = '';
    let errorMessage = '';
    let errorDetails = {};

    async function onSubmit() {
      isLoading = true;
      errorMessage = '';
      errorDetails = {};
      
      try {
        if (mode === 'login') {
          const data = await api.user.login({ email, password });
          if (data.token) {
            // Store both tokens and user data
            login(data.token, data.refresh_token, data.user);
            dispatch('login', { email });
            goto('/');
          } else {
            throw new Error('Login successful, but no token received');
          }
        } else {
          const data = await api.user.register({ username, email, password });
          login(data.token, data.refresh_token, data.user);
          dispatch('register', { username, email });
          goto('/');
        }
      } catch (error) {
        errorMessage = error.message;
        if (error.details) {
          errorDetails = error.details;
        }
      } finally {
        isLoading = false;
      }
    }
  </script>
  
  <div class={cn("grid gap-6", className)} {...$$restProps}>
    {#if errorMessage}
      <div class="text-red-500">
        <p>{errorMessage}</p>
        {#if Object.keys(errorDetails).length > 0}
          <ul class="list-disc list-inside mt-2">
            {#each Object.entries(errorDetails) as [field, messages]}
              {#each messages as message}
                <li>{field}: {message}</li>
              {/each}
            {/each}
          </ul>
        {/if}
      </div>
    {/if}
    <form on:submit|preventDefault={onSubmit}>
      <div class="grid gap-2">
        {#if mode === 'register'}
          <div class="grid gap-1">
            <Label for="username">Username</Label>
            <Input
              id="username"
              bind:value={username}
              placeholder="johndoe"
              autocomplete="username"
              disabled={isLoading}
            />
          </div>
        {/if}
        <div class="grid gap-1">
          <Label for="email">Email</Label>
          <Input
            id="email"
            bind:value={email}
            placeholder="admin@oply.com"
            type="email"
            autocapitalize="none"
            autocomplete="email"
            autocorrect="off"
            disabled={isLoading}
          />
        </div>
        <div class="grid gap-1">
          <Label for="password">Password</Label>
          <Input
            id="password"
            bind:value={password}
            placeholder="••••••••"
            type="password"
            autocapitalize="none"
            autocomplete={mode === 'login' ? "current-password" : "new-password"}
            disabled={isLoading}
          />
        </div>
        <Button type="submit" disabled={isLoading}>
          {#if isLoading}
            <Loader2 class="mr-2 h-4 w-4 animate-spin" />
          {:else if mode === 'login'}
            <Lock class="mr-2 h-4 w-4" />
          {:else}
            <UserPlus class="mr-2 h-4 w-4" />
          {/if}
          {mode === 'login' ? 'Login to Command Center' : 'Register'}
        </Button>
      </div>
    </form>
  </div>