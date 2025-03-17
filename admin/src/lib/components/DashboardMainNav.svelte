<script>
  import { page } from '$app/stores';
  import { cn } from "$lib/utils";
  import { currentNavItems } from '$lib/stores/appStore';
  import { goto } from '$app/navigation';

  let className = '';
  export { className as class };

  // Get the current path without the route group prefix
  $: currentPath = $page.url.pathname.replace(/^\/(authenticated)/, '');
  
  function handleNavigation(event, href) {
    event.preventDefault();
    // Add the route group prefix for navigation
    goto(`/(authenticated)${href}`);
  }
</script>

<nav class={cn("flex items-center space-x-4 lg:space-x-6", className)}>
  {#each $currentNavItems as item}
    <a
      href={item.href}
      on:click={(e) => handleNavigation(e, item.href)}
      class={cn(
        "text-sm font-medium transition-colors hover:text-primary flex items-center",
        currentPath === item.href
          ? "text-primary"
          : "text-muted-foreground"
      )}
    >
      <img src={item.icon} alt="" class="w-4 h-4 mr-2" />
      {item.label}
    </a>
  {/each}
</nav>