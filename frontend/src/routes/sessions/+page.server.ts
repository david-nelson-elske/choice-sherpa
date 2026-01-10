/**
 * Sessions page server-side load function.
 *
 * This is a protected route - requires authentication.
 * Demonstrates usage of the requireAuth guard.
 */

import type { PageServerLoad } from './$types';
import { requireAuth } from '$lib/auth/guards';

export const load: PageServerLoad = async (event) => {
	// Require authentication - redirects to sign-in if not authenticated
	const session = await requireAuth(event);

	// In a real app, we'd fetch sessions from the API here
	// const sessions = await authGet('/api/sessions', session);

	return {
		user: session.user,
		sessions: [] // Placeholder - would come from API
	};
};
