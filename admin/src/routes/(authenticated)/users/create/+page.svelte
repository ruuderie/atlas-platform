<script>
    import { goto } from '$app/navigation';
    import { api } from '$lib/api';
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Card, CardContent, CardHeader, CardTitle, CardFooter } from "$lib/components/ui/card";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import { ArrowLeft } from '@lucide/svelte';

    let newUser = {
        username: '',
        email: '',
        password: '',
        is_admin: false,
        is_active: true
    };

    let errorMessage = '';

    async function handleCreateUser() {
        try {
            errorMessage = '';
            await api.admin.createUser(newUser);
            goto('/users');
        } catch (error) {
            console.error('Failed to create user:', error);
            errorMessage = 'Failed to create user. Please try again.';
        }
    }

    function handleBack() {
        goto('/users');
    }
</script>

<div class="container mx-auto px-4 py-8">
    <Card>
        <CardHeader class="flex flex-row items-center justify-between">
            <div class="flex items-center space-x-4">
                <Button variant="ghost" on:click={handleBack}>
                    <ArrowLeft class="mr-2 h-4 w-4" />
                    Back
                </Button>
                <h2 class="text-2xl font-bold">Create New User</h2>
            </div>
        </CardHeader>
        <CardContent>
            <form on:submit|preventDefault={handleCreateUser}>
                <div class="grid gap-4">
                    <div class="grid gap-2">
                        <Label for="username">Username</Label>
                        <Input id="username" bind:value={newUser.username} required />
                    </div>
                    <div class="grid gap-2">
                        <Label for="email">Email</Label>
                        <Input id="email" type="email" bind:value={newUser.email} required />
                    </div>
                    <div class="grid gap-2">
                        <Label for="password">Password</Label>
                        <Input id="password" type="password" bind:value={newUser.password} required />
                    </div>
                    <div class="flex items-center space-x-2">
                        <Checkbox id="is_admin" bind:checked={newUser.is_admin} />
                        <Label for="is_admin">Admin User</Label>
                    </div>
                    <div class="flex items-center space-x-2">
                        <Checkbox id="is_active" bind:checked={newUser.is_active} />
                        <Label for="is_active">Active User</Label>
                    </div>
                </div>
                {#if errorMessage}
                    <p class="text-red-500 mt-2">{errorMessage}</p>
                {/if}
            </form>
        </CardContent>
        <CardFooter>
            <Button on:click={handleCreateUser}>Create User</Button>
        </CardFooter>
    </Card>
</div>