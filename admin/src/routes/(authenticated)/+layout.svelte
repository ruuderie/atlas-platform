<script>
    import Header from '$lib/components/Header.svelte';
    import { isAuthenticated } from '$lib/stores/authStore'
    import { checkAuth, logout } from '$lib/auth';
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import { browser } from '$app/environment';
    import { theme } from '$lib/stores/appStore';
    import { goto } from '$app/navigation';
    import { loadUser } from '$lib/stores/userStore';
    import { user } from '$lib/stores/userStore';

    let isLoading = true;

    onMount(async () => {
        if (browser) {
            await initializeApp();
        }
    });

    async function initializeApp() {
        const isAuth = await checkAuth();
        if (!isAuth) {
            goto('/login', { replaceState: true });
            return;
        }
        
        await loadUser();  // Load user data after authentication check

        const storedTheme = localStorage.getItem('theme');
        if (storedTheme) {
            theme.setTheme(storedTheme);
        } else {
            const darkMode = localStorage.getItem('darkMode') === 'true';
            theme.setTheme(darkMode ? 'dark' : 'light');
        }

        isLoading = false;
    }

    $: if (browser && !isLoading) {
        handleRouteChange($page.url.pathname);
    }

    async function handleRouteChange(newPath) {
        const isAuth = await checkAuth();
        if (!isAuth) {
            console.log('Redirecting to login');
            goto('/login', { replaceState: true });
        }
    }

    $effect(() => {
        console.log('Layout updated - Current user:', $user);
    });

    $: if ($user) {
        console.log('User data changed in layout:', $user);
    }
</script>

{#if isLoading}
    <div class="flex items-center justify-center min-h-screen">
        Loading...
    </div>
{:else}
    <slot />
{/if}
