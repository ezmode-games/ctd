/**
 * Decompress gzip request body if Content-Encoding is gzip.
 * Works in both Node.js and Workers environments.
 */
export async function decompressBody(
	body: ArrayBuffer,
	contentEncoding: string | undefined,
): Promise<string> {
	if (contentEncoding === 'gzip') {
		const ds = new DecompressionStream('gzip');
		const decompressed = new Response(
			new Blob([body]).stream().pipeThrough(ds),
		);
		return decompressed.text();
	}
	return new TextDecoder().decode(body);
}
