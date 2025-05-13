<script>
  import { onMount } from 'svelte';
  import { $page } from '$app/stores';
  import { api } from '$lib/api';
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardFooter } from "$lib/components/ui/card";
  import { ArrowLeft, DollarSign, Users, Calendar, BarChart2 } from "@lucide/svelte";
  import { goto } from '$app/navigation';

  let dealData = null;
  let loading = true;
  let error = null;

  let dealId = $derived($page.params.dealId);

  onMount(async () => {
    // Simulate loading data
    setTimeout(() => {
      // Placeholder data until API is implemented
      dealData = {
        id: dealId,
        name: "Sample Deal",
        customer: "ACME Corporation",
        amount: 15000,
        status: "Qualification",
        stage: "Meeting Scheduled",
        close_date: "2023-12-31",
        created_at: "2023-06-15",
        updated_at: "2023-06-20",
        is_active: true
      };
      loading = false;
    }, 800);
  });

  function goBack() {
    goto('/deals');
  }
</script>

<svelte:head>
  <title>{dealData ? dealData.name : 'Loading...'} | Deal Details</title>
</svelte:head>

<div class="container mx-auto px-4 py-8">
  <div class="mb-6">
    <Button variant="outline" on:click={goBack} class="flex items-center">
      <ArrowLeft class="mr-2 h-4 w-4" />
      Back to Deals
    </Button>
  </div>

  {#if loading}
    <p class="text-center text-xl">Loading deal details...</p>
  {:else if error}
    <p class="text-center text-xl text-red-500">Error: {error}</p>
  {:else if dealData}
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      <Card class="md:col-span-2">
        <CardHeader>
          <CardTitle class="text-2xl">{dealData.name}</CardTitle>
          <CardDescription>Deal ID: {dealData.id}</CardDescription>
        </CardHeader>
        <CardContent>
          <div class="grid gap-4">
            <div class="flex justify-between items-center">
              <span class="font-semibold">Customer:</span>
              <span>{dealData.customer}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="font-semibold">Amount:</span>
              <span>${dealData.amount.toLocaleString()}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="font-semibold">Status:</span>
              <span>{dealData.status}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="font-semibold">Stage:</span>
              <span>{dealData.stage}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="font-semibold">Expected Close Date:</span>
              <span>{new Date(dealData.close_date).toLocaleDateString()}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="font-semibold">Created At:</span>
              <span>{new Date(dealData.created_at).toLocaleString()}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="font-semibold">Updated At:</span>
              <span>{new Date(dealData.updated_at).toLocaleString()}</span>
            </div>
          </div>
        </CardContent>
        <CardFooter class="flex justify-between">
          <Button variant="outline">
            Edit Deal
          </Button>
          <Button variant="destructive">
            Delete Deal
          </Button>
        </CardFooter>
      </Card>

      <div class="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center">
              <DollarSign class="mr-2 h-5 w-5" />
              Deal Value
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-3xl font-bold">${dealData.amount.toLocaleString()}</div>
            <p class="text-sm text-muted-foreground mt-2">Expected revenue from this deal</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle class="flex items-center">
              <Users class="mr-2 h-5 w-5" />
              Customer Details
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p class="font-medium">{dealData.customer}</p>
            <p class="text-sm text-muted-foreground mt-2">
              Customer information will be displayed here
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle class="flex items-center">
              <Calendar class="mr-2 h-5 w-5" />
              Timeline
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p class="text-sm text-muted-foreground">
              Deal timeline will be displayed here
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle class="flex items-center">
              <BarChart2 class="mr-2 h-5 w-5" />
              Deal Analytics
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p class="text-sm text-muted-foreground">
              Deal analytics will be displayed here
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  {:else}
    <p class="text-center text-xl">Deal not found.</p>
  {/if}
</div>