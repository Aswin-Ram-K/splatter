import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import fs from "fs";

// Vite plugin to copy ghostty-vt.wasm to dist root and serve during dev
function copyGhosttyWasm() {
	return {
		name: "copy-ghostty-wasm",
		configureServer(serverInstance) {
			// Serve WASM file during dev — critical for Tauri
			serverInstance.middlewares.use((req: any, res: any) => {
				if (req.url === "/ghostty-vt.wasm") {
					const src = path.resolve(
						__dirname,
						"node_modules/ghostty-web/ghostty-vt.wasm",
					);
					res.setHeader("Content-Type", "application/wasm");
					fs.createReadStream(src).pipe(res);
				}
			});
		},
		closeBundle() {
			const src = path.resolve(
				__dirname,
				"node_modules/ghostty-web/ghostty-vt.wasm",
			);
			const dst = path.resolve(__dirname, "dist/ghostty-vt.wasm");
			fs.copyFileSync(src, dst);
		},
	};
}

export default defineConfig({
	plugins: [react(), copyGhosttyWasm()],
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "./src"),
		},
	},
	server: {
		port: 5173,
		strictPort: true,
		headers: {
			"Cross-Origin-Opener-Policy": "same-origin",
			"Cross-Origin-Embedder-Policy": "require-corp",
		},
		proxy: {
			"/ghostty-vt.wasm": {
				target: "http://localhost:5173",
				changeOrigin: true,
			},
		},
	},
	build: {
		rollupOptions: {
			output: {
				manualChunks: {
					// Split ghostty-web into its own chunk
					ghostty: ["ghostty-web"],
				},
			},
		},
	},
	// Serve WASM files during dev
	optimizeDeps: {
		exclude: ["ghostty-web"],
	},
	// Ensure WASM file is copied to dist
	assetsInlineLimit: 0,
});
