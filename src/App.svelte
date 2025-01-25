<script lang="ts">
  import { onMount } from 'svelte';
  import { selectedDate, dateRange } from './lib/stores';
  import { setupEventListeners } from './lib/events';
  import { invoke } from '@tauri-apps/api/tauri';
  import type { WindowActivity } from './types';

  let activities: WindowActivity[] = [];
  let loading = true;
  let error: string | null = null;

  $: fetchActivities($selectedDate);

  async function fetchActivities(date: Date) {
    try {
      loading = true;
      error = null;
      activities = await invoke('get_activities_for_day', {
        date: date.toISOString()
      });
    } catch (e) {
      error = e.toString();
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await setupEventListeners();
    await fetchActivities($selectedDate);
  });
</script>

<main class="container mx-auto p-4">
  <h1 class="text-2xl font-bold mb-4">Chronos Track</h1>

  {#if loading}
    <p>Loading...</p>
  {:else if error}
    <p class="text-red-500">{error}</p>
  {:else}
    <div class="space-y-4">
      <h2 class="text-xl">Activities for {$selectedDate.toLocaleDateString()}</h2>
      
      {#if activities.length === 0}
        <p>No activities recorded for this day.</p>
      {:else}
        <ul class="space-y-2">
          {#each activities as activity}
            <li class="p-2 bg-gray-100 rounded">
              <div class="font-semibold">{activity.application}</div>
              <div class="text-sm text-gray-600">{activity.title}</div>
              <div class="text-xs text-gray-500">
                {new Date(activity.start_time).toLocaleTimeString()} - 
                {new Date(activity.end_time).toLocaleTimeString()}
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  }
</style> 