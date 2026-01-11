<script lang="ts">
  import { onMount } from 'svelte';
  import { getSession, renameSession, archiveSession } from '../api/sessionApi';
  import { sessionStore } from '../stores/sessionStore';
  import type { Session } from '../types';

  export let sessionId: string;
  export let onArchived: (() => void) | null = null;

  let session: Session | null = null;
  let loading = false;
  let error: string | null = null;
  let editing = false;
  let editTitle = '';

  async function loadSession() {
    loading = true;
    error = null;

    try {
      session = await getSession(sessionId);
      sessionStore.setCurrentSession(session);
      editTitle = session.title;
    } catch (e: any) {
      error = e.message || 'Failed to load session';
    } finally {
      loading = false;
    }
  }

  async function handleRename() {
    if (!session || !editTitle.trim()) return;

    try {
      await renameSession(session.id, { title: editTitle.trim() });
      session = { ...session, title: editTitle.trim() };
      sessionStore.updateSession(session.id, { title: editTitle.trim() });
      editing = false;
    } catch (e: any) {
      error = e.message || 'Failed to rename session';
    }
  }

  async function handleArchive() {
    if (!session) return;

    const confirmed = confirm(`Are you sure you want to archive "${session.title}"?`);
    if (!confirmed) return;

    try {
      await archiveSession(session.id);
      if (onArchived) {
        onArchived();
      }
    } catch (e: any) {
      error = e.message || 'Failed to archive session';
    }
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return new Intl.DateTimeFormat('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  }

  onMount(() => {
    loadSession();
  });
</script>

<div class="session-detail">
  {#if loading}
    <div class="loading">Loading session...</div>
  {:else if error}
    <div class="error">
      <p>{error}</p>
      <button on:click={loadSession}>Retry</button>
    </div>
  {:else if session}
    <div class="session-header">
      {#if editing}
        <input
          type="text"
          bind:value={editTitle}
          on:keydown={(e) => e.key === 'Enter' && handleRename()}
          class="title-input"
        />
        <div class="edit-actions">
          <button on:click={handleRename} class="save">Save</button>
          <button on:click={() => { editing = false; editTitle = session.title; }}>Cancel</button>
        </div>
      {:else}
        <h1>{session.title}</h1>
        {#if session.status === 'active'}
          <button on:click={() => editing = true} class="edit-btn">Rename</button>
        {/if}
      {/if}

      {#if session.status === 'archived'}
        <span class="archived-badge">Archived</span>
      {/if}
    </div>

    {#if session.description}
      <p class="description">{session.description}</p>
    {/if}

    <div class="session-meta">
      <div class="meta-item">
        <strong>Cycles:</strong> {session.cycle_count}
      </div>
      <div class="meta-item">
        <strong>Created:</strong> {formatDate(session.created_at)}
      </div>
      <div class="meta-item">
        <strong>Updated:</strong> {formatDate(session.updated_at)}
      </div>
      <div class="meta-item">
        <strong>Status:</strong>
        <span class="status-badge" class:active={session.status === 'active'}>
          {session.status}
        </span>
      </div>
    </div>

    {#if session.status === 'active'}
      <div class="actions">
        <button on:click={handleArchive} class="archive-btn">Archive Session</button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .session-detail {
    padding: 2rem;
  }

  .loading,
  .error {
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

  .session-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  h1 {
    margin: 0;
    font-size: 2rem;
    flex: 1;
  }

  .title-input {
    flex: 1;
    padding: 0.5rem;
    font-size: 2rem;
    border: 1px solid var(--color-border, #e0e0e0);
    border-radius: 4px;
  }

  .edit-actions {
    display: flex;
    gap: 0.5rem;
  }

  button {
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-border, #e0e0e0);
    border-radius: 4px;
    background: white;
    cursor: pointer;
  }

  button:hover {
    background: var(--color-hover, #f5f5f5);
  }

  .edit-btn,
  .save {
    background: var(--color-primary, #1976d2);
    color: white;
    border-color: var(--color-primary, #1976d2);
  }

  .archive-btn {
    background: var(--color-error, #d32f2f);
    color: white;
    border-color: var(--color-error, #d32f2f);
  }

  .description {
    margin-bottom: 2rem;
    color: var(--color-text-secondary, #666);
  }

  .session-meta {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    padding: 1.5rem;
    background: var(--color-surface, #f5f5f5);
    border-radius: 8px;
    margin-bottom: 2rem;
  }

  .meta-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .status-badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    background: var(--color-warning, #ffa726);
    color: white;
    font-size: 0.875rem;
    font-weight: 600;
    text-transform: capitalize;
  }

  .status-badge.active {
    background: var(--color-success, #66bb6a);
  }

  .archived-badge {
    padding: 0.5rem 1rem;
    background: var(--color-warning, #ffa726);
    color: white;
    border-radius: 4px;
    font-weight: 600;
  }

  .actions {
    display: flex;
    gap: 1rem;
  }
</style>
