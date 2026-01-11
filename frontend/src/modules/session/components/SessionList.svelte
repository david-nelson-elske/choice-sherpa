<script lang="ts">
  import { onMount } from 'svelte';
  import { sessionStore } from '../stores/sessionStore';
  import { listSessions } from '../api/sessionApi';
  import type { ListSessionsQuery } from '../types';

  export let query: ListSessionsQuery = {};
  export let onSelectSession: ((sessionId: string) => void) | null = null;

  let loading = false;
  let error: string | null = null;
  let sessions: any[] = [];

  async function loadSessions() {
    loading = true;
    error = null;

    try {
      const list = await listSessions(query);
      sessionStore.setSessionList(list);
      sessions = list.items;
    } catch (e: any) {
      error = e.message || 'Failed to load sessions';
      sessionStore.setError(error);
    } finally {
      loading = false;
    }
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return new Intl.DateTimeFormat('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    }).format(date);
  }

  function handleSessionClick(sessionId: string) {
    if (onSelectSession) {
      onSelectSession(sessionId);
    }
  }

  onMount(() => {
    loadSessions();
  });
</script>

<div class="session-list">
  {#if loading}
    <div class="loading">Loading sessions...</div>
  {:else if error}
    <div class="error">
      <p>{error}</p>
      <button on:click={loadSessions}>Retry</button>
    </div>
  {:else if sessions.length === 0}
    <div class="empty">
      <p>No sessions found</p>
    </div>
  {:else}
    <div class="sessions">
      {#each sessions as session (session.id)}
        <div
          class="session-card"
          class:archived={session.status === 'archived'}
          on:click={() => handleSessionClick(session.id)}
          on:keypress={(e) => e.key === 'Enter' && handleSessionClick(session.id)}
          role="button"
          tabindex="0"
        >
          <h3 class="session-title">{session.title}</h3>
          <div class="session-meta">
            <span class="cycle-count">{session.cycle_count} cycles</span>
            <span class="updated-at">Updated {formatDate(session.updated_at)}</span>
          </div>
          {#if session.status === 'archived'}
            <span class="archived-badge">Archived</span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .session-list {
    padding: 1rem;
  }

  .loading,
  .error,
  .empty {
    padding: 2rem;
    text-align: center;
  }

  .error {
    color: var(--color-error, #d32f2f);
  }

  .error button {
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    background: var(--color-primary, #1976d2);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .sessions {
    display: grid;
    gap: 1rem;
  }

  .session-card {
    padding: 1rem;
    background: var(--color-surface, #fff);
    border: 1px solid var(--color-border, #e0e0e0);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .session-card:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    transform: translateY(-2px);
  }

  .session-card.archived {
    opacity: 0.6;
  }

  .session-title {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .session-meta {
    display: flex;
    gap: 1rem;
    font-size: 0.875rem;
    color: var(--color-text-secondary, #666);
  }

  .archived-badge {
    display: inline-block;
    margin-top: 0.5rem;
    padding: 0.25rem 0.5rem;
    background: var(--color-warning, #ffa726);
    color: white;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }
</style>
