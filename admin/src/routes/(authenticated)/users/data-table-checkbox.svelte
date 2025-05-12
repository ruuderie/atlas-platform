<script>
    import * as Checkbox from "$lib/components/ui/checkbox";
    
    let { checked = false, indeterminate = false, onCheckedChange = (value) => {} } = $props();
</script>

<Checkbox.Root 
    checked={checked} 
    onCheckedChange={onCheckedChange}
    aria-label="Select row"
/>
