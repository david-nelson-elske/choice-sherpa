/**
 * Root layout server load function.
 *
 * Loads the session on the server side and makes it available
 * to all child routes via the layout data.
 */

import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async (event) => {
	const session = await event.locals.auth();

	return {
		session
	};
};
