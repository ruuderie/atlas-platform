<script>
  import { onMount } from 'svelte';
  import { VisXYContainer, VisGroupedBar, VisAxis, VisTooltip } from '@unovis/svelte';

  // Convert props to use the $props rune
  let { 
    data = [], 
    xKey = 'label', 
    yKey = 'value', 
    title = '', 
    color = '#4bc0c0' 
  } = $props();

  // Configure tooltip
  const tooltipConfig = {
    trigger: 'hover',
    content: (d) => `${d[xKey]}: ${d[yKey].toLocaleString()}`,
  };

  // Define accessor functions for the chart
  const x = (d) => d[xKey];
  const y = (d) => d[yKey];
</script>

<div class="chart-container">
  {#if title}
    <h3 class="chart-title">{title}</h3>
  {/if}
  <VisXYContainer height={300}>
    <VisGroupedBar {data} {x} {y} {color} />
    <VisAxis type="x" />
    <VisAxis type="y" />
    <VisTooltip {...tooltipConfig} />
  </VisXYContainer>
</div>

<style>
  .chart-container {
    width: 100%;
    height: 300px;
  }
  
  .chart-title {
    text-align: center;
    margin-bottom: 1rem;
    font-weight: 600;
  }
</style>