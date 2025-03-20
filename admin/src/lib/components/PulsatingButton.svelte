<script lang="ts">
    import { cn } from "$lib/utils";
    import { createEventDispatcher, props } from 'svelte';

    const pulseColor = $props<string>("#0096ff");
    const duration = $props<string>("1.5s");
    const href = $props<string | null>(null);
    const text = $props<string>("Button");
    const className = $props<string>("");

    const dispatch = createEventDispatcher();

    function handleClick(event: MouseEvent) {
        if (!href) {
            event.preventDefault();
            dispatch('click', event);
        }
    }
</script>

{#if href}
    <a
        {href}
        class={cn(
            "relative text-center cursor-pointer flex justify-center items-center rounded-lg text-white dark:text-black bg-blue-500 dark:bg-blue-500 px-4 py-2",
            className
        )}
        style:--pulse-color={pulseColor}
        style:--duration={duration}
        on:click={handleClick}
    >
        <div class="relative z-10">
            {text}
        </div>
        <div
            class="absolute top-1/2 left-1/2 size-full rounded-lg bg-inherit animate-pulse -translate-x-1/2 -translate-y-1/2"
        ></div>
    </a>
{:else}
    <button
        class={cn(
            "relative text-center cursor-pointer flex justify-center items-center rounded-lg text-white dark:text-black bg-blue-500 dark:bg-blue-500 px-4 py-2",
            className
        )}
        style:--pulse-color={pulseColor}
        style:--duration={duration}
        on:click={handleClick}
    >
        <div class="relative z-10">
            {text}
        </div>
        <div
            class="absolute top-1/2 left-1/2 size-full rounded-lg bg-inherit animate-pulse -translate-x-1/2 -translate-y-1/2"
        ></div>
    </button>
{/if}
