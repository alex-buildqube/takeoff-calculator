/**
 * @filename: lint-staged.config.js
 * @type {import('lint-staged').Configuration}
 */
export default {
	"*.rs": (_files) => [
		"pnpm build",
		"turbo run format:rs format:fix",
		"cargo clippy --workspace --fix --allow-dirty",
	],
	"*.@(js|ts|tsx|yml|yaml|md|json)": ["pnpm exec biome check --write"],
	"*.toml": ["taplo format"],
};
