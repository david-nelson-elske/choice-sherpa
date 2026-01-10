/**
 * SvelteKit server hooks.
 *
 * This module sets up the Auth.js handler for all authentication requests.
 * It intercepts requests to /auth/* and processes them appropriately.
 *
 * @see https://kit.svelte.dev/docs/hooks
 */

import { handle as authHandle } from './auth';
import type { Handle } from '@sveltejs/kit';
import { sequence } from '@sveltejs/kit/hooks';

/**
 * Authentication handler from Auth.js.
 * Handles /auth/signin, /auth/signout, /auth/callback, etc.
 */
const authenticationHandle: Handle = authHandle;

/**
 * Session injection handler.
 * Makes the session available in event.locals for all routes.
 */
const sessionHandle: Handle = async ({ event, resolve }) => {
	// Session is already handled by authHandle, but we can add
	// additional session processing here if needed
	return resolve(event);
};

/**
 * Combined hooks handler.
 * Runs auth first, then any additional processing.
 */
export const handle: Handle = sequence(authenticationHandle, sessionHandle);
