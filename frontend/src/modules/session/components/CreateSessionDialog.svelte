<script lang="ts">
  import { createSession } from '../api/sessionApi';
  import { sessionStore } from '../stores/sessionStore';

  export let open = false;
  export let onClose: (() => void) | null = null;
  export let onSuccess: ((sessionId: string) => void) | null = null;

  let title = '';
  let description = '';
  let loading = false;
  let error: string | null = null;

  async function handleSubmit() {
    if (!title.trim()) {
      error = 'Title is required';
      return;
    }

    loading = true;
    error = null;

    try {
      const result = await createSession({
        title: title.trim(),
        description: description.trim() || undefined,
      });

      // Reset form
      title = '';
      description = '';

      if (onSuccess) {
        onSuccess(result.session_id);
      }

      if (onClose) {
        onClose();
      }
    } catch (e: any) {
      error = e.message || 'Failed to create session';
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    if (loading) return;
    title = '';
    description = '';
    error = null;
    if (onClose) {
      onClose();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
  }
</script>

{#if open}
  <div class="dialog-overlay" on:click={handleClose} on:keydown={handleKeydown} role="button" tabindex="-1">
    <div class="dialog" on:click|stopPropagation role="dialog" aria-labelledby="dialog-title">
      <h2 id="dialog-title">Create New Session</h2>

      <form on:submit|preventDefault={handleSubmit}>
        <div class="form-group">
          <label for="title">Title *</label>
          <input
            id="title"
            type="text"
            bind:value={title}
            placeholder="Enter session title"
            maxlength="500"
            disabled={loading}
            required
          />
        </div>

        <div class="form-group">
          <label for="description">Description</label>
          <textarea
            id="description"
            bind:value={description}
            placeholder="Optional description"
            rows="4"
            disabled={loading}
          />
        </div>

        {#if error}
          <div class="error">{error}</div>
        {/if}

        <div class="dialog-actions">
          <button type="button" on:click={handleClose} disabled={loading}>Cancel</button>
          <button type="submit" class="primary" disabled={loading}>
            {loading ? 'Creating...' : 'Create Session'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    max-width: 500px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
  }

  h2 {
    margin: 0 0 1.5rem 0;
    font-size: 1.5rem;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
  }

  input,
  textarea {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--color-border, #e0e0e0);
    border-radius: 4px;
    font-family: inherit;
    font-size: 1rem;
  }

  input:focus,
  textarea:focus {
    outline: none;
    border-color: var(--color-primary, #1976d2);
  }

  textarea {
    resize: vertical;
  }

  .error {
    padding: 0.75rem;
    background: #ffebee;
    color: #d32f2f;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .dialog-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
  }

  button {
    padding: 0.75rem 1.5rem;
    border: 1px solid var(--color-border, #e0e0e0);
    border-radius: 4px;
    background: white;
    cursor: pointer;
    font-size: 1rem;
  }

  button:hover:not(:disabled) {
    background: var(--color-hover, #f5f5f5);
  }

  button.primary {
    background: var(--color-primary, #1976d2);
    color: white;
    border-color: var(--color-primary, #1976d2);
  }

  button.primary:hover:not(:disabled) {
    background: var(--color-primary-dark, #1565c0);
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
