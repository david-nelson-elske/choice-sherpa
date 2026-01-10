/**
 * Route guard utilities for protecting pages.
 *
 * These helpers are used in +page.server.ts or +layout.server.ts
 * load functions to enforce authentication requirements.
 *
 * @example
 * ```typescript
 * // In +page.server.ts
 * import { requireAuth } from '$lib/auth/guards';
 *
 * export const load = async (event) => {
 *   const session = await requireAuth(event);
 *   // session is guaranteed to be non-null here
 *   return { user: session.user };
 * };
 * ```
 */

import { redirect } from '@sveltejs/kit';
import type { RequestEvent } from '@sveltejs/kit';
import type { Session } from '@auth/sveltekit';

/**
 * Require authentication for a route.
 *
 * If the user is not authenticated, redirects to the sign-in page.
 * The current URL is passed as a callback URL so the user is
 * redirected back after signing in.
 *
 * @param event - The SvelteKit request event
 * @returns The authenticated session
 * @throws Redirect to /login if not authenticated
 */
export async function requireAuth(event: RequestEvent): Promise<Session> {
	const session = await event.locals.auth();

	if (!session?.user) {
		// Redirect to sign-in with callback URL
		const callbackUrl = encodeURIComponent(event.url.pathname + event.url.search);
		redirect(303, `/login?callbackUrl=${callbackUrl}`);
	}

	return session;
}

/**
 * Get the current session, returning null if not authenticated.
 *
 * Use this for routes where authentication is optional.
 *
 * @param event - The SvelteKit request event
 * @returns The session or null
 */
export async function getSession(event: RequestEvent): Promise<Session | null> {
	return event.locals.auth();
}

/**
 * Require that the user is NOT authenticated.
 *
 * Use this for pages like sign-in or sign-up that should redirect
 * authenticated users away.
 *
 * @param event - The SvelteKit request event
 * @param redirectTo - Where to redirect authenticated users (default: '/')
 * @throws Redirect if user is authenticated
 */
export async function requireGuest(
	event: RequestEvent,
	redirectTo: string = '/'
): Promise<void> {
	const session = await event.locals.auth();

	if (session?.user) {
		redirect(303, redirectTo);
	}
}
