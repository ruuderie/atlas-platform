<script>
 import { Button } from "$lib/components/ui/button/index.js";
 import * as Card from "$lib/components/ui/card/index.js";
 
 let { listing } = $props();

 // Format price with commas
 const formatPrice = (price) => {
   return price ? price.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",") : 'N/A';
 };

 // Format date
 const formatDate = (dateString) => {
   return new Date(dateString).toLocaleDateString('en-US', {
     year: 'numeric',
     month: 'long',
     day: 'numeric'
   });
 };

 // Capitalize first letter
 const capitalize = (str) => str.charAt(0).toUpperCase() + str.slice(1);

 // Function to render object or array values
 const renderValue = (value) => {
   if (Array.isArray(value)) {
     return value.join(', ');
   } else if (typeof value === 'object' && value !== null) {
     return Object.entries(value).map(([k, v]) => `${capitalize(k.replace('_', ' '))}: ${renderValue(v)}`).join(', ');
   }
   return value;
 };

 // Fields to exclude from general rendering
 const excludeFields = ['id', 'title', 'description', 'created_at', 'updated_at', 'additional_info', 'profile_id', 'directory_id', 'category_id'];
</script>

<div class="w-full">
  <Card.Root class="h-full bg-white shadow-md rounded-lg overflow-hidden">
    <Card.Header class="p-4 border-b">
      <Card.Title class="text-xl font-semibold">{listing.title}</Card.Title>
      <Card.Description class="text-gray-500">{capitalize(listing.listing_type)}</Card.Description>
    </Card.Header>
    <Card.Content class="p-4">
      <div class="grid w-full gap-4">
        <div class="space-y-1.5">
          <p><strong>Description:</strong> {listing.description}</p>
        </div>
        
        <div class="space-y-1.5">
          <p><strong>Location:</strong> {listing.city}, {listing.state}, {listing.country}</p>
          {#if listing.neighborhood}
            <p><strong>Neighborhood:</strong> {listing.neighborhood}</p>
          {/if}
        </div>

        {#if listing.price}
          <div class="space-y-1.5">
            <p><strong>Price:</strong> ${formatPrice(listing.price)} {listing.price_type ? `per ${listing.price_type.replace('_', ' ')}` : ''}</p>
          </div>
        {/if}

        {#if listing.latitude && listing.longitude}
          <div class="space-y-1.5">
            <p><strong>Coordinates:</strong> {listing.latitude.toFixed(6)}, {listing.longitude.toFixed(6)}</p>
          </div>
        {/if}

        {#each Object.entries(listing) as [key, value]}
          {#if value && !excludeFields.includes(key) && !['price', 'price_type', 'city', 'state', 'country', 'neighborhood', 'latitude', 'longitude'].includes(key)}
            <div class="space-y-1.5">
              <p><strong>{capitalize(key.replace('_', ' '))}:</strong> {renderValue(value)}</p>
            </div>
          {/if}
        {/each}

        {#if listing.additional_info}
          <div class="space-y-1.5">
            <p><strong>Additional Info:</strong></p>
            <ul class="list-disc list-inside">
              {#each Object.entries(listing.additional_info) as [key, value]}
                <li>{capitalize(key.replace('_', ' '))}: {renderValue(value)}</li>
              {/each}
            </ul>
          </div>
        {/if}
      </div>
    </Card.Content>
    <Card.Footer class="p-4 border-t flex justify-between items-center">
      <div class="text-sm text-gray-500">
        <p>Status: {listing.status}</p>
        <p>Created: {formatDate(listing.created_at)}</p>
        {#if listing.is_featured}
          <p class="text-yellow-500 font-bold">Featured</p>
        {/if}
      </div>
      <Button variant="outline" href="/listing/{listing.id}">View Details</Button>
    </Card.Footer>
  </Card.Root>
</div>
