/**
 * Load function for the document page.
 *
 * Fetches the document for the given cycle ID.
 */

import type { PageLoad } from './$types';
import { getDocument } from '$lib/api/document';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params }) => {
	const { cycleId } = params;

	try {
		// Fetch the document for this cycle
		const document = await getDocument(cycleId);

		return {
			cycleId,
			document
		};
	} catch (err) {
		// Handle API errors
		if (err && typeof err === 'object' && 'status' in err) {
			const status = (err as { status: number }).status;
			if (status === 404) {
				throw error(404, 'Document not found');
			}
		}
		throw error(500, 'Failed to load document');
	}
};
