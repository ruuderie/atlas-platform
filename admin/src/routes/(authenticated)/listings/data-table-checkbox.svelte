<script>
    import * as Checkbox from "$lib/components/ui/checkbox";
    
    export let checked = false;
    export let indeterminate = false;
    export let onCheckedChange = (value) => {};
</script>

<Checkbox.Root 
    checked={checked} 
    onCheckedChange={onCheckedChange}
    aria-label="Select row"
/>
