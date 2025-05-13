<script>
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Avatar from "$lib/components/ui/avatar";
  import { ChevronDown } from '@lucide/svelte';
  
  // Import the images directly
  import companyIcon from '$lib/assets/icons/company_60.png';
  import teamMemberIcon from '$lib/assets/icons/team_member_60.png';

  // Import the stores
  import { selectedTeam } from '$lib/stores/appStore';

  const teams = [
    { name: 'Oply', image: companyIcon },
    { name: 'Admin', image: teamMemberIcon },
  ];

  function switchTeam(team) {
    selectedTeam.set(team.name);
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger asChild>
    <Button variant="outline" class="w-[200px] justify-between">
      <Avatar.Root class="mr-2 h-5 w-5">
        <Avatar.Image 
          src={teams.find(t => t.name === $selectedTeam)?.image} 
          alt={$selectedTeam} 
        />
        <Avatar.Fallback>{$selectedTeam.charAt(0)}</Avatar.Fallback>
      </Avatar.Root>
      {$selectedTeam}
      <ChevronDown class="ml-auto h-4 w-4 shrink-0 opacity-50" />
    </Button>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-[200px] bg-background text-foreground border border-border shadow-md">
    {#each teams as team}
      <DropdownMenu.Item 
        onclick={() => switchTeam(team)}
        class="hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground cursor-pointer"
      >
        <Avatar.Root class="mr-2 h-5 w-5">
          <Avatar.Image src={team.image} alt={team.name} />
          <Avatar.Fallback>{team.name.charAt(0)}</Avatar.Fallback>
        </Avatar.Root>
        {team.name}
      </DropdownMenu.Item>
    {/each}
  </DropdownMenu.Content>
</DropdownMenu.Root>
