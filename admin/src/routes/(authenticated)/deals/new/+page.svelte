<script>
    import { goto } from '$app/navigation';
    import { api } from '$lib/api';
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Card, CardContent, CardHeader, CardTitle, CardFooter } from "$lib/components/ui/card";
    import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "$lib/components/ui/select";
    import { ArrowLeft } from '@lucide/svelte';
    import { Checkbox } from "$lib/components/ui/checkbox";

    let newDeal = {
        name: '',
        customer_id: '',
        amount: 0,
        status: 'Prospecting', // Default status
        stage: 'Initial Contact', // Default stage
        close_date: '',
        is_active: true
    };

    const dealStatuses = [
        'Prospecting',
        'Qualification',
        'Needs Analysis',
        'Proposal',
        'Negotiation',
        'Closed Won',
        'Closed Lost'
    ];

    const dealStages = [
        'Initial Contact',
        'Meeting Scheduled',
        'Proposal Sent',
        'Contract Sent',
        'Contract Signed',
        'Implementation'
    ];

    let errorMessage = '';
    let customers = [];

    async function loadCustomers() {
        try {
            const response = await api.admin.fetchCustomers();
            customers = response.data || [];
        } catch (error) {
            console.error('Failed to load customers:', error);
            errorMessage = 'Failed to load customers. Please refresh the page.';
        }
    }

    async function handleCreateDeal() {
        try {
            errorMessage = '';
            // Convert amount to float
            const dealData = {
                ...newDeal,
                amount: parseFloat(newDeal.amount),
                close_date: newDeal.close_date ? new Date(newDeal.close_date).toISOString() : null
            };
            
            await api.admin.createDeal(dealData);
            goto('/deals');
        } catch (error) {
            console.error('Failed to create deal:', error);
            errorMessage = 'Failed to create deal. Please try again.';
        }
    }

    function handleBack() {
        goto('/deals');
    }

    // Load customers when component mounts
    import { onMount } from 'svelte';
    onMount(loadCustomers);
</script>

<div class="container mx-auto px-4 py-8">
    <Card>
        <CardHeader class="flex flex-row items-center justify-between">
            <div class="flex items-center space-x-4">
                <Button variant="ghost" on:click={handleBack}>
                    <ArrowLeft class="mr-2 h-4 w-4" />
                    Back
                </Button>
                <h2 class="text-2xl font-bold">Create New Deal</h2>
            </div>
        </CardHeader>
        <CardContent>
            <form on:submit|preventDefault={handleCreateDeal}>
                <div class="grid gap-4">
                    <div class="grid gap-2">
                        <Label for="name">Deal Name</Label>
                        <Input id="name" bind:value={newDeal.name} required />
                    </div>

                    <div class="grid gap-2">
                        <Label for="customer">Customer</Label>
                        <Select bind:value={newDeal.customer_id} required>
                            <SelectTrigger>
                                <SelectValue placeholder="Select a customer" />
                            </SelectTrigger>
                            <SelectContent>
                                {#each customers as customer}
                                    <SelectItem value={customer.id}>{customer.name}</SelectItem>
                                {/each}
                            </SelectContent>
                        </Select>
                    </div>

                    <div class="grid gap-2">
                        <Label for="amount">Amount</Label>
                        <Input 
                            id="amount" 
                            type="number" 
                            step="0.01" 
                            bind:value={newDeal.amount} 
                            required 
                        />
                    </div>

                    <div class="grid gap-2">
                        <Label for="status">Status</Label>
                        <Select bind:value={newDeal.status} required>
                            <SelectTrigger>
                                <SelectValue placeholder="Select status" />
                            </SelectTrigger>
                            <SelectContent>
                                {#each dealStatuses as status}
                                    <SelectItem value={status}>{status}</SelectItem>
                                {/each}
                            </SelectContent>
                        </Select>
                    </div>

                    <div class="grid gap-2">
                        <Label for="stage">Stage</Label>
                        <Select bind:value={newDeal.stage} required>
                            <SelectTrigger>
                                <SelectValue placeholder="Select stage" />
                            </SelectTrigger>
                            <SelectContent>
                                {#each dealStages as stage}
                                    <SelectItem value={stage}>{stage}</SelectItem>
                                {/each}
                            </SelectContent>
                        </Select>
                    </div>

                    <div class="grid gap-2">
                        <Label for="close_date">Expected Close Date</Label>
                        <Input 
                            id="close_date" 
                            type="date" 
                            bind:value={newDeal.close_date} 
                        />
                    </div>

                    <div class="flex items-center space-x-2">
                        <Checkbox id="is_active" bind:checked={newDeal.is_active} />
                        <Label for="is_active">Active Deal</Label>
                    </div>
                </div>

                {#if errorMessage}
                    <p class="text-red-500 mt-2">{errorMessage}</p>
                {/if}
            </form>
        </CardContent>
        <CardFooter>
            <Button on:click={handleCreateDeal}>Create Deal</Button>
        </CardFooter>
    </Card>
</div>