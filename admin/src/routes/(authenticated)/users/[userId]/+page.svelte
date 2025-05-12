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
  import { formatDate } from '$lib/utils';

  let userData = null;
  let user = null;
  let userAccounts = [];
  let profiles = [];
  let directories = [];
  let loginHistory = []; // New variable for login history
  let loading = true;
  let error = null;
  let editing = false;
  let showDeactivatePrompt = false;

  let userId = $derived($page.params.userId);
  let queryParams = $derived($page.url.searchParams);

  onMount(() => {
    if (queryParams.get('edit') === 'true') {
      editing = true;
    }
    if (queryParams.get('deactivate') === 'true') {
      showDeactivatePrompt = true;
    }
  });

  $effect(() => {
    if (userId) {
      loadUserData(userId);
    }
  });

  async function loadUserData(id) {
    loading = true;
    try {
      const response = await api.admin.fetchUserById(id);
      console.log("User data:", response);
      
      // Destructure the response, providing default values
      ({ user = null, user_accounts: userAccounts = [], profiles = [], directories = [], login_history: loginHistory = [] } = response);
      userData = response;  // Keep the full response if needed elsewhere
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
    if (user.is_active) {
      showDeactivatePrompt = true;
    } else {
      await toggleUserActive();
    }
  }

  async function toggleUserActive() {
    try {
      user.is_active = !user.is_active;
      await api.admin.updateUser(user.id, { is_active: user.is_active });
      showDeactivatePrompt = false;
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

  function cancelDeactivation() {
    showDeactivatePrompt = false;
  }

  function formatDateTime(dateString) {
    return dateString ? formatDate(new Date(dateString), 'yyyy-MM-dd HH:mm:ss') : 'N/A';
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
            {#if userAccounts.length > 0}
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Account ID</TableHead>
                    <TableHead>User ID</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {#each userAccounts as account}
                    <TableRow>
                      <TableCell>{account.account_id}</TableCell>
                      <TableCell>{account.user_id}</TableCell>
                    </TableRow>
                  {/each}
                </TableBody>
              </Table>
            {:else}
              <p>No user accounts found.</p>
            {/if}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Profiles</CardTitle>
          </CardHeader>
          <CardContent>
            {#if profiles.length > 0}
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Profile ID</TableHead>
                    <TableHead>Account ID</TableHead>
                    <TableHead>Directory ID</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {#each profiles as profile}
                    <TableRow>
                      <TableCell>{profile.id}</TableCell>
                      <TableCell>{profile.account_id}</TableCell>
                      <TableCell>{profile.directory_id}</TableCell>
                    </TableRow>
                  {/each}
                </TableBody>
              </Table>
            {:else}
              <p>No profiles found.</p>
            {/if}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Directories</CardTitle>
          </CardHeader>
          <CardContent>
            {#if directories.length > 0}
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Directory ID</TableHead>
                    <TableHead>Name</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {#each directories as directory}
                    <TableRow>
                      <TableCell>{directory.id}</TableCell>
                      <TableCell>{directory.name}</TableCell>
                    </TableRow>
                  {/each}
                </TableBody>
              </Table>
            {:else}
              <p>No directories found.</p>
            {/if}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Login History</CardTitle>
          </CardHeader>
          <CardContent>
            {#if loginHistory.length > 0}
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Date</TableHead>
                    <TableHead>IP Address</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {#each loginHistory as login}
                    <TableRow>
                      <TableCell>{formatDateTime(login.created_at)}</TableCell>
                      <TableCell>{login.ip_address}</TableCell>
                    </TableRow>
                  {/each}
                </TableBody>
              </Table>
            {:else}
              <p>No login history found.</p>
            {/if}
          </CardContent>
        </Card>
      </div>
    </div>
  {:else}
    <p class="text-center text-xl">User not found.</p>
  {/if}

  {#if showDeactivatePrompt}
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center">
      <Card class="w-96">
        <CardHeader>
          <CardTitle>Deactivate User</CardTitle>
        </CardHeader>
        <CardContent>
          <p>Are you sure you want to deactivate this user?</p>
        </CardContent>
        <CardFooter class="flex justify-between">
          <Button variant="outline" on:click={cancelDeactivation}>Cancel</Button>
          <Button variant="destructive" on:click={toggleUserActive}>Deactivate</Button>
        </CardFooter>
      </Card>
    </div>
  {/if}
</div>