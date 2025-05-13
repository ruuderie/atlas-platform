<script>
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardFooter } from "$lib/components/ui/card";
  import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "$lib/components/ui/table";
  import { Switch } from "$lib/components/ui/switch";
  import { ArrowLeft, Users, ListChecks, DollarSign, BarChart2, Download } from '@lucide/svelte';
  import { goto } from '$app/navigation';

  let user = null;
  let loading = true;
  let error = null;
  let editing = false;

  $: userId = $page.params.userId;

  $: if (userId) {
    loadUserData(userId);
  }

  async function loadUserData(id) {
    loading = true;
    try {
      user = await api.admin.fetchUserById(id);
      console.log("User:", user);
      // Add placeholder data
      user.name = user.username; // Use username as name
      user.phone = "123-456-7890"; // Placeholder phone
      user.lastLogin = new Date().toISOString(); // Placeholder last login
      user.accounts = [
        { name: "Main Account", role: "User", status: "Active" }
      ];
      user.loginHistory = [
        { date: new Date().toISOString(), ipAddress: "192.168.1.1", device: "Desktop" }
      ];
    } catch (err) {
      error = err.message;
    } finally {
      loading = false;
    }
  }

  async function handleSave() {
    try {
      await api.admin.updateUser(user.id, user);
      editing = false;
    } catch (err) {
      error = err.message;
    }
  }

  async function handleToggleActive() {
    try {
      user.is_active = !user.is_active;
      await api.admin.updateUser(user.id, { is_active: user.is_active });
    } catch (err) {
      error = err.message;
    }
  }

  async function handleResetPassword() {
    try {
      await api.admin.resetUserPassword(user.id);
      alert("Password reset email sent to user.");
    } catch (err) {
      error = err.message;
    }
  }

  function goBack() {
    goto('/users');
  }
</script>

<svelte:head>
  <title>{user ? user.username : 'Loading...'} | User Details</title>
</svelte:head>

<div class="container mx-auto px-4 py-8">
  <div class="mb-6">
    <Button variant="outline" on:click={goBack} class="flex items-center">
      <ArrowLeft class="mr-2 h-4 w-4" />
      Back to Users
    </Button>
  </div>

  {#if loading}
    <p class="text-center text-xl">Loading user details...</p>
  {:else if error}
    <p class="text-center text-xl text-red-500">Error: {error}</p>
  {:else if user}
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      <Card class="md:col-span-2">
        <CardHeader>
          <CardTitle class="text-2xl">{editing ? 'Edit User' : user.username}</CardTitle>
          <CardDescription>User ID: {user.id}</CardDescription>
        </CardHeader>
        <CardContent>
          {#if editing}
            <form on:submit|preventDefault={handleSave}>
              <div class="grid gap-4">
                <div class="grid gap-2">
                  <Label for="username">Username</Label>
                  <Input id="username" bind:value={user.username} required />
                </div>
                <div class="grid gap-2">
                  <Label for="email">Email</Label>
                  <Input id="email" type="email" bind:value={user.email} required />
                </div>
                <div class="grid gap-2">
                  <Label for="phone">Phone</Label>
                  <Input id="phone" type="tel" bind:value={user.phone} />
                </div>
              </div>
            </form>
          {:else}
            <div class="grid gap-4">
              <div class="flex justify-between items-center">
                <span class="font-semibold">Username:</span>
                <span>{user.username}</span>
              </div>
              <div class="flex justify-between items-center">
                <span class="font-semibold">Email:</span>
                <span>{user.email}</span>
              </div>
              <div class="flex justify-between items-center">
                <span class="font-semibold">Phone:</span>
                <span>{user.phone}</span>
              </div>
              <div class="flex justify-between items-center">
                <span class="font-semibold">Last Login:</span>
                <span>{new Date(user.lastLogin).toLocaleString()}</span>
              </div>
              <div class="flex justify-between items-center">
                <span class="font-semibold">Created At:</span>
                <span>{new Date(user.created_at).toLocaleString()}</span>
              </div>
              <div class="flex justify-between items-center">
                <span class="font-semibold">Updated At:</span>
                <span>{new Date(user.updated_at).toLocaleString()}</span>
              </div>
              <div class="flex justify-between items-center">
                <span class="font-semibold">Active Account:</span>
                <Switch id="active" checked={user.is_active} on:change={handleToggleActive} />
              </div>
              <div class="flex justify-between items-center">
                <span class="font-semibold">Admin Account:</span>
                <Switch id="admin" checked={user.is_admin} disabled />
              </div>
            </div>
          {/if}
        </CardContent>
        <CardFooter class="flex justify-between">
          <Button variant="outline" on:click={() => editing = !editing}>
            {editing ? 'Cancel' : 'Edit'}
          </Button>
          {#if editing}
            <Button on:click={handleSave}>Save Changes</Button>
          {:else}
            <Button variant="destructive" on:click={handleResetPassword}>Reset Password</Button>
          {/if}
        </CardFooter>
      </Card>

      <div class="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>User Accounts</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Account</TableHead>
                  <TableHead>Role</TableHead>
                  <TableHead>Status</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {#each user.accounts as account}
                  <TableRow>
                    <TableCell>{account.name}</TableCell>
                    <TableCell>{account.role}</TableCell>
                    <TableCell>{account.status}</TableCell>
                  </TableRow>
                {/each}
              </TableBody>
            </Table>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Login History</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Date</TableHead>
                  <TableHead>IP Address</TableHead>
                  <TableHead>Device</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {#each user.loginHistory as login}
                  <TableRow>
                    <TableCell>{new Date(login.date).toLocaleString()}</TableCell>
                    <TableCell>{login.ipAddress}</TableCell>
                    <TableCell>{login.device}</TableCell>
                  </TableRow>
                {/each}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>
    </div>
  {:else}
    <p class="text-center text-xl">User not found.</p>
  {/if}
</div>