<script>
    import { onMount } from 'svelte';
    import { checkAuth } from '$lib/auth';
    import { api } from '$lib/api';  // Update this import
    import { isAuthenticated } from '$lib/stores/authStore';
    import { Users, ListChecks, DollarSign, BarChart2, Download } from '@lucide/svelte';
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import * as Tabs from "$lib/components/ui/tabs";
    import ChartComponent from '$lib/components/ChartComponent.svelte';
    import DashboardMainNav from '$lib/components/DashboardMainNav.svelte';
    import Search from '$lib/components/Search.svelte';
    import UserNav from '$lib/components/UserNav.svelte';
    import TeamSwitcher from '$lib/components/TeamSwitcher.svelte';
    import DatePickerWithRange from '$lib/components/DatePickerWithRange.svelte';
    import { goto } from '$app/navigation';
  
    let dashboardStats = null;
    let chartData = [];
    const months = ['January', 'February', 'March', 'April', 'May', 'June', 'July'];
  
    async function loadDashboardStats() {
      console.log("loadDashboardStats called");
      try {
        dashboardStats = await api.admin.fetchDashboardStats();
        console.log("Dashboard stats:", dashboardStats);
        
        // Transform the data for Unovis
        chartData = months.map((month, index) => ({
          label: month,
          value: dashboardStats.monthlyRevenue[index] || 0
        }));
      } catch (error) {
        console.error('Failed to fetch dashboard stats:', error);
      }
    }
  
    onMount(() => {
      console.log("onMount called");
      checkAuth();
    });
  
    $effect(() => {
      if ($isAuthenticated) {
      console.log("User is authenticated, loading dashboard stats");
      loadDashboardStats();
    } else {
      console.log("User is not authenticated");
    }
  });
</script>
  
<div class="flex-col md:flex">
  <div class="border-b">
    <div class="flex h-16 items-center px-4">
      <TeamSwitcher />
      <DashboardMainNav class="mx-6" />
      <div class="ml-auto flex items-center space-x-4">
        <Search />
        <UserNav />
      </div>
    </div>
  </div>
  <div class="flex-1 space-y-4 p-8 pt-6">
    <div class="flex items-center justify-between space-y-2">
      <h2 class="text-3xl font-bold tracking-tight">Dashboard Overview</h2>
    </div>

    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4 mb-8">
      <Card.Root>
        <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
          <Card.Title class="text-sm font-medium">Total Users</Card.Title>
          <Users class="h-4 w-4 text-muted-foreground" />
        </Card.Header>
        <Card.Content>
          <div class="text-2xl font-bold">{dashboardStats?.totalUsers?.toLocaleString() || '---'}</div>
        </Card.Content>
      </Card.Root>
      <Card.Root>
        <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
          <Card.Title class="text-sm font-medium">Active Listings</Card.Title>
          <ListChecks class="h-4 w-4 text-muted-foreground" />
        </Card.Header>
        <Card.Content>
          <div class="text-2xl font-bold">{dashboardStats?.activeListings?.toLocaleString() || '---'}</div>
        </Card.Content>
      </Card.Root>
      <Card.Root>
        <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
          <Card.Title class="text-sm font-medium">Ad Purchases</Card.Title>
          <DollarSign class="h-4 w-4 text-muted-foreground" />
        </Card.Header>
        <Card.Content>
          <div class="text-2xl font-bold">{dashboardStats?.adPurchases?.toLocaleString() || '---'}</div>
        </Card.Content>
      </Card.Root>
      <Card.Root>
        <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
          <Card.Title class="text-sm font-medium">Monthly Revenue</Card.Title>
          <BarChart2 class="h-4 w-4 text-muted-foreground" />
        </Card.Header>
        <Card.Content>
          <div class="text-2xl font-bold">${dashboardStats?.revenue?.toLocaleString() || '---'}</div>
        </Card.Content>
      </Card.Root>
    </div>

    <Card.Root>
      <Card.Header>
        <Card.Title>Monthly Revenue</Card.Title>
      </Card.Header>
      <Card.Content>
        <ChartComponent 
          data={chartData} 
          xKey="label" 
          yKey="value" 
          color="rgba(75, 192, 192, 0.6)" 
        />
      </Card.Content>
    </Card.Root>
  </div>
</div>