<script>
    import { Button } from "$lib/components/ui/button";
    import { Calendar } from '@lucide/svelte';
    import { Popover } from "$lib/components/ui/popover";
    import { DateRangePicker } from "./date-range-picker.svelte";
  
    const dateRange = { from: new Date(), to: new Date() };
  
    function formatDate(date) {
      return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
    }
  </script>
  <div> testing</div>
  <!--
  <Popover.Root>
    <Popover.Trigger asChild let:builder>
      <Button variant="outline" builders={[builder]} class="w-[280px] justify-start text-left font-normal">
        <Calendar class="mr-2 h-4 w-4" />
        {formatDate(dateRange.from)} - {formatDate(dateRange.to)}
      </Button>
    </Popover.Trigger>
    <Popover.Content class="w-auto p-0">
      <DateRangePicker bind:dateRange />
    </Popover.Content>
  </Popover.Root>
  -->