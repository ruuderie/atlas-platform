<script>
   // import UserRegistration from '$lib/components/UserRegistration.svelte';
    import { api } from '$lib/api';  // Update this import
    import { goto } from '$app/navigation';
  
    let errorMessage = '';
  
    async function handleRegister(event) {
      const { username, email, password } = event.detail;
      try {
        await api.user.register({ username, email, password });  // Update this line
        goto('/login');  // Use goto for client-side navigation
      } catch (err) {
        console.error('Registration error:', err);
        errorMessage = err.message || 'Registration failed. Please try again.';
      }
    }
</script>
  
<!-- <UserRegistration on:register={handleRegister} {errorMessage} /> -->
  
{#if errorMessage}
  <p class="text-red-500 text-center mt-4">{errorMessage}</p>
{/if}