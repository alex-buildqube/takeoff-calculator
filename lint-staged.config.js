/**
 * @filename: lint-staged.config.js
 * @type {import('lint-staged').Configuration}
 */
export default {
	"*.@(js|ts|tsx)": ["pnpm exec biome check --write"],
	"*.@(js|ts|tsx|yml|yaml|md|json)": ["pnpm exec biome check --write"],
	"*.toml": ["taplo format"],
	"*.rs": (_files) => [
		"turbo run format:rs format:fix",
		"cargo clippy --workspace --fix --allow-dirty",
	],
};
